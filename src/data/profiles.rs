use derive_more::{AsRef, From};
use getset::Getters;
use native_db::{Key, ToKey};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::data::{games::Game, mods::Mod};

// #[derive(Serialize, Deserialize, Debug, Clone, Getters)]
// #[getset(get = "pub")]
// pub struct ModEntry {
//     uuid: Uuid,
//     enabled: bool,
//     notes: String,
// }
//
// impl ModEntry {
//     pub fn new(uuid: Uuid) -> Self {
//         Self {
//             uuid,
//             enabled: true,
//             notes: "".to_string(),
//         }
//     }
// }
//
// #[derive(Debug, Clone, Getters)]
// #[getset(get = "pub")]
// pub struct ResolvedModEntry {
//     mod_ref: Mod,
//     entry: ModEntry,
// }
//
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
pub struct Profile {
    id: ProfileId,
    name: String,
    // mod_entries: Vec<ModEntry>,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            name: name.to_string(),
            // mod_entries: Vec::new(),
        }
    }

    // pub fn add_mod(&mut self, new_mod: Mod) {
    //     self.mod_entries.push(ModEntry::new(new_mod.uuid()));
    // }
    //
    // pub fn resolve_mod_entries(&self, game: &Game) -> Vec<ResolvedModEntry> {
    //     self.mod_entries
    //         .iter()
    //         .filter_map(|entry| match game.mods().get(&entry.uuid) {
    //             // TODO: Use Rc<RefCell<T>> to eliminate use of clone
    //             Some(mod_ref) => Some(ResolvedModEntry {
    //                 mod_ref: mod_ref.clone(),
    //                 entry: entry.clone(),
    //             }),
    //             None => {
    //                 warn!("Mod with UUID {} not found", entry.uuid);
    //                 None
    //             }
    //         })
    //         .collect()
    // }
}
