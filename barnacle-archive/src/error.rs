use std::{
    error::Error,
    ffi::{CStr, c_int},
    fmt::{self, Display, Formatter},
    panic::Location,
};

use tracing::warn;

use crate::libarchive::{
    ARCHIVE_EOF, ARCHIVE_FAILED, ARCHIVE_FATAL, ARCHIVE_OK, ARCHIVE_RETRY, ARCHIVE_WARN, archive,
    archive_error_string,
};

pub type Result<T> = std::result::Result<T, ArchiveError>;

#[derive(Debug, Clone)]
pub struct ArchiveError {
    kind: ErrorKind,
    errno: c_int,
    details: String,
    location: &'static Location<'static>,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    Retry,
    Failed,
    Fatal,
    Unknown(c_int),
}

impl ArchiveError {
    fn from_handle(
        archive: *mut archive,
        errno: i32,
        location: &'static Location<'static>,
    ) -> Self {
        let kind = match errno {
            ARCHIVE_RETRY => ErrorKind::Retry,
            ARCHIVE_FAILED => ErrorKind::Failed,
            // TODO: We need to close the archive if this happens
            ARCHIVE_FATAL => ErrorKind::Fatal,
            unknown => ErrorKind::Unknown(unknown),
        };

        Self {
            kind,
            errno,
            details: error_details(archive),
            location,
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl Display for ArchiveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) at {}",
            self.details,
            self.errno,
            self.location.line()
        )
    }
}

impl Error for ArchiveError {}

/// Supports turning libarchive error codes into [`ArchiveError`]
pub trait Errno {
    type Output;

    /// Check the returned error code of the `libarchive` function, and returned an error if there's a
    /// negative value.
    ///
    /// Safety: `archive` must point to a valid [`archive`] in memory.
    fn check(self, archive: *mut archive) -> Result<Self::Output>;
}

impl Errno for c_int {
    type Output = Status;

    #[track_caller]
    fn check(self, archive: *mut archive) -> Result<Status> {
        let location = Location::caller();
        match self {
            ARCHIVE_OK => Ok(Status::Ok),
            ARCHIVE_EOF => Ok(Status::Eof),
            ARCHIVE_WARN => {
                let details = error_details(archive);
                warn!("{}", details);

                Ok(Status::Warning)
            }
            errno => Err(ArchiveError::from_handle(archive, errno, location)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Ok,
    Warning,
    Eof,
}

fn error_details(archive: *mut archive) -> String {
    let error_string = unsafe { CStr::from_ptr(archive_error_string(archive)).to_owned() };
    error_string
        .to_str()
        .expect("libarchive error strings are valid UTF-8")
        .to_owned()
}
