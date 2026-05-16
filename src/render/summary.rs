use std::collections::HashMap;
use std::io::{self, Write};

use super::format::format_size;
use super::RenderConfig;
use crate::classify::Category;
use crate::entry::Entry;
use crate::filter::Filter;

struct CategoryStats {
    size: u64,
    count: u64,
}

pub fn write(entry: &Entry, config: &RenderConfig, out: &mut impl Write) -> io::Result<()> {
    let mut stats: HashMap<Category, CategoryStats> = HashMap::new();
    collect_stats(entry, config.filter, &mut stats);

    // Total reflects what's actually being summarized — i.e. the
    // filtered subtree. Unfiltered total stays available via meta in
    // the structured outputs; the summary view is a single-glance
    // view so we don't bother showing both.
    let total: u64 = stats.values().map(|s| s.size).sum();
    writeln!(out, "Total: {}", format_size(total))?;
    writeln!(out)?;
    writeln!(out, "Category Summary:")?;

    let mut sorted: Vec<_> = stats.into_iter().collect();
    // Primary: size desc. Tiebreak by category label so equal-size buckets
    // come out in a stable order across runs (HashMap iteration is random).
    sorted.sort_by(|a, b| {
        b.1.size
            .cmp(&a.1.size)
            .then_with(|| a.0.label().cmp(b.0.label()))
    });

    for (category, stat) in &sorted {
        let pct = (stat.size * 100).checked_div(total).unwrap_or(0);
        writeln!(
            out,
            "  {:<8} {:>10}  {:>3}%  {} items",
            category.label(),
            format_size(stat.size),
            pct,
            stat.count,
        )?;
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
