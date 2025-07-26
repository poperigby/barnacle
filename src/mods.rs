use compress_tools::{Ownership, uncompress_archive};
use serde::{Deserialize, Serialize};
use std::{fs::File, io, path::Path};
use thiserror::Error;
use uuid::Uuid;

use crate::data_dir;

#[derive(Error, Debug)]
pub enum ModError {
    #[error("Failed to read mod archive: {0}")]
    ReadArchiveError(io::Error),
    #[error("Failed to open mod archive: {0}")]
    OpenArchiveError(io::Error),
    #[error("Failed to uncompress mod archive: {0}")]
    UncompressArchiveError(#[from] compress_tools::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mod {
    name: String,
    uuid: Uuid,
}

impl Mod {
    /// Import a new mod from the given path
    pub fn new(name: String, path: &Path) -> Result<Self, ModError> {
        let archive = File::open(path).map_err(ModError::OpenArchiveError)?;
        let uuid = Uuid::new_v4();
        let output_dir = data_dir().join("mods").join(uuid.to_string());
        uncompress_archive(archive, &output_dir, Ownership::Preserve)?;

        Ok(Self { name, uuid })
    }

    /// Return the mod UUID
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}
