use std::{
    fs::{create_dir_all, set_permissions},
    path::{Path, PathBuf},
};

use barnacle_db::models::{Game, Mod, Profile};
use walkdir::WalkDir;

#[derive(PartialEq)]
pub enum Permissions {
    ReadOnly,
    // ReadWrite,
}

pub fn change_dir_permissions(path: &Path, permissions: Permissions) {
    use Permissions::*;

    for entry in WalkDir::new(path) {
        let mut perms = entry.as_ref().unwrap().metadata().unwrap().permissions();
        perms.set_readonly(permissions == ReadOnly);
        set_permissions(entry.unwrap().path(), perms).unwrap();
    }
}

/// Returns the path to the Barnacle configuration directory. If it doesn't exist when this
/// function is called, it will be created.
pub fn config_dir() -> PathBuf {
    let path = xdg::BaseDirectories::with_prefix("barnacle")
        .get_config_home()
        .unwrap();

    create_dir_all(&path).unwrap();

    path
}

/// Returns the path to the Barnacle data directory. If it doesn't exist when this function is
/// called, it will be created.
pub fn data_dir() -> PathBuf {
    let path = xdg::BaseDirectories::with_prefix("barnacle")
        .get_data_home()
        .unwrap();

    create_dir_all(&path).unwrap();

    path
}

/// Path to a specific `Game`'s directory
pub fn game_dir(game: &Game) -> PathBuf {
    data_dir().join("games").join(game.name())
}

/// Path to a specific `Profile`'s directory
pub fn profile_dir(game: &Game, profile: &Profile) -> PathBuf {
    game_dir(game).join("profiles").join(profile.name())
}

pub fn mod_dir(game: &Game, mod_: &Mod) -> PathBuf {
    game_dir(game).join("mods").join(mod_.name())
}
