use std::{
    ffi::{CString, c_void},
    io::{self, Read, Seek},
};

use crate::{
    State,
    libarchive::{self, archive_set_error},
};

/// The read callback is invoked whenever the library requires raw bytes from
/// the archive. It should read data into a buffer, set the `const void **buffer` argument
/// to point to the available data, and return a count of the number of bytes available.
///
/// The library will invoke the read callback again only after it has consumed this data.
/// The library imposes no constraints on the size of the data blocks returned.
///
/// On end-of-file, the read callback should return zero. On error, the read callback should
/// invoke `archive_set_error()` to register an error code and message and return -1.
pub extern "C" fn read<R: Read + Seek>(
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

pub extern "C" fn seek<R: Read + Seek>() {}
