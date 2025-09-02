use std::cmp::Ordering;

use agdb::{DbId, DbType};

use crate::ModId;

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub struct Mod {
    db_id: Option<DbId>,
    /// A pretty name to display in the UI
    name: String,
}

impl Mod {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
        }
    }

    pub fn id(&self) -> Option<ModId> {
        Some(ModId(self.db_id?))
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, DbType, PartialEq, Eq)]
pub struct ModEntry {
    db_id: Option<DbId>,
    enabled: bool,
    load_order: u32,
    notes: String,
}

impl ModEntry {
    pub fn new(enabled: bool, load_order: u32, notes: &str) -> Self {
        ModEntry {
            db_id: None,
            enabled,
            load_order,
            notes: notes.to_string(),
        }
    }
}

impl Ord for ModEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.load_order.cmp(&other.load_order)
    }
}

impl PartialOrd for ModEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
