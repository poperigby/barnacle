use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{games::Game, mods::Mod};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModEntry {
    uuid: Uuid,
    enabled: bool,
    notes: String,
}

impl ModEntry {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            enabled: true,
            notes: "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedModEntry {
    mod_ref: Mod,
    entry: ModEntry,
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

    pub fn resolve_mods(&self, game: &Game) -> Vec<ResolvedModEntry> {
        self.mod_entries
            .iter()
            .filter_map(|entry| match game.mods().get(&entry.uuid) {
                // TODO: Use Rc<RefCell<T>> to eliminate use of clone
                Some(mod_ref) => Some(ResolvedModEntry {
                    mod_ref: mod_ref.clone(),
                    entry: entry.clone(),
                }),
                None => {
                    warn!("Mod with UUID {} not found", entry.uuid);
                    None
                }
            })
            .collect()
    }
}
