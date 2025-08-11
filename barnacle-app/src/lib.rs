use std::{
    fs::{File, set_permissions},
    io,
    path::{Path, PathBuf},
};

use barnacle_data::v1::{
    games::{DeployType, Game},
    mods::Mod,
};
use compress_tools::{Ownership, uncompress_archive};
use thiserror::Error;
use tracing::warn;
use walkdir::WalkDir;

use crate::database::Database;

pub mod database;

#[derive(Error, Debug)]
pub enum AddModError {
    #[error("Failed to read mod archive: {0}")]
    ReadArchive(io::Error),
    #[error("Failed to open mod archive: {0}")]
    OpenArchive(io::Error),
    #[error("Failed to uncompress mod archive: {0}")]
    UncompressArchive(#[from] compress_tools::Error),
}

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

pub fn add_game(db: &Database, name: &str, game_type: DeployType, game_dir: &Path) {
    if !game_dir.exists() {
        warn!(
            "The game directory '{}' does not exist",
            game_dir.to_str().unwrap()
        );
    };

    let new_game = Game::new(name, game_type, game_dir);
    db.insert_game(new_game);
}

pub fn add_mod(db: &Database, input_path: &Path, name: Option<&str>) -> Result<(), AddModError> {
    // If mod name isn't provided, infer it from the file's name
    let name = name
        // TODO: Infer from directory name if the input path is a directory instead of an
        // archive
        .unwrap_or_else(|| input_path.file_stem().unwrap().to_str().unwrap())
        .to_string();

    let new_mod = Mod::new(&name);
    let dir = data_dir().join("mods").join(new_mod.id().to_string());

    // TODO: Only do attempt to open the archive if the input_path is an archive
    let archive = File::open(input_path).map_err(AddModError::OpenArchive)?;
    uncompress_archive(archive, &dir, Ownership::Preserve)?;

    db.insert_mod(new_mod);

    change_dir_permissions(&dir, Permissions::ReadOnly);

    Ok(())
}
