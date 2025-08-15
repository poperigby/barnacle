use std::str::FromStr;

use agdb::{DbError, DbValue, UserValue, UserValueMarker};
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
    /// A unique identifier to refer to the mod by
    id: ModId,
    /// A pretty name to display in the UI
    name: String,
}

impl Mod {
    pub fn new(name: &str) -> Self {
        Self {
            id: ModId(Uuid::new_v4()),
            name: name.to_string(),
        }
    }

    pub fn id(&self) -> ModId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
