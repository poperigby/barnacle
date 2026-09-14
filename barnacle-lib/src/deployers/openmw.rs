use std::path::PathBuf;

use openmw_config::OpenMWConfiguration;
use tokio::process::Command;
use walkdir::WalkDir;

use crate::{Game, Mod};

#[derive(Debug, Clone)]
pub struct FoundItem {
    name: String,
    path: PathBuf,
    group_key: String,
}

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

    pub async fn parse_items(mod_: &Mod) -> Vec<FoundItem> {
        let paths: Vec<PathBuf> = WalkDir::new(mod_.dir().await.unwrap())
            .into_iter()
            .map(|e| e.unwrap().into_path())
            .collect();

        let mut found = Vec::new();

        for path in paths {
            let mod_dir = mod_.dir().await.unwrap();
            let name = path.file_name();
            let path = path.strip_prefix(mod_dir).unwrap().to_path_buf();

            if let Some(extension) = path.extension() {
                let group_key = match extension.to_str().unwrap() {
                    "esp" => "openmw.plugins",
                    "bsa" => "openmw.archives",
                    _ => "",
                }
                .to_string();

                if !group_key.is_empty() {
                    found.push(FoundItem {
                        name: name.unwrap().to_str().unwrap().to_string(),
                        path,
                        group_key,
                    });
                }
            }
        }

        found
    }
}
