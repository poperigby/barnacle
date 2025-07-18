use blake3::Hash;
use compress_tools::{Ownership, uncompress_archive};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io,
    path::Path,
};
use thiserror::Error;

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
    hash: Hash,
}

impl Mod {
    /// Import a new mod from the given path
    pub fn new(name: String, path: &Path) -> Result<Self, ModError> {
        let hash = blake3::hash(&fs::read(&path).map_err(ModError::ReadArchiveError)?);

        let archive = File::open(&path).map_err(ModError::OpenArchiveError)?;
        let output_dir = data_dir().join("store").join(hash.to_string());
        uncompress_archive(archive, &output_dir, Ownership::Preserve)?;

        Ok(Self { name, hash })
    }
}
