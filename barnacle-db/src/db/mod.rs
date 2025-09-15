use std::{path::Path, sync::Arc};

use agdb::{DbAny, QueryBuilder};
use tokio::sync::RwLock;

use crate::{DatabaseError, Result};

pub mod games;
pub mod mods;
pub mod profiles;

/// Graph database for storing data related to Barnacle
#[derive(Debug)]
pub struct Database(Arc<RwLock<DbAny>>);

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let path_str = path.to_str().ok_or(DatabaseError::PathInvalidUnicode)?;
        Self::init(DbAny::new_file(path_str)?)
    }

    /// Create a memory backed database for use in tests
    fn new_memory() -> Result<Self> {
        Self::init(DbAny::new_memory("data.db")?)
    }

    fn init(mut db: DbAny) -> Result<Self> {
        // Insert aliases if they don't exist
        if db.exec(QueryBuilder::select().aliases().query())?.result == 0 {
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases(["games", "profiles", "mods", "current_profile"])
                    .query(),
            )?;
        }

        // TODO: perform any migrations here

        Ok(Database(Arc::new(RwLock::new(db))))
    }
}
