use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub struct ModModel {
    db_id: Option<DbId>,
    /// A human friendly display name
    name: String,
}

impl ModModel {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, DbType, Default, PartialEq, PartialOrd)]
pub struct ModEntryModel {
    db_id: Option<DbId>,
    enabled: bool,
    notes: String,
}
