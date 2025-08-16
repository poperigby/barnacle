use agdb::{DbId, UserValue};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, UserValue)]
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
}
