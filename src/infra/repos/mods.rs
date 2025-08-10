use std::{
    fs::{File, remove_dir_all, set_permissions},
    io,
    path::Path,
};

use compress_tools::{Ownership, uncompress_archive};
use native_db::Database;
use thiserror::Error;
use walkdir::WalkDir;

use crate::domain::v1::mods::{Mod, ModId};

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

pub struct ModsRepo<'a> {
    db: &'a Database<'a>,
}

impl<'a> ModsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn add_mod(&self, input_path: &Path, name: Option<&str>) -> Result<(), AddModError> {
        // If mod name isn't provided, infer it from the file's name
        let name = name
            // TODO: Infer from directory name if the input path is a directory instead of an
            // archive
            .unwrap_or_else(|| input_path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        let new_mod = Mod::new(name);

        // TODO: Only do attempt to open the archive if the input_path is an archive
        let archive = File::open(input_path).map_err(AddModError::OpenArchive)?;
        uncompress_archive(archive, &new_mod.dir(), Ownership::Preserve)?;

        change_dir_permissions(&new_mod.dir(), Permissions::ReadOnly);

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_mod).unwrap();
        rw.commit().unwrap();

        Ok(())
    }

    pub fn delete_mod(&self, mod_id: ModId) {
        let rw = self.db.rw_transaction().unwrap();
        let found_mod: Mod = rw.get().primary(mod_id).unwrap().unwrap();

        change_dir_permissions(&found_mod.dir(), Permissions::ReadWrite);
        remove_dir_all(found_mod.dir()).unwrap();

        rw.remove(found_mod).unwrap();
        rw.commit().unwrap();
    }
}
