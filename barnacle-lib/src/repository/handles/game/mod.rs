use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

mod error;

use heck::ToSnakeCase;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, TransactionTrait};
use tracing::info;

pub use error::*;

use crate::{
    DeployKind::OpenMW,
    deployers, mod_, profile,
    repository::{
        Cfg, DeployKind,
        db::{
            Db,
            models::games::{ActiveModel, Entity, Model},
        },
        handles::{
            Target,
            error::{
                GetFieldError, LoadModelError, ModelKind, is_unique_violation,
                map_transaction_error,
            },
            mod_::Mod,
            profile::Profile,
            target,
        },
        state,
    },
};

/// Represents a game entity in the Barnacle system.
///
/// Provides methods to inspect and modify this game's data, including
/// managing profiles and mods. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Game {
    // TODO: Do these need to be public now?
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Game {
    /// Load some existing [`Game`] from the database
    pub(crate) fn from_id(id: i32, db: &Db, cfg: &Cfg) -> Self {
        Self {
            id,
            db: db.clone(),
            cfg: cfg.clone(),
        }
    }

    async fn model(&self, conn: &impl ConnectionTrait) -> Result<Model, LoadModelError> {
        Entity::find_by_id(self.id)
            .one(conn)
            .await
            .map_err(|source| LoadModelError::query(ModelKind::Game, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Game, self.id))
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }

    pub async fn deploy(&self) {
        match self.deploy_kind().await.unwrap() {
            DeployKind::OpenMW => {
                let mut deployer = deployers::openmw::OpenMw::load();
                deployer.deploy(self).await;
            }
            _ => println!("I DON'T UNDERSTAND THIS DEPLOY TYPE :("),
        };
    }

    pub async fn name(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("name", source))?
            .name)
    }

    pub async fn set_name(&self, new_name: &str) -> Result<(), SetNameError> {
        let game = self.clone();

        let new_name = new_name.to_string();

        let old_dir = self.dir().await.map_err(SetNameError::CurrentDir)?;
        let new_dir = Self::dir_from_name(&self.cfg, &new_name);

        // TODO: We need to roll back the filesystem if the database commit fails
        self.db
            .conn()
            .transaction(|txn| {
                Box::pin(async move {
                    let mut active_model =
                        game.active_model(txn).await.map_err(SetNameError::Load)?;

                    active_model.name.set_if_not_equals(new_name.to_string());

                    active_model.update(txn).await.map_err(|source| {
                        if is_unique_violation(&source) {
                            SetNameError::Duplicate { name: new_name }
                        } else {
                            SetNameError::Update(source)
                        }
                    })?;

                    fs::rename(&old_dir, &new_dir).map_err(|source| SetNameError::RenameDir {
                        from: old_dir,
                        to: new_dir,
                        source,
                    })?;

                    Ok(())
                })
            })
            .await
            .map_err(|source| map_transaction_error(source, SetNameError::Transaction))
    }

    // pub fn targets(&self) -> Result<Vec<PathBuf>> {
    //     self.get_field("targets")
    // }

    pub async fn deploy_kind(&self) -> Result<DeployKind, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("deploy_kind", source))?
            .deploy_kind)
    }

    pub async fn dir(&self) -> Result<PathBuf, DirError> {
        let name = self.name().await.map_err(|source| DirError { source })?;

        Ok(Self::dir_from_name(&self.cfg, &name))
    }

    fn dir_from_name(cfg: &Cfg, name: &str) -> PathBuf {
        let library_dir = cfg.read().library_dir().to_path_buf();
        library_dir.join(name.to_snake_case())
    }

    pub async fn remove(self) -> Result<(), RemoveError> {
        // We have to store these so we can still access them once the game is deleted
        let name = self.name().await.map_err(RemoveError::Name)?;
        let dir = self.dir().await.map_err(RemoveError::Dir)?;

        // We want to roll back the database mutations if the directory removal fails
        self.db
            .conn()
            .transaction(|txn| {
                Box::pin(async move {
                    Entity::delete_by_id(self.id)
                        .exec(txn)
                        .await
                        .map_err(RemoveError::Delete)?;

                    fs::remove_dir_all(dir).map_err(|source| RemoveError::RemoveDir { source })?;

                    Ok::<(), RemoveError>(())
                })
            })
            .await
            .map_err(|source| map_transaction_error(source, RemoveError::Transaction))?;

        info!("Removed game: {name}");

        Ok(())
    }

    /// Insert a new [`Game`] into the database. The [`Game`] must have a unique name.
    pub(crate) async fn add(
        db: &Db,
        cfg: &Cfg,
        name: &str,
        deploy_kind: DeployKind,
    ) -> Result<Self, AddError> {
        let name = name.to_string();
        let dir = Self::dir_from_name(cfg, &name);

        let model = ActiveModel {
            name: Set(name.clone()),
            deploy_kind: Set(deploy_kind),
            ..Default::default()
        };

        // We want to roll back the database mutations if the directory creation fails
        let game = db
            .conn()
            .transaction(|txn| {
                let name = name.clone();
                let db = db.clone();
                let cfg = cfg.clone();

                Box::pin(async move {
                    let id = Entity::insert(model)
                        .exec(txn)
                        .await
                        .map_err(|source| {
                            if is_unique_violation(&source) {
                                AddError::DuplicateName { name }
                            } else {
                                AddError::Insert(source)
                            }
                        })?
                        .last_insert_id;

                    fs::create_dir_all(&dir)
                        .map_err(|source| AddError::CreateDir { path: dir, source })?;

                    Ok(Game::from_id(id, &db, &cfg))
                })
            })
            .await
            .map_err(|source| map_transaction_error(source, AddError::Transaction))?;

        info!("Created new game: {name}");

        Ok(game)
    }

    pub(crate) async fn list(db: Db, cfg: Cfg) -> Result<Vec<Game>, ListError> {
        Ok(Entity::find()
            .order_by_id_desc()
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Game::from_id(model.id, &db, &cfg))
            .collect())
    }

    /// Search for a game by name
    // TODO: This is a bad name because you're just directly finding a game by its exact name. Not
    // exactly searching. We should have actual searching.
    pub(crate) async fn search(db: Db, cfg: Cfg, name: &str) -> Result<Option<Game>, SearchError> {
        Ok(Entity::find_by_name(name)
            .one(db.conn())
            .await
            .map_err(|source| SearchError {
                name: name.to_string(),
                source,
            })?
            .map(|model| Game::from_id(model.id, &db, &cfg)))
    }

    /// Make this game the active one
    pub async fn activate(&self) -> Result<(), ActivateError> {
        state::set_active_game_id(self.db.conn(), Some(self.id))
            .await
            .map_err(ActivateError::SetActiveGameId)
    }

    pub async fn is_active(&self) -> Result<bool, IsActiveError> {
        let active_game = Self::active(&self.db, &self.cfg)
            .await
            .map_err(IsActiveError::Active)?;

        Ok(active_game.as_ref() == Some(self))
    }

    pub(crate) async fn active(db: &Db, cfg: &Cfg) -> Result<Option<Game>, ActiveError> {
        Ok(Self::resolve_active_id(db.conn())
            .await
            .map_err(ActiveError::Resolve)?
            .map(|id| Game::from_id(id, db, cfg)))
    }

    // Returns the active game ID, selecting a fallback if none is set.
    async fn resolve_active_id(
        conn: &impl ConnectionTrait,
    ) -> Result<Option<i32>, ResolveActiveIdError> {
        if let Some(id) = state::active_game_id(conn)
            .await
            .map_err(ResolveActiveIdError::ActiveGameId)?
        {
            // We already have an active game set
            return Ok(Some(id));
        }

        // Pick the oldest game as the fallback
        let fallback_id = Entity::find()
            .order_by_id_asc()
            .one(conn)
            .await
            .map_err(ResolveActiveIdError::FindFallbackGame)?
            .map(|game| game.id);

        state::set_active_game_id(conn, fallback_id)
            .await
            .map_err(ResolveActiveIdError::SetActiveGameId)?;

        Ok(fallback_id)
    }

    // Child operations

    pub(crate) async fn active_profile_id(&self) -> Result<Option<i32>, LoadModelError> {
        Ok(self.model(self.db.conn()).await?.active_profile_id)
    }

    pub(crate) async fn set_active_profile_id(
        &self,
        profile_id: Option<i32>,
    ) -> Result<(), SetActiveProfileIdError> {
        let conn = self.db.conn();

        let mut active_model = self
            .active_model(conn)
            .await
            .map_err(SetActiveProfileIdError::Load)?;

        active_model.active_profile_id.set_if_not_equals(profile_id);

        active_model
            .update(conn)
            .await
            .map_err(SetActiveProfileIdError::Update)?;

        Ok(())
    }

    pub async fn active_profile(&self) -> Result<Option<Profile>, profile::ActiveError> {
        Profile::active(&self.db, &self.cfg, self).await
    }

    pub async fn search_profile(
        &self,
        name: &str,
    ) -> Result<Option<Profile>, profile::SearchError> {
        Profile::search(self.db.clone(), self.cfg.clone(), self, name).await
    }

    pub async fn add_profile(&self, name: &str) -> Result<Profile, profile::AddError> {
        Profile::add(&self.db, &self.cfg, self, name).await
    }

    pub async fn profiles(&self) -> Result<Vec<Profile>, profile::ListError> {
        Profile::list(&self.db, &self.cfg, self).await
    }

    pub async fn add_target(&self, name: &str, path: &Path) -> Result<Target, target::AddError> {
        Target::add(&self.db, &self.cfg, self, name, path).await
    }

    pub async fn targets(&self) -> Result<Vec<Target>, target::ListError> {
        Target::list(&self.db, &self.cfg, self).await
    }

    pub async fn add_mod(&self, name: &str, path: Option<&Path>) -> Result<Mod, mod_::AddError> {
        Mod::add(self.db.clone(), self.cfg.clone(), self, name, path).await
    }

    pub async fn mods(&self) -> Result<Vec<Mod>, mod_::ListError> {
        Mod::list(&self.db.clone(), &self.cfg.clone(), self).await
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use crate::Repository;

    use super::*;

    #[tokio::test]
    async fn test_add() {
        let repo = Repository::in_memory().await;

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
        let repo = Repository::in_memory().await;

        let _game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        assert_matches!(
            repo.add_game("Morrowind", DeployKind::OpenMW).await,
            Err(AddError::DuplicateName { .. }),
        )
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let profile = game.add_profile("test_profile_1").await.unwrap();
        let mod_ = game.add_mod("test_mod", None).await.unwrap();

        assert_eq!(repo.games().await.unwrap().len(), 1);

        let dir = game.dir().await.unwrap();

        game.remove().await.unwrap();

        // Attempt to remove already removed profile and mod entries
        assert!(profile.remove().await.is_err());
        assert!(mod_.remove().await.is_err());

        assert!(!dir.exists());
        assert_eq!(repo.games().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_remove_made_next_game_active() {
        let repo = Repository::in_memory().await;
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
        let repo = Repository::in_memory().await;

        assert_eq!(repo.games().await.unwrap().len(), 0);

        repo.add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();

        assert_eq!(repo.games().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_name() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        game.name().await.unwrap();
    }

    #[tokio::test]
    async fn test_set_name() {
        let repo = Repository::in_memory().await;

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
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        game.deploy_kind().await.unwrap();
    }

    #[tokio::test]
    async fn test_dir() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        let expected_dir = repo
            .cfg
            .read()
            .library_dir()
            .join(game.name().await.unwrap().to_snake_case());

        assert_eq!(game.dir().await.unwrap(), expected_dir);
    }

    #[tokio::test]
    async fn test_activate() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        game.activate().await.unwrap();

        assert!(game.is_active().await.unwrap());
        assert_eq!(repo.active_game().await.unwrap().unwrap(), game);
    }
}
