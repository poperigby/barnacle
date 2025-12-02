use std::io;

use thiserror::Error;

use crate::repository::entities;

// mod deployers;
pub mod fs;
pub mod repository;

pub use repository::Repository;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Archive error: {0}")]
    Archive(#[from] compress_tools::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Entity error: {0}")]
    Entity(#[from] entities::Error),
}
