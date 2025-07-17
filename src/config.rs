use std::{
    fs::{File, create_dir_all},
    io,
};

use crate::{config_dir, games::Game};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    ReadError(io::Error),
    #[error("Failed to write to configuration file: {0}")]
    WriteError(io::Error),
    #[error("Failed to deserialize configuration file: {0}")]
    DeserializeError(serde_yml::Error),
    #[error("Failed to serialize configuration file: {0}")]
    SerializeError(serde_yml::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub games: Vec<Game>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_dir().join("config.yaml");
        if path.exists() {
            let file = File::open(path).map_err(ConfigError::ReadError)?;
            Ok(serde_yml::from_reader(file).map_err(ConfigError::DeserializeError)?)
        } else {
            create_dir_all(config_dir()).unwrap();
            Ok(Config { games: Vec::new() })
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let file =
            File::create(config_dir().join("config.yaml")).map_err(ConfigError::WriteError)?;

        serde_yml::to_writer(file, &self).map_err(ConfigError::SerializeError)?;

        Ok(())
    }
}
