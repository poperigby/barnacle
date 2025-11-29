//! Insert-only payload structs for persistence
//!
//! This module defines the internal data structures used when creating new
//! games, mods, and profiles in the Barnacle database. These types are not
//! returned to callers. The public API exposes `Game`, `Profile`, and `Mod`
//! handle types instead. The structs here exist solely to provide the data
//! required for inserts. Migration between schema versions is handled
//! internally.

mod v1;

// Re-export current version of models
pub mod games {
    pub use super::v1::games::*;
}
pub mod mods {
    pub use super::v1::mods::*;
}
pub mod profiles {
    pub use super::v1::profiles::*;
}
pub mod tools {
    pub use super::v1::tools::*;
}

// Also re-export the main types at `models` level for convenience
pub use games::*;
pub use mods::*;
pub use profiles::*;
pub use tools::*;

use agdb::{DbId, DbType};

/// Represents the current version of the database models used by Barnacle.
///
/// This value increments whenever the structure or layout of stored data
/// changes in a way that requires migration. It is independent of the
/// Barnacle application version and is used solely to determine whether
/// migrations need to be applied when initializing the database.
pub(crate) const CURRENT_MODEL_VERSION: u64 = 1;

/// Holds the model version of the local database. If this value is lower than
/// [`CURRENT_MODEL_VERSION`], migrations will be performed until the database
/// is up to date.
#[derive(Debug, Clone, DbType, PartialEq, PartialOrd)]
pub(crate) struct ModelVersion {
    db_id: Option<DbId>,
    version: u64,
}

impl ModelVersion {
    pub fn version(&self) -> u64 {
        self.version
    }
}

impl Default for ModelVersion {
    fn default() -> Self {
        Self {
            db_id: None,
            version: CURRENT_MODEL_VERSION,
        }
    }
}
