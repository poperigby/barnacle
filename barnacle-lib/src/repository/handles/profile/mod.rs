use std::{fmt::Debug, fs, path::PathBuf};

mod error;

use heck::ToSnakeCase;
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
    state,
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
    pub(crate) fn from_id(id: i32, db: Db, cfg: Cfg) -> Self {
        Self { id, db, cfg }
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
        let old_dir = self.dir().await.map_err(SetNameError::CurrentDir)?;

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

        let new_dir = self.dir().await.map_err(SetNameError::CurrentDir)?;
        fs::rename(&old_dir, &new_dir).map_err(|source| SetNameError::RenameDir {
            from: old_dir,
            to: new_dir,
            source,
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
            .join(self.name().await.map_err(DirError::Name)?.to_snake_case()))
    }

    /// Make this profile the active one
    pub async fn activate(&self) -> Result<(), ActivateError> {
        let conn = self.db.conn();

        let active_game_id = state::active_game_id(conn)
            .await
            .map_err(ActivateError::ActiveGameId)?;
        let profile_game_id = self.model(conn).await.map_err(ActivateError::Load)?.game_id;

        if active_game_id == Some(profile_game_id) {
            state::set_active_profile_id(conn, Some(self.id))
                .await
                .map_err(ActivateError::SetActiveProfile)
        } else {
            Err(ActivateError::ProfileNotInActiveGame)
        }
    }

    pub async fn is_active(&self) -> Result<bool, IsActiveError> {
        Ok(state::active_profile_id(self.db.conn())
            .await
            .map_err(IsActiveError::ActiveProfileId)?
            == Some(self.id))
    }

    pub(crate) async fn active(db: Db, cfg: Cfg) -> Result<Option<Profile>, ActiveError> {
        state::reconcile(db.conn())
            .await
            .map_err(ActiveError::Reconcile)?;
        Ok(state::active_profile_id(db.conn())
            .await
            .map_err(ActiveError::ActiveProfileId)?
            .map(|id| Profile::from_id(id, db.clone(), cfg.clone())))
    }

    /// Returns the parent [`Game`] of this [`Profile`]
    pub async fn parent(&self) -> Result<Game, ParentError> {
        let parent_game_id = self
            .model(self.db.conn())
            .await
            .map_err(ParentError::Load)?
            .game_id;
        Ok(Game::from_id(
            parent_game_id,
            self.db.clone(),
            self.cfg.clone(),
        ))
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

        state::reconcile(self.db.conn())
            .await
            .map_err(RemoveError::Reconcile)?;

        info!("Removed profile: {name}");

        Ok(())
    }

    pub(crate) async fn add(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Result<Self, AddError> {
        let model = ActiveModel {
            name: Set(name.to_string()),
            game_id: Set(game.id),
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

        let profile = Profile::from_id(id, db.clone(), cfg.clone());
        let dir = profile.dir().await.map_err(AddError::Dir)?;
        fs::create_dir_all(&dir).map_err(|source| AddError::CreateDir { path: dir, source })?;

        state::reconcile(db.conn())
            .await
            .map_err(AddError::Reconcile)?;

        info!("Added profile: {name}");

        Ok(profile)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>, ListError> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id))
            .order_by_id_desc()
            .all(db.conn())
            .await
            .map_err(ListError)?
            .iter()
            .map(|model| Profile::from_id(model.id, db.clone(), cfg.clone()))
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
            Entity::find_by_profile_name_per_game((name.to_string(), game.id))
                .one(db.conn())
                .await
                .map_err(|source| SearchError {
                    name: name.to_string(),
                    source,
                })?
                .map(|model| Profile::from_id(model.id, db.clone(), cfg.clone())),
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
    use crate::{
        Repository, game::AddProfileError, profile::AddError as ProfileAddError,
        repository::DeployKind,
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
            Err(AddProfileError(ProfileAddError::DuplicateName { .. }))
        ))
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::mock().await;
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

        assert!(matches!(mod_entry.remove().await, Err(_)));
        assert!(!dir.exists());
        assert_eq!(game.profiles().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_list() {
        let repo = Repository::mock().await;
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
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let profile = game.add_profile("Test").await.unwrap();

        assert_eq!(profile.parent().await.unwrap(), game);
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

        // First profile should have been automatically set as active
        assert!(profile1.is_active().await.unwrap());

        profile2.activate().await.unwrap();

        assert!(profile2.is_active().await.unwrap());
    }

    #[tokio::test]
    async fn test_remove_made_next_profile_active() {
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
