use crate::entry::SortOrder;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "duvis", about = "Disk usage visualizer for both AI and humans")]
// At most one of --json / --analyze / --ui may be set; tree is the default
// when none are passed.
#[command(group(
    ArgGroup::new("output")
        .multiple(false)
        .args(["json", "analyze", "ui"])
))]
pub struct Cli {
    /// Target directory to analyze
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Maximum depth to display (1 = root + immediate children)
    #[arg(short, long, value_parser = positive_usize)]
    pub depth: Option<usize>,

    /// Show top N entries by size (selection by size, displayed in --sort order)
    #[arg(short = 'n', long, value_parser = positive_usize)]
    pub top: Option<usize>,

    /// Output as JSON (for AI agents and pipelines)
    #[arg(long)]
    pub json: bool,

    /// Show a per-category size summary
    #[arg(long)]
    pub analyze: bool,

    /// Open browser UI with treemap / sunburst / list views
    #[arg(long)]
    pub ui: bool,

    /// Port for UI server (used with --ui). Falls back to a free port if busy.
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
