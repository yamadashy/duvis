use std::io::{self, Write};

use super::format::format_size;
use super::{select_top, OutputConfig};
use crate::category::Category;
use crate::entry::Entry;

pub fn write(entry: &Entry, config: &OutputConfig, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{} ({})", entry.name, format_size(entry.size))?;

    if let Some(children) = &entry.children {
        let (items, dropped_count, dropped_size) = select_top(children, config.top);
        let len = items.len();
        for (i, child) in items.iter().enumerate() {
            let is_last = i == len - 1;
            write_entry(child, "", is_last, config, 1, out)?;
        }
        write_overflow(dropped_count, dropped_size, "    ", out)?;
    }
    Ok(())
}

fn write_entry(
    entry: &Entry,
    prefix: &str,
    is_last: bool,
    config: &OutputConfig,
    current_depth: usize,
    out: &mut impl Write,
) -> io::Result<()> {
    let connector = if is_last { "└── " } else { "├── " };
    let suffix = if entry.is_dir { "/" } else { "" };
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

    if let Some(max_depth) = config.depth {
        if current_depth >= max_depth {
            return Ok(());
        }
    }

    if let Some(children) = &entry.children {
        let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        let (items, dropped_count, dropped_size) = select_top(children, config.top);
        let len = items.len();
        for (i, child) in items.iter().enumerate() {
            let child_is_last = i == len - 1;
            write_entry(
                child,
                &child_prefix,
                child_is_last,
                config,
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
