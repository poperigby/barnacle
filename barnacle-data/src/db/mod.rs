use std::path::Path;

use agdb::{Db, QueryBuilder};

use crate::{DatabaseError, Result};

pub mod games;
pub mod mods;
pub mod profiles;

/// Graph database for storing data related to Barnacle
#[derive(Debug)]
pub struct Database(Db);

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let path_str = path.to_str().ok_or(DatabaseError::PathInvalidUnicode)?;
        let db = Db::new(path_str)?;
        Ok(Database(db))
    }

    /// Initialize the root nodes
    pub fn init(&mut self) -> Result<()> {
        self.0.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(["games", "current_profile"])
                .query(),
        )?;

        Ok(())
    }
}
