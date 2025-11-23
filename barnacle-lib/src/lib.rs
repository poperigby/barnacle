use std::io;

use thiserror::Error;

// mod deployers;
pub mod fs;

mod repository;

pub use barnacle_db::{
    GameId, ModId, ProfileId, ProfileMod,
    models::{DeployKind, Game, Mod, Profile, Tool},
};
pub use repository::Repository;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Archive error: {0}")]
    Archive(#[from] compress_tools::Error),
    #[error("Database error: {0}")]
    Db(#[from] barnacle_db::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}
