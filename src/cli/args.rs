// The actual `--help` text lives in `src/cli/help.rs` as a hand-formatted block
// (mirroring pdfvision's layout). clap's `override_help` swaps in that text
// for both `-h` and `--help`. The `///` doc comments below are kept for
// cargo doc / IDE tooltips but are not what users see at the CLI.

use crate::classify::Category;
use crate::entry::SortOrder;
use crate::filter::EntryType;
use crate::scan::HardlinkPolicy;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

use super::help::HELP_TEXT;

#[derive(Parser)]
#[command(name = "duvis", version, override_help = HELP_TEXT)]
// Output formats are mutually exclusive; tree is the default when none of
// --json / --ndjson / --summary / --ui is given. The arg group lists
// `ui` only when the feature is on, so a no-default-features build
// doesn't trip on a clap reference to a missing argument.
#[cfg_attr(
    feature = "ui",
    command(group(
        ArgGroup::new("output")
            .multiple(false)
            .args(["json", "toon", "ndjson", "summary", "ui"])
    ))
)]
#[cfg_attr(
    not(feature = "ui"),
    command(group(
        ArgGroup::new("output")
            .multiple(false)
            .args(["json", "toon", "ndjson", "summary"])
    ))
)]
pub(super) struct Cli {
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

    /// Emit the same `{meta, tree}` data as --json, encoded in TOON
    /// (Token-Oriented Object Notation) — an indentation-based, tabular
    /// format that costs fewer LLM tokens than JSON. Combines with
    /// --largest. Mutually exclusive with --json, --ndjson, --summary,
    /// and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub toon: bool,

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
    #[cfg(feature = "ui")]
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
    /// flat list ordered by size. Combines with --json / --toon / --ndjson
    /// for structured output. Mutually exclusive with --summary and --ui
    /// (those are different views, not just different formats).
    #[cfg_attr(feature = "ui", arg(
        long,
        value_name = "N",
        value_parser = positive_usize,
        conflicts_with_all = ["summary", "ui"],
        help_heading = "Display Options"
    ))]
    #[cfg_attr(not(feature = "ui"), arg(
        long,
        value_name = "N",
        value_parser = positive_usize,
        conflicts_with = "summary",
        help_heading = "Display Options"
    ))]
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
    #[cfg_attr(
        feature = "ui",
        arg(
            long,
            value_delimiter = ',',
            value_name = "CATEGORY",
            conflicts_with = "ui",
            help_heading = "Filters"
        )
    )]
    #[cfg_attr(
        not(feature = "ui"),
        arg(
            long,
            value_delimiter = ',',
            value_name = "CATEGORY",
            help_heading = "Filters"
        )
    )]
    pub category: Vec<Category>,

    /// Restrict displayed entries by type: `file` or `dir`.
    #[cfg_attr(
        feature = "ui",
        arg(
            long,
            value_name = "file|dir",
            conflicts_with = "ui",
            help_heading = "Filters"
        )
    )]
    #[cfg_attr(
        not(feature = "ui"),
        arg(long, value_name = "file|dir", help_heading = "Filters")
    )]
    pub r#type: Option<EntryType>,

    /// Show only entries whose disk usage is at least this size.
    /// 1024-based, case-insensitive: `100M`, `1.5G`, `50KiB`, `1024`
    /// (bare integer = bytes).
    #[cfg_attr(
        feature = "ui",
        arg(
            long,
            value_name = "SIZE",
            conflicts_with = "ui",
            help_heading = "Filters"
        )
    )]
    #[cfg_attr(
        not(feature = "ui"),
        arg(long, value_name = "SIZE", help_heading = "Filters")
    )]
    pub min_size: Option<String>,

    /// Show only entries whose name matches one of these glob patterns.
    /// Repeatable; multiple patterns are OR-combined among themselves
    /// and AND-combined with other filters: `--name "*.log" --name "*.tmp"`.
    /// Quote in the shell to keep the glob from being expanded by zsh / bash.
    #[cfg_attr(
        feature = "ui",
        arg(
            long,
            value_name = "GLOB",
            conflicts_with = "ui",
            help_heading = "Filters"
        )
    )]
    #[cfg_attr(
        not(feature = "ui"),
        arg(long, value_name = "GLOB", help_heading = "Filters")
    )]
    pub name: Vec<String>,

    /// Show only entries modified within the past <DURATION>. Suffix:
    /// `d` (days, default), `w` (7d), `m` (30d), `y` (365d). e.g.
    /// `--changed-within 7d` or `--changed-within 2w`. Field name
    /// (`changed`) leaves room for future `--accessed-within` etc.
    #[cfg_attr(
        feature = "ui",
        arg(
            long,
            value_name = "DURATION",
            conflicts_with = "ui",
            help_heading = "Filters"
        )
    )]
    #[cfg_attr(
        not(feature = "ui"),
        arg(long, value_name = "DURATION", help_heading = "Filters")
    )]
    pub changed_within: Option<String>,

    /// Show only entries modified more than <DURATION> ago. Same suffix
    /// rules as --changed-within. Combine for a window:
    /// `--changed-within 1y --changed-before 30d` = 30 days .. 1 year ago.
    #[cfg_attr(
        feature = "ui",
        arg(
            long,
            value_name = "DURATION",
            conflicts_with = "ui",
            help_heading = "Filters"
        )
    )]
    #[cfg_attr(
        not(feature = "ui"),
        arg(long, value_name = "DURATION", help_heading = "Filters")
    )]
    pub changed_before: Option<String>,

    // ----- UI Server --------------------------------------------------------
    /// Port for the --ui HTTP server (default 7515). Falls back to a free
    /// OS-assigned port if busy.
    #[cfg(feature = "ui")]
    #[arg(
        long,
        default_value = "7515",
        value_name = "PORT",
        help_heading = "UI Server Options"
    )]
    pub port: u16,

    // ----- Diagnostics ------------------------------------------------------
    /// Explain how a name would be classified, without scanning. Prints
    /// both interpretations (as-directory / as-file) and the rule that
    /// matched. Combine with --json for structured output. Useful when
    /// you see a category in a scan and want to know why.
    #[cfg_attr(feature = "ui", arg(
        long,
        value_name = "NAME",
        conflicts_with_all = ["toon", "ndjson", "summary", "ui", "largest"],
        help_heading = "Diagnostics"
    ))]
    #[cfg_attr(not(feature = "ui"), arg(
        long,
        value_name = "NAME",
        conflicts_with_all = ["toon", "ndjson", "summary", "largest"],
        help_heading = "Diagnostics"
    ))]
    pub explain_category: Option<String>,
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
