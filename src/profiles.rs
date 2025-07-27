use std::{fs::create_dir_all, path::Path};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    name: String,
    /// An ordered list of mod UUIDs to quickly refer to them
    mods: Vec<Uuid>,
}

impl Profile {
    pub fn new(name: &str, dir: &Path) -> Self {
        create_dir_all(dir).unwrap();

        Self {
            name: name.to_string(),
            mods: Vec::new(),
        }
    }
}
