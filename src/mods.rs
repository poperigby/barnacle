use compress_tools::{Ownership, uncompress_archive};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;

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

        Ok(Self { name, uuid })
    }

    /// Return the mod UUID
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn dir(&self) -> PathBuf {
        data_dir().join("mods").join(self.uuid.to_string())
    }
}
