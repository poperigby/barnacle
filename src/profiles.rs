use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{games::Game, mods::Mod};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModEntry {
    uuid: Uuid,
    enabled: bool,
}

impl ModEntry {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedModEntry<'a> {
    entry: &'a ModEntry,
    mod_data: &'a Mod,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    name: String,
    mod_entries: Vec<ModEntry>,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            mod_entries: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_mod(&mut self, new_mod: Mod) {
        self.mod_entries.push(ModEntry::new(new_mod.uuid()));
    }

    pub fn resolve_mods<'a>(&'a self, game: &'a Game) -> Vec<ResolvedModEntry<'a>> {
        self.mod_entries
            .iter()
            .filter_map(|entry| match game.mods().get(&entry.uuid) {
                Some(mod_data) => Some(ResolvedModEntry { entry, mod_data }),
                None => {
                    warn!("Mod with UUID {} not found", entry.uuid);
                    None
                }
            })
            .collect()
    }
}
