use core_engine::ledger::Ledger;
use core_storage::wal::WriteAheadLog;
use core_storage::Storage;

pub struct Server {
    ledger: Ledger,
    storage: WriteAheadLog,
}

impl Server {
    pub fn new(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            ledger: Ledger::new(),
            storage: WriteAheadLog::open(path)?,
        })
    }

    pub fn append(&mut self, payload: &[u8]) -> std::io::Result<u64> {
        let offset = self.ledger.assign_offset();

        self.storage.append(payload)?;

        Ok(offset)
    }
}
