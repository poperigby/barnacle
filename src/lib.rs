use std::path::PathBuf;

use native_db::Database;

use crate::managers::{games::GamesManager, mods::ModsManager, profiles::ProfilesManager};

pub mod domain;
pub mod managers;

pub fn config_dir() -> PathBuf {
    xdg::BaseDirectories::with_prefix("barnacle")
        .get_config_home()
        .unwrap()
}

pub fn data_dir() -> PathBuf {
    xdg::BaseDirectories::with_prefix("barnacle")
        .get_data_home()
        .unwrap()
}

pub struct AppService<'a> {
    games_repo: GamesManager<'a>,
    profiles_repo: ProfilesManager<'a>,
    mods_repo: ModsManager<'a>,
}

impl<'a> AppService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            games_repo: GamesManager::new(db),
            profiles_repo: ProfilesManager::new(db),
            mods_repo: ModsManager::new(db),
        }
    }
}
