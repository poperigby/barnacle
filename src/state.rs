use std::{
    fs::{File, create_dir_all},
    io::{self, Write},
};

use crate::{config_dir, games::Game};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    ReadError(io::Error),
    #[error("Failed to write to configuration file: {0}")]
    WriteError(io::Error),
    #[error("Failed to deserialize configuration file: {0}")]
    DeserializeError(ron::de::SpannedError),
    #[error("Failed to serialize configuration file: {0}")]
    SerializeError(ron::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub games: Vec<Game>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_dir().join("state.ron");
        if path.exists() {
            let file = File::open(path).map_err(ConfigError::ReadError)?;
            Ok(ron::de::from_reader(file).map_err(ConfigError::DeserializeError)?)
        } else {
            create_dir_all(config_dir()).unwrap();
            Ok(Config { games: Vec::new() })
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let mut file =
            File::create(config_dir().join("state.ron")).map_err(ConfigError::WriteError)?;

        let s = ron::ser::to_string_pretty(&self, PrettyConfig::default())
            .map_err(ConfigError::SerializeError)?;

        write!(file, "{}", s).map_err(ConfigError::WriteError)?;

        Ok(())
    }
}
