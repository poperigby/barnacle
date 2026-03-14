use std::{fs, path::PathBuf};

use heck::ToSnakeCase;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, QueryOrder,
};
use tracing::info;

use crate::repository::{
    Cfg,
    db::{
        Db,
        models::profiles,
    },
    entities::{Error, Game, Mod, ModEntry, Result, map_duplicate_name},
};

#[derive(Debug, Clone)]
pub struct Profile {
    pub(crate) id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Profile {
    pub(crate) fn load(row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let model = db.run(profiles::Entity::find_by_id(row_id).one(db.conn()))?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self {
            id: model.id,
            db,
            cfg,
        })
    }

    fn model(&self) -> Result<profiles::Model> {
        let model = self.db.run(profiles::Entity::find_by_id(self.id).one(self.db.conn()))?;
        model.ok_or(Error::RemovedEntity)
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.model()?.name)
    }

    pub fn set_name(&self, new_name: &str) -> Result<()> {
        if new_name == self.name()? {
            return Ok(());
        }

        let old_dir = self.dir()?;
        let mut active = self.model()?.into_active_model();
        active.name = Set(new_name.to_string());
        self.db
            .run(active.update(self.db.conn()))
            .map_err(map_duplicate_name)?;
        let new_dir = self.dir()?;
        fs::rename(old_dir, new_dir).unwrap();
        Ok(())
    }

    pub fn dir(&self) -> Result<PathBuf> {
        Ok(self
            .parent()?
            .dir()?
            .join("profiles")
            .join(self.name()?.to_snake_case()))
    }

    pub fn activate(&self) -> Result<()> {
        let game_id = self.parent()?.id;

        self.db.run(async {
            profiles::Entity::update_many()
                .filter(profiles::Column::GameId.eq(game_id))
                .col_expr(
                    profiles::Column::IsActive,
                    sea_orm::sea_query::Expr::value(false),
                )
                .exec(self.db.conn())
                .await?;

            let Some(model) = profiles::Entity::find_by_id(self.id).one(self.db.conn()).await? else {
                return Err(sea_orm::DbErr::Custom("missing profile during activation".into()));
            };
            let mut active = model.into_active_model();
            active.is_active = Set(true);
            active.update(self.db.conn()).await?;
            Ok(())
        })?;

        Ok(())
    }

    pub fn is_active(&self) -> Result<bool> {
        Ok(Profile::active(self.db.clone(), self.cfg.clone(), self.parent()?)? == Some(self.clone()))
    }

    pub(crate) fn active(db: Db, cfg: Cfg, game: Game) -> Result<Option<Profile>> {
        let model = db.run(
            profiles::Entity::find()
                .filter(profiles::Column::GameId.eq(game.id))
                .filter(profiles::Column::IsActive.eq(true))
                .order_by_asc(profiles::Column::Id)
                .one(db.conn()),
        )?;

        model
            .map(|model| Profile::load(model.id, db.clone(), cfg.clone()))
            .transpose()
    }

    pub fn parent(&self) -> Result<Game> {
        let game_id = self.model()?.game_id;
        Game::load(game_id, self.db.clone(), self.cfg.clone())
    }

    pub fn add_mod_entry(&self, mod_: Mod) -> Result<ModEntry> {
        ModEntry::add(&self.db, &self.cfg, self, mod_)
    }

    pub fn mod_entries(&self) -> Result<Vec<ModEntry>> {
        ModEntry::list(&self.db, &self.cfg, self)
    }

    pub fn remove(self) -> Result<()> {
        for entry in self.mod_entries()? {
            let entry_id = entry.entry_id;
            entry
                .remove()
                .or_else(|err| match err {
                    Error::RemovedEntity => Ok(()),
                    other => Err(other),
                })
                .unwrap_or_else(|err| {
                    panic!("Failed to remove mod entry: {entry_id:?}: {err} during profile cleanup")
                });
        }

        let parent_game = self.parent()?;
        let name = self.name()?;
        let dir = self.dir()?;
        let was_active = self.is_active()?;
        self.db.run(async {
            let Some(model) = profiles::Entity::find_by_id(self.id).one(self.db.conn()).await? else {
                return Err(sea_orm::DbErr::Custom("missing profile during delete".into()));
            };
            model.delete(self.db.conn()).await?;
            Ok(())
        })?;

        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }

        if was_active
            && let Some(first_profile) = Profile::list(&self.db, &self.cfg, &parent_game)?.first()
        {
            first_profile.activate()?;
        }

        info!("Removed profile: {name}");
        Ok(())
    }

    pub(crate) fn add(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Result<Self> {
        let inserted = db.run(async {
            let model = profiles::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                game_id: Set(game.id),
                name: Set(name.to_string()),
                is_active: Set(false),
            };
            model.insert(db.conn()).await
        })
        .map_err(map_duplicate_name)?;

        let profile = Profile::load(inserted.id, db.clone(), cfg.clone())?;
        fs::create_dir_all(profile.dir()?).unwrap();

        if Profile::active(db.clone(), cfg.clone(), game.clone())?.is_none()
            && let Some(first_profile) = Profile::list(db, cfg, game)?.first()
        {
            first_profile.activate()?;
            return Ok(first_profile.clone());
        }

        Ok(profile)
    }

    pub(crate) fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>> {
        let models = db.run(
            profiles::Entity::find()
                .filter(profiles::Column::GameId.eq(game.id))
                .order_by_asc(profiles::Column::Id)
                .all(db.conn()),
        )?;

        models
            .into_iter()
            .map(|model| Profile::load(model.id, db.clone(), cfg.clone()))
            .collect()
    }

    pub(crate) fn search(db: Db, cfg: Cfg, game: &Game, name: &str) -> Result<Option<Profile>> {
        let model = db.run(
            profiles::Entity::find()
                .filter(profiles::Column::GameId.eq(game.id))
                .filter(profiles::Column::Name.eq(name))
                .one(db.conn()),
        )?;

        model
            .map(|model| Profile::load(model.id, db.clone(), cfg.clone()))
            .transpose()
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

    #[test]
    fn test_add() {
        let repo = Repository::mock();
        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        let profile = game.add_profile("Test").unwrap();
        assert!(profile.dir().unwrap().exists());
    }

    #[test]
    fn test_add_duplicate() {
        let repo = Repository::mock();
        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        game.add_profile("Test").unwrap();

        assert!(matches!(game.add_profile("Test"), Err(Error::DuplicateName)));
    }

    #[test]
    fn test_remove() {
        let repo = Repository::mock();
        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        let mod_ = game.add_mod("test_mod", None).unwrap();

        let profile = game.add_profile("Test").unwrap();
        let mod_entry = profile.add_mod_entry(mod_).unwrap();

        assert_eq!(game.profiles().unwrap().len(), 1);

        let dir = profile.dir().unwrap();

        profile.remove().unwrap();

        assert!(matches!(mod_entry.remove(), Err(Error::RemovedEntity)));
        assert!(!dir.exists());
        assert_eq!(game.profiles().unwrap().len(), 0);
    }

    #[test]
    fn test_list() {
        let repo = Repository::mock();
        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();

        assert_eq!(game.profiles().unwrap().len(), 0);
        game.add_profile("Cool Profile").unwrap();
        assert_eq!(repo.games().unwrap().len(), 1);
    }

    #[test]
    fn test_parent() {
        let repo = Repository::mock();
        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        let profile = game.add_profile("Test").unwrap();
        assert_eq!(profile.parent().unwrap(), game);
    }

    #[test]
    fn test_activate() {
        let repo = Repository::mock();
        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();

        let profile1 = game.add_profile("Test1").unwrap();
        let profile2 = game.add_profile("Test2").unwrap();

        assert!(profile1.is_active().unwrap());

        profile2.activate().unwrap();

        assert!(profile2.is_active().unwrap());
    }

    #[test]
    fn test_remove_made_next_profile_active() {
        let repo = Repository::mock();
        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();

        let profile1 = game.add_profile("Test1").unwrap();
        let profile2 = game.add_profile("Test2").unwrap();

        profile1.activate().unwrap();
        assert!(profile1.is_active().unwrap());

        profile1.remove().unwrap();
        assert!(profile2.is_active().unwrap());
    }
}
