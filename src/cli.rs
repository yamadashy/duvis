use crate::entry::SortOrder;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// How duvis renders the scan result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[value(rename_all = "kebab-case")]
pub enum OutputFormat {
    /// Indented tree to stdout (the human default).
    #[default]
    Tree,
    /// Structured JSON to stdout (for AI agents and pipelines).
    Json,
    /// Per-category size summary with reclaimable hints.
    Analyze,
    /// Open the browser UI (treemap / sunburst / list).
    Ui,
}

#[derive(Parser)]
#[command(name = "duvis", about = "Disk usage visualizer for both AI and humans")]
pub struct Cli {
    /// Target directory to analyze
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output format (one of: tree, json, analyze, ui)
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,

    /// Maximum depth to display (1 = root + immediate children)
    #[arg(short, long, value_parser = positive_usize)]
    pub depth: Option<usize>,

    /// Show top N entries by size (selection by size, displayed in --sort order)
    #[arg(short = 'n', long, value_parser = positive_usize)]
    pub top: Option<usize>,

    /// Port for UI server (used with --format ui). Falls back to a free port if busy.
    #[arg(long, default_value = "7515")]
    pub port: u16,

    /// Sort order
    #[arg(long, default_value = "size")]
    pub sort: SortOrder,

    /// Reverse sort order
    #[arg(long)]
    pub reverse: bool,
}

/// `clap` value parser used by `--depth` / `--top`. Rejects 0 so depth=0 isn't
/// silently equivalent to depth=1 (was previously inconsistent across formats).
fn positive_usize(s: &str) -> Result<usize, String> {
    let n: usize = s
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())?;
    if n == 0 {
        Err("must be ≥ 1".to_string())
    } else {
        Ok(n)
    }
}
