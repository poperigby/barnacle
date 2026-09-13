use openmw_config::OpenMWConfiguration;
use tokio::process::Command;

use crate::Game;

#[derive(Debug)]
pub struct OpenMw {}

impl OpenMw {
    pub async fn prepare(game: &Game) -> Self {
        let mut config = OpenMWConfiguration::from_env_or_user_config().unwrap();

        let active_profile = game.active_profile().await.unwrap().unwrap();

        for mod_entry in active_profile.mod_entries().await.unwrap() {
            if mod_entry.enabled().await.unwrap() {
                config.add_data_directory(&mod_entry.dir().await.unwrap());
            }
        }

        config.save_user().unwrap();

        let game_process = Command::new("echo 'Hello'");

        Self {}
    }
}
