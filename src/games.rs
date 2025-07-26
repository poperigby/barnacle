use crate::{data_dir, mods::Mod, profiles::Profile};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::create_dir_all, path::PathBuf};
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
            mods: HashMap::new(),
            game_dir,
        }
    }

    pub fn create_profile(&mut self, name: String) {
        create_dir_all(self.dir().join(&name)).unwrap();

        self.profiles.push(Profile::new(name));
    }

    pub fn import_mod(&mut self, mod_path: PathBuf, name: Option<String>) {
        match name {
            Some(n) => {
                let new_mod = Mod::new(n, &mod_path).unwrap();
                self.mods.insert(new_mod.uuid(), new_mod);
            }
            None => {
                // Infer name from mod_path
                let n = mod_path.file_stem().unwrap().to_str().unwrap().to_string();
                let new_mod = Mod::new(n, &mod_path).unwrap();
                self.mods.insert(new_mod.uuid(), new_mod);
            }
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the path of the game specific profile directory
    pub fn dir(&self) -> PathBuf {
        data_dir().join("profiles").join(&self.name)
    }
}
