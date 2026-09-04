//! Core domain entities for Barnacle.
//!
//! These types represent games, profiles, mods, and other elements managed by
//! the system. They provide a unified interface for inspecting and mutating
//! these elements, handling all necessary operations behind the scenes.

use std::{fmt::Debug, path::PathBuf};

use sea_orm::{DbErr, SqlErr};
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
    Db(#[from] sea_orm::DbErr),

    #[error("The referenced object no longer exists")]
    StaleHandle,

    #[error("No matching object was found")]
    NotFound,

    #[error("The application state singleton is missing")]
    StateMissing,

    #[error("The give profile doesn't not belong to the active game")]
    ProfileNotInActiveGame,

    #[error("Error while attempting to read the archive: {0}")]
    Archive(#[from] compress_tools::Error),

    #[error("The given archive is empty: {0}")]
    EmptyArchive(PathBuf),

    #[error("The given path does not point to an archive: {0}")]
    InvalidArchivePath(PathBuf),

    #[error("A game named '{0}' already exists")]
    DuplicateGameName(String),

    #[error("A profile named '{0}' already exists for this game")]
    DuplicateProfileName(String),

    #[error("A mod named '{0}' already exists for this game")]
    DuplicateModName(String),

    #[error("This profile already contains that mod")]
    DuplicateModEntry,
}

fn map_insert_error(err: DbErr, duplicate_error: Error) -> Error {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => duplicate_error,
        _ => err.into(),
    }
}
