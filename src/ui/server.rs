//! HTTP server: router wiring, `serve` entry point, and the `/`,
//! `/data.json`, `/rescan` handlers. Tests live here because they
//! drive the router directly via `tower::ServiceExt::oneshot`.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde_json::json;

use crate::entry::SortOrder;
use crate::scan::HardlinkPolicy;
use crate::wire::entry::WireEntry;

use super::assets::HTML_TEMPLATE;
use super::reveal::reveal;
use super::state::{start_scan, AppState, Inner, STALE_DAYS};

pub(crate) async fn serve(
    scan_root: PathBuf,
    port: u16,
    sort: SortOrder,
    reverse: bool,
    hardlinks: HardlinkPolicy,
) -> Result<()> {
    // Canonicalize once at boot so `/reveal` can compare a request's
    // canonical resolved path against `scan_root` with `starts_with`. If
    // we kept the user-supplied path (which may be relative, contain `..`,
    // or traverse symlinks like macOS's `/var` → `/private/var`) the
    // comparison would spuriously reject valid in-tree paths.
    let scan_root = scan_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize scan root: {}", scan_root.display()))?;
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
fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/data.json", get(data_json))
        .route("/rescan", post(rescan))
        .route("/reveal", post(reveal))
        .with_state(state)
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
            "tree": WireEntry::from_entry(tree),
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

    use crate::classify::Category;
    use crate::entry::Entry;

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
