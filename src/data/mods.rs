use compress_tools::{Ownership, uncompress_archive};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, Permissions, set_permissions},
    io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{Result, data_dir};

#[derive(Error, Debug)]
pub enum ModError {
    #[error("Failed to read mod archive: {0}")]
    ReadArchive(io::Error),
    #[error("Failed to open mod archive: {0}")]
    OpenArchive(io::Error),
    #[error("Failed to uncompress mod archive: {0}")]
    UncompressArchive(compress_tools::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mod {
    /// A unique identifier to refer to the mod by
    uuid: Uuid,
    /// The path to the mod
    path: PathBuf,
    /// A pretty name to display in the UI
    name: String,
}

impl Mod {
    /// Import a new mod from the given path
    pub fn new(path: &Path, name: Option<&str>) -> Result<Self> {
        // If mod name isn't provided, infer it from the file's name
        let name = name
            // TODO: Infer from directory name if the input path is a directory instead of an
            // archive
            .unwrap_or_else(|| path.file_stem().unwrap().to_str().unwrap())
            .to_string();
        let uuid = Uuid::new_v4();

        let archive = File::open(path).map_err(ModError::OpenArchive)?;
        let output_dir = data_dir().join("mods").join(uuid.to_string());
        uncompress_archive(archive, &output_dir, Ownership::Preserve)
            .map_err(ModError::UncompressArchive)?;

        // Make mod read-only on disk
        for entry in WalkDir::new(output_dir) {
            let entry = entry.unwrap();
            let mode = if entry.metadata().unwrap().is_dir() {
                0o550
            } else {
                0o440
            };

            set_permissions(entry.path(), Permissions::from_mode(mode)).unwrap();
        }

        let path = data_dir().join("mods").join(uuid.to_string());
        Ok(Self { name, path, uuid })
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
