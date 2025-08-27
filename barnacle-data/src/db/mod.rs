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
        let mut db = Db::new(path_str)?;

        // Insert aliases if they don't exist
        if db.exec(QueryBuilder::select().aliases().query())?.result == 0 {
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases(["games", "current_profile"])
                    .query(),
            )?;
        }

        Ok(Database(db))
    }
}
