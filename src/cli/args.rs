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
// Two independent axes, one ArgGroup each.
//
//   view   — *what* is computed: tree (default) / summary / largest / ui
//   format — *how* it is encoded: text (default) / json / toon / ndjson
//
// Members of a group are mutually exclusive, but the groups don't
// constrain each other, so `--summary --json` is a valid combination.
// Before v0.2.0 both axes shared a single group, which made the summary
// view unreachable in any structured format while `--largest --json`
// worked — the inconsistency this split removes.
//
// `ui` is listed in the view group only when the feature is on, so a
// no-default-features build doesn't trip on a clap reference to a
// missing argument. `--ui` additionally conflicts with the format flags
// (declared on the arg itself) because it serves a browser, not stdout.
#[cfg_attr(
    feature = "ui",
    command(group(
        ArgGroup::new("view")
            .multiple(false)
            .args(["summary", "largest", "ui"])
    ))
)]
#[cfg_attr(
    not(feature = "ui"),
    command(group(
        ArgGroup::new("view")
            .multiple(false)
            .args(["summary", "largest"])
    ))
)]
#[command(group(
    ArgGroup::new("format")
        .multiple(false)
        .args(["json", "toon", "ndjson"])
))]
pub(super) struct Cli {
    /// Target file or directory to scan. Defaults to "." (current
    /// directory) when at least one flag is given; running `duvis` with
    /// no arguments at all prints --help instead.
    ///
    /// Deliberately an `Option` with no clap `default_value`: the default
    /// is applied in `plan.rs`. That keeps "the user actually typed a
    /// PATH" distinguishable, which is what lets `--explain-category`
    /// reject a PATH instead of accepting and ignoring it.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    // ----- Output Format ----------------------------------------------------
    /// Emit structured JSON to stdout. Shape is `{meta, tree}` for the
    /// default view, or `{meta, summary}` / `{meta, largest}` when paired
    /// with those views; `meta` carries `scan_root`, `wire_version`,
    /// `hardlinks`, scan counters, etc. Mutually exclusive with the other
    /// formats (--toon, --ndjson) and with --ui.
    #[arg(long, help_heading = "Output Format")]
    pub json: bool,

    /// Emit the same data as --json, encoded in TOON (Token-Oriented
    /// Object Notation) — an indentation-based, tabular format that costs
    /// fewer LLM tokens than JSON. Mutually exclusive with the other
    /// formats (--json, --ndjson) and with --ui.
    #[arg(long, help_heading = "Output Format")]
    pub toon: bool,

    /// Stream records as newline-delimited JSON (one per line). First
    /// line is `{type:"meta",...}`; the rest are entries in DFS pre-order,
    /// or category rollups under --summary. Designed for jq / streaming
    /// agents. Mutually exclusive with the other formats (--json, --toon)
    /// and with --ui.
    #[arg(long, help_heading = "Output Format")]
    pub ndjson: bool,

    // ----- Views ------------------------------------------------------------
    /// Print a per-category size summary (cache / build / log / media /
    /// vcs / ide / other). Combines with --json / --toon / --ndjson for
    /// structured output. Mutually exclusive with --largest and --ui.
    #[arg(long, help_heading = "Views")]
    pub summary: bool,

    /// Open a browser UI with treemap, sunburst, and list views. Starts an
    /// embedded HTTP server (see --port) and launches your default
    /// browser.
    ///
    /// Serves a browser rather than stdout, so it conflicts with every
    /// format flag and with the display limits the server doesn't apply.
    #[cfg(feature = "ui")]
    #[arg(
        long,
        conflicts_with_all = ["json", "toon", "ndjson", "max_depth", "top"],
        help_heading = "Views"
    )]
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
    /// via the `view` group — those are different views, not just
    /// different formats.
    #[arg(
        long,
        value_name = "N",
        value_parser = positive_usize,
        help_heading = "Views"
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
    ///
    /// `requires` makes the dependency on --ui a parse error rather than
    /// a flag that quietly does nothing: `duvis . --port 8080` is
    /// rejected. And the default lives in `plan.rs`, not in a clap
    /// `default_value` — a defaulted arg counts as present, which would
    /// make `requires` demand --ui on every run.
    #[cfg(feature = "ui")]
    #[arg(
        long,
        value_name = "PORT",
        requires = "ui",
        help_heading = "UI Server Options"
    )]
    pub port: Option<u16>,

    // ----- Diagnostics ------------------------------------------------------
    /// Explain how a name would be classified, without scanning. Prints
    /// both interpretations (as-directory / as-file) and the rule that
    /// matched. Useful when you see a category in a scan and want to know
    /// why.
    ///
    /// This is a query against the classifier rule table, not a view of a
    /// scan: it never touches the filesystem. So it rejects PATH and
    /// every scan-shaped flag rather than accepting them and producing
    /// output that ignored them. `--json` is the one format it renders
    /// (a two-field record has nothing for TOON's tabular encoding or
    /// NDJSON's streaming to do), so the other two are rejected as well.
    #[cfg_attr(feature = "ui", arg(
        long,
        value_name = "NAME",
        conflicts_with_all = [
            "path", "toon", "ndjson",
            "summary", "largest", "ui",
            "max_depth", "top", "reverse",
            "category", "type", "min_size", "name",
            "changed_within", "changed_before",
        ],
        help_heading = "Diagnostics"
    ))]
    #[cfg_attr(not(feature = "ui"), arg(
        long,
        value_name = "NAME",
        conflicts_with_all = [
            "path", "toon", "ndjson",
            "summary", "largest",
            "max_depth", "top", "reverse",
            "category", "type", "min_size", "name",
            "changed_within", "changed_before",
        ],
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

#[cfg(test)]
mod tests {
    #[test]
    fn positive_usize_rejects_zero_and_accepts_one() {
        assert!(super::positive_usize("0").is_err());
        assert_eq!(super::positive_usize("1").unwrap(), 1);
    }
}
