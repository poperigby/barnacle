use std::{
    ffi::CString,
    os::unix::ffi::OsStrExt,
    path::Path,
    ptr::{self, NonNull},
};

use crate::{
    error::{Errno, Result, Status},
    libarchive::{
        archive, archive_read_free, archive_read_new, archive_read_next_header,
        archive_read_open_filename, archive_read_support_filter_all,
        archive_read_support_format_all,
    },
};

mod error;
mod libarchive;

pub struct Archive {
    handle: NonNull<archive>,
}

impl Archive {
    pub fn open(path: &Path) -> Result<Self> {
        let filename = CString::new(path.as_os_str().as_bytes())
            .expect("real Unix paths never have NUL bytes");

        let handle = unsafe {
            let a = archive_read_new();
            archive_read_support_filter_all(a).check(a)?;
            archive_read_support_format_all(a).check(a)?;

            archive_read_open_filename(a, filename.as_ptr(), 10240).check(a)?;

            a
        };

        Ok(Self {
            handle: NonNull::new(handle).expect("libarchive won't give us a null pointer"),
        })
    }

    pub fn next_entry(&'_ mut self) -> Result<Option<ArchiveEntry<'_>>> {
        let ptr = ptr::null_mut();

        let status = unsafe {
            archive_read_next_header(self.handle.as_ptr(), ptr).check(self.handle.as_ptr())?
        };

        match status {
            Status::Ok | Status::Warning => Ok(Some(ArchiveEntry { archive: self })),
            Status::Eof => Ok(None),
        }
    }
}

impl Drop for Archive {
    fn drop(&mut self) {
        unsafe {
            // This can return an error code if the internal archive_read_close fails, but we just ignore
            // it. There's nothing useful that can be done with it, as the resources are already gone.
            archive_read_free(self.handle.as_ptr());
        }
    }
}

pub struct ArchiveEntry<'a> {
    archive: &'a mut Archive,
}
