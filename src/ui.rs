//! Browser UI server. Spins up an axum HTTP server that serves the
//! single-page bundle and a small JSON API (`/data.json`, `/rescan`,
//! `/reveal`).
//!
//! Submodule layout:
//! - [`assets`] — the bundled HTML shell (`HTML_TEMPLATE`).
//! - [`state`] — shared `AppState` / `ScanState`, the background scan
//!   spawner, and the `STALE_DAYS` constant the UI shows.
//! - [`server`] — `serve` entry point, router wiring, `/`, `/data.json`,
//!   `/rescan` handlers, and the HTTP-level test suite.
//! - [`reveal`] — `/reveal` handler + the `reveal_in_filer` cfg
//!   family (macOS / Windows / fallback).

pub mod assets;
pub mod reveal;
pub mod server;
pub mod state;

pub use server::serve;
