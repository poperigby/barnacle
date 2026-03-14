use std::{
    fs,
    path::{Path, PathBuf},
};

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
        models::{DeployKind, games, mods},
    },
    entities::{Error, Mod, Profile, Result, map_duplicate_name},
};

#[derive(Debug, Clone)]
pub struct Game {
    pub(crate) id: i64,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Game {
    pub(crate) fn load(row_id: i64, db: Db, cfg: Cfg) -> Result<Self> {
        let model = db.run(games::Entity::find_by_id(row_id).one(db.conn()))?;
        let Some(model) = model else {
            return Err(Error::RemovedEntity);
        };
        Ok(Self {
            id: model.id,
            db,
            cfg,
        })
    }

    fn model(&self) -> Result<games::Model> {
        let model = self.db.run(games::Entity::find_by_id(self.id).one(self.db.conn()))?;
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

    pub fn targets(&self) -> Result<Vec<PathBuf>> {
        let model = self.model()?;
        serde_json::from_str(&model.targets_json)
            .map_err(|err| Error::Serialization(err.to_string()))
    }

    pub fn deploy_kind(&self) -> Result<DeployKind> {
        let model = self.model()?;
        model
            .deploy_kind
            .parse()
            .map_err(|err| Error::Serialization(format!("invalid deploy kind: {err}")))
    }

    pub fn set_deploy_kind(&self, new_deploy_kind: DeployKind) -> Result<()> {
        if new_deploy_kind == self.deploy_kind()? {
            return Ok(());
        }

        let mut active = self.model()?.into_active_model();
        active.deploy_kind = Set(new_deploy_kind.to_string());
        self.db.run(active.update(self.db.conn()))?;
        Ok(())
    }

    pub fn dir(&self) -> Result<PathBuf> {
        Ok(self
            .cfg
            .read()
            .library_dir()
            .join(self.name()?.to_snake_case()))
    }

    pub fn remove(self) -> Result<()> {
        for profile in self.profiles()? {
            let profile_name = profile.name().unwrap();
            profile
                .remove()
                .or_else(|err| match err {
                    Error::RemovedEntity => Ok(()),
                    other => Err(other),
                })
                .unwrap_or_else(|_| {
                    panic!("Failed to remove profile: {profile_name} during game cleanup")
                });
        }

        for mod_ in self.mods()? {
            let mod_name = mod_.name().unwrap();
            mod_
                .remove()
                .or_else(|err| match err {
                    Error::RemovedEntity => Ok(()),
                    other => Err(other),
                })
                .unwrap_or_else(|_| panic!("Failed to remove mod: {mod_name} during game cleanup"));
        }

        let name = self.name()?;
        let dir = self.dir()?;
        let was_active = self.is_active()?;
        self.db.run(async {
            let Some(model) = games::Entity::find_by_id(self.id).one(self.db.conn()).await? else {
                return Err(sea_orm::DbErr::Custom("missing game during delete".into()));
            };
            model.delete(self.db.conn()).await?;
            Ok(())
        })?;

        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }

        if was_active && let Some(first_game) = Game::list(self.db.clone(), self.cfg.clone())?.first() {
            first_game.activate()?;
        }

        info!("Removed game: {name}");
        Ok(())
    }

    pub fn add_profile(&self, name: &str) -> Result<Profile> {
        Profile::add(&self.db, &self.cfg, self, name)
    }

    pub fn profiles(&self) -> Result<Vec<Profile>> {
        Profile::list(&self.db, &self.cfg, self)
    }

    pub fn mods(&self) -> Result<Vec<Mod>> {
        let models = self.db.run(
            mods::Entity::find()
                .filter(mods::Column::GameId.eq(self.id))
                .order_by_asc(mods::Column::Id)
                .all(self.db.conn()),
        )?;

        models
            .into_iter()
            .map(|model| Mod::load(model.id, self.db.clone(), self.cfg.clone()))
            .collect()
    }

    pub fn add_mod(&self, name: &str, path: Option<&Path>) -> Result<Mod> {
        Mod::add(self.db.clone(), self.cfg.clone(), self, name, path)
    }

    pub(crate) fn add(db: &Db, cfg: Cfg, name: &str, deploy_kind: DeployKind) -> Result<Self> {
        let inserted = db.run(async {
            let model = games::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                name: Set(name.to_string()),
                targets_json: Set("[]".to_string()),
                deploy_kind: Set(deploy_kind.to_string()),
                is_active: Set(false),
            };
            model.insert(db.conn()).await
        })
        .map_err(map_duplicate_name)?;

        let game = Game::load(inserted.id, db.clone(), cfg.clone())?;
        fs::create_dir_all(game.dir().unwrap()).unwrap();

        if Game::active(db.clone(), cfg.clone())?.is_none()
            && let Some(first_game) = Game::list(db.clone(), cfg.clone())?.first()
        {
            first_game.activate()?;
        }

        info!("Created new game: {}", game.name()?);
        Ok(game)
    }

    pub(crate) fn list(db: Db, cfg: Cfg) -> Result<Vec<Game>> {
        let models = db.run(
            games::Entity::find()
                .order_by_asc(games::Column::Name)
                .all(db.conn()),
        )?;

        models
            .into_iter()
            .map(|model| Game::load(model.id, db.clone(), cfg.clone()))
            .collect()
    }

    pub(crate) fn search(db: Db, cfg: Cfg, name: &str) -> Result<Option<Game>> {
        let model = db.run(
            games::Entity::find()
                .filter(games::Column::Name.eq(name))
                .one(db.conn()),
        )?;

        model
            .map(|model| Game::load(model.id, db.clone(), cfg.clone()))
            .transpose()
    }

    pub fn activate(&self) -> Result<()> {
        self.db.run(async {
            games::Entity::update_many()
                .col_expr(games::Column::IsActive, sea_orm::sea_query::Expr::value(false))
                .exec(self.db.conn())
                .await?;

            let Some(model) = games::Entity::find_by_id(self.id).one(self.db.conn()).await? else {
                return Err(sea_orm::DbErr::Custom("missing game during activation".into()));
            };
            let mut active = model.into_active_model();
            active.is_active = Set(true);
            active.update(self.db.conn()).await?;
            Ok(())
        })?;

        Ok(())
    }

    pub fn is_active(&self) -> Result<bool> {
        Ok(Game::active(self.db.clone(), self.cfg.clone())? == Some(self.clone()))
    }

    pub(crate) fn active(db: Db, cfg: Cfg) -> Result<Option<Game>> {
        let model = db.run(
            games::Entity::find()
                .filter(games::Column::IsActive.eq(true))
                .order_by_asc(games::Column::Id)
                .one(db.conn()),
        )?;

        model
            .map(|model| Game::load(model.id, db.clone(), cfg.clone()))
            .transpose()
    }

    pub fn active_profile(&self) -> Result<Option<Profile>> {
        Profile::active(self.db.clone(), self.cfg.clone(), self.clone())
    }

    pub fn search_profile(&self, name: &str) -> Result<Option<Profile>> {
        Profile::search(self.db.clone(), self.cfg.clone(), self, name)
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

    #[test]
    fn test_add() {
        let repo = Repository::mock();

        let game1 = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();

        let games = repo.games().unwrap();

        assert!(game1.dir().unwrap().exists());
        assert_eq!(games.len(), 2);
        assert_eq!(games.first().unwrap().name().unwrap(), "Morrowind");
        assert_eq!(
            games.last().unwrap().deploy_kind().unwrap(),
            DeployKind::CreationEngine
        );
    }

    #[test]
    fn test_add_duplicate() {
        let repo = Repository::mock();

        let _game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();

        assert!(matches!(
            repo.add_game("Morrowind", DeployKind::OpenMW),
            Err(Error::DuplicateName)
        ));
    }

    #[test]
    fn test_remove() {
        let repo = Repository::mock();

        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        let profile = game.add_profile("test_profile_1").unwrap();
        let mod_ = game.add_mod("test_mod", None).unwrap();

        assert_eq!(repo.games().unwrap().len(), 1);

        let dir = game.dir().unwrap();

        game.remove().unwrap();

        assert!(matches!(profile.remove(), Err(Error::RemovedEntity)));
        assert!(matches!(mod_.remove(), Err(Error::RemovedEntity)));

        assert!(!dir.exists());
        assert_eq!(repo.games().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_made_next_game_active() {
        let repo = Repository::mock();
        let game1 = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        let game2 = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();

        game1.activate().unwrap();
        assert!(game1.is_active().unwrap());

        game1.remove().unwrap();
        assert!(game2.is_active().unwrap());
    }

    #[test]
    fn test_list() {
        let repo = Repository::mock();

        assert_eq!(repo.games().unwrap().len(), 0);
        repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();
        assert_eq!(repo.games().unwrap().len(), 1);
    }

    #[test]
    fn test_name() {
        let repo = Repository::mock();

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .unwrap();

        game.name().unwrap();
    }

    #[test]
    fn test_set_name() {
        let repo = Repository::mock();
        let game = repo.add_game("Skyrim", DeployKind::CreationEngine).unwrap();

        assert_eq!(game.name().unwrap(), "Skyrim");

        game.set_name("Skyrim 3: Electric Boogaloo").unwrap();

        assert_eq!(game.name().unwrap(), "Skyrim 3: Electric Boogaloo");
    }

    #[test]
    fn test_deploy_kind() {
        let repo = Repository::mock();

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .unwrap();

        game.deploy_kind().unwrap();
    }

    #[test]
    fn test_dir() {
        let repo = Repository::mock();

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .unwrap();

        let expected_dir = repo
            .cfg
            .read()
            .library_dir()
            .join(game.name().unwrap().to_snake_case());

        assert_eq!(game.dir().unwrap(), expected_dir);
    }

    #[test]
    fn test_activate() {
        let repo = Repository::mock();

        let game = repo.add_game("Morrowind", DeployKind::OpenMW).unwrap();
        game.activate().unwrap();

        assert!(game.is_active().unwrap());
        assert_eq!(repo.active_game().unwrap().unwrap(), game);
    }
}
