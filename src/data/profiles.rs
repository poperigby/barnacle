use derive_more::{AsRef, From};
use getset::Getters;
use native_db::{Key, ToKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::mods::{Mod, ModId};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, AsRef, From)]
pub struct ProfileId(Uuid);

impl ToKey for ProfileId {
    fn to_key(&self) -> Key {
        Key::new(self.0.as_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["Profile ID".to_string()]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct ModEntry {
    mod_id: ModId,
    enabled: bool,
    notes: String,
}

impl ModEntry {
    pub fn new(mod_id: ModId) -> Self {
        Self {
            mod_id,
            enabled: true,
            notes: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Profile {
    id: ProfileId,
    name: String,
    mod_entries: Vec<ModEntry>,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            name: name.to_string(),
            mod_entries: Vec::new(),
        }
    }

    pub fn add_mod(&mut self, new_mod: Mod) {
        self.mod_entries.push(ModEntry::new(new_mod.id()));
    }
}
