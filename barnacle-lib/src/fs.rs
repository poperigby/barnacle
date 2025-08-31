use std::{
    fs::set_permissions,
    path::{Path, PathBuf},
};

use barnacle_data::schema::v1::{games::Game, profiles::Profile};
use walkdir::WalkDir;

#[derive(PartialEq)]
enum Permissions {
    ReadOnly,
    ReadWrite,
}

fn change_dir_permissions(path: &Path, permissions: Permissions) {
    use Permissions::*;

    for entry in WalkDir::new(path) {
        let mut perms = entry.as_ref().unwrap().metadata().unwrap().permissions();
        perms.set_readonly(permissions == ReadOnly);
        set_permissions(entry.unwrap().path(), perms).unwrap();
    }
}

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

/// Path to a specific `Game`'s directory
pub fn game_dir(game: &Game) -> PathBuf {
    data_dir().join("games").join(game.name())
}

/// Path to a specific `Profile`'s directory
pub fn profile_dir(game: &Game, profile: &Profile) -> PathBuf {
    game_dir(game).join("profiles").join(profile.name())
}
