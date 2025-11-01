use std::{
    fs::{File, create_dir_all},
    io,
    path::Path,
};

use barnacle_db::Database;

use compress_tools::{Ownership, uncompress_archive};
use thiserror::Error;

use crate::fs::{Permissions, change_dir_permissions, data_dir, game_dir, mod_dir, profile_dir};

// mod deployers;
mod fs;
mod state;

pub use barnacle_db::{
    GameId, ModId, ProfileId, ProfileMod,
    models::{DeployKind, Game, Mod, Profile, Tool},
};

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
