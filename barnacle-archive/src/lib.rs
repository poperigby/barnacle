use std::{
    ffi::{CString, c_void},
    io::{Read, Seek},
    ptr::{self, NonNull},
};

use crate::{
    error::{Errno, Result, Status},
    libarchive::{
        archive, archive_entry, archive_read_free, archive_read_new, archive_read_next_header,
        archive_read_open, archive_read_support_filter_all, archive_read_support_format_all,
        archive_set_error,
    },
};

mod error;
mod libarchive;

pub struct Archive<R>
where
    R: Read + Seek,
{
    handle: NonNull<archive>,
    state: NonNull<State<R>>,
}

struct State<R: Read + Seek> {
    reader: R,
    buffer: Vec<u8>,
}

impl<R> Archive<R>
where
    R: Read + Seek,
{
    pub fn open(source: R) -> Result<Self> {
        let state = NonNull::new(Box::into_raw(Box::new(State {
            reader: source,
            buffer: Vec::new(),
        })))
        .expect("the pointer to the newly allocated State will not be null");

        let handle = unsafe {
            let a = archive_read_new();
            archive_read_support_filter_all(a).check(a)?;
            archive_read_support_format_all(a).check(a)?;

            archive_read_open(
                a,
                state.as_ptr().cast(),
                None,
                Some(read_callback::<R>),
                None,
            )
            .check(a)?;

            a
        };

        Ok(Self {
            handle: NonNull::new(handle).expect("libarchive won't give us a null pointer"),
            state,
        })
    }

    pub fn next_entry(&'_ mut self) -> Result<Option<ArchiveEntry<'_, R>>> {
        let mut entry: *mut archive_entry = ptr::null_mut();

        let status = unsafe {
            archive_read_next_header(self.handle.as_ptr(), &mut entry)
                .check(self.handle.as_ptr())?
        };

        match status {
            Status::Ok | Status::Warning => Ok(Some(ArchiveEntry { archive: self })),
            Status::Eof => Ok(None),
        }
    }
}

impl<R> Drop for Archive<R>
where
    R: Read + Seek,
{
    fn drop(&mut self) {
        unsafe {
            // This can return an error code if the internal archive_read_close fails, but we just ignore
            // it. There's nothing useful that can be done with it, as the resources are already gone.
            archive_read_free(self.handle.as_ptr());

            drop(Box::from_raw(self.state.as_ptr()));
        }
    }
}

/// The read callback is invoked whenever the library requires raw bytes from
/// the archive. The read callback should read data into a buffer, set the
/// `const void **buffer` argument to point to the available data, and return a
/// count of the number of bytes available. The library will invoke the read
/// callback again only after it has consumed this data. The library imposes
/// no constraints on the size of the data blocks returned. On end-of-file,
/// the read callback should return zero. On error, the read callback should
/// invoke `archive_set_error()` to register an error code and message and return -1.
extern "C" fn read_callback<R: Read + Seek>(
    archive: *mut archive,
    state: *mut c_void,
    buffer: *mut *const c_void,
) -> isize {
    let state = unsafe { &mut *state.cast::<State<R>>() };

    match state.reader.read_to_end(&mut state.buffer) {
        Err(e) => {
            let error_code = e.raw_os_error().unwrap_or(-1);
            let error_string =
                CString::new(e.to_string()).expect("an error string will not have NUL bytes");

            unsafe {
                archive_set_error(archive, error_code, error_string.as_ptr());
            };

            -1
        }
        Ok(0) => 0,
        Ok(_) => {
            unsafe { *buffer = state.buffer.as_mut_ptr().cast() };

            state.buffer.len().cast_signed()
        }
    }
}

pub struct ArchiveEntry<'a, R>
where
    R: Read + Seek,
{
    archive: &'a mut Archive<R>,
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Write};

    use tempfile::tempfile;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::Archive;

    fn memory() -> Archive<Cursor<Vec<u8>>> {
        let mut zip = ZipWriter::new(Cursor::new(vec![]));
        zip.add_directory("test/", SimpleFileOptions::default())
            .unwrap();

        let lorem = "Aut quia quam culpa hic. Quas labore ipsa magnam ut. Ad perferendis excepturi iure ratione accusamus numquam.";
        zip.start_file("test/lorem.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(lorem.as_bytes()).unwrap();

        let buffer = zip.finish().unwrap();

        Archive::open(buffer).unwrap()
    }

    #[test]
    fn open_memory() {
        memory();
    }

    #[test]
    fn open_file() {
        let mut zip = ZipWriter::new(tempfile().unwrap());
        zip.add_directory("test/", SimpleFileOptions::default())
            .unwrap();

        let lorem = "Aut quia quam culpa hic. Quas labore ipsa magnam ut. Ad perferendis excepturi iure ratione accusamus numquam.";
        zip.start_file("test/lorem.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(lorem.as_bytes()).unwrap();

        let file = zip.finish().unwrap();

        Archive::open(file).unwrap();
    }
}
