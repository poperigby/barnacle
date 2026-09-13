use derive_more::{Deref, DerefMut};
use openmw_config::OpenMWConfiguration;
use tempfile::{TempDir, tempdir};
use tokio::process::Command;

use crate::Game;

#[derive(Debug, Deref, DerefMut)]
struct Config {
    #[deref]
    #[deref_mut]
    config: OpenMWConfiguration,
    dir: TempDir,
}

impl Config {
    pub fn load() -> Self {
        let dir = tempdir().unwrap();
        let config = OpenMWConfiguration::new_empty(dir.path()).unwrap();

        Self { config, dir }
    }
}

#[derive(Debug)]
pub struct OpenMw {
    config: Config,
    game_process: Command,
}

impl OpenMw {
    pub async fn prepare(game: &Game) -> Self {
        let mut config = Config::load();

        let active_profile = game.active_profile().await.unwrap().unwrap();

        for mod_entry in active_profile.mod_entries().await.unwrap() {
            if mod_entry.enabled().await.unwrap() {
                config.add_data_directory(&mod_entry.dir().await.unwrap());
            }
        }

        config.save_user().unwrap();

        let game_process = Command::new("echo 'Hello'");

        Self {
            config,
            game_process,
        }
    }
}
