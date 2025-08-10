use std::path::PathBuf;

use native_db::Database;

use crate::infra::repos::{games::GamesRepo, mods::ModsRepo, profiles::ProfilesRepo};

pub mod domain;
pub mod infra;

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
    games_repo: GamesRepo<'a>,
    profiles_repo: ProfilesRepo<'a>,
    mods_repo: ModsRepo<'a>,
}

impl<'a> AppService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            games_repo: GamesRepo::new(db),
            profiles_repo: ProfilesRepo::new(db),
            mods_repo: ModsRepo::new(db),
        }
    }
}
