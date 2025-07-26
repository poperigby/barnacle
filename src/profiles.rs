use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    name: String,
    /// An ordered list of mod UUIDs to quickly refer to them
    mods: Vec<Uuid>,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            mods: Vec::new(),
        }
    }
}
