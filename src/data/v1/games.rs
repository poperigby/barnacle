use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use derive_more::{AsRef, From};
use native_db::{Key, ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{
    data::v1::{
        mods::{Mod, ModId},
        profiles::Profile,
    },
    data_dir,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, AsRef, From)]
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
    name: String,
    deploy_type: DeployType,
    game_dir: PathBuf,
    profiles: Vec<Profile>,
    mod_ids: Vec<ModId>,
}

impl Game {
    pub fn setup(name: &str, game_type: DeployType, game_dir: &Path) -> Self {
        create_dir_all(data_dir().join("profiles").join(name)).unwrap();

        if !game_dir.exists() {
            warn!(
                "The game directory '{}' does not exist",
                game_dir.to_str().unwrap()
            );
        };

        Self {
            id: Uuid::new_v4().into(),
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

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn mods(&self) -> &[ModId] {
        &self.mod_ids
    }

    pub fn game_dir(&self) -> &Path {
        &self.game_dir
    }

    /// Return the path of the game specific profile directory
    pub fn dir(&self) -> PathBuf {
        data_dir().join("profiles").join(&self.name)
    }

    pub fn create_profile(&mut self, name: &str) {
        let dir = self.dir().join(name);
        create_dir_all(&dir).unwrap();

        // TODO: Initialize deployer

        self.profiles.push(Profile::new(name));
    }

    pub fn import_mod(&mut self, mod_path: &Path, name: Option<&str>) {
        let new_mod = Mod::new(mod_path, name).unwrap();
        self.mod_ids.push(new_mod.id());
    }
}
