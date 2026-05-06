pub mod analyze;
mod format;
pub mod json;
pub mod tree;

use std::io::Write;

use crate::entry::Entry;

pub struct OutputConfig {
    pub depth: Option<usize>,
    pub top: Option<usize>,
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

/// Dispatch to the appropriate output backend. Each backend takes a
/// `&mut impl Write` so tests can capture output into a buffer.
///
/// `--json` and `--analyze` are mutually exclusive (enforced by clap's
/// ArgGroup); when neither is set we fall back to the human-friendly tree.
/// `--ui` is dispatched by main.rs since it spins up an async server
/// instead of writing to a stream.
pub fn render(
    entry: &Entry,
    config: &OutputConfig,
    json: bool,
    analyze: bool,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    if analyze {
        analyze::write(entry, out)?;
    } else if json {
        json::write(entry, config, out)?;
    } else {
        tree::write(entry, config, out)?;
    }
    Ok(())
}
