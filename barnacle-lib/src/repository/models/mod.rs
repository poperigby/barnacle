//! Core domain models

use sea_orm::DbErr;
use thiserror::Error;

mod games;
mod mod_entries;
mod mods;
mod profiles;
mod tools;

pub use games::{Game, entity::DeployKind};
pub use mod_entries::ModEntry;
pub use mods::Mod;
pub use profiles::Profile;
pub use tools::Tool;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal database error {0}")]
    Internal(#[from] DbErr),
    #[error("This entity has been deleted")]
    RemovedEntity,
    #[error("Internal database error {0}")]
    Serialization(String),
}
