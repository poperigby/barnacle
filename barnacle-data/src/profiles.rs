use std::str::FromStr;

use agdb::{DbError, DbValue, UserValue, UserValueMarker};
use derive_more::{AsRef, Display};
use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::mods::{Mod, ModId};

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Copy, AsRef, Display, UserValueMarker,
)]
pub struct ProfileId(Uuid);

impl From<ProfileId> for DbValue {
    fn from(value: ProfileId) -> Self {
        DbValue::String(value.0.to_string())
    }
}

impl TryFrom<DbValue> for ProfileId {
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

#[derive(Serialize, Deserialize, Debug, Clone, Getters, UserValue, UserValueMarker)]
#[getset(get = "pub")]
pub struct ModEntry {
    mod_id: ModId,
    enabled: bool,
    notes: String,
}

impl ModEntry {
    pub fn new(mod_id: ModId) -> Self {
        Self {
            mod_id,
            enabled: true,
            notes: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Getters, UserValue, UserValueMarker)]
#[getset(get = "pub")]
pub struct Profile {
    id: ProfileId,
    name: String,
    mod_entries: Vec<ModEntry>,
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            id: ProfileId(Uuid::new_v4()),
            name: name.to_string(),
            mod_entries: Vec::new(),
        }
    }

    pub fn add_mod(&mut self, new_mod: Mod) {
        self.mod_entries.push(ModEntry::new(new_mod.id()));
    }
}
