use std::{
    ffi::CStr,
    io::{self, Read, Seek},
    path::PathBuf,
    ptr::NonNull,
};

use thiserror::Error;

use crate::{
    Archive,
    error::ReturnCode,
    libarchive::{
        self, AE_IFBLK, AE_IFCHR, AE_IFDIR, AE_IFIFO, AE_IFLNK, AE_IFREG, AE_IFSOCK,
        archive_entry_filetype, archive_entry_pathname, archive_entry_pathname_utf8,
        archive_entry_pathname_w, archive_entry_size, archive_read_data,
    },
    locale::Utf8LocaleGuard,
};

/// # Invariants
///
/// `handle` is non-null and was returned by `archive_read_next_header` on `archive`.
/// Because we hold `&'a mut Archive`, the entry (and every buffer libarchive hands out
/// from it) stays valid for `'a`: no next-header read or `archive_read_free` can happen.
/// `ArchiveEntry` is `!Sync`, so libarchive's internal caching on the entry is never
/// accessed concurrently.
pub struct ArchiveEntry<'a, R>
where
    R: Read + Seek,
{
    handle: NonNull<libarchive::archive_entry>,
    archive: &'a mut Archive<R>,
}

impl<'a, R> ArchiveEntry<'a, R>
where
    R: Read + Seek,
{
    pub(crate) fn new(handle: *mut libarchive::archive_entry, archive: &'a mut Archive<R>) -> Self {
        Self {
            handle: NonNull::new(handle)
                .expect("libarchive just wrote to our pointer, so it won't be null"),
            archive,
        }
    }

    /// Returns a relative path with `/` separators, and no root or drive prefixes. It may still
    /// return a path with `..`, so this will need to be checked against an extraction destination directory.
    pub fn path(&self) -> Result<PathBuf, PathError> {
        let _utf8_guard = Utf8LocaleGuard::new();

        // SAFETY: `handle` is valid per the type invariants.
        let ptr = unsafe {
            // This will attempt to convert to UTF8, and return null if the conversion isn't
            // possible, or if there isn't a pathname.
            archive_entry_pathname_utf8(self.handle.as_ptr())
        };

        if ptr.is_null() {
            // Check if this is a UTF8 conversion error or a missing path.
            // archive_entry_pathname{_w} will also return null if it's actually missing.

            // SAFETY: `handle` is valid per the type invariants.
            return Err(
                if unsafe { archive_entry_pathname(self.handle.as_ptr()) }.is_null()
                    && unsafe { archive_entry_pathname_w(self.handle.as_ptr()) }.is_null()
                {
                    PathError::MissingPath
                } else {
                    PathError::InvalidEncoding
                },
            );
        }

        // SAFETY: We just checked `ptr` is non-null, libarchive gives us a nul-terminated string,
        // and we immediately copy the slice into a new string with the `.replace()`, so we are no
        // longer pointing to the libarchive entry's internal buffer.
        let s = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .map_err(|_| PathError::InvalidEncoding)?
            // This would technically falsely rename legitimate Linux files that contain `\`, but no
            // sane file will do that.
            //
            // This creates a new `String` and copies the data, so we don't have to worry about `s`
            // pointing to possibly invalid data.
            .replace('\\', "/");

        if s.is_empty() {
            return Err(PathError::Empty);
        }

        if s.starts_with('/') {
            return Err(PathError::ContainsRoot);
        }

        // We have to parse for a prefix ourselves, because `Path` doesn't recognize something like
        // `C:/` as a prefix on Linux
        let mut chars = s.chars();
        if matches!((chars.next(), chars.next()), (Some(c), Some(':')) if c.is_ascii_alphabetic()) {
            return Err(PathError::ContainsPrefix);
        }

        Ok(PathBuf::from(s))
    }

    pub fn size(&self) -> Option<i64> {
        // SAFETY: `handle` is valid per the type invariants.
        let size = unsafe { archive_entry_size(self.handle.as_ptr()) };

        match size {
            // The file size wasn't set
            0 => None,
            n => Some(n),
        }
    }

    pub fn file_type(&self) -> FileType {
        // SAFETY: `handle` is valid per the type invariants.
        let file_type = unsafe { archive_entry_filetype(self.handle.as_ptr()) };

        match file_type {
            AE_IFREG => FileType::Regular,
            AE_IFLNK => FileType::SymbolicLink,
            AE_IFSOCK => FileType::Socket,
            AE_IFCHR => FileType::CharacterDevice,
            AE_IFBLK => FileType::BlockDevice,
            AE_IFDIR => FileType::Directory,
            AE_IFIFO => FileType::NamedPipe,
            other => FileType::Unknown(other),
        }
    }
}

impl<'a, R> Read for ArchiveEntry<'a, R>
where
    R: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // SAFETY: `handle` is valid per the type invariants, we pass in a pointer to a buffer
        // that is valid for `'a`, and libarchive won't read past the end of the buffer, as we gave
        // it the length.
        let bytes_read = unsafe {
            archive_read_data(
                self.archive.handle.as_ptr(),
                buf.as_mut_ptr().cast(),
                buf.len(),
            )
        }
        .check(self.archive)
        .map_err(io::Error::other)?;

        Ok(bytes_read.try_into().expect(
            "bytes_read won't be negative, because negative return values are converted to errors",
        ))
    }
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("this entry doesn't list a path in its header")]
    MissingPath,
    #[error("libarchive failed to convert the path to UTF8")]
    InvalidEncoding,
    #[error("this path is empty")]
    Empty,
    #[error("this path contains a root component (e.g. '/')")]
    ContainsRoot,
    #[error("this path contains a prefix component (e.g. 'C:/')")]
    ContainsPrefix,
}

#[derive(Debug, PartialEq)]
pub enum FileType {
    Regular,
    SymbolicLink,
    Socket,
    CharacterDevice,
    BlockDevice,
    Directory,
    NamedPipe,
    Unknown(u32),
}
