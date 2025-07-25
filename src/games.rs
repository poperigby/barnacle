use crate::{data_dir, mods::Mod, profiles::Profile};
use serde::{Deserialize, Serialize};
use std::{fs::create_dir_all, path::PathBuf};
use tracing::warn;

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    name: String,
    profiles: Vec<Profile>,
    mods: Vec<Mod>,
    game_dir: PathBuf,
}

impl Game {
    pub fn new(name: String, game_dir: PathBuf) -> Self {
        create_dir_all(data_dir().join("profiles").join(&name)).unwrap();

        if !game_dir.exists() {
            warn!(
                "The game directory \"{}\" does not exist",
                game_dir.to_str().unwrap()
            );
        };

        Self {
            name,
            profiles: Vec::new(),
            mods: Vec::new(),
            game_dir,
        }
    }

    pub fn create_profile(&mut self, name: String) {
        create_dir_all(self.dir().join(&name)).unwrap();

        self.profiles.push(Profile::new(name));
    }

    pub fn add_mod(&mut self, r#mod: Mod) {
        self.mods.push(r#mod);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the path of the game specific profile directory
    pub fn dir(&self) -> PathBuf {
        data_dir().join("profiles").join(&self.name)
    }
}
