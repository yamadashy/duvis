use std::io::{self, Write};

use super::filter::{precompute_subtree_match, subtree_visible, SubtreeMatch};
use super::format::format_size;
use super::{select_top, select_top_refs, RenderConfig};
use crate::category::Category;
use crate::entry::Entry;

pub fn write(entry: &Entry, config: &RenderConfig, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{} ({})", entry.name, format_size(entry.size))?;

    // `None` when no filter is active so the "not applicable" case is
    // encoded in the type, matching json.rs / ndjson.rs. This avoids a
    // latent foot-gun where a placeholder empty `SubtreeMatch` would
    // silently treat all entries as invisible if the short-circuit
    // ever moved.
    let visible_map = if config.filter.is_empty() {
        None
    } else {
        Some(precompute_subtree_match(entry, config.filter))
    };
    let visible = visible_map.as_ref();

    if let Some(children) = entry.children() {
        let (items, dropped_count, dropped_size) = visible_children(children, config, visible);
        let len = items.len();
        for (i, child) in items.iter().enumerate() {
            let is_last = i == len - 1;
            write_entry(child, "", is_last, config, visible, 1, out)?;
        }
        write_overflow(dropped_count, dropped_size, "    ", out)?;
    }
    Ok(())
}

/// Apply `--top` and the optional filter together. Filter is applied
/// first so `--top N` selects N out of the *visible* set (matches +
/// filter-relevant ancestors), not N out of the raw children.
fn visible_children<'a>(
    children: &'a [Entry],
    config: &RenderConfig,
    visible: Option<&SubtreeMatch>,
) -> (Vec<&'a Entry>, usize, u64) {
    let Some(map) = visible else {
        return select_top(children, config.top);
    };
    let kept_by_filter: Vec<&Entry> = children
        .iter()
        .filter(|c| subtree_visible(c, map))
        .collect();
    select_top_refs(&kept_by_filter, config.top)
}

fn write_entry(
    entry: &Entry,
    prefix: &str,
    is_last: bool,
    config: &RenderConfig,
    visible: Option<&SubtreeMatch>,
    current_depth: usize,
    out: &mut impl Write,
) -> io::Result<()> {
    let connector = if is_last { "└── " } else { "├── " };
    let suffix = if entry.is_dir() { "/" } else { "" };
    let category_tag = format_category_tag(entry.category);
    writeln!(
        out,
        "{}{}{}{}  {}{}",
        prefix,
        connector,
        entry.name,
        suffix,
        category_tag,
        format_size(entry.size),
    )?;

    if let Some(max_depth) = config.max_depth {
        if current_depth >= max_depth {
            return Ok(());
        }
    }

    if let Some(children) = entry.children() {
        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let (items, dropped_count, dropped_size) = visible_children(children, config, visible);
        let len = items.len();
        for (i, child) in items.iter().enumerate() {
            let child_is_last = i == len - 1;
            write_entry(
                child,
                &child_prefix,
                child_is_last,
                config,
                visible,
                current_depth + 1,
                out,
            )?;
        }
        write_overflow(
            dropped_count,
            dropped_size,
            &format!("{}    ", child_prefix),
            out,
        )?;
    }
    Ok(())
}

fn format_category_tag(category: Category) -> String {
    if category == Category::Other {
        String::new()
    } else {
        format!("[{}] ", category.label())
    }
}

fn write_overflow(
    dropped_count: usize,
    dropped_size: u64,
    prefix: &str,
    out: &mut impl Write,
) -> io::Result<()> {
    if dropped_count > 0 {
        writeln!(
            out,
            "{}... and {} more ({})",
            prefix,
            dropped_count,
            format_size(dropped_size),
        )?;
    }
    Ok(())
}
