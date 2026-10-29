use std::{
    fs::File,
    io::{self, Read, Seek},
    path::Path,
};

use compress_tools::{Ownership, uncompress_archive};
use thiserror::Error;

use crate::progress::ProgressReader;

pub use crate::progress::Progress;

mod progress;

/// Extract the `archive` to `destination`. Whenever extraction progresses, `on_progress` is called.
/// Archive ownership is ignored.
pub fn extract(
    archive: &Path,
    destination: &Path,
    on_progress: impl Fn(Progress) + 'static,
) -> Result<(), ExtractError> {
    let mut file = File::open(archive).map_err(ExtractError::Read)?;

    let mut header = Vec::with_capacity(8);
    let _ = file
        .by_ref()
        .take(8)
        .read_to_end(&mut header)
        .map_err(ExtractError::Read)?;
    if !infer::is_archive(&header) {
        return Err(ExtractError::NotAnArchive);
    }

    file.rewind().map_err(ExtractError::Read)?;

    let reader = ProgressReader::create(file, on_progress).map_err(ExtractError::Read)?;

    uncompress_archive(reader, destination, Ownership::Ignore)?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("failed to read the given file: {0}")]
    Read(#[source] io::Error),
    #[error("the given file is not an archive")]
    NotAnArchive,
    #[error("failed to uncompress the given archive: {0}")]
    Uncompress(#[from] compress_tools::Error),
}
