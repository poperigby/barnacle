use agdb::UserValue;
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Getters, UserValue)]
#[getset(get = "pub")]
pub struct Profile {
    name: String,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
