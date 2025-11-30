use std::io;

use thiserror::Error;

// mod deployers;
pub mod fs;
pub mod repository;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Archive error: {0}")]
    Archive(#[from] compress_tools::Error),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}
