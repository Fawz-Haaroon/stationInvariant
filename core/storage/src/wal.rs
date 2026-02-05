use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

pub struct WriteAheadLog {
    file: File,
}

impl WriteAheadLog {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;

        Ok(Self { file })
    }

    pub fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.file.write_all(bytes)?;
        self.file.flush()?;
        Ok(())
    }

    /// Expose the underlying file for replay / recovery.
    /// Read-only usage is expected.
    pub fn file(&self) -> &File {
        &self.file
    }
}
