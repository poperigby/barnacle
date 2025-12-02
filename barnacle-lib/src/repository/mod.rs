use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    Result,
    repository::{config::CoreConfig, db::DbHandle, entities::game::Game, models::DeployKind},
};

pub mod config;
pub mod db;
pub mod entities;
mod models;

pub type CoreConfigHandle = Arc<RwLock<CoreConfig>>;

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
        Ok(Game::add(
            self.db.clone(),
            self.cfg.clone(),
            name,
            deploy_kind,
        )?)
    }

    pub fn remove_game(&self, target: Game) -> Result<()> {
        target.remove()?;

        Ok(())
    }

    pub fn games(&self) -> Result<Vec<Game>> {
        Ok(Game::list(self.db.clone(), self.cfg.clone())?)
    }
}
