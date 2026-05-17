//! `/reveal` handler + the OS-specific `reveal_in_filer` family.
//!
//! Validates the requested path segments against the scan root and asks
//! the OS file manager to highlight the resolved target.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::wire::ui::WireRevealReq;

use super::state::AppState;

/// Why a `/reveal` request couldn't be resolved into a concrete path
/// inside the scan root. Each variant maps 1:1 to an HTTP status in
/// [`reveal`]; kept separate from the handler so the resolution logic
/// is testable in isolation (the handler itself launches the OS file
/// manager, which can't run in unit tests).
#[derive(Debug, PartialEq, Eq)]
pub(super) enum RevealError {
    /// A segment was empty, `.`, `..`, or contained a path separator.
    InvalidSegment,
    /// The joined path didn't exist on disk.
    NotFound,
    /// The canonicalized target escaped the scan root.
    Outside,
}

/// Build the absolute target path from `scan_root` + `segments`, rejecting
/// traversal-shaped segments and verifying the canonical resolution
/// stays inside `scan_root`. `scan_root` is expected to already be
/// canonical (see `serve()`); the comparison is `starts_with` against
/// that canonical form.
pub(super) fn resolve_target(
    scan_root: &Path,
    segments: &[String],
) -> Result<PathBuf, RevealError> {
    // Segment hygiene first — these come from the page, not from the user
    // typing freely, but we still don't want a hostile script to escape.
    let mut path = scan_root.to_path_buf();
    for seg in segments {
        if seg.is_empty() || seg == "." || seg == ".." || seg.contains('/') || seg.contains('\\') {
            return Err(RevealError::InvalidSegment);
        }
        path.push(seg);
    }
    let canonical = path.canonicalize().map_err(|_| RevealError::NotFound)?;
    if !canonical.starts_with(scan_root) {
        return Err(RevealError::Outside);
    }
    Ok(canonical)
}

pub(super) async fn reveal(
    State(state): State<Arc<AppState>>,
    axum::Json(req): axum::Json<WireRevealReq>,
) -> Response {
    let canonical = match resolve_target(&state.scan_root, &req.segments) {
        Ok(p) => p,
        Err(RevealError::InvalidSegment) => {
            return (StatusCode::BAD_REQUEST, "invalid segment").into_response()
        }
        Err(RevealError::NotFound) => {
            return (StatusCode::NOT_FOUND, "path does not exist").into_response()
        }
        Err(RevealError::Outside) => {
            return (StatusCode::FORBIDDEN, "outside scan root").into_response()
        }
    };

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

#[cfg(test)]
mod tests {
    //! Path-resolution unit tests for `/reveal`. We can't end-to-end test
    //! the success path of the handler itself because `reveal_in_filer`
    //! launches the OS file manager — instead we exercise the pure
    //! `resolve_target` extract directly. The handler's HTTP-level
    //! rejection-path coverage lives in `super::super::server::tests`.

    use super::*;
    use std::fs;

    #[test]
    fn accepts_file_inside_canonicalized_root() {
        // Tempdirs on macOS resolve through `/var` → `/private/var`. The
        // canonicalize-at-boot fix in `serve()` makes `scan_root` already
        // canonical so `starts_with` matches the canonical join below.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        fs::write(root.join("hello.txt"), b"").unwrap();

        let target = resolve_target(&root, &["hello.txt".to_string()]).unwrap();

        assert_eq!(target, root.join("hello.txt"));
    }

    #[test]
    fn rejects_non_canonical_root_that_would_escape_starts_with() {
        // Sanity: confirm the bug existed pre-fix. If `scan_root` is left
        // un-canonical (the macOS `/var` shape) and contains a symlink,
        // the canonical resolution of an in-tree file fails the
        // `starts_with` comparison. This justifies why `serve()`
        // canonicalizes at boot.
        let dir = tempfile::tempdir().unwrap();
        let canonical_root = dir.path().canonicalize().unwrap();
        if dir.path() == canonical_root {
            // Platform doesn't surface the symlink shape (e.g. linux
            // tmpfs), so the bug isn't observable. Skip rather than
            // produce a false negative.
            return;
        }
        fs::write(canonical_root.join("hello.txt"), b"").unwrap();

        let err = resolve_target(dir.path(), &["hello.txt".to_string()]).unwrap_err();

        assert_eq!(err, RevealError::Outside);
    }

    #[test]
    fn rejects_traversal_segments() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        for bad in ["..", ".", "", "a/b", "a\\b"] {
            assert_eq!(
                resolve_target(&root, &[bad.to_string()]),
                Err(RevealError::InvalidSegment),
                "expected InvalidSegment for {bad:?}",
            );
        }
    }

    #[test]
    fn rejects_missing_segment_with_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();

        assert_eq!(
            resolve_target(&root, &["does-not-exist".to_string()]),
            Err(RevealError::NotFound),
        );
    }
}
