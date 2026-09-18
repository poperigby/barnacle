use std::path::{Path, PathBuf};

use openmw_config::OpenMWConfiguration;
use strum::EnumIter;
use tokio::process::Command;
use walkdir::WalkDir;

use crate::{
    Game,
    deployers::{Deployer, FoundItem, ItemGroupSpec},
};

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, PartialOrd)]
pub enum ItemGroup {
    Plugin,
    Archive,
}

impl ItemGroup {
    fn from_file_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "esp" | "esm" | "omwscript" | "omwplugin" | "omwgame" => Some(Self::Plugin),
            "bsa" => Some(Self::Archive),
            _ => None,
        }
    }
}

impl ItemGroupSpec for ItemGroup {
    fn key(self) -> &'static str {
        match self {
            Self::Plugin => "openmw.plugin",
            Self::Archive => "openmw.archive",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Plugin => "Plugin",
            Self::Archive => "Archive",
        }
    }

    fn ordered(self) -> bool {
        match self {
            Self::Plugin => true,
            Self::Archive => false,
        }
    }
}

#[derive(Debug)]
pub struct OpenMw {}

impl Deployer for OpenMw {
    fn item_groups() -> Vec<impl ItemGroupSpec> {
        vec![ItemGroup::Plugin, ItemGroup::Archive]
    }
}

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

        for plugin in active_profile.item_entries(ItemGroup::Plugin).await {
            if plugin.enabled().await {
                config
                    .add_content_file(&plugin.relative_path().await.to_string_lossy())
                    .unwrap();
            }
        }

        for archive in active_profile.item_entries(ItemGroup::Archive).await {
            if archive.enabled().await {
                config
                    .add_archive_file(&archive.relative_path().await.to_string_lossy())
                    .unwrap();
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

    pub fn find_items(mod_dir: &Path) -> Vec<FoundItem<ItemGroup>> {
        let paths: Vec<PathBuf> = WalkDir::new(mod_dir)
            .into_iter()
            .map(|e| e.unwrap().into_path())
            .collect();

        let mut found = Vec::new();
        for path in paths {
            let relative_path = path.strip_prefix(mod_dir).unwrap().to_path_buf();

            if let Some(extension) = relative_path.extension() {
                let Some(group) = ItemGroup::from_file_extension(&extension.to_string_lossy())
                else {
                    continue;
                };

                found.push(FoundItem {
                    key: path.file_name().unwrap().to_string_lossy().to_lowercase(),
                    relative_path: relative_path.to_str().unwrap().to_string(),
                    group,
                });
            }
        }

        found.sort_by_key(|k| k.key.clone());

        found
    }
}

#[cfg(test)]
mod tests {
    use test_files::TestFiles;

    use super::*;

    #[test]
    fn test_find_items() {
        let dir = TestFiles::new();

        dir.file("Immersive Spoons.esp", "plugin")
            .file("BIGGER BOOBA.BSA", "archive")
            .file("optional/hyper realistic horkers.ESP", "plugin");

        let found_items = OpenMw::find_items(dir.path());

        assert_eq!(
            found_items,
            [
                FoundItem {
                    group: ItemGroup::Archive,
                    key: "bigger booba.bsa".to_string(),
                    relative_path: "BIGGER BOOBA.BSA".to_string(),
                },
                FoundItem {
                    group: ItemGroup::Plugin,
                    key: "hyper realistic horkers.esp".to_string(),
                    relative_path: "optional/hyper realistic horkers.ESP".to_string(),
                },
                FoundItem {
                    group: ItemGroup::Plugin,
                    key: "immersive spoons.esp".to_string(),
                    relative_path: "Immersive Spoons.esp".to_string(),
                },
            ]
        );
    }
}
