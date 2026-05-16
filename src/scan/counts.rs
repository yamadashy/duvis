//! Atomic progress counters shared between the scanner and any
//! observer (the UI server polls these from another thread).

use std::sync::atomic::{AtomicU64, Ordering};

/// Live counters shared between the scanner and any observer. `items_scanned`
/// drives the "12,345 items…" progress UI; `items_skipped` reports filesystem
/// entries we could not read (permission denied, races against deletion, …)
/// so the user can tell that the reported total is incomplete.
#[derive(Default, Debug)]
pub struct ScanCounts {
    pub items_scanned: AtomicU64,
    pub items_skipped: AtomicU64,
}

impl ScanCounts {
    pub fn scanned(&self) -> u64 {
        self.items_scanned.load(Ordering::Relaxed)
    }

    pub fn skipped(&self) -> u64 {
        self.items_skipped.load(Ordering::Relaxed)
    }
}
