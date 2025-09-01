use agdb::{DbId, DbType};

use crate::ProfileId;

#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub struct Profile {
    db_id: Option<DbId>,
    name: String,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
        }
    }

    pub fn id(&self) -> Option<ProfileId> {
        Some(ProfileId(self.db_id?))
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
