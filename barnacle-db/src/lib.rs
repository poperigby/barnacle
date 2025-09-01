use agdb::{DbError, DbId};
use derive_more::Display;
use thiserror::Error;

mod db;

pub mod models;

pub use db::Database;

type Result<T> = std::result::Result<T, DatabaseError>;

/// ID pointing to a [`models::games::Game`] in the database
#[derive(Debug)]
pub struct GameId(DbId);

/// ID pointing to a [`models::profiles::Profile`] in the database
#[derive(Debug)]
pub struct ProfileId(DbId);

/// ID pointing to a [`models::mods::Mod`] in the database
#[derive(Debug)]
pub struct ModId(DbId);

#[derive(Debug, Error, PartialEq)]
pub enum DatabaseError {
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
