//! NDJSON output: one record per line, streamable, jq / DB / agent friendly.
//!
//! The first record is a `meta` object (same content as `--json`'s `meta`
//! block, plus a `type: "meta"` discriminator). Every subsequent record is
//! an `entry` in DFS pre-order so an agent reading the stream sees a
//! parent before its children. Each entry carries enough context
//! (`relative_path`, `depth`, `file_count`, `dir_count`, `truncated_*`)
//! to be processed independently — there's no cross-line state required
//! to reconstruct the tree.

use std::io::Write;

use anyhow::Result;
use serde::Serialize;

use super::format::format_size;
use super::{precompute_subtree_counts, select_top, OutputConfig, SubtreeCounts};
use crate::category::Category;
use crate::entry::Entry;
use crate::scanner::HardlinkPolicy;

const WIRE_VERSION: u32 = 2;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Record<'a> {
    Meta(MetaRecord<'a>),
    Entry(EntryRecord<'a>),
}

#[derive(Serialize)]
struct MetaRecord<'a> {
    wire_version: u32,
    duvis_version: &'static str,
    scan_root: String,
    hardlinks: &'a str,
    items_scanned: u64,
    items_skipped: u64,
}

#[derive(Serialize)]
struct EntryRecord<'a> {
    name: &'a str,
    relative_path: String,
    depth: u32,
    size: u64,
    size_human: String,
    is_dir: bool,
    category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_days_ago: Option<u64>,
    file_count: u64,
    dir_count: u64,
    /// Direct children dropped at this level by `--top`. Zero (omitted)
    /// when no top filter applied.
    #[serde(skip_serializing_if = "is_zero_u64")]
    truncated_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    truncated_size: u64,
}

fn is_zero_u64(n: &u64) -> bool {
    *n == 0
}

fn hardlinks_label(p: HardlinkPolicy) -> &'static str {
    match p {
        HardlinkPolicy::CountOnce => "count-once",
        HardlinkPolicy::CountEach => "count-each",
    }
}

/// `/`-normalized path. Mirrors json.rs::child_relative_path.
fn child_relative_path(parent: &str, name: &str) -> String {
    let normalized = name.replace('\\', "/");
    if parent == "." {
        normalized
    } else {
        format!("{parent}/{normalized}")
    }
}

fn write_meta(out: &mut impl Write, config: &OutputConfig) -> Result<()> {
    let rec = Record::Meta(MetaRecord {
        wire_version: WIRE_VERSION,
        duvis_version: env!("CARGO_PKG_VERSION"),
        scan_root: config.scan_root.display().to_string().replace('\\', "/"),
        hardlinks: hardlinks_label(config.hardlinks),
        items_scanned: config.counts.scanned(),
        items_skipped: config.counts.skipped(),
    });
    serde_json::to_writer(&mut *out, &rec)?;
    writeln!(out)?;
    Ok(())
}

fn write_entry(
    entry: &Entry,
    relative_path: String,
    depth: u32,
    max_depth: Option<usize>,
    top: Option<usize>,
    counts: &SubtreeCounts,
    out: &mut impl Write,
) -> Result<()> {
    // Counts always reflect the *full* scanned subtree at this entry, not
    // the truncated view. Agents need this so they can detect "we only
    // see N of M children" even when --depth or --top is in play.
    let (file_count, dir_count_with_self) = counts
        .get(&(entry as *const Entry))
        .copied()
        .unwrap_or((0, 0));
    let dir_count = dir_count_with_self.saturating_sub(if entry.is_dir() { 1 } else { 0 });

    let render_children = !matches!(max_depth, Some(max) if depth as usize >= max);

    let mut truncated_count: u64 = 0;
    let mut truncated_size: u64 = 0;
    let mut to_render: Vec<&Entry> = Vec::new();
    if let Some(children) = entry.children() {
        if render_children {
            let (kept, dropped_count, dropped_size) = select_top(children, top);
            truncated_count = dropped_count as u64;
            truncated_size = dropped_size;
            to_render = kept;
        }
    }

    let rec = Record::Entry(EntryRecord {
        name: &entry.name,
        relative_path: relative_path.clone(),
        depth,
        size: entry.size,
        size_human: format_size(entry.size),
        is_dir: entry.is_dir(),
        category: entry.category,
        modified_days_ago: entry.modified_days_ago,
        file_count,
        dir_count,
        truncated_count,
        truncated_size,
    });
    serde_json::to_writer(&mut *out, &rec)?;
    writeln!(out)?;

    for child in to_render {
        let child_path = child_relative_path(&relative_path, &child.name);
        write_entry(child, child_path, depth + 1, max_depth, top, counts, out)?;
    }
    Ok(())
}

pub fn write(entry: &Entry, config: &OutputConfig, out: &mut impl Write) -> Result<()> {
    write_meta(out, config)?;
    let counts = precompute_subtree_counts(entry);
    write_entry(
        entry,
        ".".to_string(),
        0,
        config.depth,
        config.top,
        &counts,
        out,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Category;
    use crate::entry::Entry;
    use std::path::PathBuf;

    fn dir(name: &str, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), Category::Other, None, children)
    }

    fn file(name: &str, size: u64) -> Entry {
        Entry::file(name.to_string(), size, Category::Other, None)
    }

    fn parse_lines(buf: &[u8]) -> Vec<serde_json::Value> {
        std::str::from_utf8(buf)
            .unwrap()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).expect("each ndjson line is valid json"))
            .collect()
    }

    #[test]
    fn first_line_is_meta_then_dfs_pre_order_entries() {
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scanner::ScanCounts::default();
        let cfg = OutputConfig {
            depth: None,
            top: None,
            scan_root: &scan_root,
            counts: &counts,
            hardlinks: HardlinkPolicy::CountOnce,
        };
        let tree = dir(
            "proj",
            vec![
                dir("src", vec![file("a.txt", 1), file("b.txt", 2)]),
                file("c.txt", 3),
            ],
        );
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let lines = parse_lines(&buf);
        assert_eq!(lines[0]["type"], "meta");
        assert_eq!(lines[1]["type"], "entry");
        // Pre-order: parent before children.
        let paths: Vec<String> = lines[1..]
            .iter()
            .map(|r| r["relative_path"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(paths, vec![".", "src", "src/a.txt", "src/b.txt", "c.txt"]);
    }

    #[test]
    fn each_entry_carries_full_subtree_counts() {
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scanner::ScanCounts::default();
        let cfg = OutputConfig {
            depth: None,
            top: None,
            scan_root: &scan_root,
            counts: &counts,
            hardlinks: HardlinkPolicy::CountOnce,
        };
        let tree = dir(
            "proj",
            vec![dir("src", vec![file("a.txt", 1)]), file("b.txt", 2)],
        );
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let lines = parse_lines(&buf);
        let root = lines.iter().find(|l| l["relative_path"] == ".").unwrap();
        assert_eq!(root["file_count"], 2);
        assert_eq!(root["dir_count"], 1);
        let src = lines.iter().find(|l| l["relative_path"] == "src").unwrap();
        assert_eq!(src["file_count"], 1);
        assert_eq!(src["dir_count"], 0);
    }
}
