use crate::entry::SortOrder;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "duvis",
    version,
    about = "Disk usage visualizer for both AI and humans",
    long_about = "duvis (/ˈduːvɪs/) is a fast, read-only disk usage analyzer.

Run it against any directory to see what's filling your disk. The default \
output is a colorized terminal tree. Pass --analyze for a per-category size \
summary, --json for AI-agent-friendly structured output, or --ui to open an \
interactive browser treemap.

duvis is strictly read-only — it shows you what's there, never deletes \
anything, never recommends what to delete.",
    after_help = "EXAMPLES:
  Tree view of the current directory (default output)
  $ duvis .

  Limit depth and show only the top N largest entries at each level
  $ duvis ~/projects --depth 2 --top 10

  Per-category summary (cache / build / log / media / vcs / ide / other)
  $ duvis ~/projects --analyze

  Structured JSON for AI agents and scripts
  $ duvis ~/projects --json | jq '.children[] | {name, size_human, category}'

  Open the interactive browser UI on the default port (7515)
  $ duvis ~/projects --ui

  Pick a custom UI port (falls back to a free OS-assigned port if busy)
  $ duvis ~/projects --ui --port 8080"
)]
// Output formats are mutually exclusive; tree is the default when none of
// --json / --analyze / --ui is given.
#[command(group(
    ArgGroup::new("output")
        .multiple(false)
        .args(["json", "analyze", "ui"])
))]
pub struct Cli {
    /// Target directory to scan.
    ///
    /// Symlinks inside the tree are not followed. Defaults to "." (current
    /// directory) when at least one flag is given. Running `duvis` with no
    /// arguments at all prints --help instead — use `duvis .` to scan the
    /// current directory explicitly.
    #[arg(default_value = ".", value_name = "PATH")]
    pub path: PathBuf,

    // ----- Output Format ----------------------------------------------------
    /// Emit a structured JSON tree to stdout (for AI agents and scripts).
    ///
    /// Each entry has `name`, `size`, `size_human`, `is_dir`, `category`,
    /// and `modified_days_ago`. Designed for piping into `jq`, feeding into
    /// MCP servers, or persisting as a snapshot. Mutually exclusive with
    /// --analyze and --ui; default output (no flag) is a colorized terminal
    /// tree.
    #[arg(long, help_heading = "Output Format")]
    pub json: bool,

    /// Print a per-category size summary instead of the tree view.
    ///
    /// Categories: cache / build / log / media / vcs / ide / other. Output
    /// is fact-only (size, percentage, item count) — duvis intentionally
    /// does not flag anything as "safe to delete". Mutually exclusive with
    /// --json and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub analyze: bool,

    /// Open a browser UI with treemap, sunburst, and list views.
    ///
    /// Starts an embedded HTTP server (default port 7515; see --port) and
    /// launches your default browser. Cells are color-coded by category;
    /// click to drill in, hover for details. The bundle is embedded in the
    /// binary — no internet access required. Mutually exclusive with
    /// --json and --analyze.
    #[arg(long, help_heading = "Output Format")]
    pub ui: bool,

    // ----- Display ----------------------------------------------------------
    /// Maximum depth to display in the output (≥ 1).
    ///
    /// Depth 1 shows the root and its immediate children, depth 2 adds
    /// grandchildren, and so on. Affects only what is *displayed* — sizes
    /// are always summed from the full scanned subtree.
    #[arg(
        short,
        long,
        value_parser = positive_usize,
        value_name = "N",
        help_heading = "Display Options",
    )]
    pub depth: Option<usize>,

    /// Show only the largest N entries at each level (≥ 1).
    ///
    /// Selection is always by size; the displayed order still follows
    /// --sort. Combine with --depth to spot the biggest stuff fast:
    /// `duvis ~/projects -d 2 -n 10`.
    #[arg(
        short = 'n',
        long,
        value_parser = positive_usize,
        value_name = "N",
        help_heading = "Display Options",
    )]
    pub top: Option<usize>,

    /// Sort order for tree / JSON / UI output.
    ///
    /// `size` (default, largest first) or `name` (alphabetical). The
    /// *selection* of entries under --top is always by size regardless of
    /// this flag; --sort only affects display order.
    #[arg(
        long,
        default_value = "size",
        value_name = "size|name",
        help_heading = "Display Options"
    )]
    pub sort: SortOrder,

    /// Reverse the --sort order.
    ///
    /// With `--sort size`, this puts the smallest entries first; with
    /// `--sort name`, descending alphabetical.
    #[arg(long, help_heading = "Display Options")]
    pub reverse: bool,

    // ----- UI Server --------------------------------------------------------
    /// Port for the --ui HTTP server.
    ///
    /// Defaults to 7515 (see the README for why this number). If the port
    /// is already bound, duvis falls back to a free OS-assigned port
    /// automatically and prints the resolved URL on stderr. Ignored when
    /// --ui is not set.
    #[arg(
        long,
        default_value = "7515",
        value_name = "PORT",
        help_heading = "UI Server Options"
    )]
    pub port: u16,
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
