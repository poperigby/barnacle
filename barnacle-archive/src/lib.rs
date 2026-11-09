use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
    ptr::{self, NonNull},
};

use nix::fcntl::{RenameFlags, renameat2};
use tempfile::tempdir_in;

use crate::{
    callback::read_callback,
    entry::{ArchiveEntry, FileType},
    error::{Result, ReturnCode, Status},
    libarchive::{
        archive_read_free, archive_read_new, archive_read_next_header, archive_read_open,
        archive_read_support_filter_all, archive_read_support_format_all,
    },
};

mod callback;
mod entry;
mod error;
mod libarchive;

/// # Invariants
///
/// `handle` is non-null and was returned by [`archive_read_new`]. It is valid for the lifetime of [`Archive`],
/// as it is only freed on [`drop`].
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
        // SAFETY: `archive_read_new` takes no arguments and has no preconditions.
        // It only allocates and initializes a fresh `archive`. It may return
        // NULL on allocation failure, which is checked below before the pointer is used.
        // That's very unlikely to happen, though.
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

        // SAFETY: `handle` is valid per the type invariants.
        unsafe {
            archive_read_support_filter_all(handle).check_from_ptr(handle)?;
            archive_read_support_format_all(handle).check_from_ptr(handle)?;
        };

        // SAFETY: `handle` is valid per the type invariants. The pointer to `state` will be valid for
        // the lifetime of `Archive`, as it only deallocated when `Archive` is dropped.
        unsafe {
            archive_read_open(
                handle,
                state.as_ptr().cast(),
                None,
                Some(read_callback::<R>),
                None,
            )
        }
        .check_from_ptr(handle)?;

        archive
    }

    /// Get the next entry in the archive, if there is one. Calling this will drop the previous entry.
    pub fn next_entry(&mut self) -> Result<Option<ArchiveEntry<'_, R>>> {
        let mut entry: *mut libarchive::archive_entry = ptr::null_mut();

        // SAFETY: `handle` is valid per the type invariants. We're passing in a newly created null
        // pointer that libarchive will assign to the next `entry`.
        let status =
            unsafe { archive_read_next_header(self.handle.as_ptr(), &mut entry).check(self)? };

        // We take a mutable reference to the archive, because calling `archive_read_next_header`
        // moves the inner cursor that points to the current entry, which would invalidate
        // previous entries. We want the caller to only be able to hold onto one entry at a time.
        match status {
            Status::Ok | Status::Warning => Ok(Some(ArchiveEntry::new(entry, self))),
            Status::Eof => Ok(None),
        }
    }

    // TODO: Perhaps pass in a library specific staging directory to this function
    pub fn extract(&mut self, destination: &Path) -> Result<()> {
        assert!(destination.is_dir());

        // Extract into a temporary directory alongside the destination (same filesystem),
        // then atomically swap it into place. If extraction fails, the temporary directory is
        // discarded, so the destination never contains a partially extracted archive.
        let staging_dir = tempdir_in(destination.parent().unwrap()).unwrap();

        // TODO: Create a budget of (free drive space - max(10GB,5%free)).
        // Keep a running byte counter, and terminate if we exceed the budget.
        while let Some(entry) = self.next_entry()? {
            if !matches!(entry.file_type(), FileType::Regular | FileType::Directory) {
                continue;
            }

            let path = match entry.path() {
                Some(path) => path,
                None => continue,
            };

            let destination_path = destination.join(path_clean::clean(path));

            // TODO: Ensure entry:
            // * Would not escape destination
            // * Reject entries where: compression ratio > 100:1 && output > 100MB. This will help to
            // prevent zip bombs where there's an absurd compression ratio, but not falsely flag
            // small files where large compression ratios are normal.
        }

        renameat2(
            File::open(staging_dir.path()).unwrap(),
            staging_dir.path(),
            File::open(destination).unwrap(),
            destination,
            RenameFlags::RENAME_EXCHANGE,
        )
        .unwrap();

        Ok(())
    }
}

impl<R> Drop for Archive<R>
where
    R: Read + Seek,
{
    fn drop(&mut self) {
        // SAFETY:
        // - `handle` points to a live `archive` allocated with `archive_read_new`. It is only every freed here.
        // - `state` was originally allocated by a `Box::into_raw`, so it follows the memory layout of `Box`.
        //    It is only ever freed here.
        unsafe {
            // This can return an error code if the internal archive_read_close fails, but we just ignore
            // it. There's nothing useful that can be done with it, as the resources are already gone.
            archive_read_free(self.handle.as_ptr());

            drop(Box::from_raw(self.state.as_ptr()));
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{Cursor, Read, Seek, Write},
        path::PathBuf,
    };

    use tempfile::tempfile;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::{Archive, entry::FileType};

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
        let mut archive = Archive::open(write_zip(source)).unwrap();

        let _test_dir = archive.next_entry().unwrap().unwrap();
        let mut lorem_txt = archive.next_entry().unwrap().unwrap();

        assert_eq!(lorem_txt.path(), Some(PathBuf::from("test/lorem.txt")));
        assert_eq!(lorem_txt.size(), Some(LOREM.len().cast_signed() as i64));
        assert_eq!(lorem_txt.file_type(), FileType::Regular);

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
