use std::{
    cmp::max,
    fs::{File, create_dir_all},
    io::{self, Read, Seek},
    path::Path,
    ptr::{self, NonNull},
};

use rustix::fs::{RenameFlags, renameat_with, statvfs};
use tempfile::tempdir_in;

use crate::{
    entry::{ArchiveEntry, FileType},
    error::{Result, ReturnCode, Status},
    libarchive::{
        archive_filter_bytes, archive_read_free, archive_read_new, archive_read_next_header,
        archive_read_open, archive_read_support_filter_all, archive_read_support_format_all,
    },
    locale::Utf8LocaleGuard,
};

mod callback;
mod entry;
mod error;
mod libarchive;
mod locale;

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
    /// The compressed length of the archive, in bytes.
    len: u64,
}

struct State<R: Read + Seek> {
    reader: R,
    buffer: Vec<u8>,
}

impl<R> Archive<R>
where
    R: Read + Seek,
{
    pub fn open(mut source: R) -> Result<Self> {
        let _ut8_guard = Utf8LocaleGuard::new();

        // SAFETY: `archive_read_new` takes no arguments and has no preconditions.
        // It only allocates and initializes a fresh `archive`. It may return
        // NULL on allocation failure, which is checked below before the pointer is used.
        // That's very unlikely to happen, though.
        let handle = unsafe { archive_read_new() };

        let len = source.seek(io::SeekFrom::End(0)).unwrap();
        source.rewind().unwrap();

        let state = NonNull::new(Box::into_raw(Box::new(State {
            reader: source,
            // Allocate a fixed size buffer so we don't read the whole stream into memory
            buffer: vec![0; 64 << 10],
        })))
        .expect("the pointer to the newly allocated State will not be null");

        let archive = Ok(Self {
            handle: NonNull::new(handle).expect("libarchive won't give us a null pointer"),
            state,
            len,
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
                Some(callback::read::<R>),
                None,
            )
        }
        .check_from_ptr(handle)?;

        archive
    }

    /// Extract this [`Archive`] to the given `destination`.
    ///
    /// # Errors
    ///
    /// This method will return an error in the following cases:
    /// * `destination` does not exist
    /// * `destination` is not a directory
    pub fn extract(&mut self, destination: &Path, on_progress: impl Fn(Progress)) -> Result<()> {
        let _utf8_guard = Utf8LocaleGuard::new();

        assert!(destination.is_dir());

        // Extract into a temporary directory alongside the destination (same filesystem),
        // then atomically swap it into place. If extraction fails, the temporary directory is
        // discarded, so the destination never contains a partially extracted archive.
        // TODO: Perhaps pass in a library specific staging directory to this function
        let staging_dir = tempdir_in(destination.parent().unwrap()).unwrap();

        let free = statvfs(destination)
            .map(|s| s.f_bsize * s.f_bavail)
            .unwrap();
        let budget = free - max(10u64 << 30, free / 20);

        let mut bytes_written = 0;
        while let Some(mut entry) = self.next_entry()? {
            if !matches!(entry.file_type(), FileType::Regular | FileType::Directory) {
                // TODO: Do we really want to silently continue here?
                continue;
            }

            let path = path_clean::clean(staging_dir.path().join(entry.path().unwrap()));
            if !path.starts_with(staging_dir.path()) {
                // We want to terminate extraction, because we can no longer trust this archive.
                panic!("ENTRY ESCAPES DESTINATION");
            }

            if entry.file_type() == FileType::Directory {
                create_dir_all(path).unwrap();
            } else if entry.file_type() == FileType::Regular {
                create_dir_all(path.parent().unwrap()).unwrap();

                let mut file = File::create(path).unwrap();

                let written = io::copy(&mut entry, &mut file).unwrap();

                bytes_written += written;
            }

            if self.bytes_read() > self.len {
                panic!("More compressed bytes were read from this archive than it contains. Hmmm")
            }

            if bytes_written > budget {
                panic!("We exceeded our budget. Uh oh")
            }

            on_progress(Progress {
                current: self.bytes_read(),
                total: self.len,
            });

            // TODO: Ensure entry:
            // * Reject entries where: compression ratio > 100:1 && output > 100MB. This will help to
            // prevent zip bombs where there's an absurd compression ratio, but not falsely flag
            // small files where large compression ratios are normal.
        }

        renameat_with(
            File::open(staging_dir.path()).unwrap(),
            staging_dir.path(),
            File::open(destination).unwrap(),
            destination,
            RenameFlags::EXCHANGE,
        )
        .unwrap();

        Ok(())
    }

    /// Get the next entry in the archive, if there is one. Calling this will drop the previous entry.
    fn next_entry(&mut self) -> Result<Option<ArchiveEntry<'_, R>>> {
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

    /// Returns the amount of compressed bytes that have been read from this archive.
    fn bytes_read(&self) -> u64 {
        // -1 is an alias for the highest numbered filter, which will be the pseudo-filter that
        // wraps the archive read functions. This gives us the amount of compressed bytes read.
        const RAW_INPUT_FILTER: i32 = -1;

        // SAFETY: `handle` is valid per the type invariants. It's OK to cast this to u64, as the
        // returned integer will never be negative.
        unsafe { archive_filter_bytes(self.handle.as_ptr(), RAW_INPUT_FILTER) }.cast_unsigned()
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

#[derive(Debug)]
pub struct Progress {
    current: u64,
    total: u64,
}

#[cfg(test)]
mod test {
    use std::{
        fs,
        io::{Cursor, Read, Seek, Write},
        path::{Path, PathBuf},
    };

    use tempfile::{tempdir, tempfile};
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    use crate::{Archive, entry::FileType};

    const LOREM: &str = "Aut quia quam culpa hic. Quas labore ipsa magnam ut. Ad perferendis excepturi iure ratione accusamus numquam.";

    struct TestFile {
        path: String,
        contents: Vec<u8>,
        compression: CompressionMethod,
    }

    /// Describes a zip archive to build for a test, so the test can check against it afterwards.
    struct TestArchive {
        /// Directories that get explicit entries. Paths end with `/`.
        dirs: Vec<String>,
        files: Vec<TestFile>,
    }

    impl TestArchive {
        fn lorem() -> Self {
            Self {
                dirs: vec!["test/".to_owned()],
                files: vec![TestFile {
                    path: "test/lorem.txt".to_owned(),
                    contents: LOREM.as_bytes().to_vec(),
                    compression: CompressionMethod::Deflated,
                }],
            }
        }

        /// A Bethesda-style mod with everything inside `root`, which is either empty or ends with `/`.
        ///
        /// It covers:
        /// * nested directories, only some of which have explicit entries
        /// * an empty directory
        /// * stored and deflated entries
        /// * binary content that isn't valid UTF-8
        /// * an empty file
        /// * files bigger than the 64 KiB read buffer
        /// * spaces and non-ASCII characters in names
        fn cool_mod(root: &str) -> Self {
            use CompressionMethod::{Deflated, Stored};

            // Pseudo-random bytes so deflate can't shrink them much
            let texture: Vec<u8> = (0..200_000u32)
                .map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
                .collect();

            // Very compressible, like a big mesh with lots of padding
            let padding = vec![0u8; 300_000];

            let files: Vec<(&str, Vec<u8>, CompressionMethod)> = vec![
                ("CoolMod.esp", b"TES4\x00\x01\xff\xfe".to_vec(), Stored),
                (
                    "CoolMod - Main.bsa",
                    b"BSA\x00\x68\x00\x00\x00".to_vec(),
                    Stored,
                ),
                ("meshes/armor/cool/cuirass.nif", padding, Deflated),
                ("textures/armor/cool/cuirass_d.dds", texture, Deflated),
                (
                    "textures/armor/cool/cuirass_n.dds",
                    b"DDS \x7c\x00".to_vec(),
                    Deflated,
                ),
                (
                    "scripts/source/CoolScript.psc",
                    b"Scriptname CoolScript extends Quest\n".to_vec(),
                    Deflated,
                ),
                (
                    "fomod/info.xml",
                    "<fomod><Name>Cool Mod — Édition</Name></fomod>".into(),
                    Deflated,
                ),
                ("fomod/ModuleConfig.xml", b"<config/>".to_vec(), Deflated),
                (
                    "Docs/Read Me.txt",
                    b"Install with a mod manager.\r\n".to_vec(),
                    Deflated,
                ),
                ("Docs/Änderungen.txt", "Größere Rüstung".into(), Deflated),
                ("Docs/empty.txt", Vec::new(), Stored),
            ];

            let mut dirs: Vec<String> = ["fomod/", "Docs/", "SKSE/Plugins/"]
                .into_iter()
                .map(|dir| format!("{root}{dir}"))
                .collect();
            if !root.is_empty() {
                dirs.insert(0, root.to_owned());
            }

            Self {
                dirs,
                files: files
                    .into_iter()
                    .map(|(path, contents, compression)| TestFile {
                        path: format!("{root}{path}"),
                        contents,
                        compression,
                    })
                    .collect(),
            }
        }

        /// Writes this archive into `writer`, and rewinds it so it's ready to be read.
        fn write<W: Write + Seek>(&self, writer: W) -> W {
            let mut zip = ZipWriter::new(writer);

            for dir in &self.dirs {
                zip.add_directory(dir, SimpleFileOptions::default())
                    .unwrap();
            }

            for file in &self.files {
                let options = SimpleFileOptions::default().compression_method(file.compression);
                zip.start_file(&file.path, options).unwrap();
                zip.write_all(&file.contents).unwrap();
            }

            let mut writer = zip.finish().unwrap();
            writer.rewind().unwrap();
            writer
        }

        fn open(&self) -> Archive<Cursor<Vec<u8>>> {
            Archive::open(self.write(Cursor::new(Vec::new()))).unwrap()
        }

        /// Asserts that `destination` contains exactly this archive's directories and files.
        fn assert_extracted_to(&self, destination: &Path) {
            for dir in &self.dirs {
                assert!(destination.join(dir).is_dir(), "{dir} is missing");
            }

            for file in &self.files {
                let on_disk = destination.join(&file.path);
                assert!(on_disk.is_file(), "{} is missing", file.path);
                assert_eq!(
                    fs::read(&on_disk).unwrap(),
                    file.contents,
                    "{} has the wrong contents",
                    file.path
                );
            }

            let mut extracted: Vec<PathBuf> = files_under(destination)
                .into_iter()
                .map(|path| path.strip_prefix(destination).unwrap().to_owned())
                .collect();
            let mut expected: Vec<PathBuf> = self
                .files
                .iter()
                .map(|file| PathBuf::from(&file.path))
                .collect();
            extracted.sort();
            expected.sort();
            assert_eq!(extracted, expected, "unexpected files in the destination");
        }
    }

    /// Every non-directory under `dir`, recursively.
    fn files_under(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                files.extend(files_under(&path));
            } else {
                files.push(path);
            }
        }
        files
    }

    fn assert_reads_lorem<S: Read + Write + Seek>(source: S) {
        let mut archive = Archive::open(TestArchive::lorem().write(source)).unwrap();

        let test_dir = archive.next_entry().unwrap().unwrap();
        assert_eq!(test_dir.path().unwrap(), PathBuf::from("test/"));
        assert_eq!(test_dir.file_type(), FileType::Directory);

        let mut lorem_txt = archive.next_entry().unwrap().unwrap();
        assert_eq!(lorem_txt.path().unwrap(), PathBuf::from("test/lorem.txt"));
        assert_eq!(lorem_txt.size(), i64::try_from(LOREM.len()).ok());
        assert_eq!(lorem_txt.file_type(), FileType::Regular);

        let mut contents = String::new();
        lorem_txt.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, LOREM);

        assert!(archive.next_entry().unwrap().is_none());
    }

    #[test]
    fn read_lorem_memory() {
        assert_reads_lorem(Cursor::new(Vec::new()));
    }

    #[test]
    fn read_lorem_file() {
        assert_reads_lorem(tempfile().unwrap());
    }

    fn assert_extracts(test_archive: &TestArchive) {
        let destination = tempdir().unwrap();

        let progress = |progress| {
            dbg!(progress);
        };

        test_archive
            .open()
            .extract(destination.path(), progress)
            .unwrap();

        test_archive.assert_extracted_to(destination.path());
    }

    #[test]
    fn extract_lorem() {
        assert_extracts(&TestArchive::lorem());
    }

    #[test]
    fn extract_mod_in_wrapper_folder() {
        assert_extracts(&TestArchive::cool_mod("Cool Mod 1.2/"));
    }

    #[test]
    fn extract_mod_at_root() {
        assert_extracts(&TestArchive::cool_mod(""));
    }
}
