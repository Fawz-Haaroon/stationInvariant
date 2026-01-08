use crate::entry::Entry;

/*
INVARIENT cannot be modified or removed & the order is deterministic
// ordered log of ENTRIES (append-only)
*/

#[derive(Debug)]
pub struct Log {
    entries: Vec<Entry>,
}

impl Log {
    // Create an empty log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    // Append a new entry to the log.
    // Returns the offset at which it was inserted.
    pub fn append(&mut self, entry: Entry) -> usize {
        let offset = self.entries.len();
        self.entries.push(entry);
        offset
    }

    // Number of entries in the log.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    // Check if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // Get an entry by offset (read-only).
    pub fn get(&self, offset: usize) -> Option<&Entry> {
        self.entries.get(offset)
    }

    // Replay entries starting from a given offset.
    pub fn replay_from(&self, offset: usize) -> impl Iterator<Item = &Entry> {
        self.entries.iter().skip(offset)
    }
}

