use std::path::{Path, PathBuf};

use derive_more::{AsRef, Display};
use native_db::{Key, ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::v1::{mods::ModId, profiles::ProfileId};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, AsRef, Display)]
pub struct GameId(Uuid);

impl ToKey for GameId {
    fn to_key(&self) -> Key {
        Key::new(self.0.as_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["Game ID".to_string()]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DeployType {
    /// Deploys directly to the game directory with OverlayFS.
    Overlay,
    /// Same as the overlay type, but with support for Gamebryo/Creation Engine `plugins.txt`.
    Gamebryo,
    CreationEngine,
    /// Deploys mods to an intermediary staging directory with OverlayFS, preventing the mod store
    /// from needing to be modified. The individual mod directories are then added to `openmw.cfg`.
    /// Plugins are also handled.
    OpenMW,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Game {
    #[primary_key]
    id: GameId,
    #[secondary_key(unique)]
    name: String,
    deploy_type: DeployType,
    game_dir: PathBuf,
    profiles: Vec<ProfileId>,
    mod_ids: Vec<ModId>,
}

impl Game {
    pub fn new(name: &str, game_type: DeployType, game_dir: &Path) -> Self {
        Self {
            id: GameId(Uuid::new_v4()),
            name: name.to_string(),
            deploy_type: game_type,
            game_dir: game_dir.to_path_buf(),
            profiles: Vec::new(),
            mod_ids: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profiles(&self) -> &[ProfileId] {
        &self.profiles
    }

    pub fn mods(&self) -> &[ModId] {
        &self.mod_ids
    }

    pub fn game_dir(&self) -> &Path {
        &self.game_dir
    }

    pub fn add_profile(&mut self, id: ProfileId) {
        self.profiles.push(id);
    }

    pub fn add_mod(&mut self, id: ModId) {
        self.mod_ids.push(id);
    }
}
