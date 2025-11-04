use agdb::{DbId, DbType};

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub struct Profile {
    db_id: Option<DbId>,
    name: String,
    // TODO: Just to differentiate profiles and mods until I add more fields
    is_profile: bool,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
            is_profile: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
