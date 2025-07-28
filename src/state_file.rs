use std::{
    fs::{File, create_dir_all},
    io::{self, Write},
};

use crate::{Result, data_dir, games::Game};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateFileError {
    #[error("Failed to read state file: {0}")]
    Read(io::Error),
    #[error("Failed to write to state file: {0}")]
    Write(io::Error),
    #[error("Failed to create the data directory: {0}")]
    CreateDataDir(io::Error),
    #[error("Failed to deserialize state file: {0}")]
    Deserialize(ron::de::SpannedError),
    #[error("Failed to serialize state file: {0}")]
    Serialize(ron::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub games: Vec<Game>,
}

impl State {
    pub fn load() -> Result<Self> {
        let path = data_dir().join("state.ron");
        if path.exists() {
            let file = File::open(path).map_err(StateFileError::Read)?;
            Ok(ron::de::from_reader(file).map_err(StateFileError::Deserialize)?)
        } else {
            create_dir_all(data_dir()).map_err(StateFileError::CreateDataDir)?;
            Ok(State { games: Vec::new() })
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut file = File::create(data_dir().join("state.ron")).map_err(StateFileError::Write)?;

        let s = ron::ser::to_string_pretty(&self, PrettyConfig::default())
            .map_err(StateFileError::Serialize)?;

        write!(file, "{}", s).map_err(StateFileError::Write)?;

        Ok(())
    }
}
