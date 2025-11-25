use std::{
    fs,
    path::{Path, PathBuf},
};

use barnacle_db::models::{Game, Mod, Profile};
use heck::ToSnakeCase;
use serde::{Deserialize, Serialize};

use crate::fs::config_dir;

const CURRENT_CONFIG_VERSION: u16 = 1;
const FILE_NAME: &str = "core.toml";

/// The backend's core configuration, serialized to TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    version: u16,
    library_dir: PathBuf,
}

impl CoreConfig {
    pub fn load() -> Self {
        let path = config_dir().join(FILE_NAME);

        if path.exists() {
            let contents = fs::read_to_string(path).unwrap();
            toml::from_str(&contents).unwrap_or_default()
        } else {
            let cfg = Self::default();
            cfg.save();
            cfg
        }
    }

    pub fn save(&self) {
        let contents = toml::to_string_pretty(self).unwrap();

        // Make sure config_dir exists
        fs::create_dir_all(config_dir()).unwrap();

        fs::write(config_dir().join(FILE_NAME), contents).unwrap();
    }

    /// Returns the path to the Barnacle library directory. This is where
    /// Barnacle stores its game, profile, and mod files. If it doesn't
    /// exist when this function is called, it will be created.
    pub fn library_dir(&self) -> &Path {
        fs::create_dir_all(&self.library_dir).unwrap();
        &self.library_dir
    }

    /// Path to a specific [`Game`]'s directory
    pub fn game_dir(&self, game: &Game) -> PathBuf {
        self.library_dir().join(game.name().to_snake_case())
    }

    /// Path to a specific [`Profile`]'s directory
    pub fn profile_dir(&self, game: &Game, profile: &Profile) -> PathBuf {
        self.game_dir(game)
            .join("profiles")
            .join(profile.name().to_snake_case())
    }

    /// Path to a specific [`Mod`]'s directory
    pub fn mod_dir(&self, game: &Game, mod_: &Mod) -> PathBuf {
        self.game_dir(game)
            .join("mods")
            .join(mod_.name().to_snake_case())
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            library_dir: xdg::BaseDirectories::with_prefix("barnacle")
                .get_data_home()
                .unwrap()
                .join("library"),
        }
    }
}
