use std::{fmt::Debug, fs, path::PathBuf};

use super::Error;
use heck::ToSnakeCase;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryFilter};
use tracing::info;

use crate::repository::{
    Cfg, Game, Mod, ModEntry,
    db::{
        Db,
        models::profiles::{ActiveModel, COLUMN, Entity, Model},
    },
    handles::{Result, map_insert_error},
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

    async fn model(&self) -> Result<Model> {
        Entity::find_by_id(self.id)
            .one(self.db.conn())
            .await?
            .ok_or(Error::StaleHandle)
    }

    async fn active_model(&self) -> Result<ActiveModel> {
        Ok(self.model().await?.into())
    }

    // Fields

    pub async fn name(&self) -> Result<String> {
        Ok(self.model().await?.name)
    }

    pub async fn set_name(&self, new_name: &str) -> Result<()> {
        let old_dir = self.dir().await?;

        let mut active_model = self.active_model().await?;
        active_model.name.set_if_not_equals(new_name.to_string());
        active_model.update(self.db.conn()).await?;

        let new_dir = self.dir().await?;
        fs::rename(old_dir, new_dir).unwrap();

        Ok(())
    }

    pub async fn dir(&self) -> Result<PathBuf> {
        Ok(self
            .parent()
            .await?
            .dir()
            .await?
            .join("profiles")
            .join(self.name().await?.to_snake_case()))
    }

    /// Make this profile the active one
    pub async fn activate(&self) -> Result<()> {
        let conn = self.db.conn();

        if state::active_game_id(conn).await? == Some(self.model().await?.game_id) {
            return state::set_active_profile_id(conn, Some(self.id)).await;
        } else {
            Err(Error::ProfileNotInActiveGame)
        }
    }

    pub async fn is_active(&self) -> Result<bool> {
        Ok(state::active_profile_id(self.db.conn()).await? == Some(self.id))
    }

    pub(crate) async fn active(db: Db, cfg: Cfg) -> Result<Option<Profile>> {
        Ok(state::active_profile_id(db.conn())
            .await?
            .map(|id| Profile::from_id(id, db.clone(), cfg.clone())))
    }

    /// Returns the parent [`Game`] of this [`Profile`]
    pub async fn parent(&self) -> Result<Game> {
        let parent_game_id = self.model().await?.game_id;
        Ok(Game::from_id(
            parent_game_id,
            self.db.clone(),
            self.cfg.clone(),
        ))
    }

    // Operations

    /// Add a new [`ModEntry`] to a [`Profile`] that points to the [`Mod`] given by ID.
    pub async fn add_mod_entry(&self, mod_: Mod) -> Result<ModEntry> {
        ModEntry::add(&self.db, &self.cfg, self, mod_).await
    }

    pub async fn mod_entries(&self) -> Result<Vec<ModEntry>> {
        ModEntry::list(&self.db, &self.cfg, self).await
    }

    pub async fn remove(self) -> Result<()> {
        // We have to store these so we can still access them once the profile is deleted
        let name = self.name().await?;
        let dir = self.dir().await?;

        Entity::delete_by_id(self.id).exec(self.db.conn()).await?;

        fs::remove_dir_all(dir).unwrap();

        state::reconcile(self.db.conn()).await?;

        info!("Removed profile: {name}");

        Ok(())
    }

    pub(crate) async fn add(db: &Db, cfg: &Cfg, game: &Game, name: &str) -> Result<Self> {
        let model = ActiveModel {
            name: Set(name.to_string()),
            game_id: Set(game.id),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(db.conn())
            .await
            .map_err(|e| map_insert_error(e, Error::DuplicateProfileName(name.into())))?
            .last_insert_id;

        let profile = Profile::from_id(id, db.clone(), cfg.clone());

        // TODO: Try to recover from this based on the problem
        fs::create_dir_all(profile.dir().await?).unwrap();

        state::reconcile(db.conn()).await?;

        info!("Added profile: {name}");

        Ok(profile)
    }

    pub(crate) async fn list(db: &Db, cfg: &Cfg, game: &Game) -> Result<Vec<Self>> {
        Ok(Entity::find()
            .filter(COLUMN.game_id.eq(game.id))
            .order_by_id_desc()
            .all(db.conn())
            .await?
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
    ) -> Result<Option<Profile>> {
        Ok(
            Entity::find_by_profile_name_per_game((name.to_string(), game.id))
                .one(db.conn())
                .await?
                .map(|model| Profile::from_id(model.id, db.clone(), cfg.clone())),
        )
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
        repository::{DeployKind, handles::Error},
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
            Err(Error::DuplicateProfileName(_))
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

        assert!(matches!(mod_entry.remove().await, Err(Error::StaleHandle)));
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
