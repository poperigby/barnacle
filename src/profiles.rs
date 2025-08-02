use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::mods::Mod;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    name: String,
    /// An ordered list of mod UUIDs to quickly refer to them
    mod_ids: Vec<Uuid>,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            mod_ids: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mod_ids(&self) -> &[Uuid] {
        &self.mod_ids
    }

    pub fn add_mod(&mut self, new_mod: Mod) {
        self.mod_ids.push(new_mod.uuid());
    }

    // pub fn get_enabled_mods(&self) {
    //     let enabled_mods: Vec<_> = &self
    //         .mod_ids()
    //         .iter()
    //         .filter_map(|id| game.mods().get(id))
    //         .collect()
    // }
}
