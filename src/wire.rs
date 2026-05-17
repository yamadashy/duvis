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

// Submodules are `pub(crate)` (not `pub`) because the parent `wire`
// module is itself private from the crate root — these DTOs are an
// internal serialization detail, not part of the library's public API.
// Renderers / ui / cli access them by path, which is why they're not
// `mod`.
pub(crate) mod category;
pub(crate) mod explain;
pub(crate) mod largest;
pub(crate) mod tree;

// Only consumed by the `ui` feature today (UI server's /data.json tree
// payload and /reveal request). Gated so the no-default-features build
// stays warning-clean.
#[cfg(feature = "ui")]
pub(crate) mod entry;
#[cfg(feature = "ui")]
pub(crate) mod ui;
