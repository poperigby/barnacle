use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType)]
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
}

#[derive(Debug, Clone, DbType, Default)]
pub struct ModEntry {
    db_id: Option<DbId>,
    enabled: bool,
    notes: String,
}
