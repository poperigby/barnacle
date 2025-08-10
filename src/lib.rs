use std::path::PathBuf;

use thiserror::Error;

use crate::data::v1::mods::ModError;

pub mod data;
pub mod managers;
// pub mod deployers;

pub type Result<T> = std::result::Result<T, BarnacleError>;

#[derive(Debug, Error)]
pub enum BarnacleError {
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
