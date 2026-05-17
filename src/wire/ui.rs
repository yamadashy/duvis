//! Wire DTOs for the UI HTTP boundary (`/data.json` payloads,
//! `/reveal` request body).
//!
//! `/data.json` itself is still assembled via `serde_json::json!` in
//! `ui.rs` (the body shape varies by scan status and we don't gain
//! much by typing each variant individually); what lives here is what
//! the macro can't express on its own — the `Deserialize` side of
//! `/reveal` and the lean tree shape (`WireEntry`, re-exported from
//! [`super::entry`]) the macro embeds.

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct WireRevealReq {
    /// Path segments below the scanned root, e.g. ["target", "debug"].
    /// An empty array reveals the root itself.
    pub(crate) segments: Vec<String>,
}
