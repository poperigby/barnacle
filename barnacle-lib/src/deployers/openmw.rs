use openmw_config::OpenMWConfiguration;
use tokio::process::Command;

use crate::Game;

#[derive(Debug)]
pub struct OpenMw {}

impl OpenMw {
    pub async fn prepare(game: &Game) -> Command {
        let active_profile = game.active_profile().await.unwrap().unwrap();

        let config_path = active_profile.generated_dir().await.unwrap();
        let mut config = OpenMWConfiguration::new_empty(&config_path).unwrap();

        for mod_entry in active_profile.mod_entries().await.unwrap() {
            if mod_entry.enabled().await.unwrap() {
                config.add_data_directory(&mod_entry.dir().await.unwrap());
            }
        }

        config.save_user().unwrap();

        let mut program = game.launch_command().await;

        program.args([
            // Skip the user configuration file
            "--replace",
            "config",
            // Use our generated configuration file
            "--config",
            config_path.to_str().unwrap(),
        ]);

        program
    }
}
