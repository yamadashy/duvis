//! `--largest N`: flat list of the N largest entries (files + directories)
//! across the entire scanned tree, ordered by size descending.
//!
//! Output goes to whichever format flag was passed alongside `--largest`:
//! - default text → human-readable aligned table
//! - `--json` → `{meta, largest: [...]}` (no `tree` field; this is a flat
//!   query, not a hierarchical view)
//! - `--ndjson` → meta line + one `entry` line per result
//!
//! Mutually exclusive with `--analyze` and `--ui` (different views), but
//! orthogonal to `--json` / `--ndjson` (formats).

use std::io::{self, Write};

use anyhow::Result;
use serde::Serialize;

use super::format::format_size;
use super::{
    child_relative_path, hardlinks_label, is_zero_u64, precompute_subtree_counts,
    scan_root_for_wire, OutputConfig, SubtreeCounts, WIRE_VERSION,
};
use crate::category::Category;
use crate::entry::Entry;

/// Output target for `--largest` results. Selected by which format flag
/// (if any) was passed alongside `--largest`.
#[derive(Debug, Clone, Copy)]
pub enum LargestFormat {
    Text,
    Json,
    Ndjson,
}

/// One row in the largest-N list. Holds a borrow into the original tree
/// so we don't clone Entry data unnecessarily.
struct Row<'a> {
    entry: &'a Entry,
    relative_path: String,
    depth: u32,
}

/// Walk the tree and collect every entry. The scan root is normally
/// excluded — for a directory scan it's the implicit "everything under
/// here" container and would always rank #1 by size, swamping the list.
/// But when the user points duvis at a single *file* (`duvis some.bin
/// --largest 1`), the root *is* the only thing there is, and excluding
/// it would produce an empty list. So we include the root when it's a
/// file leaf.
fn collect_all<'a>(entry: &'a Entry, rel_path: String, depth: u32, out: &mut Vec<Row<'a>>) {
    if depth > 0 {
        out.push(Row {
            entry,
            relative_path: rel_path.clone(),
            depth,
        });
    } else if !entry.is_dir() {
        // File-as-scan-root special case: surface the file with its
        // name (not "."), matching what the user typed on the CLI.
        out.push(Row {
            entry,
            relative_path: entry.name.clone(),
            depth,
        });
    }
    if let Some(children) = entry.children() {
        for child in children {
            let child_path = child_relative_path(&rel_path, &child.name);
            collect_all(child, child_path, depth + 1, out);
        }
    }
}

/// Pick the top-N rows by size descending. Tie-break by `relative_path`
/// alphabetically so the output is deterministic across runs (rayon
/// scan order would otherwise leak through for equal-sized entries).
fn select_largest<'a>(rows: &mut Vec<Row<'a>>, n: usize) {
    rows.sort_by(|a, b| {
        b.entry
            .size
            .cmp(&a.entry.size)
            .then_with(|| a.relative_path.cmp(&b.relative_path))
    });
    rows.truncate(n);
}

pub fn write(
    entry: &Entry,
    config: &OutputConfig,
    n: usize,
    format: LargestFormat,
    out: &mut impl Write,
) -> Result<()> {
    let mut rows: Vec<Row<'_>> = Vec::new();
    collect_all(entry, ".".to_string(), 0, &mut rows);
    let total_entries = rows.len() as u64;
    select_largest(&mut rows, n);

    match format {
        LargestFormat::Text => write_text(&rows, config, n, total_entries, out)?,
        LargestFormat::Json => write_json(entry, &rows, config, n, total_entries, out)?,
        LargestFormat::Ndjson => write_ndjson(entry, &rows, config, n, total_entries, out)?,
    }
    Ok(())
}

/// Maximum width of the rendered path column in text output. A single
/// pathological deep path would otherwise push the rest of each row off
/// the screen.
const PATH_DISPLAY_CAP: usize = 60;

fn write_text(
    rows: &[Row<'_>],
    config: &OutputConfig,
    n_requested: usize,
    total_entries: u64,
    out: &mut impl Write,
) -> io::Result<()> {
    writeln!(
        out,
        "Largest {} entries in {} (of {} total):",
        rows.len().min(n_requested),
        config.scan_root.display(),
        total_entries,
    )?;
    writeln!(out)?;

    if rows.is_empty() {
        writeln!(out, "(scan tree is empty)")?;
        return Ok(());
    }

    // Pre-render every path so width and printed value match. Without
    // this the `{:<width$}` directive treated `width` as a *minimum*,
    // letting long paths overflow into the category column.
    let rendered: Vec<String> = rows.iter().map(render_path_for_text).collect();
    let path_width = rendered
        .iter()
        .map(|p| p.chars().count())
        .max()
        .unwrap_or(0);

    for (row, path) in rows.iter().zip(rendered.iter()) {
        let size = format_size(row.entry.size);
        let cat = format!("[{}]", row.entry.category.label());
        let kind = if row.entry.is_dir() { "dir" } else { "file" };
        writeln!(
            out,
            "{:>10}  {:<width$}  {:<13}  {}",
            size,
            path,
            cat,
            kind,
            width = path_width,
        )?;
    }
    Ok(())
}

/// `relative_path` with a trailing `/` for directories, truncated from
/// the *front* with `…` if it exceeds [`PATH_DISPLAY_CAP`]. The tail of
/// a path is more identifying than the head (deepest segment names the
/// actual file), so we keep that side.
fn render_path_for_text(row: &Row<'_>) -> String {
    let raw = if row.entry.is_dir() {
        format!("{}/", row.relative_path)
    } else {
        row.relative_path.clone()
    };
    let count = raw.chars().count();
    if count <= PATH_DISPLAY_CAP {
        return raw;
    }
    let keep = PATH_DISPLAY_CAP.saturating_sub(1);
    let tail: String = raw.chars().skip(count - keep).collect();
    format!("…{tail}")
}

#[derive(Serialize)]
struct LargestRoot<'a> {
    meta: Meta<'a>,
    largest: Vec<LargestEntry>,
}

#[derive(Serialize)]
struct Meta<'a> {
    wire_version: u32,
    duvis_version: &'static str,
    scan_root: String,
    hardlinks: &'a str,
    items_scanned: u64,
    items_skipped: u64,
    /// Surfaced so an agent can tell whether the list it sees was
    /// truncated by `--largest N` or genuinely smaller than N. Pair with
    /// `total_entries` to compute "we saw 10 of 1234".
    largest_requested: usize,
    total_entries: u64,
}

#[derive(Serialize)]
struct LargestEntry {
    name: String,
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
}

fn make_meta<'a>(config: &'a OutputConfig<'a>, n_requested: usize, total_entries: u64) -> Meta<'a> {
    Meta {
        wire_version: WIRE_VERSION,
        duvis_version: env!("CARGO_PKG_VERSION"),
        scan_root: scan_root_for_wire(config.scan_root),
        hardlinks: hardlinks_label(config.hardlinks),
        items_scanned: config.counts.scanned(),
        items_skipped: config.counts.skipped(),
        largest_requested: n_requested,
        total_entries,
    }
}

fn build_entry(row: &Row<'_>, counts: &SubtreeCounts) -> LargestEntry {
    let (file_count, dir_count_with_self) = counts
        .get(&(row.entry as *const Entry))
        .copied()
        .unwrap_or((0, 0));
    let dir_count = dir_count_with_self.saturating_sub(if row.entry.is_dir() { 1 } else { 0 });
    LargestEntry {
        name: row.entry.name.clone(),
        relative_path: row.relative_path.clone(),
        depth: row.depth,
        size: row.entry.size,
        size_human: format_size(row.entry.size),
        is_dir: row.entry.is_dir(),
        category: row.entry.category,
        modified_days_ago: row.entry.modified_days_ago,
        file_count,
        dir_count,
    }
}

fn write_json(
    tree_root: &Entry,
    rows: &[Row<'_>],
    config: &OutputConfig,
    n_requested: usize,
    total_entries: u64,
    out: &mut impl Write,
) -> Result<()> {
    let counts = precompute_subtree_counts(tree_root);
    let entries: Vec<LargestEntry> = rows.iter().map(|r| build_entry(r, &counts)).collect();
    let root = LargestRoot {
        meta: make_meta(config, n_requested, total_entries),
        largest: entries,
    };
    serde_json::to_writer_pretty(&mut *out, &root)?;
    writeln!(out)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum NdjsonRecord<'a> {
    Meta(Meta<'a>),
    Entry(NdjsonEntryRecord<'a>),
}

#[derive(Serialize)]
struct NdjsonEntryRecord<'a> {
    name: &'a str,
    relative_path: &'a str,
    depth: u32,
    size: u64,
    size_human: String,
    is_dir: bool,
    category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_days_ago: Option<u64>,
    file_count: u64,
    dir_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    truncated_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    truncated_size: u64,
}

fn write_ndjson(
    tree_root: &Entry,
    rows: &[Row<'_>],
    config: &OutputConfig,
    n_requested: usize,
    total_entries: u64,
    out: &mut impl Write,
) -> Result<()> {
    let meta_rec = NdjsonRecord::Meta(make_meta(config, n_requested, total_entries));
    serde_json::to_writer(&mut *out, &meta_rec)?;
    writeln!(out)?;

    let counts = precompute_subtree_counts(tree_root);
    for row in rows {
        let (file_count, dir_count_with_self) = counts
            .get(&(row.entry as *const Entry))
            .copied()
            .unwrap_or((0, 0));
        let dir_count = dir_count_with_self.saturating_sub(if row.entry.is_dir() { 1 } else { 0 });
        let rec = NdjsonRecord::Entry(NdjsonEntryRecord {
            name: &row.entry.name,
            relative_path: &row.relative_path,
            depth: row.depth,
            size: row.entry.size,
            size_human: format_size(row.entry.size),
            is_dir: row.entry.is_dir(),
            category: row.entry.category,
            modified_days_ago: row.entry.modified_days_ago,
            file_count,
            dir_count,
            // truncated_* never set in flat largest mode — there's no
            // parent / child relationship being clipped here. Kept on
            // the record so the schema matches the tree-form NDJSON for
            // downstream consumers that share parsing logic.
            truncated_count: 0,
            truncated_size: 0,
        });
        serde_json::to_writer(&mut *out, &rec)?;
        writeln!(out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{HardlinkPolicy, ScanCounts};
    use std::path::PathBuf;

    fn dir(name: &str, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), Category::Other, None, children)
    }

    fn file(name: &str, size: u64, cat: Category) -> Entry {
        Entry::file(name.to_string(), size, cat, None)
    }

    fn fixture() -> Entry {
        // proj/
        //   target/  (build, big)
        //     app    (200_000)
        //   src/
        //     a.rs   (100)
        //     b.rs   (50)
        //   .git/    (vcs)
        //     HEAD   (10)
        dir(
            "proj",
            vec![
                dir("target", vec![file("app", 200_000, Category::Build)]),
                dir(
                    "src",
                    vec![
                        file("a.rs", 100, Category::Other),
                        file("b.rs", 50, Category::Other),
                    ],
                ),
                dir("git", vec![file("HEAD", 10, Category::Vcs)]),
            ],
        )
    }

    fn cfg<'a>(scan_root: &'a PathBuf, counts: &'a ScanCounts) -> OutputConfig<'a> {
        OutputConfig {
            depth: None,
            top: None,
            scan_root,
            counts,
            hardlinks: HardlinkPolicy::CountOnce,
        }
    }

    #[test]
    fn collect_all_excludes_root_and_visits_every_descendant() {
        let tree = fixture();
        let mut rows: Vec<Row<'_>> = Vec::new();
        collect_all(&tree, ".".to_string(), 0, &mut rows);
        let paths: Vec<String> = rows.iter().map(|r| r.relative_path.clone()).collect();
        // 3 dirs + 4 files = 7 descendants, scan root not included.
        assert_eq!(rows.len(), 7);
        assert!(paths.contains(&"target".to_string()));
        assert!(paths.contains(&"target/app".to_string()));
        assert!(paths.contains(&"src/a.rs".to_string()));
        assert!(!paths.contains(&".".to_string()), "root should be excluded");
    }

    #[test]
    fn collect_all_includes_file_scan_root() {
        // `duvis --largest 1 some_file.txt` should still produce a result.
        let root = file("solo.bin", 1024, Category::Other);
        let mut rows: Vec<Row<'_>> = Vec::new();
        collect_all(&root, ".".to_string(), 0, &mut rows);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relative_path, "solo.bin");
        assert!(!rows[0].entry.is_dir());
    }

    #[test]
    fn render_path_for_text_truncates_long_paths_from_the_front() {
        let long_name = "a".repeat(80);
        let row = Row {
            entry: &file(&long_name, 0, Category::Other),
            relative_path: long_name.clone(),
            depth: 1,
        };
        let rendered = render_path_for_text(&row);
        assert!(
            rendered.chars().count() <= PATH_DISPLAY_CAP,
            "truncated path exceeded cap: {rendered:?}",
        );
        assert!(
            rendered.starts_with('…'),
            "ellipsis should mark front-truncation: {rendered:?}",
        );
        // Tail (the identifying part) is preserved.
        assert!(
            rendered.ends_with("aaaaa"),
            "tail not preserved: {rendered:?}"
        );
    }

    #[test]
    fn select_largest_orders_by_size_desc_with_path_tiebreak() {
        let tree = fixture();
        let mut rows: Vec<Row<'_>> = Vec::new();
        collect_all(&tree, ".".to_string(), 0, &mut rows);
        select_largest(&mut rows, 3);
        let paths: Vec<&str> = rows.iter().map(|r| r.relative_path.as_str()).collect();
        // target dir (200_000) > target/app (200_000, tie broken by path → "target" < "target/app")
        // > src dir (150)
        assert_eq!(paths, vec!["target", "target/app", "src"]);
    }

    #[test]
    fn write_text_renders_aligned_table_with_dir_marker() {
        let tree = fixture();
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = ScanCounts::default();
        let cfg = cfg(&scan_root, &counts);
        let mut buf: Vec<u8> = Vec::new();
        write(&tree, &cfg, 3, LargestFormat::Text, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Largest 3 entries"));
        // Dir entries get a trailing slash.
        assert!(output.contains("target/"));
        // Files don't.
        assert!(output.contains("target/app  "));
        // Header mentions total entry count for context.
        assert!(output.contains("of 7 total"));
    }

    #[test]
    fn write_json_returns_meta_plus_flat_largest_array() {
        let tree = fixture();
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = ScanCounts::default();
        let cfg = cfg(&scan_root, &counts);
        let mut buf: Vec<u8> = Vec::new();
        write(&tree, &cfg, 2, LargestFormat::Json, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(v["meta"]["wire_version"], 2);
        assert_eq!(v["meta"]["largest_requested"], 2);
        assert_eq!(v["meta"]["total_entries"], 7);
        // No `tree` field — this is a flat list, not a hierarchical
        // view. Agents shouldn't expect to walk children here.
        assert!(v.get("tree").is_none());
        let largest = v["largest"].as_array().unwrap();
        assert_eq!(largest.len(), 2);
        assert_eq!(largest[0]["relative_path"], "target");
    }

    #[test]
    fn write_ndjson_emits_meta_then_one_line_per_result() {
        let tree = fixture();
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = ScanCounts::default();
        let cfg = cfg(&scan_root, &counts);
        let mut buf: Vec<u8> = Vec::new();
        write(&tree, &cfg, 3, LargestFormat::Ndjson, &mut buf).unwrap();
        let lines: Vec<serde_json::Value> = std::str::from_utf8(&buf)
            .unwrap()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 4); // meta + 3 entries
        assert_eq!(lines[0]["type"], "meta");
        assert_eq!(lines[0]["largest_requested"], 3);
        for line in &lines[1..] {
            assert_eq!(line["type"], "entry");
        }
    }
}
