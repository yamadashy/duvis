//! Shared scan state + the background scan spawner.
//!
//! `AppState` is the axum router's `State`; handlers in [`super::server`]
//! and [`super::reveal`] read it. `start_scan` runs the actual scan on
//! a blocking tokio task so the HTTP handlers stay responsive.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::entry::{Entry, SortOrder};
use crate::scan::{HardlinkPolicy, ScanCounts};

/// Days threshold for "stale" classification. Files modified longer ago than
/// this are counted toward the "Stale" stat in the UI.
pub(super) const STALE_DAYS: u32 = 90;

/// Live scan progress + result, shared between the HTTP handlers and the
/// background scan task.
pub(super) struct ScanState {
    pub(super) started_at: Instant,
    /// Live counters. Bumped from inside the scanner; read from the
    /// /data.json handler so the UI can show "12,345 items / 3 skipped" live.
    pub(super) counts: Arc<ScanCounts>,
    /// Generation counter used to drop results from a scan that was
    /// superseded by a newer /rescan before it could finish.
    pub(super) scan_id: u64,
    pub(super) inner: Inner,
}

pub(super) enum Inner {
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

pub(super) struct AppState {
    pub(super) scan_root: PathBuf,
    pub(super) sort: SortOrder,
    pub(super) reverse: bool,
    pub(super) hardlinks: HardlinkPolicy,
    /// Monotonic id. Each call to `start_scan` bumps it; the spawned task
    /// commits its result only if the id still matches when it finishes.
    next_scan_id: AtomicU64,
    /// Gate that prevents two scans from running concurrently. /rescan calls
    /// while a scan is already in flight are coalesced into a no-op so we
    /// don't pile up redundant disk I/O on a long scan.
    scan_in_flight: AtomicBool,
    pub(super) state: Mutex<ScanState>,
}

impl AppState {
    /// Construct a fresh `AppState` in the `Scanning` placeholder state.
    pub(super) fn new(
        scan_root: PathBuf,
        sort: SortOrder,
        reverse: bool,
        hardlinks: HardlinkPolicy,
    ) -> Self {
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
pub(super) fn start_scan(state: Arc<AppState>) {
    // CAS gate: only one scan in flight at a time. Avoids piling up disk I/O
    // when /rescan is spam-clicked during a long scan.
    if state
        .scan_in_flight
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    // Construct the guard immediately after winning the CAS so that
    // *any* failure path between here and the task body — a poisoned
    // mutex on the `state.lock()` below, an OOM on Arc allocation, the
    // runtime declining to schedule the blocking task — still releases
    // the gate. Without this the gate could stay stuck `true` forever
    // and silently no-op every subsequent /rescan.
    let guard = InFlightGuard(Arc::clone(&state));
    let scan_id = state.next_scan_id.fetch_add(1, Ordering::Relaxed);
    let fresh = ScanState::scanning(scan_id);
    let counts = Arc::clone(&fresh.counts);
    {
        let mut s = state.state.lock().unwrap();
        *s = fresh;
    }
    tokio::task::spawn_blocking(move || {
        // Bind the moved guard so its lifetime spans the whole task
        // body — drop on closure exit (success or panic) releases the
        // gate. If the closure is never executed (runtime shutdown),
        // dropping the closure itself still drops the guard.
        let _guard = guard;
        let started = Instant::now();
        let scan_result =
            crate::scan::scan_with_progress(&state.scan_root, &counts, state.hardlinks);
        // Sort *before* acquiring the mutex — sorting a multi-million-entry
        // tree can take a noticeable fraction of a second, and `/data.json`
        // handlers compete for the same lock.
        let next_inner = match scan_result {
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
        let mut s = state.state.lock().unwrap();
        if s.scan_id == scan_id {
            s.inner = next_inner;
        }
    });
}

/// Releases the `scan_in_flight` gate on drop. Owning an `Arc<AppState>`
/// (rather than borrowing the atomic) lets the guard be constructed
/// before `spawn_blocking` and moved into the `'static` closure, so the
/// gate clears on every drop path — scan panic, sort panic, mutex
/// poison, or the task never being scheduled.
struct InFlightGuard(Arc<AppState>);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.scan_in_flight.store(false, Ordering::Release);
    }
}
