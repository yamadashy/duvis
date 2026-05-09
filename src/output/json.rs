use std::io::Write;

use anyhow::Result;
use serde::Serialize;

use super::filter::{precompute_subtree_match, subtree_visible, SubtreeMatch};
use super::format::format_size;
use super::{
    child_relative_path, hardlinks_label, is_zero_u64, precompute_subtree_counts,
    scan_root_for_wire, select_top_refs, OutputConfig, SubtreeCounts, WIRE_VERSION,
};
use crate::category::Category;
use crate::entry::Entry;

#[derive(Serialize)]
struct JsonRoot<'a> {
    meta: Meta<'a>,
    tree: JsonOutput,
}

#[derive(Serialize)]
struct Meta<'a> {
    wire_version: u32,
    duvis_version: &'static str,
    scan_root: String,
    hardlinks: &'a str,
    items_scanned: u64,
    items_skipped: u64,
}

#[derive(Serialize)]
struct JsonOutput {
    name: String,
    /// Path from the scan root, `/`-separated. Root itself is `"."`.
    relative_path: String,
    /// 0 at scan root, +1 per directory level.
    depth: u32,
    size: u64,
    size_human: String,
    is_dir: bool,
    category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_days_ago: Option<u64>,
    /// Total regular files in this subtree (recursive). Constant for a
    /// given Entry — does *not* change with `--top` / `--depth` since
    /// those only affect what we emit, not what was measured.
    file_count: u64,
    /// Total directories in this subtree, excluding self.
    dir_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<JsonOutput>>,
    /// Direct children dropped at this level by `--top`. Zero (omitted)
    /// when no top filter applied. Distinct from depth-truncation: that
    /// shows up as a non-zero `file_count`/`dir_count` with no
    /// `children` array, and is implicit.
    #[serde(skip_serializing_if = "is_zero_u64")]
    truncated_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    truncated_size: u64,
}

/// Build the JSON view, looking up precomputed counts in `counts` so
/// each entry is touched O(1) times. When a filter is active, children
/// outside the visible set (no self-match and no descendant match) are
/// skipped.
fn build(
    entry: &Entry,
    relative_path: String,
    depth: u32,
    max_depth: Option<usize>,
    top: Option<usize>,
    counts: &SubtreeCounts,
    visible: Option<&SubtreeMatch>,
) -> JsonOutput {
    let render_children = !matches!(max_depth, Some(max) if depth as usize >= max);

    let mut children_out: Option<Vec<JsonOutput>> = None;
    let mut truncated_count: u64 = 0;
    let mut truncated_size: u64 = 0;

    if let Some(children) = entry.children() {
        if render_children {
            // Filter survives → --top, so an active filter narrows the
            // pool first and `truncated_*` reports drops *within* the
            // surviving set.
            let filtered: Vec<&Entry> = match visible {
                None => children.iter().collect(),
                Some(map) => children
                    .iter()
                    .filter(|c| subtree_visible(c, map))
                    .collect(),
            };
            let (kept, dropped_count, dropped_size) = select_top_refs(&filtered, top);
            truncated_count = dropped_count as u64;
            truncated_size = dropped_size;

            let mut rendered = Vec::with_capacity(kept.len());
            for child in kept {
                let child_path = child_relative_path(&relative_path, &child.name);
                rendered.push(build(
                    child,
                    child_path,
                    depth + 1,
                    max_depth,
                    top,
                    counts,
                    visible,
                ));
            }
            children_out = Some(rendered);
        }
    }

    // Counts always reflect the *full* scanned subtree, even when we
    // don't render the children (depth limit) or drop some of them
    // (--top). This is what lets an agent tell "you only see 5 of 200
    // children".
    let (file_count, dir_count_with_self) = counts
        .get(&(entry as *const Entry))
        .copied()
        .unwrap_or((0, 0));
    let dir_count = dir_count_with_self.saturating_sub(if entry.is_dir() { 1 } else { 0 });

    JsonOutput {
        name: entry.name.clone(),
        relative_path,
        depth,
        size: entry.size,
        size_human: format_size(entry.size),
        is_dir: entry.is_dir(),
        category: entry.category,
        modified_days_ago: entry.modified_days_ago,
        file_count,
        dir_count,
        children: children_out,
        truncated_count,
        truncated_size,
    }
}

pub fn write(entry: &Entry, config: &OutputConfig, out: &mut impl Write) -> Result<()> {
    let counts = precompute_subtree_counts(entry);
    let visible_map = if config.filter.is_empty() {
        None
    } else {
        Some(precompute_subtree_match(entry, config.filter))
    };
    let tree = build(
        entry,
        ".".to_string(),
        0,
        config.depth,
        config.top,
        &counts,
        visible_map.as_ref(),
    );
    let root = JsonRoot {
        meta: Meta {
            wire_version: WIRE_VERSION,
            duvis_version: env!("CARGO_PKG_VERSION"),
            scan_root: scan_root_for_wire(config.scan_root),
            hardlinks: hardlinks_label(config.hardlinks),
            items_scanned: config.counts.scanned(),
            items_skipped: config.counts.skipped(),
        },
        tree,
    };
    serde_json::to_writer_pretty(&mut *out, &root)?;
    writeln!(out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Category;
    use crate::entry::Entry;
    use crate::output::filter::Filter;
    use crate::scanner::HardlinkPolicy;
    use std::path::PathBuf;
    use std::sync::atomic::Ordering;

    fn dir(name: &str, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), Category::Other, None, children)
    }

    fn file(name: &str, size: u64) -> Entry {
        Entry::file(name.to_string(), size, Category::Other, None)
    }

    fn fake_config<'a>(
        scan_root: &'a PathBuf,
        counts: &'a crate::scanner::ScanCounts,
        filter: &'a Filter,
    ) -> OutputConfig<'a> {
        OutputConfig {
            depth: None,
            top: None,
            scan_root,
            counts,
            hardlinks: HardlinkPolicy::CountOnce,
            filter,
        }
    }

    #[test]
    fn meta_block_carries_scan_root_and_hardlinks() {
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scanner::ScanCounts::default();
        counts.items_scanned.store(42, Ordering::Relaxed);
        let filter = Filter::default();
        let cfg = fake_config(&scan_root, &counts, &filter);
        let tree = dir("proj", vec![file("a.txt", 10)]);
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(v["meta"]["wire_version"], 2);
        assert_eq!(v["meta"]["scan_root"], "/tmp/proj");
        assert_eq!(v["meta"]["hardlinks"], "count-once");
        assert_eq!(v["meta"]["items_scanned"], 42);
    }

    #[test]
    fn relative_path_root_is_dot_and_descendants_use_forward_slashes() {
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scanner::ScanCounts::default();
        let filter = Filter::default();
        let cfg = fake_config(&scan_root, &counts, &filter);
        let tree = dir(
            "proj",
            vec![dir("src", vec![file("main.rs", 5)]), file("readme.md", 3)],
        );
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(v["tree"]["relative_path"], ".");
        assert_eq!(v["tree"]["depth"], 0);
        let children = v["tree"]["children"].as_array().unwrap();
        let src = children
            .iter()
            .find(|c| c["name"] == "src")
            .expect("src entry");
        assert_eq!(src["relative_path"], "src");
        assert_eq!(src["depth"], 1);
        let main_rs = src["children"][0].clone();
        assert_eq!(main_rs["relative_path"], "src/main.rs");
        assert_eq!(main_rs["depth"], 2);
    }

    #[test]
    fn file_and_dir_counts_aggregate_recursively() {
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scanner::ScanCounts::default();
        let filter = Filter::default();
        let cfg = fake_config(&scan_root, &counts, &filter);
        // proj/  (1 dir self + 1 dir src) → dir_count = 1 ; file_count = 3
        //   src/
        //     a.txt
        //     b.txt
        //   c.txt
        let tree = dir(
            "proj",
            vec![
                dir("src", vec![file("a.txt", 1), file("b.txt", 2)]),
                file("c.txt", 3),
            ],
        );
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(v["tree"]["file_count"], 3);
        assert_eq!(v["tree"]["dir_count"], 1);
    }

    #[test]
    fn truncated_counts_reflect_top_drops() {
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scanner::ScanCounts::default();
        let filter = Filter::default();
        let cfg = OutputConfig {
            depth: None,
            top: Some(1),
            scan_root: &scan_root,
            counts: &counts,
            hardlinks: HardlinkPolicy::CountOnce,
            filter: &filter,
        };
        // 3 files, --top 1 keeps the largest, 2 are dropped.
        let tree = dir(
            "proj",
            vec![
                file("big.bin", 100),
                file("mid.bin", 50),
                file("small.bin", 5),
            ],
        );
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(v["tree"]["truncated_count"], 2);
        assert_eq!(v["tree"]["truncated_size"], 55);
        // file_count still reflects the full subtree, not just the kept one.
        assert_eq!(v["tree"]["file_count"], 3);
    }
}
