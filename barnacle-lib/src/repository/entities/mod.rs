//! Core domain entities for Barnacle.
//!
//! These types represent games, profiles, mods, and other elements managed by
//! the system. They provide a unified interface for inspecting and mutating
//! these elements, handling all necessary operations behind the scenes.

use std::fmt::Debug;

use agdb::{DbId, DbValue, QueryBuilder};
use thiserror::Error;

use crate::repository::db::DbHandle;

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
    Internal(#[from] agdb::DbError),
    #[error("This entity refers to a model that has been deleted")]
    StaleEntity,
}

pub(crate) fn get_field<T>(db: &DbHandle, id: DbId, field: &str) -> Result<T>
where
    T: TryFrom<DbValue>,
    T::Error: Debug,
{
    let value = db
        .read()
        .exec(QueryBuilder::select().values(field).ids(id).query())?
        .elements
        .pop()
        .expect("A successful query should not be empty")
        .values
        .pop()
        .expect("A successful query should not be empty")
        .value;

    Ok(T::try_from(value)
        .expect("Conversion from a `DbValue` must succeed. Perhaps the wrong type was expected from this field."))
}

pub(crate) fn set_field<T>(db: &mut DbHandle, id: DbId, field: &str, value: T) -> Result<()>
where
    T: Into<DbValue>,
{
    db.write().exec_mut(
        QueryBuilder::insert()
            .values([[(field, value).into()]])
            .ids(id)
            .query(),
    )?;

    Ok(())
}
