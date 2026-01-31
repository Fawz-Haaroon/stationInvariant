//! System invariants.
//!
//! These are rules the system relies on being true.
//! Violating them means data corruption, not a bug.

/// Offsets are strictly increasing per stream.
pub fn assert_monotonic_offset(prev: u64, next: u64) {
    assert!(
        next > prev,
        "offset regression detected: {} -> {}",
        prev,
        next
    );
}
