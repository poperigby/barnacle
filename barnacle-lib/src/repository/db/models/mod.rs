//! Insert-only payload structs for persistence
//!
//! This module defines the internal data structures used when creating new
//! games, mods, and profiles in the Barnacle database. These types are not
//! returned to callers. The public API exposes `Game`, `Profile`, and `Mod`
//! handle types instead. The structs here exist solely to provide the data
//! required for inserts. Migration between schema versions is handled
//! internally.

pub mod games;
pub mod mod_entries;
pub mod mods;
pub mod profiles;
pub mod state;
pub mod tools;

pub use games::DeployKind;
