use std::str::FromStr;

use agdb::{DbError, DbId, DbValue, UserValue, UserValueMarker};
use derive_more::{AsRef, Display};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, AsRef, Copy, Display, UserValueMarker)]
pub struct ModId(Uuid);

impl From<ModId> for DbValue {
    fn from(value: ModId) -> Self {
        DbValue::String(value.0.to_string())
    }
}

impl TryFrom<DbValue> for ModId {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        if let DbValue::String(s) = value {
            Ok(Self(
                Uuid::from_str(&s).map_err(|_| DbError::from("Failed to parse UUID"))?,
            ))
        } else {
            Err(DbError::from("Expected string for ModId"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, UserValue)]
pub struct Mod {
    db_id: Option<DbId>,
    /// A unique identifier to refer to the mod by
    id: ModId,
    /// A pretty name to display in the UI
    name: String,
}

impl Mod {
    pub fn new(name: &str) -> Self {
        Self {
            db_id: None,
            id: ModId(Uuid::new_v4()),
            name: name.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, UserValue, UserValueMarker)]
pub struct ModEntry {
    db_id: Option<DbId>,
    mod_id: ModId,
    enabled: bool,
    notes: String,
}

impl ModEntry {
    pub fn new(mod_id: ModId) -> Self {
        Self {
            db_id: None,
            mod_id,
            enabled: true,
            notes: "".to_string(),
        }
    }
}
