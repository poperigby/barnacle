//! Filesystem helpers

use std::{
    fs::{create_dir_all, set_permissions},
    path::{Path, PathBuf},
};

use walkdir::WalkDir;
use xdg::BaseDirectories;

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
    let path = xdg_prefix().get_config_home().expect("$HOME must exist");

    create_dir_all(&path).unwrap();

    path
}

/// Returns the path to the Barnacle data directory. If it doesn't exist when this function is
/// called, it will be created.
pub fn data_dir() -> PathBuf {
    let path = xdg_prefix().get_data_home().expect("$HOME must exist");

    create_dir_all(&path).unwrap();

    path
}

/// Returns the path to the Barnacle state directory. If it doesn't exist when this function is
/// called, it will be created.
pub fn state_dir() -> PathBuf {
    let path = xdg_prefix().get_state_home().expect("$HOME must exist");

    create_dir_all(&path).unwrap();

    path
}

fn xdg_prefix() -> BaseDirectories {
    xdg::BaseDirectories::with_prefix("barnacle")
}
