//! Core domain entities for Barnacle.

use sea_orm::DbErr;
use thiserror::Error;

mod game;
mod mod_;
mod mod_entry;
mod profile;
mod tool;

pub use game::Game;
pub use mod_::Mod;
pub use mod_entry::ModEntry;
pub use profile::Profile;
pub use tool::Tool;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal database error {0}")]
    Internal(#[from] DbErr),
    #[error("This entity has been deleted")]
    RemovedEntity,
    #[error("An entity with the given name already exists")]
    DuplicateName,
    #[error("Internal database error {0}")]
    Serialization(String),
}

pub(crate) fn map_duplicate_name(err: Error) -> Error {
    match err {
        Error::Internal(db_err) if db_err.to_string().contains("UNIQUE constraint failed") => {
            Error::DuplicateName
        }
        other => other,
    }
}
