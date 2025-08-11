use std::{
    fs::{File, create_dir_all},
    io::{self, Write},
};

use barnacle::data_dir;
use barnacle_data::v1::{games::GameId, profiles::ProfileId};
use ron::{
    de::from_reader,
    ser::{PrettyConfig, to_string_pretty},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type Result<T> = std::result::Result<T, AppStateError>;

#[derive(Error, Debug)]
pub enum AppStateError {
    #[error("Failed to read state file: {0}")]
    Read(io::Error),
    #[error("Failed to write to state file: {0}")]
    Write(io::Error),
    #[error("Failed to deserialize state file: {0}")]
    Deserialize(ron::de::SpannedError),
    #[error("Failed to serialize state file: {0}")]
    Serialize(ron::Error),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct AppState {
    pub selected_game: Option<GameId>,
    pub selected_profile: Option<ProfileId>,
}

impl AppState {
    pub fn load() -> Result<Self> {
        let path = data_dir().join("state.ron");
        if path.exists() {
            let file = File::open(path).map_err(AppStateError::Read)?;
            Ok(from_reader(file).map_err(AppStateError::Deserialize)?)
        } else {
            create_dir_all(data_dir()).unwrap();
            Ok(AppState::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut file = File::create(data_dir().join("state.ron")).map_err(AppStateError::Write)?;

        let s =
            to_string_pretty(&self, PrettyConfig::default()).map_err(AppStateError::Serialize)?;

        write!(file, "{s}").map_err(AppStateError::Write)?;

        Ok(())
    }
}
