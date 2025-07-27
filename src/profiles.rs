use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    name: String,
    /// An ordered list of mod UUIDs to quickly refer to them
    mods: Vec<Uuid>,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            mods: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
