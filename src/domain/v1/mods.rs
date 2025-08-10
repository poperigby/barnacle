use std::path::PathBuf;

use derive_more::{AsRef, Display};
use native_db::{Key, ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data_dir;

#[derive(Serialize, Deserialize, Clone, Debug, AsRef, Copy, Display)]
pub struct ModId(Uuid);

impl ToKey for ModId {
    fn to_key(&self) -> Key {
        Key::new(self.0.as_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["Mod ID".to_string()]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct Mod {
    /// A unique identifier to refer to the mod by
    #[primary_key]
    id: ModId,
    /// A pretty name to display in the UI
    name: String,
}

impl Mod {
    pub fn new(name: String) -> Self {
        Self {
            id: ModId(Uuid::new_v4()),
            name,
        }
    }

    pub fn id(&self) -> ModId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dir(&self) -> PathBuf {
        data_dir().join("mods").join(self.id.0.to_string())
    }
}
