use std::io;

pub trait Storage {
    /// Persist bytes durably.
    /// If this returns Ok, data must survive crash.
    fn append(&mut self, bytes: &[u8]) -> io::Result<()>;
}

pub mod wal;
