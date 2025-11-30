use agdb::DbId;

use crate::repository::db::{DbHandle, get_field};

/// Represents a mod entity in the Barnacle system.
///
/// Provides methods to inspect and modify this mod's data.
/// Always reflects the current database state.
pub struct Mod {
    id: DbId,
    db: DbHandle,
}

impl Mod {
    pub fn name(&self) -> String {
        let db = self.db.read();

        get_field(&db, "name", self.id).unwrap()
    }
}
