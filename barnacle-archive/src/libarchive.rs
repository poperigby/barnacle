#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    use std::{
        ffi::{CStr, CString, c_void},
        ptr,
    };

    use libc::S_IFREG;

    use super::{
        ARCHIVE_OK, archive, archive_entry_free, archive_entry_new, archive_entry_set_filetype,
        archive_entry_set_pathname, archive_entry_set_perm, archive_entry_set_size,
        archive_error_string, archive_read_data, archive_read_new, archive_read_next_header,
        archive_read_open_memory, archive_read_support_filter_all, archive_read_support_format_all,
        archive_write_add_filter_gzip, archive_write_close, archive_write_data, archive_write_free,
        archive_write_header, archive_write_new, archive_write_open_memory,
        archive_write_set_format_pax_restricted,
    };

    #[test]
    fn round_trip_compression_decompression() {
        unsafe {
            let input = "Aut quia quam culpa hic. Quas labore ipsa magnam ut. Ad perferendis excepturi iure ratione accusamus numquam.";

            // Write a new archive
            let archive = archive_write_new();
            archive_write_add_filter_gzip(archive);
            archive_write_set_format_pax_restricted(archive);

            let mut output = vec![0u8; 256];
            let mut used: usize = 0;
            archive_write_open_memory(
                archive,
                output.as_mut_ptr() as *mut c_void,
                output.len(),
                &raw mut used,
            );

            let entry = archive_entry_new();
            archive_entry_set_pathname(entry, CString::new("lorem.txt").unwrap().as_ptr());
            archive_entry_set_size(entry, input.len().try_into().unwrap());
            archive_entry_set_filetype(entry, S_IFREG);
            archive_entry_set_perm(entry, 0o0644);

            check(archive, archive_write_header(archive, entry));
            // archive_write_data returns the number of bytes written on success
            assert_eq!(
                archive_write_data(archive, input.as_ptr() as *const c_void, input.len()),
                input.len().try_into().unwrap()
            );

            archive_entry_free(entry);
            check(archive, archive_write_close(archive));
            check(archive, archive_write_free(archive));

            // Read back the archive
            let archive = archive_read_new();
            archive_read_support_filter_all(archive);
            archive_read_support_format_all(archive);

            let mut buffer: Vec<u8> = vec![0u8; 256];
            check(
                archive,
                archive_read_open_memory(archive, output.as_ptr() as *const c_void, output.len()),
            );

            let mut entry = ptr::null_mut();
            archive_read_next_header(archive, &mut entry);

            archive_read_data(archive, buffer.as_mut_ptr() as *mut c_void, buffer.len());

            let s = String::from_utf8(buffer).unwrap();
            // there are some left over 0s from the end of the buffer
            let expected = s.trim_end_matches(char::from(0));

            assert_eq!(expected, input);
        }
    }

    fn check(archive: *mut archive, result: i32) {
        unsafe {
            if result != ARCHIVE_OK {
                let error = CStr::from_ptr(archive_error_string(archive))
                    .to_str()
                    .to_owned()
                    .unwrap();
                panic!("libarchive function failed: {}", error);
            };
        }
    }
}
