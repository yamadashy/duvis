// =============================================================================
// EDITING NOTE — `--help` for AI agents and humans
// =============================================================================
// duvis is used by AI agents as well as humans, so `--help` should give them
// enough orientation to start driving the tool. But duvis is also strictly
// read-only — an agent that wants to know a flag's exact behavior can just
// TRY it. Running the command answers questions in milliseconds and can't
// damage anything.
//
// So the goal here is "point them in the right direction" — not "pre-document
// every edge case". We deliberately don't try to spell out:
//   - exact JSON schema field semantics ("size" units, when "children" is
//     omitted, etc.) — `duvis . --json` shows it instantly
//   - which flag is ignored in which output mode — pass it and look
//   - exact stderr / exit-code behavior — observable on a single run
//
// What IS worth stating in help:
//   - the basic purpose of each flag
//   - mutual exclusivity (so a wrong combination errors out clearly with
//     a clap message instead of silently picking one mode)
//   - the read-only stance (so an agent doesn't waste cycles looking for
//     a delete option that intentionally doesn't exist)
//
// Mechanical conventions:
//   - Single paragraph per `///` block (no blank lines) so `-h` and
//     `--help` produce identical output.
//   - Use `help_heading` to group related flags by purpose.
//   - The `after_help` EXAMPLES block sticks to patterns an agent is most
//     likely to reach for first; avoid `duvis .` as an example because
//     the unbounded recursive output is a poor first impression.
//
// Rule of thumb: if a sentence's job could be done by the agent running
// the command itself, it probably doesn't belong here.
// =============================================================================

use crate::category::Category;
use crate::entry::SortOrder;
use crate::output::filter::EntryType;
use crate::scanner::HardlinkPolicy;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "duvis",
    version,
    about = "duvis (/ˈduːvɪs/) is a fast, read-only disk usage analyzer for both AI agents and humans. \
Default output is a colorized terminal tree; pass --summary for a per-category summary, --json for \
structured output, or --ui for an interactive browser treemap. Strictly read-only — duvis never \
deletes anything and never recommends what to delete.",
    after_help = "EXAMPLES:
  Tree view, depth-limited
  $ duvis ~/projects --max-depth 2 --top 10

  Per-category summary
  $ duvis ~/projects --summary

  Structured JSON for scripts and agents
  $ duvis ~/projects --json | jq '.children[] | {name, size_human, category}'

  Browser UI
  $ duvis ~/projects --ui"
)]
// Output formats are mutually exclusive; tree is the default when none of
// --json / --ndjson / --summary / --ui is given.
#[command(group(
    ArgGroup::new("output")
        .multiple(false)
        .args(["json", "ndjson", "summary", "ui"])
))]
pub struct Cli {
    /// Target file or directory to scan. Defaults to "." (current directory)
    /// when at least one flag is given. Running `duvis` with no arguments
    /// at all prints --help instead.
    #[arg(default_value = ".", value_name = "PATH")]
    pub path: PathBuf,

    // ----- Output Format ----------------------------------------------------
    /// Emit a structured JSON tree to stdout. Top-level shape is
    /// `{meta, tree}`; `meta` carries `scan_root`, `wire_version`,
    /// `hardlinks`, scan counters, etc. Mutually exclusive with
    /// --ndjson, --summary, and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub json: bool,

    /// Stream entries as newline-delimited JSON (one record per line).
    /// First line is `{type:"meta",...}`, subsequent lines are
    /// `{type:"entry",...}` in DFS pre-order. Designed for jq /
    /// streaming agents. Mutually exclusive with --json, --summary,
    /// and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub ndjson: bool,

    /// Print a per-category size summary (cache / build / log / media /
    /// vcs / ide / other). Mutually exclusive with --json, --ndjson,
    /// and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub summary: bool,

    /// Open a browser UI with treemap, sunburst, and list views. Starts an
    /// embedded HTTP server (default port 7515; see --port) and launches
    /// your default browser. Mutually exclusive with --json and --summary.
    #[arg(long, help_heading = "Output Format")]
    pub ui: bool,

    // ----- Display ----------------------------------------------------------
    /// Maximum depth to display (≥ 1). Affects only what is shown — sizes
    /// are always summed from the full scanned subtree.
    #[arg(
        short = 'd',
        long = "max-depth",
        value_parser = positive_usize,
        value_name = "N",
        help_heading = "Display Options"
    )]
    pub max_depth: Option<usize>,

    /// Show only the largest N entries at each level (≥ 1). Selection is
    /// by size; display order follows --sort.
    #[arg(
        short = 'n',
        long,
        value_parser = positive_usize,
        value_name = "N",
        help_heading = "Display Options"
    )]
    pub top: Option<usize>,

    /// Sort order: `size` (default, largest first) or `name` (alphabetical).
    #[arg(
        long,
        default_value = "size",
        value_name = "size|name",
        help_heading = "Display Options"
    )]
    pub sort: SortOrder,

    /// Reverse the --sort order.
    #[arg(long, help_heading = "Display Options")]
    pub reverse: bool,

    /// Show the N largest entries (files and directories) globally as a
    /// flat list ordered by size. Combines with --json / --ndjson for
    /// structured output. Mutually exclusive with --summary and --ui
    /// (those are different views, not just different formats).
    #[arg(
        long,
        value_name = "N",
        value_parser = positive_usize,
        conflicts_with_all = ["summary", "ui"],
        help_heading = "Display Options"
    )]
    pub largest: Option<usize>,

    /// How to attribute bytes to hardlinked files. `count-once` (default)
    /// matches `du` — each inode is counted once even when reachable via
    /// multiple paths. `count-each` reports every link separately, which
    /// inflates totals on trees with many hardlinks (e.g. pnpm stores).
    /// Unix only.
    #[arg(
        long,
        default_value = "count-once",
        value_name = "count-once|count-each",
        help_heading = "Display Options"
    )]
    pub hardlinks: HardlinkPolicy,

    // ----- Filters ----------------------------------------------------------
    /// Restrict displayed entries to one or more categories. Repeatable
    /// or comma-separated: `--category cache,build` or
    /// `--category cache --category build`. AND-combined with other
    /// filters. Totals (parent dir size, scan counts) are unaffected —
    /// only what's shown is filtered.
    // Filters compose with every CLI view (tree / json / ndjson / summary /
    // largest) but are intentionally rejected with --ui: the browser already
    // has interactive controls for these axes, and silently ignoring them at
    // the CLI would be a foot-gun. clap surfaces the conflict with a clear
    // "argument cannot be used with --ui" message.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "CATEGORY",
        conflicts_with = "ui",
        help_heading = "Filters"
    )]
    pub category: Vec<Category>,

    /// Restrict displayed entries by type: `file` or `dir`.
    #[arg(
        long,
        value_name = "file|dir",
        conflicts_with = "ui",
        help_heading = "Filters"
    )]
    pub r#type: Option<EntryType>,

    /// Show only entries whose disk usage is at least this size.
    /// 1024-based, case-insensitive: `100M`, `1.5G`, `50KiB`, `1024`
    /// (bare integer = bytes).
    #[arg(
        long,
        value_name = "SIZE",
        conflicts_with = "ui",
        help_heading = "Filters"
    )]
    pub min_size: Option<String>,

    /// Show only entries whose name matches one of these glob patterns.
    /// Repeatable; multiple patterns are OR-combined among themselves
    /// and AND-combined with other filters: `--name "*.log" --name "*.tmp"`.
    /// Quote in the shell to keep the glob from being expanded by zsh / bash.
    #[arg(
        long,
        value_name = "GLOB",
        conflicts_with = "ui",
        help_heading = "Filters"
    )]
    pub name: Vec<String>,

    /// Show only entries modified within the past <DURATION>. Suffix:
    /// `d` (days, default), `w` (7d), `m` (30d), `y` (365d). e.g.
    /// `--changed-within 7d` or `--changed-within 2w`. Field name
    /// (`changed`) leaves room for future `--accessed-within` etc.
    #[arg(
        long,
        value_name = "DURATION",
        conflicts_with = "ui",
        help_heading = "Filters"
    )]
    pub changed_within: Option<String>,

    /// Show only entries modified more than <DURATION> ago. Same suffix
    /// rules as --changed-within. Combine for a window:
    /// `--changed-within 1y --changed-before 30d` = 30 days .. 1 year ago.
    #[arg(
        long,
        value_name = "DURATION",
        conflicts_with = "ui",
        help_heading = "Filters"
    )]
    pub changed_before: Option<String>,

    // ----- UI Server --------------------------------------------------------
    /// Port for the --ui HTTP server (default 7515). Falls back to a free
    /// OS-assigned port if busy.
    #[arg(
        long,
        default_value = "7515",
        value_name = "PORT",
        help_heading = "UI Server Options"
    )]
    pub port: u16,
}

/// `clap` value parser used by `--max-depth` / `--top`. Rejects 0 so a
/// zero value isn't silently equivalent to 1 (was previously inconsistent
/// across formats).
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
