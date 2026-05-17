//! Wire-format DTOs. Every type emitted via `--json` / `--ndjson` /
//! `--explain-category --json` / the UI's `/data.json` lives in this
//! module. Domain types in `entry`, `classify`, `scan` deliberately
//! don't derive `Serialize` — converting to a wire DTO is an explicit
//! step at the rendering boundary so that:
//!
//! - the on-disk wire format is one place to read,
//! - a domain refactor can't silently break a downstream agent / UI,
//! - `bumping wire_version` is a clear, localized diff.
//!
//! `Serialize` / `Deserialize` derives (and equivalent impls) are
//! confined to files under this module — CI enforces it.

pub mod category;
pub mod explain;
pub mod largest;
pub mod tree;

// Only consumed by the `ui` feature today (UI server's /data.json tree
// payload and /reveal request). Gated so the no-default-features build
// stays warning-clean.
#[cfg(feature = "ui")]
pub mod entry;
#[cfg(feature = "ui")]
pub mod ui;
