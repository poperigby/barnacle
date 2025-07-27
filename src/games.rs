use crate::{data_dir, mods::Mod, profiles::Profile};
use damascus::FuseOverlayFs;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::create_dir_all,
    path::{Path, PathBuf},
};
use tracing::warn;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    name: String,
    profiles: Vec<Profile>,
    mods: HashMap<Uuid, Mod>,
    game_dir: PathBuf,
}

impl Game {
    pub fn new(name: &str, game_dir: &Path) -> Self {
        create_dir_all(data_dir().join("profiles").join(name)).unwrap();

        if !game_dir.exists() {
            warn!(
                "The game directory '{}' does not exist",
                game_dir.to_str().unwrap()
            );
        };

        Self {
            name: name.to_string(),
            profiles: Vec::new(),
            mods: HashMap::new(),
            game_dir: game_dir.to_path_buf(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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

        let overlay_dir = dir.join("overlay");
        create_dir_all(overlay_dir.join("work")).unwrap();
        create_dir_all(overlay_dir.join("upper")).unwrap();

        self.profiles.push(Profile::new(name));
    }

    pub fn import_mod(&mut self, mod_path: &Path, name: Option<&str>) {
        let new_mod = Mod::new(mod_path, name).unwrap();
        self.mods.insert(new_mod.uuid(), new_mod);
    }

    pub fn deploy_profile(&self, profile: &Profile) {
        let overlay_dir = self.dir().join(profile.name()).join("overlay");

        let work_dir = overlay_dir.join("work");
        let upper_dir = overlay_dir.join("upper");
        let lower_dirs = vec![self.game_dir()].into_iter();

        FuseOverlayFs::new(
            lower_dirs,
            Some(upper_dir),
            Some(work_dir),
            &self.game_dir,
            false,
        )
        .unwrap();
    }
}
