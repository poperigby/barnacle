use barnacle_db::Database;

use crate::{Result, fs::data_dir};

mod games;
mod mods;
mod profiles;
mod tools;

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
}

impl Repository {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: Database::new(&data_dir().join("data.db"))?,
        })
    }
}
