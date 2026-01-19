use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use crate::protocol::{Frame, DecodeError};

pub struct ReplayReader {
    file: File,
}

impl ReplayReader {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self { file })
    }

    pub fn read_all(&mut self) -> io::Result<Vec<Frame>> {
        let mut frames = Vec::new();

        loop {
            match Self::read_one(&mut self.file) {
                Ok(Some(frame)) => frames.push(frame),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(frames)
    }

    fn read_one(file: &mut File) -> io::Result<Option<Frame>> {
        // read header
        let mut header = [0u8; Frame::HEADER_LEN];

        match file.read_exact(&mut header) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Partial write (if it crash)
                return Ok(None);
            }
            Err(e) => return Err(e),
        }

        // Extract payload length from header
        let len_bytes = &header[17..21];
        let payload_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;

        // read payload
        let mut payload = vec![0u8; payload_len];
        file.read_exact(&mut payload)?;

        // reconstruct full frame buffer
        let mut full = Vec::with_capacity(Frame::HEADER_LEN + payload_len);
        full.extend_from_slice(&header);
        full.extend_from_slice(&payload);

        // decode
        match Frame::decode(&full) {
            Ok(frame) => Ok(Some(frame)),
            Err(DecodeError::LengthMismatch) => Ok(None),
            Err(_) => Ok(None),
        }
    }
}

