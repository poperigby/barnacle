use std::{
    fs::{File, Permissions, set_permissions},
    io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use compress_tools::{Ownership, uncompress_archive};
use derive_more::{AsRef, From};
use native_db::{Key, ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Clone, Debug, AsRef, From, Copy)]
pub struct ModId(Uuid);

impl ToKey for ModId {
    fn to_key(&self) -> Key {
        Key::new(self.0.as_bytes().to_vec())
    }
    fn key_names() -> Vec<String> {
        vec!["Mod ID".to_string()]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct Mod {
    /// A unique identifier to refer to the mod by
    #[primary_key]
    id: ModId,
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
        let uuid = uuid::Uuid::new_v4();

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
        Ok(Self {
            id: uuid.into(),
            name,
            path,
        })
    }

    pub fn id(&self) -> ModId {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
