use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    Result,
    repository::{
        config::{CoreConfig, CoreConfigHandle},
        db::DbHandle,
        models::GameModel,
    },
};

mod db;
mod models;

pub mod config;
pub mod entities;

pub use entities::{Game, Mod, ModEntry, Profile, Tool};
pub use models::DeployKind;

/// Central access point for all persistent data.
///
/// The [`Repository`] handles both on-disk filesystem operations and all
/// database and configuration file queries. It provides a single, consistent interface
/// for reading and writing game data, mods, and profiles.
#[derive(Clone, Debug)]
pub struct Repository {
    db: DbHandle,
    cfg: CoreConfigHandle,
}

impl Repository {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: DbHandle::new(),
            cfg: Arc::new(RwLock::new(CoreConfig::load())),
        })
    }

    pub fn add_game(&self, name: &str, deploy_kind: DeployKind) -> Result<Game> {
        let model = GameModel::new(name, deploy_kind);
        Ok(Game::add(self.db.clone(), self.cfg.clone(), model)?)
    }

    pub fn remove_game(&self, target: Game) -> Result<()> {
        target.remove()?;

        Ok(())
    }

    pub fn games(&self) -> Result<Vec<Game>> {
        Ok(Game::list(self.db.clone(), self.cfg.clone())?)
    }

    pub fn set_current_profile(&self, profile: &Profile) -> Result<()> {
        Ok(Profile::set_current(self.db.clone(), profile)?)
    }

    pub fn current_profile(&self) -> Result<Profile> {
        Ok(Profile::current(self.db.clone(), self.cfg.clone())?)
    }
}
