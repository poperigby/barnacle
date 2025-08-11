// TODO: Move business logic out of here

use std::{
    fs::{File, create_dir_all, remove_dir_all, set_permissions},
    io,
    path::Path,
};

use barnacle_data::v1::{
    games::{DeployType, Game},
    mods::{Mod, ModId},
    profiles::Profile,
};
use compress_tools::{Ownership, uncompress_archive};
use thiserror::Error;
use tracing::warn;
use walkdir::WalkDir;

use crate::data_dir;

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

#[derive(Error, Debug)]
pub enum AddModError {
    #[error("Failed to read mod archive: {0}")]
    ReadArchive(io::Error),
    #[error("Failed to open mod archive: {0}")]
    OpenArchive(io::Error),
    #[error("Failed to uncompress mod archive: {0}")]
    UncompressArchive(#[from] compress_tools::Error),
}

pub struct Database<'a> {
    db: native_db::Database<'a>,
}

impl<'a> Database<'a> {
    pub fn new(db: native_db::Database<'a>) -> Self {
        Self { db }
    }

    pub fn add_game(&self, name: &str, game_type: DeployType, game_dir: &Path) {
        if !game_dir.exists() {
            warn!(
                "The game directory '{}' does not exist",
                game_dir.to_str().unwrap()
            );
        };

        let new_game = Game::new(name, game_type, game_dir);

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_game).unwrap();
        rw.commit().unwrap();
    }

    pub fn add_profile(&self, name: &str) {
        let new_profile = Profile::new(name);

        create_dir_all(
            data_dir()
                .join("profiles")
                .join(new_profile.id().to_string()),
        )
        .unwrap();

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_profile).unwrap();
        rw.commit().unwrap();
    }

    pub fn add_mod(&self, input_path: &Path, name: Option<&str>) -> Result<(), AddModError> {
        // If mod name isn't provided, infer it from the file's name
        let name = name
            // TODO: Infer from directory name if the input path is a directory instead of an
            // archive
            .unwrap_or_else(|| input_path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        let new_mod = Mod::new(name);
        let dir = data_dir().join("mods").join(new_mod.id().to_string());

        // TODO: Only do attempt to open the archive if the input_path is an archive
        let archive = File::open(input_path).map_err(AddModError::OpenArchive)?;
        uncompress_archive(archive, &dir, Ownership::Preserve)?;

        change_dir_permissions(&dir, Permissions::ReadOnly);

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_mod).unwrap();
        rw.commit().unwrap();

        Ok(())
    }

    pub fn delete_mod(&self, mod_id: ModId) {
        let rw = self.db.rw_transaction().unwrap();
        let found_mod: Mod = rw.get().primary(mod_id).unwrap().unwrap();
        let dir = data_dir().join("mods").join(found_mod.id().to_string());

        change_dir_permissions(&dir, Permissions::ReadWrite);
        remove_dir_all(&dir).unwrap();

        rw.remove(found_mod).unwrap();
        rw.commit().unwrap();
    }
}
