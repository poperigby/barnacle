//! Data models for Barnacle database entities
//!
//! This module contains the data structures used to represent games, mods, and profiles
//! in the Barnacle database. The types exported from this module represent the current
//! data model version. Migrations are handled internally.

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

// Re-export the main types at models:: level for convenience
pub use games::*;
pub use mods::*;
pub use profiles::*;
