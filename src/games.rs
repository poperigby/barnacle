use crate::{data_dir, mods::Mod, profiles::Profile};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::create_dir_all,
    path::{Path, PathBuf},
};
use tracing::warn;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameType {
    /// Deploys directly to the game directory with OverlayFS.
    Generic,
    /// Same as the generic type, but with support for Gamebryo/Creation Engine `plugins.txt`.
    Bethesda,
    /// Deploys mods to an intermediary staging directory with OverlayFS, preventing the mod store
    /// from needing to be modified. The individual mod directories are then added to `openmw.cfg`.
    /// Plugins are also handled.
    OpenMW,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    name: String,
    /// The deployment logic will change based on which game type is chosen.
    game_type: GameType,
    profiles: Vec<Profile>,
    mods: HashMap<Uuid, Mod>,
    game_dir: PathBuf,
}

impl Game {
    pub fn new(name: &str, game_type: GameType, game_dir: &Path) -> Self {
        create_dir_all(data_dir().join("profiles").join(name)).unwrap();

        if !game_dir.exists() {
            warn!(
                "The game directory '{}' does not exist",
                game_dir.to_str().unwrap()
            );
        };

        Self {
            name: name.to_string(),
            game_type,
            profiles: Vec::new(),
            mods: HashMap::new(),
            game_dir: game_dir.to_path_buf(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn mods(&self) -> &HashMap<Uuid, Mod> {
        &self.mods
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

        // Create overlay specific directories
        let overlay_dir = dir.join("overlay");
        create_dir_all(overlay_dir.join("work")).unwrap();
        create_dir_all(overlay_dir.join("upper")).unwrap();

        self.profiles.push(Profile::new(name));
    }

    pub fn import_mod(&mut self, mod_path: &Path, name: Option<&str>) {
        let new_mod = Mod::new(mod_path, name).unwrap();
        self.mods.insert(new_mod.uuid(), new_mod);
    }
}
