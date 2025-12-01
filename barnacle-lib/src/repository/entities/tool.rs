use std::path::PathBuf;

use agdb::DbId;

use crate::repository::{db::DbHandle, entities::get_field};

/// Represents a tool entity in the Barnacle system.
///
/// Provides methods to inspect and modify this tool's data.
/// Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct Tool {
    id: DbId,
    db: DbHandle,
}

impl Tool {
    pub fn name(&self) -> String {
        get_field(&self.db, self.id, "name").unwrap()
    }

    pub fn path(&self) -> PathBuf {
        get_field(&self.db, self.id, "path").unwrap()
    }

    // TODO: This can actually be Option<String>
    pub fn args(&self) -> String {
        get_field(&self.db, self.id, "args").unwrap()
    }
}
