use std::path::PathBuf;

use thiserror::Error;

use crate::{mods::ModError, state_file::StateFileError};

pub mod games;
pub mod mods;
pub mod overlay;
pub mod profiles;
pub mod state_file;

pub type Result<T> = std::result::Result<T, BarnacleError>;

#[derive(Debug, Error)]
pub enum BarnacleError {
    #[error("Problem detected with state file: {0}")]
    StateFileError(#[from] StateFileError),
    #[error("Problem detected with mod: {0}")]
    ModError(#[from] ModError),
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
