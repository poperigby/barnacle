//! Core domain entities for Barnacle.
//!
//! These types represent games, profiles, mods, and other elements managed by
//! the system. They provide a unified interface for inspecting and mutating
//! these elements, handling all necessary operations behind the scenes.

use agdb::{DbId, DbValue, QueryBuilder};
use thiserror::Error;

use crate::repository::db::DbHandle;

pub mod game;
pub mod mod_;
pub mod mod_entry;
pub mod profile;
pub mod tool;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to convert field {0}")]
    Conversion(String),
    #[error("Empty query elements")]
    EmptyElements,
    #[error("Internal database error {0}")]
    Internal(#[from] agdb::DbError),
}

pub(crate) fn get_field<T>(db: &DbHandle, id: DbId, field: &str) -> Result<T>
where
    T: TryFrom<DbValue>,
{
    db.read()
        .exec(QueryBuilder::select().values(field).ids(id).query())?
        .elements
        .pop()
        .expect("successful result values cannot be empty")
        .values
        .pop()
        .expect("successful result values cannot be empty")
        .value
        .try_into()
        .map_err(|_| Error::Conversion(field.into()))
}
