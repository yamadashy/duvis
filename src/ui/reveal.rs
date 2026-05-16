//! `/reveal` handler + the OS-specific `reveal_in_filer` family.
//!
//! Validates the requested path segments against the scan root and asks
//! the OS file manager to highlight the resolved target.

use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::wire::ui::WireRevealReq;

use super::state::AppState;

pub(super) async fn reveal(
    State(state): State<Arc<AppState>>,
    axum::Json(req): axum::Json<WireRevealReq>,
) -> Response {
    // Build the absolute path while rejecting anything that smells like
    // path traversal — the segments come from the page, not from the user
    // typing freely, but we still don't want a hostile script to escape.
    let mut path = state.scan_root.clone();
    for seg in &req.segments {
        if seg.is_empty() || seg == "." || seg == ".." || seg.contains('/') || seg.contains('\\') {
            return (StatusCode::BAD_REQUEST, "invalid segment").into_response();
        }
        path.push(seg);
    }

    // Confirm the resolved path stays inside the scan root. If the user
    // scanned a directory full of symlinks we still want to forbid escaping.
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "path does not exist").into_response(),
    };
    if !canonical.starts_with(&state.scan_root) {
        return (StatusCode::FORBIDDEN, "outside scan root").into_response();
    }

    if let Err(e) = reveal_in_filer(&canonical) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    StatusCode::OK.into_response()
}

/// Open the OS file manager focused on `path`.
/// On macOS / Windows we ask the system to highlight the file/dir.
/// Elsewhere we fall back to opening the parent directory (or the dir itself).
#[cfg(target_os = "macos")]
fn reveal_in_filer(path: &Path) -> std::io::Result<()> {
    let status = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "`open -R` exited with {status}"
        )));
    }
    Ok(())
}

// Windows: `explorer.exe /select,...` is documented to exit with non-zero
// even on success (returns 1 in many cases), so we deliberately do NOT
// validate the status code here. A spawn failure still surfaces via `?`.
#[cfg(target_os = "windows")]
fn reveal_in_filer(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .status()?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn reveal_in_filer(path: &Path) -> std::io::Result<()> {
    let target = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    open::that(target).map_err(std::io::Error::other)
}
