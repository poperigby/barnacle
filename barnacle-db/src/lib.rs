use agdb::{DbError, DbId};
use derive_more::Display;
use thiserror::Error;

// Documentation imports
#[allow(unused_imports)]
use crate::models::{Game, Mod, Profile};

mod db;

pub mod models;

pub use db::Database;

type Result<T> = std::result::Result<T, Error>;

/// A handle to a [`Game`] in the database.
///
/// This serves as a parent context for game-specific profiles and mods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameCtx {
    pub(crate) id: DbId,
}

/// A handle to a [`Profile`] within a specific [`Game`].
///
/// This context ensures that profile operations are scoped to the correct game
/// and enables validation that profiles and mods belong to the same game before
/// linking them together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileCtx {
    pub(crate) id: DbId,
    pub(crate) game_id: DbId,
}

/// A handle to a [`Mod`] within a specific [`Game`].
///
/// This context ensures that mod operations are scoped to the correct game
/// and enables validation that mods and profiles belong to the same game before
/// linking them together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModCtx {
    pub(crate) id: DbId,
    pub(crate) game_id: DbId,
}

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] DbError),
    #[error("The given path is invalid unicode")]
    PathInvalidUnicode,
    #[error("Unique, constraint violated for: {0}")]
    UniqueViolation(UniqueConstraint),
}

#[derive(Debug, Display, PartialEq)]
pub enum UniqueConstraint {
    GameName,
    ProfileName,
}
