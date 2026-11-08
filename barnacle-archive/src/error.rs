use std::{
    error::Error,
    ffi::{CStr, c_int},
    fmt::{self, Display, Formatter},
    io::{Read, Seek},
    panic::Location,
};

use tracing::warn;

use crate::{
    Archive,
    libarchive::{
        ARCHIVE_EOF, ARCHIVE_FAILED, ARCHIVE_FATAL, ARCHIVE_OK, ARCHIVE_RETRY, ARCHIVE_WARN,
        archive, archive_error_string,
    },
};

pub type Result<T> = std::result::Result<T, ArchiveError>;

/// Represents an error that can be returned from `libarchive` functions, converted from return codes.
#[derive(Debug, Clone)]
pub struct ArchiveError {
    kind: ErrorKind,
    details: String,
    location: &'static Location<'static>,
}

impl ArchiveError {
    fn from_handle(
        archive: *mut archive,
        return_code: i32,
        location: &'static Location<'static>,
    ) -> Self {
        Self {
            kind: return_code.into(),
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
            self.kind,
            self.location.line()
        )
    }
}

impl Error for ArchiveError {}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    Retry,
    Failed,
    Fatal,
    Unknown(c_int),
}

impl From<i32> for ErrorKind {
    fn from(value: i32) -> Self {
        match value {
            ARCHIVE_RETRY => Self::Retry,
            ARCHIVE_FAILED => Self::Failed,
            // TODO: We need to close the archive if this happens
            ARCHIVE_FATAL => Self::Fatal,
            unknown => Self::Unknown(unknown),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let return_code = match *self {
            Self::Retry => ARCHIVE_RETRY,
            Self::Failed => ARCHIVE_FAILED,
            Self::Fatal => ARCHIVE_FATAL,
            Self::Unknown(n) => n,
        };

        write!(f, "{}", return_code)
    }
}

/// Supports turning libarchive return codes into [`ArchiveError`]
pub trait ReturnCode: Sized {
    type Output;

    /// Check the return code of the `libarchive` function, and returned an error if there's a
    /// negative value.
    ///
    /// Safety: `archive` must point to a valid [`archive`] in memory.
    fn check_from_ptr(self, archive: *mut archive) -> Result<Self::Output>;

    /// Check the return code of the `libarchive` function, and returned an error if there's a
    /// negative value.
    #[track_caller]
    fn check<R: Read + Seek>(self, archive: &Archive<R>) -> Result<Self::Output> {
        self.check_from_ptr(archive.handle.as_ptr())
    }
}

// This value is used by `libarchive` for pure return codes
impl ReturnCode for i32 {
    type Output = Status;

    #[track_caller]
    fn check_from_ptr(self, archive: *mut archive) -> Result<Status> {
        let location = Location::caller();
        match self {
            ARCHIVE_OK => Ok(Status::Ok),
            ARCHIVE_EOF => Ok(Status::Eof),
            ARCHIVE_WARN => {
                let details = error_details(archive);
                warn!("{}", details);

                Ok(Status::Warning)
            }
            other => Err(ArchiveError::from_handle(archive, other, location)),
        }
    }
}

// This value is used by `libarchive` for return codes, or an amount of bytes read
impl ReturnCode for isize {
    type Output = isize;

    #[track_caller]
    fn check_from_ptr(self, archive: *mut archive) -> Result<Self::Output> {
        let location = Location::caller();

        if self < 0 {
            Err(ArchiveError::from_handle(archive, self as i32, location))
        } else {
            Ok(self)
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
