use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use agdb::{DbError, DbId, DbValue, UserValue, UserValueMarker};
use derive_more::AsRef;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug, Clone, Display, EnumString, UserValueMarker, Default)]
pub enum DeployType {
    /// Deploys directly to the game directory with OverlayFS.
    #[default]
    Overlay,
    /// Same as the overlay type, but with support for Gamebryo/Creation Engine `plugins.txt`.
    Gamebryo,
    CreationEngine,
    /// Deploys mods to an intermediary staging directory with OverlayFS, preventing the mod store
    /// from needing to be modified. The individual mod directories are then added to `openmw.cfg`.
    /// Plugins are also handled.
    OpenMW,
}

impl From<DeployType> for DbValue {
    fn from(value: DeployType) -> Self {
        DbValue::String(value.to_string())
    }
}

impl TryFrom<DbValue> for DeployType {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        if let DbValue::String(s) = value {
            s.parse()
                .map_err(|_| DbError::from(format!("Invalid DeployType string: {s}")))
        } else {
            Err(DbError::from("Expected string for DeployType"))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, AsRef, UserValueMarker)]
pub struct GameDir(PathBuf);

impl From<GameDir> for DbValue {
    fn from(value: GameDir) -> Self {
        DbValue::String(value.0.to_string_lossy().to_string())
    }
}

impl TryFrom<DbValue> for GameDir {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        if let DbValue::String(s) = value {
            Ok(GameDir(PathBuf::from_str(&s).map_err(|_| {
                DbError::from(format!("Invalid GameDir string: {s}"))
            })?))
        } else {
            Err(DbError::from("Expected string for GameDir"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, UserValue)]
pub struct Game {
    db_id: Option<DbId>,
    name: String,
    deploy_type: DeployType,
    game_dir: GameDir,
}

impl Game {
    pub fn new(name: &str, game_type: DeployType, game_dir: &Path) -> Self {
        Self {
            db_id: None,
            name: name.to_string(),
            deploy_type: game_type,
            game_dir: GameDir(game_dir.to_path_buf()),
        }
    }
}
