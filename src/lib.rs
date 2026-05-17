//! `duvis` library surface.
//!
//! The crate is shipped primarily as a CLI binary, but the same logic
//! is also reachable as a library. To keep refactors safe, only the
//! handful of items re-exported here are part of the stable public
//! surface — the internal module layout (`scan/`, `classify/`, `cli/`,
//! `render/`, `wire/`, `filter/`, `ui/`) is an implementation detail and
//! may change between minor versions.
//!
//! Typical library use:
//! ```no_run
//! use duvis::{scan, HardlinkPolicy};
//! let (tree, counts) = scan(std::path::Path::new("."), HardlinkPolicy::CountOnce).unwrap();
//! println!("scanned {} items", counts.scanned());
//! for child in tree.children().into_iter().flatten() {
//!     println!("{}", child.name);
//! }
//! ```
//!
//! Binary use is just [`run_cli`], which `main.rs` delegates to.

mod classify;
mod cli;
mod entry;
mod filter;
mod render;
mod scan;
#[cfg(feature = "ui")]
mod ui;
mod wire;

pub use classify::{
    classify_dir, classify_file, explain_dir, explain_file, Category, Classification,
    ClassificationReason, Tier,
};
pub use entry::{Entry, EntryKind, SortOrder};
pub use scan::{scan, scan_with_progress, HardlinkPolicy, ScanCounts};

/// Binary entry. `main.rs` calls this and returns the resulting exit
/// code. All CLI / dispatch / exit-code work lives inside the lib crate
/// so that downstream code (and integration tests) can drive the full
/// pipeline without going through a subprocess.
pub fn run_cli() -> std::process::ExitCode {
    cli::run()
}
