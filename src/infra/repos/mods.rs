use std::{
    fs::{File, Permissions, set_permissions},
    io,
    os::unix::fs::PermissionsExt,
    path::Path,
};

use compress_tools::{Ownership, uncompress_archive};
use native_db::Database;
use thiserror::Error;
use walkdir::WalkDir;

use crate::domain::v1::mods::Mod;

#[derive(Error, Debug)]
pub enum AddModError {
    #[error("Failed to read mod archive: {0}")]
    ReadArchive(io::Error),
    #[error("Failed to open mod archive: {0}")]
    OpenArchive(io::Error),
    #[error("Failed to uncompress mod archive: {0}")]
    UncompressArchive(compress_tools::Error),
}

pub struct ModRepo<'a> {
    db: Database<'a>,
}

impl<'a> ModRepo<'a> {
    pub fn new(db: Database<'a>) -> Self {
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
        uncompress_archive(archive, &new_mod.dir(), Ownership::Preserve)
            .map_err(AddModError::UncompressArchive)?;

        // Make mod read-only on disk
        for entry in WalkDir::new(&new_mod.dir()) {
            let entry = entry.unwrap();
            let mode = if entry.metadata().unwrap().is_dir() {
                0o550
            } else {
                0o440
            };

            set_permissions(entry.path(), Permissions::from_mode(mode)).unwrap();
        }

        let rw = self.db.rw_transaction().unwrap();
        rw.insert(new_mod).unwrap();
        rw.commit().unwrap();

        Ok(())
    }
}
