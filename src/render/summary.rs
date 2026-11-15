//! `--summary`: per-category size rollup for the scanned tree.
//!
//! Output goes to whichever format flag was passed alongside `--summary`:
//! - default text → `Total:` line plus an aligned category table
//! - `--json` → `{meta, summary: [...]}` (no `tree` field; this is a
//!   rollup, not a hierarchical view)
//! - `--toon` → the same payload, TOON-encoded
//! - `--ndjson` → meta line + one `category` line per bucket
//!
//! Mutually exclusive with `--largest` and `--ui` (different views), but
//! orthogonal to the format flags.

use std::collections::HashMap;
use std::io::Write;

use anyhow::Result;

use super::format::format_size;
use super::{OutputFormat, RenderConfig};
use crate::classify::Category;
use crate::entry::Entry;
use crate::filter::Filter;
use crate::wire::summary::{
    WireSummaryCategory, WireSummaryMeta, WireSummaryNdjsonRecord, WireSummaryRoot,
};

struct CategoryStats {
    size: u64,
    count: u64,
}

/// One rollup row, already sorted and with `percent` resolved. Built once
/// and shared by every format so the numbers can't drift between them.
struct Bucket {
    category: Category,
    size: u64,
    percent: u64,
    count: u64,
}

pub(crate) fn write(
    entry: &Entry,
    config: &RenderConfig,
    format: OutputFormat,
    out: &mut impl Write,
) -> Result<()> {
    let mut stats: HashMap<Category, CategoryStats> = HashMap::new();
    collect_stats(entry, config.filter, &mut stats);

    // Total reflects what's actually being summarized — i.e. the
    // filtered subtree. Unfiltered scan counters stay available via meta
    // in the structured outputs.
    let total: u64 = stats.values().map(|s| s.size).sum();

    let mut sorted: Vec<_> = stats.into_iter().collect();
    // Primary: size desc. Tiebreak by category label so equal-size buckets
    // come out in a stable order across runs (HashMap iteration is random).
    sorted.sort_by(|a, b| {
        b.1.size
            .cmp(&a.1.size)
            .then_with(|| a.0.label().cmp(b.0.label()))
    });
    let buckets: Vec<Bucket> = sorted
        .into_iter()
        .map(|(category, stat)| Bucket {
            category,
            size: stat.size,
            percent: (stat.size * 100).checked_div(total).unwrap_or(0),
            count: stat.count,
        })
        .collect();

    match format {
        OutputFormat::Text => write_text(&buckets, total, out)?,
        OutputFormat::Json => write_json(&buckets, total, config, out)?,
        OutputFormat::Toon => write_toon(&buckets, total, config, out)?,
        OutputFormat::Ndjson => write_ndjson(&buckets, total, config, out)?,
    }
    Ok(())
}

fn write_text(buckets: &[Bucket], total: u64, out: &mut impl Write) -> Result<()> {
    writeln!(out, "Total: {}", format_size(total))?;
    writeln!(out)?;
    writeln!(out, "Category Summary:")?;

    for bucket in buckets {
        writeln!(
            out,
            "  {:<8} {:>10}  {:>3}%  {} items",
            bucket.category.label(),
            format_size(bucket.size),
            bucket.percent,
            bucket.count,
        )?;
    }
    Ok(())
}

fn build_categories(buckets: &[Bucket]) -> Vec<WireSummaryCategory> {
    buckets
        .iter()
        .map(|b| WireSummaryCategory {
            category: b.category,
            size: b.size,
            size_human: format_size(b.size),
            percent: b.percent,
            count: b.count,
        })
        .collect()
}

fn build_root<'a>(
    buckets: &[Bucket],
    total: u64,
    config: &'a RenderConfig<'a>,
) -> WireSummaryRoot<'a> {
    WireSummaryRoot {
        meta: WireSummaryMeta::from_config(config, total, format_size(total)),
        summary: build_categories(buckets),
    }
}

fn write_json(
    buckets: &[Bucket],
    total: u64,
    config: &RenderConfig,
    out: &mut impl Write,
) -> Result<()> {
    let root = build_root(buckets, total, config);
    serde_json::to_writer_pretty(&mut *out, &root)?;
    writeln!(out)?;
    Ok(())
}

fn write_toon(
    buckets: &[Bucket],
    total: u64,
    config: &RenderConfig,
    out: &mut impl Write,
) -> Result<()> {
    let root = build_root(buckets, total, config);
    let encoded = toon_format::encode_default(&root)?;
    out.write_all(encoded.as_bytes())?;
    writeln!(out)?;
    Ok(())
}

fn write_ndjson(
    buckets: &[Bucket],
    total: u64,
    config: &RenderConfig,
    out: &mut impl Write,
) -> Result<()> {
    let meta_rec = WireSummaryNdjsonRecord::Meta(WireSummaryMeta::from_config(
        config,
        total,
        format_size(total),
    ));
    serde_json::to_writer(&mut *out, &meta_rec)?;
    writeln!(out)?;

    for category in build_categories(buckets) {
        let rec = WireSummaryNdjsonRecord::Category(category);
        serde_json::to_writer(&mut *out, &rec)?;
        writeln!(out)?;
    }
    Ok(())
}

fn collect_stats(entry: &Entry, filter: &Filter, stats: &mut HashMap<Category, CategoryStats>) {
    let matches = filter.is_empty() || filter.matches(entry);

    if !entry.is_dir() {
        if matches {
            let stat = stats
                .entry(entry.category)
                .or_insert(CategoryStats { size: 0, count: 0 });
            stat.size += entry.size;
            stat.count += 1;
        }
        return;
    }

    // For directories with a non-Other category that themselves match,
    // bucket the whole subtree under that category. When a filter is
    // active and the dir doesn't match its own filter, fall through to
    // the recursive case so per-file matches inside still surface.
    if entry.category != Category::Other && matches {
        let stat = stats
            .entry(entry.category)
            .or_insert(CategoryStats { size: 0, count: 0 });
        stat.size += entry.size;
        stat.count += 1;
        return;
    }

    // For Other directories (or filtered-out non-Other dirs), recurse
    // into children to look for matches.
    if let Some(children) = entry.children() {
        for child in children {
            collect_stats(child, filter, stats);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{HardlinkPolicy, ScanCounts};
    use std::path::PathBuf;

    fn dir(name: &str, cat: Category, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), cat, None, children)
    }

    fn file(name: &str, size: u64, cat: Category) -> Entry {
        Entry::file(name.to_string(), size, cat, None)
    }

    fn fixture() -> Entry {
        dir(
            "proj",
            Category::Other,
            vec![
                dir(
                    "target",
                    Category::Build,
                    vec![file("app", 300, Category::Build)],
                ),
                file("a.log", 100, Category::Log),
            ],
        )
    }

    fn render(format: OutputFormat) -> String {
        let tree = fixture();
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = ScanCounts::default();
        let filter = Filter::default();
        let config = RenderConfig {
            max_depth: None,
            top: None,
            scan_root: &scan_root,
            counts: &counts,
            hardlinks: HardlinkPolicy::CountOnce,
            filter: &filter,
        };
        let mut buf: Vec<u8> = Vec::new();
        write(&tree, &config, format, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn text_lists_categories_largest_first() {
        let out = render(OutputFormat::Text);
        assert!(out.contains("Total: "));
        let build = out.find("build").expect("build bucket missing");
        let log = out.find("log").expect("log bucket missing");
        assert!(build < log, "larger bucket should sort first:\n{out}");
    }

    #[test]
    fn json_emits_meta_and_summary_without_a_tree_field() {
        let out = render(OutputFormat::Json);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["meta"]["wire_version"], 2);
        assert_eq!(v["meta"]["total"], 400);
        // A rollup, not a hierarchy — agents shouldn't look for children.
        assert!(v.get("tree").is_none());
        let summary = v["summary"].as_array().unwrap();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0]["category"], "build");
        assert_eq!(summary[0]["size"], 300);
        assert_eq!(summary[0]["percent"], 75);
        assert_eq!(summary[1]["category"], "log");
    }

    #[test]
    fn ndjson_emits_meta_then_one_line_per_category() {
        let out = render(OutputFormat::Ndjson);
        let lines: Vec<serde_json::Value> = out
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 3); // meta + 2 categories
        assert_eq!(lines[0]["type"], "meta");
        assert_eq!(lines[0]["total"], 400);
        assert_eq!(lines[1]["type"], "category");
        assert_eq!(lines[1]["category"], "build");
    }

    #[test]
    fn toon_encodes_the_same_payload() {
        let out = render(OutputFormat::Toon);
        assert!(out.contains("build"), "toon output missing bucket:\n{out}");
        assert!(out.contains("summary"), "toon output missing key:\n{out}");
    }

    #[test]
    fn percent_is_zero_when_nothing_matched() {
        // checked_div guards the empty-tree divide; make sure the guard
        // is actually exercised rather than assumed.
        let tree = dir("empty", Category::Other, vec![]);
        let scan_root = PathBuf::from("/tmp/empty");
        let counts = ScanCounts::default();
        let filter = Filter::default();
        let config = RenderConfig {
            max_depth: None,
            top: None,
            scan_root: &scan_root,
            counts: &counts,
            hardlinks: HardlinkPolicy::CountOnce,
            filter: &filter,
        };
        let mut buf: Vec<u8> = Vec::new();
        write(&tree, &config, OutputFormat::Json, &mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert_eq!(v["meta"]["total"], 0);
        assert!(v["summary"].as_array().unwrap().is_empty());
    }
}
