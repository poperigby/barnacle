use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use agdb::{DbError, DbValue, UserValue, UserValueMarker};
use derive_more::AsRef;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::{mods::ModId, profiles::ProfileId};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, AsRef, Copy, UserValueMarker)]
pub struct GameId(Uuid);

impl From<GameId> for DbValue {
    fn from(value: GameId) -> Self {
        DbValue::String(value.0.to_string())
    }
}

impl TryFrom<DbValue> for GameId {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        if let DbValue::String(s) = value {
            Ok(Self(
                Uuid::from_str(&s).map_err(|_| DbError::from("Failed to parse UUID"))?,
            ))
        } else {
            Err(DbError::from("Expected string for GameId"))
        }
    }
}

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
                .map_err(|_| DbError::from(format!("Invalid DeployType string: {}", s)))
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
                DbError::from(format!("Invalid GameDir string: {}", s))
            })?))
        } else {
            Err(DbError::from("Expected string for GameDir"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, UserValue)]
pub struct Game {
    id: GameId,
    name: String,
    deploy_type: DeployType,
    game_dir: GameDir,
    profiles: Vec<ProfileId>,
    mod_ids: Vec<ModId>,
}

impl Game {
    pub fn new(name: &str, game_type: DeployType, game_dir: &Path) -> Self {
        Self {
            id: GameId(Uuid::new_v4()),
            name: name.to_string(),
            deploy_type: game_type,
            game_dir: GameDir(game_dir.to_path_buf()),
            profiles: Vec::new(),
            mod_ids: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profiles(&self) -> &[ProfileId] {
        &self.profiles
    }

    pub fn mods(&self) -> &[ModId] {
        &self.mod_ids
    }

    pub fn game_dir(&self) -> &Path {
        &self.game_dir.as_ref()
    }

    pub fn add_profile(&mut self, id: ProfileId) {
        self.profiles.push(id);
    }

    pub fn add_mod(&mut self, id: ModId) {
        self.mod_ids.push(id);
    }
}
