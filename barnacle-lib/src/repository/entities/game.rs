use std::{
    fs,
    path::{Path, PathBuf},
};

use heck::ToSnakeCase;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter,
    QueryOrder,
};
use tracing::info;

use crate::repository::{
    Cfg,
    db::{
        Db,
        models::{DeployKind, games, mods},
    },
    entities::{Error, Mod, Profile, Result},
};

#[derive(Debug, Clone)]
pub struct Game {
    pub(crate) id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Game {
    pub(crate) async fn load(id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let model = games::Entity::find_by_id(id).one(db.conn()).await?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self {
            id: model.id,
            db,
            cfg,
        })
    }

    async fn model(&self) -> Result<games::Model> {
        let model = games::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?;
        model.ok_or(Error::RemovedEntity)
    }

    pub async fn name(&self) -> Result<String> {
        Ok(self.model().await?.name)
    }

    pub async fn set_name(&self, new_name: &str) -> Result<()> {
        // Prevents us from performing FS operations if we don't need to
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

    pub async fn targets(&self) -> Result<Vec<PathBuf>> {
        let model = self.model().await?;
        serde_json::from_str(&model.targets).map_err(|err| Error::Serialization(err.to_string()))
    }

    pub async fn deploy_kind(&self) -> Result<DeployKind> {
        let model = self.model().await?;
        model
            .deploy_kind
            .parse()
            .map_err(|err| Error::Serialization(format!("invalid deploy kind: {err}")))
    }

    pub async fn set_deploy_kind(&self, new_deploy_kind: DeployKind) -> Result<()> {
        let mut active = self.model().await?.into_active_model();
        active.deploy_kind = Set(new_deploy_kind.to_string());
        active.update(self.db.conn()).await?;
        Ok(())
    }

    pub async fn dir(&self) -> Result<PathBuf> {
        let name = self.name().await?;
        let library_dir = self.cfg.read().library_dir().to_path_buf();
        Ok(library_dir.join(name.to_snake_case()))
    }

    pub async fn remove(self) -> Result<()> {
        for profile in self.profiles().await? {
            let profile_name = profile.name().await.unwrap();
            profile
                .remove()
                .await
                .or_else(|err| match err {
                    Error::RemovedEntity => Ok(()),
                    other => Err(other),
                })
                .unwrap_or_else(|_| {
                    panic!("Failed to remove profile: {profile_name} during game cleanup")
                });
        }

        for mod_ in self.mods().await? {
            let mod_name = mod_.name().await.unwrap();
            mod_.remove()
                .await
                .or_else(|err| match err {
                    Error::RemovedEntity => Ok(()),
                    other => Err(other),
                })
                .unwrap_or_else(|_| panic!("Failed to remove mod: {mod_name} during game cleanup"));
        }

        let name = self.name().await?;
        let dir = self.dir().await?;
        let was_active = self.is_active().await?;
        let Some(model) = games::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
        else {
            return Err(Error::Internal(sea_orm::DbErr::Custom(
                "missing game during delete".into(),
            )));
        };
        model.delete(self.db.conn()).await?;

        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }

        if was_active
            && let Some(first_game) = Game::list(self.db.clone(), self.cfg.clone()).await?.first()
        {
            first_game.activate().await?;
        }

        info!("Removed game: {name}");
        Ok(())
    }

    pub async fn add_profile(&self, name: &str) -> Result<Profile> {
        Profile::add(&self.db, &self.cfg, self, name).await
    }

    pub async fn profiles(&self) -> Result<Vec<Profile>> {
        Profile::list(&self.db, &self.cfg, self).await
    }

    pub async fn mods(&self) -> Result<Vec<Mod>> {
        let models = mods::Entity::find()
            .filter(mods::COLUMN.game_id.eq(self.id))
            .order_by_asc(mods::COLUMN.id)
            .all(self.db.conn())
            .await?;

        let mut out = Vec::with_capacity(models.len());
        for model in models {
            out.push(Mod::load(model.id, self.db.clone(), self.cfg.clone()).await?);
        }
        Ok(out)
    }

    pub async fn add_mod(&self, name: &str, path: Option<&Path>) -> Result<Mod> {
        Mod::add(self.db.clone(), self.cfg.clone(), self, name, path).await
    }

    pub(crate) async fn add(
        db: &Db,
        cfg: Cfg,
        name: &str,
        deploy_kind: DeployKind,
    ) -> Result<Self> {
        let model = games::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            name: Set(name.to_string()),
            targets: Set("[]".to_string()),
            deploy_kind: Set(deploy_kind.to_string()),
            is_active: Set(false),
        };
        let inserted = model.insert(db.conn()).await.map_err(Error::from)?;

        let game = Game::load(inserted.id, db.clone(), cfg.clone()).await?;
        fs::create_dir_all(game.dir().await.unwrap()).unwrap();

        if Game::active(db.clone(), cfg.clone()).await?.is_none()
            && let Some(first_game) = Game::list(db.clone(), cfg.clone()).await?.first()
        {
            first_game.activate().await?;
        }

        info!("Created new game: {}", game.name().await?);
        Ok(game)
    }

    pub(crate) async fn list(db: Db, cfg: Cfg) -> Result<Vec<Game>> {
        let models = games::Entity::find()
            .order_by_asc(games::COLUMN.name)
            .all(db.conn())
            .await?;

        let mut out = Vec::with_capacity(models.len());
        for model in models {
            out.push(Game::load(model.id, db.clone(), cfg.clone()).await?);
        }
        Ok(out)
    }

    pub(crate) async fn search(db: Db, cfg: Cfg, name: &str) -> Result<Option<Game>> {
        let model = games::Entity::find()
            .filter(games::COLUMN.name.eq(name))
            .one(db.conn())
            .await?;

        if let Some(model) = model {
            Ok(Some(Game::load(model.id, db.clone(), cfg.clone()).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn activate(&self) -> Result<()> {
        games::Entity::update_many()
            .col_expr(
                games::COLUMN.is_active,
                sea_orm::sea_query::Expr::value(false),
            )
            .exec(self.db.conn())
            .await?;

        let Some(model) = games::Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
        else {
            return Err(Error::Internal(sea_orm::DbErr::Custom(
                "missing game during activation".into(),
            )));
        };
        let mut active = model.into_active_model();
        active.is_active = Set(true);
        active.update(self.db.conn()).await?;

        Ok(())
    }

    pub async fn is_active(&self) -> Result<bool> {
        Ok(Game::active(self.db.clone(), self.cfg.clone()).await? == Some(self.clone()))
    }

    pub(crate) async fn active(db: Db, cfg: Cfg) -> Result<Option<Game>> {
        let model = games::Entity::find()
            .filter(games::COLUMN.is_active.eq(true))
            .order_by_asc(games::COLUMN.id)
            .one(db.conn())
            .await?;

        if let Some(model) = model {
            Ok(Some(Game::load(model.id, db.clone(), cfg.clone()).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn active_profile(&self) -> Result<Option<Profile>> {
        Profile::active(self.db.clone(), self.cfg.clone(), self.clone()).await
    }

    pub async fn search_profile(&self, name: &str) -> Result<Option<Profile>> {
        Profile::search(self.db.clone(), self.cfg.clone(), self, name).await
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Repository;

    #[tokio::test]
    async fn test_add() {
        let repo = Repository::mock().await;

        let game1 = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        repo.add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        let games = repo.games().await.unwrap();

        assert!(game1.dir().await.unwrap().exists());
        assert_eq!(games.len(), 2);
        assert_eq!(games.first().unwrap().name().await.unwrap(), "Morrowind");
        assert_eq!(
            games.last().unwrap().deploy_kind().await.unwrap(),
            DeployKind::CreationEngine
        );
    }

    #[tokio::test]
    async fn test_add_duplicate() {
        let repo = Repository::mock().await;

        let _game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        assert!(matches!(
            repo.add_game("Morrowind", DeployKind::OpenMW).await,
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
        let profile = game.add_profile("test_profile_1").await.unwrap();
        let mod_ = game.add_mod("test_mod", None).await.unwrap();

        assert_eq!(repo.games().await.unwrap().len(), 1);

        let dir = game.dir().await.unwrap();

        game.remove().await.unwrap();

        assert!(matches!(profile.remove().await, Err(Error::RemovedEntity)));
        assert!(matches!(mod_.remove().await, Err(Error::RemovedEntity)));
        assert!(!dir.exists());

        assert_eq!(repo.games().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_remove_active() {
        let repo = Repository::mock().await;

        let game1 = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let game2 = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        game1.activate().await.unwrap();
        assert!(game1.is_active().await.unwrap());

        game1.remove().await.unwrap();
        assert!(game2.is_active().await.unwrap());
    }

    #[tokio::test]
    async fn test_list() {
        let repo = Repository::mock().await;

        assert_eq!(repo.games().await.unwrap().len(), 0);
        repo.add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        assert_eq!(repo.games().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_name() {
        let repo = Repository::mock().await;

        repo.add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap()
            .name()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_set_name() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();

        assert_eq!(game.name().await.unwrap(), "Skyrim");

        game.set_name("Skyrim 3: Electric Boogaloo").await.unwrap();

        assert_eq!(game.name().await.unwrap(), "Skyrim 3: Electric Boogaloo");
    }

    #[tokio::test]
    async fn test_deploy_kind() {
        let repo = Repository::mock().await;

        repo.add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap()
            .deploy_kind()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_dir() {
        let repo = Repository::mock().await;
        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        let expected_dir = repo
            .clone()
            .games()
            .await
            .unwrap()
            .first()
            .unwrap()
            .cfg
            .read()
            .library_dir()
            .join(game.name().await.unwrap().to_snake_case());

        assert_eq!(game.dir().await.unwrap(), expected_dir);
    }

    #[tokio::test]
    async fn test_activate() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        game.activate().await.unwrap();

        assert!(game.is_active().await.unwrap());
        assert_eq!(repo.active_game().await.unwrap().unwrap(), game);
    }
}
