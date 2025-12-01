use agdb::DbId;

use crate::repository::db::{DbHandle, get_field};

/// Represents a mod entity in the Barnacle system.
///
/// Provides methods to inspect and modify this mod's data.
/// Always reflects the current database state.
pub struct Mod {
    pub(crate) id: DbId,
    pub(crate) db: DbHandle,
}

impl Mod {
    pub fn name(&self) -> String {
        get_field(&self.db, self.id, "name").unwrap()
    }
}
