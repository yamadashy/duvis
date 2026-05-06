use std::io::Write;

use anyhow::Result;
use serde::Serialize;

use super::format::format_size;
use super::{select_top, OutputConfig};
use crate::category::Category;
use crate::entry::Entry;

#[derive(Serialize)]
struct JsonOutput {
    name: String,
    size: u64,
    size_human: String,
    is_dir: bool,
    category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_days_ago: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<JsonOutput>>,
}

fn to_json_output(
    entry: &Entry,
    depth: Option<usize>,
    top: Option<usize>,
    current_depth: usize,
) -> JsonOutput {
    let children = match depth {
        Some(max_depth) if current_depth >= max_depth => None,
        _ => entry.children().map(|c| {
            let (kept, _, _) = select_top(c, top);
            kept.into_iter()
                .map(|child| to_json_output(child, depth, top, current_depth + 1))
                .collect()
        }),
    };

    JsonOutput {
        name: entry.name.clone(),
        size: entry.size,
        size_human: format_size(entry.size),
        is_dir: entry.is_dir(),
        category: entry.category,
        modified_days_ago: entry.modified_days_ago,
        children,
    }
}

pub fn write(entry: &Entry, config: &OutputConfig, out: &mut impl Write) -> Result<()> {
    let output = to_json_output(entry, config.depth, config.top, 0);
    serde_json::to_writer_pretty(&mut *out, &output)?;
    writeln!(out)?;
    Ok(())
}
