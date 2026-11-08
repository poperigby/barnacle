use std::{
    ffi::{CString, c_void},
    io::{self, Read, Seek},
    ptr::{self, NonNull},
};

use crate::{
    error::{Result, ReturnCode, Status},
    libarchive::{
        archive_read_data, archive_read_free, archive_read_new, archive_read_next_header,
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
    handle: NonNull<libarchive::archive>,
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
        let handle = unsafe { archive_read_new() };

        let state = NonNull::new(Box::into_raw(Box::new(State {
            reader: source,
            // Allocate a fixed size buffer so we don't read the whole stream into memory
            buffer: vec![0; 64 * 1024],
        })))
        .expect("the pointer to the newly allocated State will not be null");

        let archive = Ok(Self {
            handle: NonNull::new(handle).expect("libarchive won't give us a null pointer"),
            state,
        });

        unsafe {
            archive_read_support_filter_all(handle).check_from_ptr(handle)?;
            archive_read_support_format_all(handle).check_from_ptr(handle)?;

            archive_read_open(
                handle,
                state.as_ptr().cast(),
                None,
                Some(read_callback::<R>),
                None,
            )
            .check_from_ptr(handle)?;
        };

        archive
    }

    pub fn next_entry(&self) -> Result<Option<ArchiveEntry<'_, R>>> {
        let mut entry: *mut libarchive::archive_entry = ptr::null_mut();

        let status =
            unsafe { archive_read_next_header(self.handle.as_ptr(), &mut entry).check(self)? };

        match status {
            Status::Ok | Status::Warning => Ok(Some(ArchiveEntry {
                handle: NonNull::new(entry)
                    .expect("libarchive just wrote to our pointer, so it won't be null"),
                archive: self,
            })),
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

pub struct ArchiveEntry<'a, R>
where
    R: Read + Seek,
{
    handle: NonNull<libarchive::archive_entry>,
    archive: &'a Archive<R>,
}

impl<'a, R> Read for ArchiveEntry<'a, R>
where
    R: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
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

/// The read callback is invoked whenever the library requires raw bytes from
/// the archive. It should read data into a buffer, set the `const void **buffer` argument
/// to point to the available data, and return a count of the number of bytes available.
///
/// The library will invoke the read callback again only after it has consumed this data.
/// The library imposes no constraints on the size of the data blocks returned.
///
/// On end-of-file, the read callback should return zero. On error, the read callback should
/// invoke `archive_set_error()` to register an error code and message and return -1.
extern "C" fn read_callback<R: Read + Seek>(
    archive: *mut libarchive::archive,
    state: *mut c_void,
    buffer: *mut *const c_void,
) -> isize {
    let state = unsafe { &mut *state.cast::<State<R>>() };

    loop {
        match state.reader.read(&mut state.buffer) {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                continue;
            }
            Err(e) => {
                let error_code = e.raw_os_error().unwrap_or(-1);
                let error_string =
                    CString::new(e.to_string()).expect("an error string will not have NUL bytes");

                unsafe {
                    archive_set_error(archive, error_code, error_string.as_ptr());
                };

                return -1;
            }
            Ok(n) if n > state.buffer.len() => {
                // We hit an incorrect `Read` implementation >:(
                // https://doc.rust-lang.org/src/alloc/io/read.rs.html#115

                let error_string = CString::new(
                    "The amount of bytes read was greater than the size of the buffer.\n
                    This indicates an incorrect implementation of Read.",
                )
                .expect("the string contains no NUL bytes");

                unsafe {
                    archive_set_error(archive, -1, error_string.as_ptr());
                };

                return -1;
            }
            Ok(0) => return 0,
            Ok(n) => {
                unsafe { *buffer = state.buffer.as_mut_ptr().cast() };

                return n
                    .try_into()
                    // https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety
                    .expect("Rust guarantees slices can be no larger than isize::MAX");
            }
        };
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read, Seek, Write};

    use tempfile::tempfile;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::Archive;

    const LOREM: &str = "Aut quia quam culpa hic. Quas labore ipsa magnam ut. Ad perferendis excepturi iure ratione accusamus numquam.";

    fn write_zip<W: Write + Seek>(writer: W) -> W {
        let options = SimpleFileOptions::default();
        let mut zip = ZipWriter::new(writer);

        zip.add_directory("test/", options).unwrap();
        zip.start_file("test/lorem.txt", options).unwrap();
        zip.write_all(LOREM.as_bytes()).unwrap();

        let mut writer = zip.finish().unwrap();
        writer.rewind().unwrap();
        writer
    }

    fn assert_reads_lorem<S: Read + Write + Seek>(source: S) {
        let archive = Archive::open(write_zip(source)).unwrap();

        let _test_dir = archive.next_entry().unwrap().unwrap();
        let mut lorem_txt = archive.next_entry().unwrap().unwrap();

        let mut contents = String::new();
        lorem_txt.read_to_string(&mut contents).unwrap();

        assert_eq!(contents, LOREM);
        assert!(archive.next_entry().unwrap().is_none());
    }

    #[test]
    fn read_lorem_memory() {
        assert_reads_lorem(Cursor::new(vec![]));
    }

    #[test]
    fn read_lorem_file() {
        assert_reads_lorem(tempfile().unwrap());
    }
}
