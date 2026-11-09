use std::{
    ffi::CStr,
    io::{self, Read, Seek},
    path::PathBuf,
    ptr::NonNull,
};

use crate::{
    Archive,
    error::ReturnCode,
    libarchive::{
        self, AE_IFBLK, AE_IFCHR, AE_IFDIR, AE_IFIFO, AE_IFLNK, AE_IFREG, AE_IFSOCK,
        archive_entry_filetype, archive_entry_pathname_utf8, archive_entry_size, archive_read_data,
    },
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

    pub fn path(&self) -> Option<PathBuf> {
        // SAFETY: `handle` is valid per the type invariants.
        let ptr = unsafe {
            // This will attempt to convert to UTF8, and return NULL if the conversion isn't
            // possible, or if there isn't a pathname.
            archive_entry_pathname_utf8(self.handle.as_ptr())
        };

        if ptr.is_null() {
            return None;
        }

        // SAFETY: We just checked `ptr` is non-null, libarchive gives us a nul-terminated string,
        // and we immediately copy the slice into a new string with the `.replace()`, so we are no
        // longer pointing to the libarchive entry's internal buffer.
        let s = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .ok()?
            // Creates a new `String` and copies the data
            .replace('\\', "/");

        Some(PathBuf::from(s))
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
