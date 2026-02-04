use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

pub struct ReplayReader {
    file: File,
}

impl ReplayReader {
    pub fn new(mut file: File) -> io::Result<Self> {
        file.seek(SeekFrom::Start(0))?;
        Ok(Self { file })
    }

    /// Read the log from start to end.
    /// Returns raw payloads in write order.
    pub fn replay_all(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut records = Vec::new();

        loop {
            match Self::read_one(&mut self.file)? {
                Some(record) => records.push(record),
                None => break,
            }
        }

        Ok(records)
    }

    fn read_one(file: &mut File) -> io::Result<Option<Vec<u8>>> {
        let mut len_buf = [0u8; 8];

        // length prefix
        if let Err(e) = file.read_exact(&mut len_buf) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(e);
        }

        let len = u64::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];

        file.read_exact(&mut payload)?;

        Ok(Some(payload))
    }
}
