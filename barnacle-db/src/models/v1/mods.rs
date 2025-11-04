use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub struct Mod {
    db_id: Option<DbId>,
    /// A human friendly display name
    name: String,
    // TODO: Just to differentiate profiles and mods until I add more fields
    is_mod: bool,
}

impl Mod {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            name: name.into(),
            is_mod: true,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, DbType, Default, PartialEq, PartialOrd)]
pub struct ModEntry {
    db_id: Option<DbId>,
    enabled: bool,
    notes: String,
}

impl ModEntry {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }
}
