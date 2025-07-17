use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    name: String,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
