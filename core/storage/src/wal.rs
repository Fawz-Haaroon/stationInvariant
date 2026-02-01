use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub struct WriteAheadLog {
    file: std::fs::File,
}

impl WriteAheadLog {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self { file })
    }

    pub fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.file.write_all(bytes)?;
        self.file.flush()?; // durability boundary
        Ok(())
    }
}

