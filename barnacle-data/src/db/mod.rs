use std::path::Path;

use agdb::{Db, DbError, DbId, QueryBuilder};
use thiserror::Error;

pub mod games;
pub mod mods;
pub mod profiles;

type Result<T> = std::result::Result<T, DatabaseError>;

/// ID representing a Game in the database
#[derive(Debug)]
pub struct GameId(DbId);

/// ID representing a Profile in the database
#[derive(Debug)]
pub struct ProfileId(DbId);

/// ID representing a Mod in the database
#[derive(Debug)]
pub struct ModId(DbId);

/// Graph database for storing data related to Barnacle
#[derive(Debug)]
pub struct Database(Db);

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Db(#[from] DbError),
    #[error("The given path is invalid unicode")]
    PathInvalidUnicode,
}

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
