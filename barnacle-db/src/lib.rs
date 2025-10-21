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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameId(DbId);

/// A handle to a [`Profile`] within a specific [`Game`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileId(DbId);

/// A handle to a [`Mod`] within a specific [`Game`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModId(DbId);

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
