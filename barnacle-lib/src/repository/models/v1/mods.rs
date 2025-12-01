use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub(crate) struct ModModel {
    pub(crate) db_id: Option<DbId>,
    /// A human friendly display name
    pub(crate) name: String,
}

impl ModModel {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            name: name.into(),
        }
    }
}
