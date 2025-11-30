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
    pub async fn name(&self) -> String {
        let db = self.db.read().await;

        get_field(&db, "name", self.id).unwrap()
    }

    pub async fn path(&self) -> PathBuf {
        let db = self.db.read().await;

        get_field(&db, "path", self.id).unwrap()
    }

    // TODO: This can actually be Option<String>
    pub async fn args(&self) -> String {
        let db = self.db.read().await;

        get_field(&db, "args", self.id).unwrap()
    }
}
