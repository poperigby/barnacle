use std::{
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use super::Error;
use heck::ToSnakeCase;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, ModelTrait};
use tracing::info;

use crate::repository::{
    Cfg, DeployKind,
    db::{
        Db,
        models::{
            games::{ActiveModel, Entity, Model},
            mods::Entity as ModEntity,
        },
    },
    handles::{Result, map_insert_error, mod_::Mod, profile::Profile},
    state,
};

/// Represents a game entity in the Barnacle system.
///
/// Provides methods to inspect and modify this game's data, including
/// managing profiles and mods. Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Game {
    pub(crate) id: i32,
    pub(crate) db: Db,
    pub(crate) cfg: Cfg,
}

impl Game {
    /// Load some existing [`Game`] from the database
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

    pub async fn name(&self) -> Result<String> {
        Ok(self.model().await?.name)
    }

    // TODO: We need to have some way to reconcile the on disk state if the fs::rename() fails
    pub async fn set_name(&self, new_name: &str) -> Result<()> {
        let old_dir = self.dir().await?;

        let mut active_model = self.active_model().await?;
        active_model.name.set_if_not_equals(new_name.to_string());
        active_model.update(self.db.conn()).await?;

        let new_dir = self.dir().await?;

        // TODO: Try to recover from this based on the problem
        fs::rename(old_dir, new_dir).unwrap();

        Ok(())
    }

    // pub fn targets(&self) -> Result<Vec<PathBuf>> {
    //     self.get_field("targets")
    // }

    pub async fn deploy_kind(&self) -> Result<DeployKind> {
        Ok(self.model().await?.deploy_kind)
    }

    pub async fn set_deploy_kind(&self, new_deploy_kind: DeployKind) -> Result<()> {
        let mut active_model = self.active_model().await?;
        active_model.deploy_kind.set_if_not_equals(new_deploy_kind);
        active_model.update(self.db.conn()).await?;

        Ok(())
    }

    pub async fn dir(&self) -> Result<PathBuf> {
        let library_dir = self.cfg.read().library_dir().to_path_buf();

        Ok(library_dir.join(self.name().await?.to_snake_case()))
    }

    pub async fn remove(self) -> Result<()> {
        // We have to store these so we can still access them once the game is deleted
        let name = self.name().await?;
        let dir = self.dir().await?;

        Entity::delete_by_id(self.id).exec(self.db.conn()).await?;

        // TODO: Try to recover from this based on the problem
        fs::remove_dir_all(dir).unwrap();

        state::reconcile(self.db.conn()).await?;

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
        Mod::list(&self.db.clone(), &self.cfg.clone(), self).await
    }

    pub async fn add_mod(&self, name: &str, path: Option<&Path>) -> Result<Mod> {
        Mod::add(self.db.clone(), self.cfg.clone(), self, name, path).await
    }

    /// Insert a new [`Game`] into the database. The [`Game`] must have a unique name.
    pub(crate) async fn add(
        db: &Db,
        cfg: Cfg,
        name: &str,
        deploy_kind: DeployKind,
    ) -> Result<Self> {
        let model = ActiveModel {
            name: Set(name.to_string()),
            deploy_kind: Set(deploy_kind),
            ..Default::default()
        };

        let id = Entity::insert(model)
            .exec(db.conn())
            .await
            .map_err(|e| map_insert_error(e, Error::DuplicateGameName(name.into())))?
            .last_insert_id;

        let game = Game::from_id(id, db.clone(), cfg.clone());

        // TODO: Try to recover from this based on the problem
        fs::create_dir_all(game.dir().await?).unwrap();

        state::reconcile(db.conn()).await?;

        info!("Created new game: {}", game.name().await?);

        Ok(game)
    }

    pub(crate) async fn list(db: Db, cfg: Cfg) -> Result<Vec<Game>> {
        Ok(Entity::find()
            .order_by_id_desc()
            .all(db.conn())
            .await?
            .iter()
            .map(|model| Game::from_id(model.id, db.clone(), cfg.clone()))
            .collect())
    }

    /// Search for a game by name
    // TODO: This is a bad name because you're just directly finding a game by its exact name. Not
    // exactly searching. We should have actual searching.
    pub(crate) async fn search(db: Db, cfg: Cfg, name: &str) -> Result<Option<Game>> {
        Ok(Entity::find_by_name(name)
            .one(db.conn())
            .await?
            .map(|model| Game::from_id(model.id, db.clone(), cfg.clone())))
    }

    /// Make this game the active one
    pub async fn activate(&self) -> Result<()> {
        state::set_active_game_id(self.db.conn(), Some(self.id)).await
    }

    pub async fn is_active(&self) -> Result<bool> {
        Ok(state::active_game_id(self.db.conn()).await? == Some(self.id))
    }

    pub(crate) async fn active(db: Db, cfg: Cfg) -> Result<Option<Game>> {
        Ok(state::active_game_id(db.conn())
            .await?
            .map(|id| Game::from_id(id, db.clone(), cfg.clone())))
    }

    pub async fn active_profile(&self) -> Result<Option<Profile>> {
        Profile::active(self.db.clone(), self.cfg.clone()).await
    }

    /// Search for the given profile by name
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
    use crate::Repository;

    use super::*;

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
            Err(Error::DuplicateGameName(_)),
        ))
    }

    #[tokio::test]
    async fn test_remove() {
        let repo = Repository::mock().await;

        let game = repo
            .add_game("Skyrim", DeployKind::CreationEngine)
            .await
            .unwrap();
        let profile = game.add_profile("test_profile_1").await.unwrap();
        let _mod = game.add_mod("test_mod", None).await.unwrap();

        assert_eq!(repo.games().await.unwrap().len(), 1);

        let dir = game.dir().await.unwrap();

        game.remove().await.unwrap();

        // Attempt to remove already removed profile and mod entries
        assert!(matches!(profile.remove().await, Err(Error::StaleHandle)));
        assert!(matches!(_mod.remove().await, Err(Error::StaleHandle)));

        assert!(!dir.exists());
        assert_eq!(repo.games().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_remove_made_next_game_active() {
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

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        game.name().await.unwrap();
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

        let game = repo
            .add_game("Fallout: New Vegas", DeployKind::Gamebryo)
            .await
            .unwrap();

        game.deploy_kind().await.unwrap();
    }

    #[tokio::test]
    async fn test_dir() {
        let repo = Repository::mock().await;

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
