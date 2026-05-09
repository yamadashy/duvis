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

use crate::entry::SortOrder;
use crate::scanner::HardlinkPolicy;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "duvis",
    version,
    about = "duvis (/ˈduːvɪs/) is a fast, read-only disk usage analyzer for both AI agents and humans. \
Default output is a colorized terminal tree; pass --analyze for a per-category summary, --json for \
structured output, or --ui for an interactive browser treemap. Strictly read-only — duvis never \
deletes anything and never recommends what to delete.",
    after_help = "EXAMPLES:
  Tree view, depth-limited
  $ duvis ~/projects --depth 2 --top 10

  Per-category summary
  $ duvis ~/projects --analyze

  Structured JSON for scripts and agents
  $ duvis ~/projects --json | jq '.children[] | {name, size_human, category}'

  Browser UI
  $ duvis ~/projects --ui"
)]
// Output formats are mutually exclusive; tree is the default when none of
// --json / --analyze / --ui is given.
#[command(group(
    ArgGroup::new("output")
        .multiple(false)
        .args(["json", "analyze", "ui"])
))]
pub struct Cli {
    /// Target file or directory to scan. Defaults to "." (current directory)
    /// when at least one flag is given. Running `duvis` with no arguments
    /// at all prints --help instead.
    #[arg(default_value = ".", value_name = "PATH")]
    pub path: PathBuf,

    // ----- Output Format ----------------------------------------------------
    /// Emit a structured JSON tree to stdout. Mutually exclusive with
    /// --analyze and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub json: bool,

    /// Print a per-category size summary (cache / build / log / media /
    /// vcs / ide / other). Mutually exclusive with --json and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub analyze: bool,

    /// Open a browser UI with treemap, sunburst, and list views. Starts an
    /// embedded HTTP server (default port 7515; see --port) and launches
    /// your default browser. Mutually exclusive with --json and --analyze.
    #[arg(long, help_heading = "Output Format")]
    pub ui: bool,

    // ----- Display ----------------------------------------------------------
    /// Maximum depth to display (≥ 1). Affects only what is shown — sizes
    /// are always summed from the full scanned subtree.
    #[arg(
        short,
        long,
        value_parser = positive_usize,
        value_name = "N",
        help_heading = "Display Options"
    )]
    pub depth: Option<usize>,

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
