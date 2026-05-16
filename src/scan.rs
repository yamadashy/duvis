//! Filesystem scan facade. Re-exports the policy enum, the progress
//! counters, and the two entry points (`scan` / `scan_with_progress`)
//! so callers can `use crate::scan::{scan, HardlinkPolicy, ScanCounts}`
//! without caring which submodule each item lives in.
//!
//! Submodule layout:
//! - [`hardlinks`] — `HardlinkPolicy` enum + Display / FromStr (CLI surface).
//! - [`counts`] — `ScanCounts` atomic progress signal shared with UI.
//! - [`metadata`] — per-file metadata reads (`file_disk_usage` with
//!   Unix `st_blocks` + hardlink dedup, `days_since_modified`).
//! - [`walk`] — the parallel rayon walk + `scan` / `scan_with_progress`
//!   entry points and the bulk of the tests.

pub mod counts;
pub mod hardlinks;
pub mod metadata;
pub mod walk;

pub use counts::ScanCounts;
pub use hardlinks::HardlinkPolicy;
pub use walk::{scan, scan_with_progress};
