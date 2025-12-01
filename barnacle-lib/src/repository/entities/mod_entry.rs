use agdb::DbId;

use crate::repository::{
    CoreConfigHandle,
    db::DbHandle,
    entities::{Result, get_field},
};

/// Represents a mod entry in the Barnacle system.
///
/// Provides methods to inspect and modify this mod entry's data.
/// Always reflects the current database state.
#[derive(Debug, Clone)]
pub struct ModEntry {
    /// The ID of the ModEntryModel
    pub(crate) entry_id: DbId,
    /// The ID of the ModModel the entry points to
    pub(crate) mod_id: DbId,
    pub(crate) db: DbHandle,
}

impl ModEntry {
    pub(crate) fn from_id(entry_id: DbId, mod_id: DbId, db: DbHandle) -> Self {
        Self {
            entry_id,
            mod_id,
            db,
        }
    }

    pub fn name(&self) -> Result<String> {
        get_field(&self.db, self.mod_id, "name")
    }

    pub fn enabled(&self) -> Result<bool> {
        get_field(&self.db, self.entry_id, "enabled")
    }

    pub fn notes(&self) -> Result<String> {
        get_field(&self.db, self.entry_id, "notes")
    }
}
