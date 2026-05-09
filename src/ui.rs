use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::entry::{Entry, SortOrder};
use crate::scanner::{HardlinkPolicy, ScanCounts};

/// Days threshold for "stale" classification. Files modified longer ago than
/// this are counted toward the "Stale" stat in the UI.
const STALE_DAYS: u32 = 90;

const HTML_TEMPLATE: &str = include_str!(concat!(env!("OUT_DIR"), "/ui.html"));

/// Live scan progress + result, shared between the HTTP handlers and the
/// background scan task.
struct ScanState {
    started_at: Instant,
    /// Live counters. Bumped from inside the scanner; read from the
    /// /data.json handler so the UI can show "12,345 items / 3 skipped" live.
    counts: Arc<ScanCounts>,
    /// Generation counter used to drop results from a scan that was
    /// superseded by a newer /rescan before it could finish.
    scan_id: u64,
    inner: Inner,
}

enum Inner {
    Scanning,
    Ready { tree: Entry, scanned_in_ms: u64 },
    Error(String),
}

impl ScanState {
    fn scanning(scan_id: u64) -> Self {
        Self {
            started_at: Instant::now(),
            counts: Arc::new(ScanCounts::default()),
            scan_id,
            inner: Inner::Scanning,
        }
    }
}

struct AppState {
    scan_root: PathBuf,
    sort: SortOrder,
    reverse: bool,
    hardlinks: HardlinkPolicy,
    /// Monotonic id. Each call to `start_scan` bumps it; the spawned task
    /// commits its result only if the id still matches when it finishes.
    next_scan_id: AtomicU64,
    /// Gate that prevents two scans from running concurrently. /rescan calls
    /// while a scan is already in flight are coalesced into a no-op so we
    /// don't pile up redundant disk I/O on a long scan.
    scan_in_flight: AtomicBool,
    state: Mutex<ScanState>,
}

pub async fn serve(
    scan_root: PathBuf,
    port: u16,
    sort: SortOrder,
    reverse: bool,
    hardlinks: HardlinkPolicy,
) -> Result<()> {
    let app_state = Arc::new(AppState::new(scan_root, sort, reverse, hardlinks));

    // Kick off the initial scan immediately so the browser can render a
    // "scanning..." UI while we wait.
    start_scan(Arc::clone(&app_state));

    let app = build_router(app_state);

    let (listener, bound_port) = bind_with_fallback(port).await?;
    let url = format!("http://127.0.0.1:{bound_port}");

    if bound_port != port {
        println!("Port {port} is busy, falling back to {bound_port}.");
    }
    println!("Starting UI server at {url}");
    println!("Press Ctrl+C to stop.");
    let _ = open::that(&url);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Wires the HTTP routes to a shared `AppState`. Internal — the public entry
/// point is `serve()`. The `#[cfg(test)] mod tests` below uses this directly
/// to drive HTTP-level tests via `tower::ServiceExt::oneshot`.
fn build_router(state: Arc<AppState>) -> axum::Router {
    Router::new()
        .route("/", get(index))
        .route("/data.json", get(data_json))
        .route("/rescan", post(rescan))
        .route("/reveal", post(reveal))
        .with_state(state)
}

impl AppState {
    /// Construct a fresh `AppState` in the `Scanning` placeholder state.
    fn new(scan_root: PathBuf, sort: SortOrder, reverse: bool, hardlinks: HardlinkPolicy) -> Self {
        Self {
            scan_root,
            sort,
            reverse,
            hardlinks,
            next_scan_id: AtomicU64::new(1),
            scan_in_flight: AtomicBool::new(false),
            state: Mutex::new(ScanState::scanning(0)),
        }
    }
}

/// Spawn a background blocking task that re-runs the scan. The current
/// status flips to `Scanning` immediately so the UI can react.
///
/// Coalesces concurrent calls: if a scan is already running, this returns
/// without starting a second one. `scan_id` is kept as defense-in-depth so
/// that even if a stray scan slips through, only the in-flight generation can
/// commit `Inner::Ready` to the shared state.
fn start_scan(state: Arc<AppState>) {
    // CAS gate: only one scan in flight at a time. Avoids piling up disk I/O
    // when /rescan is spam-clicked during a long scan.
    if state
        .scan_in_flight
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    let scan_id = state.next_scan_id.fetch_add(1, Ordering::Relaxed);
    let fresh = ScanState::scanning(scan_id);
    let counts = Arc::clone(&fresh.counts);
    {
        let mut s = state.state.lock().unwrap();
        *s = fresh;
    }
    tokio::task::spawn_blocking(move || {
        let started = Instant::now();
        let scan_result =
            crate::scanner::scan_with_progress(&state.scan_root, &counts, state.hardlinks);
        {
            let mut s = state.state.lock().unwrap();
            if s.scan_id == scan_id {
                s.inner = match scan_result {
                    Ok(mut tree) => {
                        tree.sort(&state.sort, state.reverse);
                        let scanned_in_ms = started.elapsed().as_millis() as u64;
                        Inner::Ready {
                            tree,
                            scanned_in_ms,
                        }
                    }
                    Err(e) => Inner::Error(e.to_string()),
                };
            }
        }
        // Release the gate so the next /rescan can proceed.
        state.scan_in_flight.store(false, Ordering::Release);
    });
}

/// Try to bind on `127.0.0.1:<port>`. If that port is taken (and isn't 0),
/// fall back to an OS-assigned free port so the user always gets a working
/// server. Returns the listener and the port it actually bound to.
async fn bind_with_fallback(port: u16) -> Result<(tokio::net::TcpListener, u16)> {
    match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
        Ok(listener) => {
            let actual = listener.local_addr()?.port();
            Ok((listener, actual))
        }
        Err(e) if port != 0 && e.kind() == std::io::ErrorKind::AddrInUse => {
            let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
            let actual = listener.local_addr()?.port();
            Ok((listener, actual))
        }
        Err(e) => Err(e.into()),
    }
}

async fn index() -> Html<String> {
    // The placeholder used to be replaced with the scan JSON. We now serve a
    // shell: the page fetches /data.json itself and polls until ready.
    let html = HTML_TEMPLATE.replacen("__DUVIS_DATA__", "null", 1);
    Html(html)
}

async fn data_json(State(s): State<Arc<AppState>>) -> Response {
    let state = s.state.lock().unwrap();
    let scan_root = s.scan_root.display().to_string();
    let body = match &state.inner {
        Inner::Scanning => json!({
            "status": "scanning",
            "elapsed_ms": state.started_at.elapsed().as_millis() as u64,
            "items_scanned": state.counts.scanned(),
            "items_skipped": state.counts.skipped(),
            "scan_root": scan_root,
        }),
        Inner::Ready {
            tree,
            scanned_in_ms,
        } => json!({
            "status": "ready",
            "scanned_in_ms": scanned_in_ms,
            "items_scanned": state.counts.scanned(),
            "items_skipped": state.counts.skipped(),
            "scan_root": scan_root,
            "tree": tree,
            "meta": meta_block(),
        }),
        Inner::Error(msg) => json!({
            "status": "error",
            "message": msg,
            "scan_root": scan_root,
        }),
    };
    axum::Json(body).into_response()
}

/// Server-derived constants the UI needs at boot.
fn meta_block() -> serde_json::Value {
    json!({
        "stale_days": STALE_DAYS,
    })
}

async fn rescan(State(s): State<Arc<AppState>>) -> StatusCode {
    start_scan(s);
    StatusCode::ACCEPTED
}

#[derive(Deserialize)]
struct RevealReq {
    /// Path segments below the scanned root, e.g. ["target", "debug"].
    /// An empty array reveals the root itself.
    segments: Vec<String>,
}

async fn reveal(
    State(state): State<Arc<AppState>>,
    axum::Json(req): axum::Json<RevealReq>,
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

#[cfg(test)]
mod tests {
    //! HTTP-level tests for the duvis UI server. We drive the router directly
    //! via `tower::ServiceExt::oneshot` so we don't need a real port or a live
    //! scan, and so internals (`AppState`, `build_router`) stay private to
    //! the crate.

    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::category::Category;

    fn fake_entry(name: &str) -> Entry {
        Entry::dir(
            name.to_string(),
            Category::Other,
            Some(1),
            vec![Entry::file(
                "file.txt".to_string(),
                4096,
                Category::Other,
                Some(1),
            )],
        )
    }

    fn ready_state(scan_root: PathBuf) -> Arc<AppState> {
        let state = Arc::new(AppState::new(
            scan_root,
            SortOrder::Size,
            false,
            HardlinkPolicy::CountOnce,
        ));
        {
            let mut s = state.state.lock().unwrap();
            s.inner = Inner::Ready {
                tree: fake_entry("root"),
                scanned_in_ms: 42,
            };
        }
        state
    }

    async fn body_to_json(response: axum::response::Response) -> Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).expect("response body is valid json")
    }

    async fn body_to_string(response: axum::response::Response) -> String {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn data_json_returns_scanning_initially() {
        let dir = tempfile::tempdir().unwrap();
        let state = Arc::new(AppState::new(
            dir.path().to_path_buf(),
            SortOrder::Size,
            false,
            HardlinkPolicy::CountOnce,
        ));
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/data.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_to_json(response).await;
        assert_eq!(json["status"], "scanning");
        assert!(json.get("items_scanned").is_some());
        assert!(json.get("items_skipped").is_some());
        assert!(json.get("scan_root").is_some());
    }

    #[tokio::test]
    async fn data_json_returns_ready_with_meta() {
        let dir = tempfile::tempdir().unwrap();
        let state = ready_state(dir.path().to_path_buf());
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .uri("/data.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_to_json(response).await;
        assert_eq!(json["status"], "ready");
        assert_eq!(json["scanned_in_ms"], 42);
        assert_eq!(json["tree"]["name"], "root");
        let meta = &json["meta"];
        assert!(meta["stale_days"].is_number());
    }

    #[tokio::test]
    async fn rescan_returns_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let state = ready_state(dir.path().to_path_buf());
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/rescan")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn reveal_rejects_path_traversal_segments() {
        let dir = tempfile::tempdir().unwrap();
        let state = ready_state(dir.path().to_path_buf());
        let router = build_router(state);

        for bad in [
            r#"{"segments":[".."]}"#,
            r#"{"segments":["."]}"#,
            r#"{"segments":[""]}"#,
            r#"{"segments":["a/b"]}"#,
            r#"{"segments":["a\\b"]}"#,
        ] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/reveal")
                        .header("content-type", "application/json")
                        .body(Body::from(bad))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "expected 400 for payload {bad}",
            );
            let body = body_to_string(response).await;
            assert!(body.contains("invalid segment"), "body was {body}");
        }
    }

    #[tokio::test]
    async fn reveal_rejects_nonexistent_segment() {
        let dir = tempfile::tempdir().unwrap();
        let state = ready_state(dir.path().to_path_buf());
        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/reveal")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"segments":["does-not-exist"]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn index_serves_html_shell() {
        let dir = tempfile::tempdir().unwrap();
        let state = Arc::new(AppState::new(
            dir.path().to_path_buf(),
            SortOrder::Size,
            false,
            HardlinkPolicy::CountOnce,
        ));
        let response = build_router(state)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_to_string(response).await;
        assert!(
            body.contains("<html") || body.contains("<!DOCTYPE"),
            "expected HTML shell, got: {}",
            &body[..body.len().min(200)],
        );
    }
}
