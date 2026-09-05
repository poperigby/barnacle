use std::sync::Arc;

use parking_lot::RwLock;

use crate::repository::{
    config::{Cfg, CoreConfig},
    db::Db,
};

mod db;
mod state;

pub mod config;
pub mod handles;

pub use db::models::DeployKind;
pub use handles::{Game, Mod, ModEntry, Profile, Tool};

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
    pub async fn new() -> Self {
        Self {
            db: Db::new().await.unwrap(),
            cfg: Arc::new(RwLock::new(CoreConfig::load())),
        }
    }

    pub async fn add_game(&self, name: &str, deploy_kind: DeployKind) -> handles::Result<Game> {
        Game::add(&self.db.clone(), self.cfg.clone(), name, deploy_kind).await
    }

    pub async fn games(&self) -> handles::Result<Vec<Game>> {
        Game::list(self.db.clone(), self.cfg.clone()).await
    }

    pub async fn search_game(&self, name: &str) -> handles::Result<Option<Game>> {
        Game::search(self.db.clone(), self.cfg.clone(), name).await
    }

    pub async fn active_game(&self) -> handles::Result<Option<Game>> {
        Game::active(self.db.clone(), self.cfg.clone()).await
    }

    #[cfg(test)]
    /// A mock version of a [`Repository`] with an in-memory database and configuration
    /// file, for using in tests.
    pub(crate) async fn mock() -> Self {
        Self {
            db: Db::in_memory().await.unwrap(),
            cfg: Arc::new(RwLock::new(CoreConfig::mock())),
        }
    }
}
