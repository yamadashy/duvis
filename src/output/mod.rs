pub mod analyze;
mod format;
pub mod json;
pub mod ndjson;
pub mod tree;

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use crate::entry::Entry;
use crate::scanner::{HardlinkPolicy, ScanCounts};

pub struct OutputConfig<'a> {
    pub depth: Option<usize>,
    pub top: Option<usize>,
    /// Absolute, canonicalized scan root. Surfaced via `meta.scan_root` in
    /// JSON / NDJSON so an agent that pipes the output elsewhere can still
    /// reconstruct full paths from the per-entry `relative_path`.
    pub scan_root: &'a Path,
    /// Final scan counters. Forwarded into `meta` for the structured
    /// outputs so callers can tell "we visited 1.2M items, 3 were skipped"
    /// without parsing stderr.
    pub counts: &'a ScanCounts,
    /// Which hardlink dedup mode produced the sizes. Recorded in `meta`
    /// so consumers can tell whether a per-path size already accounts for
    /// shared inodes or not.
    pub hardlinks: HardlinkPolicy,
}

/// Pick the largest `n` children by size while preserving their relative order
/// in `children` (which already follows --sort). Returns the selected slice
/// plus the count and total size of what was dropped, so the renderer can
/// emit a "and N more (X)" overflow line.
///
/// `--top` advertises "by size" (per --help and README), but the children
/// are presented in --sort order, so we must split selection from display.
pub(crate) fn select_top(children: &[Entry], top: Option<usize>) -> (Vec<&Entry>, usize, u64) {
    match top {
        None => (children.iter().collect(), 0, 0),
        Some(n) if n >= children.len() => (children.iter().collect(), 0, 0),
        Some(n) => {
            // Indices of the n largest by size. Tie-break by original index to
            // be deterministic when sizes are equal.
            let mut by_size: Vec<(usize, u64)> = children
                .iter()
                .enumerate()
                .map(|(i, e)| (i, e.size))
                .collect();
            by_size.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            by_size.truncate(n);
            let keep: std::collections::BTreeSet<usize> = by_size.iter().map(|&(i, _)| i).collect();

            let kept: Vec<&Entry> = children
                .iter()
                .enumerate()
                .filter(|(i, _)| keep.contains(i))
                .map(|(_, e)| e)
                .collect();
            let dropped_size: u64 = children
                .iter()
                .enumerate()
                .filter(|(i, _)| !keep.contains(i))
                .map(|(_, e)| e.size)
                .sum();
            let dropped_count = children.len() - kept.len();
            (kept, dropped_count, dropped_size)
        }
    }
}

/// Which structured / human output mode to render. Selected by clap's
/// mutually-exclusive ArgGroup, so exactly one variant reaches the
/// dispatcher.
#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    Tree,
    Json,
    Ndjson,
    Analyze,
}

/// Precomputed `(file_count, dir_count_including_self)` for every entry
/// in a scanned tree, keyed by entry pointer. JSON / NDJSON renderers
/// look up here instead of recomputing per visit, which would otherwise
/// be O(N²) on deep trees because every node would re-walk its full
/// subtree. Pointers are stable for the duration of a render call since
/// the tree is held by main and never moved.
pub(crate) type SubtreeCounts = HashMap<*const Entry, (u64, u64)>;

/// Walk the tree once bottom-up and record per-entry subtree counts.
/// `(file_count, dir_count_including_self)` so a parent can sum a
/// child's counts directly into its own without per-child branching.
pub(crate) fn precompute_subtree_counts(entry: &Entry) -> SubtreeCounts {
    let mut map: SubtreeCounts = HashMap::new();
    walk_counts(entry, &mut map);
    map
}

fn walk_counts(entry: &Entry, map: &mut SubtreeCounts) -> (u64, u64) {
    let mut files = if entry.is_dir() { 0 } else { 1 };
    let mut dirs = if entry.is_dir() { 1 } else { 0 };
    if let Some(children) = entry.children() {
        for child in children {
            let (cfc, cdc) = walk_counts(child, map);
            files += cfc;
            dirs += cdc;
        }
    }
    map.insert(entry as *const Entry, (files, dirs));
    (files, dirs)
}

/// Dispatch to the appropriate output backend. Each backend takes a
/// `&mut impl Write` so tests can capture output into a buffer. `--ui`
/// is handled in main.rs since it spins up an async server instead of
/// writing to a stream.
pub fn render(
    entry: &Entry,
    config: &OutputConfig,
    mode: OutputMode,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    match mode {
        OutputMode::Tree => tree::write(entry, config, out)?,
        OutputMode::Json => json::write(entry, config, out)?,
        OutputMode::Ndjson => ndjson::write(entry, config, out)?,
        OutputMode::Analyze => analyze::write(entry, out)?,
    }
    Ok(())
}
