use crate::invariants::assert_monotonic_offset;

pub struct Ledger {
    next_offset: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self { next_offset: 0 }
    }

    /// Assigns the next offset.
    /// Does NOT persist. That is storage’s job.
    pub fn assign_offset(&mut self) -> u64 {
        let current = self.next_offset;
        let next = current + 1;

        assert_monotonic_offset(current, next);

        self.next_offset = next;
        current
    }
}
