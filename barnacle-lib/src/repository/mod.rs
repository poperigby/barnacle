use std::sync::Arc;

use parking_lot::RwLock;

use crate::repository::{
    config::{Cfg, CoreConfig},
    db::Db,
};

mod db;

pub mod config;
pub mod entities;

pub use db::models::DeployKind;
pub use entities::{Game, Mod, ModEntry, Profile, Tool};

/// Central access point for all persistent data.
///
/// The [`Repository`] handles both on-disk filesystem operations and all
/// database and configuration file queries. It provides a single, consistent interface
/// for reading and writing game data, mods, and profiles.
#[derive(Clone, Debug)]
pub struct Repository {
    db: Db,
    cfg: Cfg,
}

impl Repository {
    pub fn new() -> Self {
        Self {
            db: Db::new(),
            cfg: Arc::new(RwLock::new(CoreConfig::load())),
        }
    }

    pub fn add_game(&self, name: &str, deploy_kind: DeployKind) -> entities::Result<Game> {
        Game::add(&self.db.clone(), self.cfg.clone(), name, deploy_kind)
    }

    pub fn games(&self) -> entities::Result<Vec<Game>> {
        Game::list(self.db.clone(), self.cfg.clone())
    }

    pub fn search_game(&self, name: &str) -> entities::Result<Option<Game>> {
        Game::search(self.db.clone(), self.cfg.clone(), name)
    }

    pub fn active_game(&self) -> entities::Result<Option<Game>> {
        Game::active(self.db.clone(), self.cfg.clone())
    }

    #[cfg(test)]
    /// A mock version of a [`Repository`] with an in-memory database and configuration
    /// file, for using in tests.
    pub(crate) fn mock() -> Self {
        Self {
            db: Db::in_memory(),
            cfg: Arc::new(RwLock::new(CoreConfig::mock())),
        }
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}
