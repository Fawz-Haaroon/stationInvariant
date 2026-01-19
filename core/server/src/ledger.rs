use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::disk::DiskLog;
use crate::protocol::{Frame, FrameType};

pub struct Ledger {
    // Next offset to assign (global for now)
    next_offset: Mutex<u64>,

    // One disk log per stream
    logs: Mutex<HashMap<u64, DiskLog>>,

    // Base directory for data files
    data_dir: PathBuf,
}

impl Ledger {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            next_offset: Mutex::new(0),
            logs: Mutex::new(HashMap::new()),
            data_dir,
        }
    }

    pub fn append(&self, stream_id: u64, payload: Vec<u8>) -> std::io::Result<u64> {
        // Lock offset (this defines global order)
        let mut offset_guard = self.next_offset.lock().unwrap();
        let offset = *offset_guard;

        // Get or open disk log for this stream
        let mut logs = self.logs.lock().unwrap();
        let log = logs.entry(stream_id).or_insert_with(|| {
            let path = self.data_dir.join(format!("stream-{stream_id:016x}.log"));
            DiskLog::open(path).expect("failed to open disk log")
        });

        // Build frame
        let frame = Frame {
            frame_type: FrameType::Publish,
            stream_id,
            offset,
            payload,
        };

        // Write to disk (durable)
        log.append(&frame.encode())?;

        // Commit offset
        *offset_guard += 1;

        Ok(offset)
    }
}

