use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub struct DiskLog {
    file: File,
}

impl DiskLog {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self { file })
    }

    pub fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        // 1. Write bytes to the file
        self.file.write_all(bytes)?;

        // 2. Flush userspace buffers
        self.file.flush()?;

        // 3. Force data to stable storage
        self.file.sync_all()?;

        Ok(())
    }
}

