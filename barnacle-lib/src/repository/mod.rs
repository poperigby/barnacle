use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard};

use crate::{
    Result,
    repository::{config::CoreConfig, db::DbHandle},
};

pub mod config;
mod db;
pub mod entities;
mod models;

/// Central access point for all persistent data.
///
/// The [`Repository`] handles both on-disk filesystem operations and all
/// database and configuration file queries. It provides a single, consistent interface
/// for reading and writing game data, mods, and profiles.
#[derive(Clone, Debug)]
pub struct Repository {
    db: DbHandle,
    cfg: Arc<RwLock<CoreConfig>>,
}

impl Repository {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: DbHandle::new(),
            cfg: Arc::new(RwLock::new(CoreConfig::load())),
        })
    }

    pub async fn cfg(&'_ self) -> RwLockReadGuard<'_, CoreConfig> {
        self.cfg.read()
    }
}
