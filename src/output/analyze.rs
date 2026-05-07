use std::collections::HashMap;
use std::io::{self, Write};

use super::format::format_size;
use crate::category::Category;
use crate::entry::Entry;

struct CategoryStats {
    size: u64,
    count: u64,
}

pub fn write(entry: &Entry, out: &mut impl Write) -> io::Result<()> {
    let mut stats: HashMap<Category, CategoryStats> = HashMap::new();
    collect_stats(entry, &mut stats);

    writeln!(out, "Total: {}", format_size(entry.size))?;
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
        let pct = (stat.size * 100).checked_div(entry.size).unwrap_or(0);
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

fn collect_stats(entry: &Entry, stats: &mut HashMap<Category, CategoryStats>) {
    if !entry.is_dir() {
        let stat = stats
            .entry(entry.category)
            .or_insert(CategoryStats { size: 0, count: 0 });
        stat.size += entry.size;
        stat.count += 1;
        return;
    }

    // For directories with a non-Other category, count the whole dir as that category
    if entry.category != Category::Other {
        let stat = stats
            .entry(entry.category)
            .or_insert(CategoryStats { size: 0, count: 0 });
        stat.size += entry.size;
        stat.count += 1;
        return;
    }

    // For Other directories, recurse into children
    if let Some(children) = entry.children() {
        for child in children {
            collect_stats(child, stats);
        }
    }
}
