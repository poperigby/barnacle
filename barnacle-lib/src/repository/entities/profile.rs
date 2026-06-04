use std::{fs, path::PathBuf};

use heck::ToSnakeCase;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter,
    QueryOrder,
};
use tracing::info;

use crate::repository::{
    Cfg,
    db::{Db, models::profiles},
    entities::{Error, Game, Mod, ModEntry, Result},
};

#[derive(Debug, Clone)]
pub struct Profile {
    pub(crate) id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Profile {
    pub(crate) async fn load(row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let model = profiles::Entity::find_by_id(row_id).one(db.conn()).await?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self {
            id: model.id,
            db,
            cfg,
        })
    }

    async fn model(&self) -> Result<profiles::Model> {
        let model = profiles::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?;
        model.ok_or(Error::RemovedEntity)
    }

    pub async fn name(&self) -> Result<String> {
        Ok(self.model().await?.name)
    }

    pub async fn set_name(&self, new_name: &str) -> Result<()> {
        if new_name == self.name().await? {
            return Ok(());
        }

        let old_dir = self.dir().await?;
        let mut active = self.model().await?.into_active_model();
        active.name = Set(new_name.to_string());
        active.update(self.db.conn()).await.map_err(Error::from)?;
        let new_dir = self.dir().await?;
        fs::rename(old_dir, new_dir).unwrap();
        Ok(())
    }

    pub async fn dir(&self) -> Result<PathBuf> {
        let parent_dir = self.parent().await?.dir().await?;
        let name = self.name().await?;
        Ok(parent_dir.join("profiles").join(name.to_snake_case()))
    }

    pub async fn activate(&self) -> Result<()> {
        let game_id = self.parent().await?.id;

        profiles::Entity::update_many()
            .filter(profiles::COLUMN.game_id.eq(game_id))
            .col_expr(
                profiles::COLUMN.is_active,
                sea_orm::sea_query::Expr::value(false),
            )
            .exec(self.db.conn())
            .await?;

        let Some(model) = profiles::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
        else {
            return Err(Error::Internal(sea_orm::DbErr::Custom(
                "missing profile during activation".into(),
            )));
        };
        let mut active = model.into_active_model();
        active.is_active = Set(true);
        active.update(self.db.conn()).await?;

        Ok(())
    }

    pub async fn is_active(&self) -> Result<bool> {
        Ok(
            Profile::active(self.db.clone(), self.cfg.clone(), self.parent().await?).await?
                == Some(self.clone()),
        )
    }

    pub(crate) async fn active(db: Db, cfg: Cfg, game: Game) -> Result<Option<Profile>> {
        let model = profiles::Entity::find()
            .filter(profiles::COLUMN.game_id.eq(game.id))
            .filter(profiles::COLUMN.is_active.eq(true))
            .order_by_asc(profiles::COLUMN.id)
            .one(db.conn())
            .await?;

        if let Some(model) = model {
            Ok(Some(
                Profile::load(model.id, db.clone(), cfg.clone()).await?,
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn parent(&self) -> Result<Game> {
        let game_id = self.model().await?.game_id;
        Game::load(game_id, self.db.clone(), self.cfg.clone()).await
    }

    pub async fn add_mod_entry(&self, mod_: Mod) -> Result<ModEntry> {
        ModEntry::add(&self.db, &self.cfg, self, mod_).await
    }

    pub async fn mod_entries(&self) -> Result<Vec<ModEntry>> {
        ModEntry::list(&self.db, &self.cfg, self).await
    }

    pub async fn remove(self) -> Result<()> {
        for entry in self.mod_entries().await? {
            let entry_id = entry.entry_id;
            entry
                .remove()
                .await
                .or_else(|err| match err {
                    Error::RemovedEntity => Ok(()),
                    other => Err(other),
                })
                .unwrap_or_else(|err| {
                    panic!("Failed to remove mod entry: {entry_id:?}: {err} during profile cleanup")
                });
        }

        let parent_game = self.parent().await?;
        let name = self.name().await?;
        let dir = self.dir().await?;
        let was_active = self.is_active().await?;
        let Some(model) = profiles::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
        else {
            return Err(Error::Internal(sea_orm::DbErr::Custom(
                "missing profile during delete".into(),
            )));
        };
        model.delete(self.db.conn()).await?;

        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }

        if was_active
            && let Some(first_profile) = Profile::list(&self.db, &self.cfg, &parent_game)
                .await?
                .first()
        {
            first_profile.activate().await?;
        }

        info!("Removed profile: {name}");
        Ok(())
    }

    pub(crate) async fn add(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Result<Self> {
        let model = profiles::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            game_id: Set(game.id),
            name: Set(name.to_string()),
            is_active: Set(false),
        };
        let inserted = model.insert(db.conn()).await.map_err(Error::from)?;

        let profile = Profile::load(inserted.id, db.clone(), cfg.clone()).await?;
        fs::create_dir_all(profile.dir().await?).unwrap();

        if Profile::active(db.clone(), cfg.clone(), game.clone())
            .await?
            .is_none()
            && let Some(first_profile) = Profile::list(db, cfg, game).await?.first()
        {
            first_profile.activate().await?;
            return Ok(first_profile.clone());
        }

        Ok(profile)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>> {
        let models = profiles::Entity::find()
            .filter(profiles::COLUMN.game_id.eq(game.id))
            .order_by_asc(profiles::COLUMN.id)
            .all(db.conn())
            .await?;

        let mut out = Vec::with_capacity(models.len());
        for model in models {
            out.push(Profile::load(model.id, db.clone(), cfg.clone()).await?);
        }
        Ok(out)
    }

    pub(crate) async fn search(
        db: Db,
        cfg: Cfg,
        game: &Game,
        name: &str,
    ) -> Result<Option<Profile>> {
        let model = profiles::Entity::find()
            .filter(profiles::COLUMN.game_id.eq(game.id))
            .filter(profiles::COLUMN.name.eq(name))
            .one(db.conn())
            .await?;

        if let Some(model) = model {
            Ok(Some(
                Profile::load(model.id, db.clone(), cfg.clone()).await?,
            ))
        } else {
            Ok(None)
        }
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod test {
    use crate::{
        Repository,
        repository::{DeployKind, entities::Error},
    };

    #[tokio::test]
    async fn test_add() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();
        assert!(profile.dir().await.unwrap().exists());
    }

    #[tokio::test]
    async fn test_add_duplicate() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        game.add_profile("Test").await.unwrap();

        assert!(matches!(
            game.add_profile("Test").await,
            Err(Error::Internal(err)) if err.to_string().contains("UNIQUE constraint failed")
        ));
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let mod_ = game.add_mod("test_mod", None).await.unwrap();

        let profile = game.add_profile("Test").await.unwrap();
        let mod_entry = profile.add_mod_entry(mod_).await.unwrap();

        assert_eq!(game.profiles().await.unwrap().len(), 1);

        let dir = profile.dir().await.unwrap();

        profile.remove().await.unwrap();

        assert!(matches!(
            mod_entry.remove().await,
            Err(Error::RemovedEntity)
        ));
        assert!(!dir.exists());
    }

    #[tokio::test]
    async fn test_parent() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        game.add_profile("Cool Profile").await.unwrap();
        assert_eq!(repo.games().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_name() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();
        profile.name().await.unwrap();
    }

    #[tokio::test]
    async fn test_activate() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        let profile1 = game.add_profile("Test1").await.unwrap();
        let profile2 = game.add_profile("Test2").await.unwrap();

        assert!(profile1.is_active().await.unwrap());
        profile2.activate().await.unwrap();
        assert!(profile2.is_active().await.unwrap());
    }

    #[tokio::test]
    async fn test_remove_active() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();

        let profile1 = game.add_profile("Test1").await.unwrap();
        let profile2 = game.add_profile("Test2").await.unwrap();

        profile1.activate().await.unwrap();
        assert!(profile1.is_active().await.unwrap());

        profile1.remove().await.unwrap();
        assert!(profile2.is_active().await.unwrap());
    }
}
