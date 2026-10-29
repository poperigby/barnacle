use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    /// The amount of bytes that have been read.
    pub current: u64,
    /// The total size of the archive file, in bytes.
    pub total: u64,
}

pub struct ProgressReader {
    file: File,
    progress: Progress,
    on_progress: Box<dyn Fn(Progress)>,
}

impl ProgressReader {
    pub fn create(file: File, on_progress: impl Fn(Progress) + 'static) -> io::Result<Self> {
        let archive_size = file.metadata()?.len();

        Ok(Self {
            file,
            progress: Progress {
                current: 0,
                total: archive_size,
            },
            on_progress: Box::new(on_progress),
        })
    }
}

impl Read for ProgressReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.file.read(buf)?;

        // We clamp the amount of read bytes to the total because: `uncompress_archive` doesn't just
        // read from start to end contigously, it jumps around and reads some parts multiple times,
        // which can cause `current` to slightly overshoot `total`.
        self.progress.current = self
            .progress
            .total
            .min(self.progress.current + bytes_read as u64);

        (self.on_progress)(self.progress);

        Ok(bytes_read)
    }
}

impl Seek for ProgressReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}
