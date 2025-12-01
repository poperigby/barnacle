use std::path::PathBuf;

use agdb::DbId;

use crate::repository::db::{DbHandle, get_field};

/// Represents a tool entity in the Barnacle system.
///
/// Provides methods to inspect and modify this tool's data.
/// Always reflects the current database state.
pub struct Tool {
    id: DbId,
    db: DbHandle,
}

impl Tool {
    pub fn name(&self) -> String {
        get_field(&self.db, "name", self.id).unwrap()
    }

    pub fn path(&self) -> PathBuf {
        get_field(&self.db, "path", self.id).unwrap()
    }

    // TODO: This can actually be Option<String>
    pub fn args(&self) -> String {
        get_field(&self.db, "args", self.id).unwrap()
    }
}
