use std::{fmt::Debug, fs, path::PathBuf};

mod error;

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, QueryFilter};
use tracing::info;

pub use error::*;

use crate::repository::{
    Cfg, Game, Mod, ModEntry,
    db::{
        Db,
        models::profiles::{ActiveModel, COLUMN, Entity, Model},
    },
    handles::{
        error::{GetFieldError, LoadModelError, ModelKind, is_unique_violation},
        mod_entry,
    },
};

/// Represents a profile entity in the Barnacle system.
///
/// Provides methods to inspect and modify this profile's data, including
/// managing mod entries. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Profile {
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Profile {
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
            .map_err(|source| LoadModelError::query(ModelKind::Profile, self.id, source))?
            .ok_or_else(|| LoadModelError::stale(ModelKind::Profile, self.id))
    }

    async fn active_model(
        &self,
        conn: &impl ConnectionTrait,
    ) -> Result<ActiveModel, LoadModelError> {
        Ok(self.model(conn).await?.into())
    }

    // Fields

    pub async fn name(&self) -> Result<String, GetFieldError> {
        Ok(self
            .model(self.db.conn())
            .await
            .map_err(|source| self.field_error("name", source))?
            .name)
    }

    pub async fn set_name(&self, new_name: &str) -> Result<(), SetNameError> {
        let mut active_model = self
            .active_model(self.db.conn())
            .await
            .map_err(SetNameError::Load)?;

        active_model.name.set_if_not_equals(new_name.to_string());
        active_model
            .update(self.db.conn())
            .await
            .map_err(|source| {
                if is_unique_violation(&source) {
                    SetNameError::Duplicate {
                        name: new_name.to_string(),
                    }
                } else {
                    SetNameError::Update(source)
                }
            })?;

        Ok(())
    }

    pub async fn dir(&self) -> Result<PathBuf, DirError> {
        Ok(self
            .parent()
            .await
            .map_err(DirError::Parent)?
            .dir()
            .await
            .map_err(DirError::ParentDir)?
            .join("profiles")
            .join(self.id.to_string()))
    }

    /// Activate this profile
    pub async fn activate(&self) -> Result<(), ActivateError> {
        let parent_game = self.parent().await.map_err(ActivateError::Parent)?;

        parent_game
            .set_active_profile_id(Some(self.id))
            .await
            .map_err(ActivateError::SetActiveProfile)?;

        Ok(())
    }

    pub async fn is_active(&self) -> Result<bool, IsActiveError> {
        let parent_game = self.parent().await.map_err(IsActiveError::Parent)?;

        let active_profile = Self::active(&self.db, &self.cfg, &parent_game)
            .await
            .map_err(IsActiveError::Active)?;

        Ok(active_profile.as_ref() == Some(self))
    }

    pub(crate) async fn active(
        db: &Db,
        cfg: &Cfg,
        game: &Game,
    ) -> Result<Option<Profile>, ActiveError> {
        Ok(Self::resolve_active_id(db, game)
            .await
            .map_err(ActiveError::Resolve)?
            .map(|id| Profile::from_id(id, db, cfg)))
    }

    // Returns the active profile ID, selecting a fallback if none is set.
    async fn resolve_active_id(db: &Db, game: &Game) -> Result<Option<i32>, ResolveActiveIdError> {
        if let Some(id) = game
            .active_profile_id()
            .await
            .map_err(ResolveActiveIdError::ActiveProfileId)?
        {
            // We already have an active profile
            return Ok(Some(id));
        }

        let conn = db.conn();

        // Make the oldest profile the fallback
        let fallback_id = Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .order_by_id_asc()
            .one(conn)
            .await
            .map_err(ResolveActiveIdError::FindFallbackProfile)?
            .map(|game| game.id);

        game.set_active_profile_id(fallback_id)
            .await
            .map_err(ResolveActiveIdError::SetActiveProfile)?;

        Ok(fallback_id)
    }

    /// Returns the parent [`Game`] of this [`Profile`]
    pub async fn parent(&self) -> Result<Game, ParentError> {
        let parent_game_id = self
            .model(self.db.conn())
            .await
            .map_err(ParentError::Load)?
            .game_id;

        Ok(Game::from_id(parent_game_id, &self.db, &self.cfg))
    }

    // Operations

    pub async fn remove(self) -> Result<(), RemoveError> {
        // We have to store these so we can still access them once the profile is deleted
        let name = self.name().await.map_err(RemoveError::Name)?;
        let dir = self.dir().await.map_err(RemoveError::Dir)?;

        Entity::delete_by_id(self.id)
            .exec(self.db.conn())
            .await
            .map_err(RemoveError::Delete)?;

        fs::remove_dir_all(&dir).map_err(|source| RemoveError::RemoveDir { path: dir, source })?;

        info!("Removed profile: {name}");

        Ok(())
    }

    pub(crate) async fn add(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Result<Self, AddError> {
        let model = ActiveModel {
            name: Set(name.to_string()),
            game_id: Set(game.id()),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(db.conn())
            .await
            .map_err(|source| {
                if is_unique_violation(&source) {
                    AddError::DuplicateName {
                        name: name.to_string(),
                    }
                } else {
                    AddError::Insert(source)
                }
            })?
            .last_insert_id;

        let profile = Profile::from_id(id, db, cfg);
        let dir = profile.dir().await.map_err(AddError::Dir)?;
        fs::create_dir_all(&dir).map_err(|source| AddError::CreateDir { path: dir, source })?;

        info!("Added profile: {name}");

        Ok(profile)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>, ListError> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id()))
            .order_by_id_desc()
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Profile::from_id(model.id, db, cfg))
            .collect())
    }

    /// Search for a profile under the given game by name
    pub(crate) async fn search(
        db: Db,
        cfg: Cfg,
        game: &Game,
        name: &str,
    ) -> Result<Option<Profile>, SearchError> {
        Ok(
            Entity::find_by_profile_name_per_game((name.to_string(), game.id()))
                .one(db.conn())
                .await
                .map_err(|source| SearchError {
                    name: name.to_string(),
                    source,
                })?
                .map(|model| Profile::from_id(model.id, &db, &cfg)),
        )
    }

    // Children

    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub async fn add_mod_entry(&self, mod_: Mod) -> Result<ModEntry, mod_entry::AddError> {
        ModEntry::add(&self.db, &self.cfg, self, mod_).await
    }

    pub async fn mod_entries(&self) -> Result<Vec<ModEntry>, mod_entry::ListError> {
        ModEntry::list(&self.db, &self.cfg, self).await
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod test {
    use crate::{Repository, profile, repository::DeployKind};

    #[tokio::test]
    async fn test_add() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        assert!(profile.dir().await.unwrap().exists());
    }

    #[tokio::test]
    async fn test_add_duplicate() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        game.add_profile("Test").await.unwrap();

        assert!(matches!(
            game.add_profile("Test").await,
            Err(profile::AddError::DuplicateName { .. })
        ))
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::in_memory().await;
        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let _mod = game.add_mod("test_mod", None).await.unwrap();

        let profile = game.add_profile("Test").await.unwrap();
        let mod_entry = profile.add_mod_entry(_mod).await.unwrap();

        assert_eq!(game.profiles().await.unwrap().len(), 1);

        let dir = profile.dir().await.unwrap();

        profile.remove().await.unwrap();

        // Check the child mod entries were also removed
        assert!(mod_entry.remove().await.is_err());

        assert!(!dir.exists());
        assert_eq!(game.profiles().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_list() {
        let repo = Repository::in_memory().await;
        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();

        assert_eq!(game.profiles().await.unwrap().len(), 0);

        game.add_profile("Cool Profile").await.unwrap();

        assert_eq!(repo.games().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_parent() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        assert_eq!(profile.parent().await.unwrap(), game);
    }

    #[tokio::test]
    async fn test_activate() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();

        let profile1 = game.add_profile("Test1").await.unwrap();
        let profile2 = game.add_profile("Test2").await.unwrap();

        // First profile should have been automatically set as active
        assert!(profile1.is_active().await.unwrap());

        profile2.activate().await.unwrap();

        assert!(profile2.is_active().await.unwrap());
    }

    #[tokio::test]
    async fn test_dir() {
        let repo = Repository::in_memory().await;

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        let profile = game.add_profile("Test").await.unwrap();

        let expected_dir = repo
            .cfg
            .read()
            .library_dir()
            .join(game.id().to_string())
            .join("profiles")
            .join(profile.id.to_string());

        assert_eq!(profile.dir().await.unwrap(), expected_dir);
    }

    #[tokio::test]
    async fn test_remove_made_next_profile_active() {
        let repo = Repository::in_memory().await;
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

    #[tokio::test]
    async fn test_switching_games_preserves_each_games_active_profile() {
        let repo = Repository::in_memory().await;

        let game1 = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        game1.activate().await.unwrap();

        let profile1 = game1.add_profile("Test1").await.unwrap();
        profile1.activate().await.unwrap();
        game1.add_profile("Test2").await.unwrap();

        let game2 = repo
            .add_game("Morrowind", DeployKind::OpenMW)
            .await
            .unwrap();
        let profile2 = game2.add_profile("Test2").await.unwrap();

        game2.activate().await.unwrap();

        assert!(profile2.is_active().await.unwrap());

        game1.activate().await.unwrap();

        assert!(profile1.is_active().await.unwrap());
    }
}
