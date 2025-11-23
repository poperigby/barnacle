use std::sync::Arc;

use barnacle_db::Database;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::{Result, fs::data_dir, repository::config::Config};

pub mod config;
pub mod db;

/// Central access point for all persistent data.
///
/// The [`Repository`] handles both on-disk filesystem operations and all
/// database queries. It provides a single, consistent interface for reading
/// and writing game data, mods, and profiles.
#[derive(Clone, Debug)]
pub struct Repository {
    db: Arc<RwLock<Database>>,
    cfg: Arc<RwLock<Config>>,
}

impl Repository {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: Arc::new(RwLock::new(Database::new(&data_dir().join("data.db"))?)),
            cfg: Arc::new(RwLock::new(Config::load())),
        })
    }

    pub async fn cfg(&'_ self) -> RwLockReadGuard<'_, Config> {
        self.cfg.read().await
    }
}
