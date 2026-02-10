use std::io;
use std::net::TcpStream;
use std::io::Write;

use core_engine::ledger::Ledger;
use core_protocol::frame::{Frame, FrameType};
use core_storage::wal::WriteAheadLog;

pub struct Server {
    ledger: Ledger,
    storage: WriteAheadLog,
}

impl Server {
    pub fn new(path: &str) -> io::Result<Self> {
        Ok(Self {
            ledger: Ledger::new(),
            storage: WriteAheadLog::open(path)?,
        })
    }

    pub fn append(&mut self, payload: &[u8]) -> io::Result<u64> {
        let offset = self.ledger.assign_offset();
        self.storage.append(payload)?;
        Ok(offset)
    }

    pub fn handle_frame(
        &mut self,
        frame: Frame,
        stream: &mut TcpStream,
    ) -> io::Result<()> {
        match frame.frame_type {
            FrameType::Publish => {
                let offset = self.append(&frame.payload)?;

                let ack = Frame {
                    frame_type: FrameType::Ack,
                    stream_id: frame.stream_id,
                    offset,
                    payload: Vec::new(),
                };

                stream.write_all(&ack.encode())?;
            }

            FrameType::Subscribe => {
                // intentionally empty for now
            }

            _ => {}
        }

        Ok(())
    }
}

