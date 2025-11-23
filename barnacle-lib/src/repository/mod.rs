use barnacle_db::Database;

use crate::{Result, fs::data_dir, repository::config::Config};

pub mod config;
pub mod db;

/// Central access point for all persistent data.
///
/// The [`Repository`] handles both on-disk filesystem operations and all
/// database queries. It provides a single, consistent interface for reading
/// and writing game data, mods, and profiles.
///
/// It can be freely cloned, as it internally uses an Arc<RwLock>.
#[derive(Clone, Debug)]
pub struct Repository {
    db: Database,
    cfg: Config,
}

impl Repository {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: Database::new(&data_dir().join("data.db"))?,
            cfg: Config::load(),
        })
    }

    pub fn cfg(&self) -> &Config {
        &self.cfg
    }
}
