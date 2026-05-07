// =============================================================================
// EDITING NOTE — `--help` IS A FIRST-CLASS DELIVERABLE
// =============================================================================
// duvis is used by AI agents as well as humans. For an agent, `duvis --help`
// IS the spec — it's the only documentation the agent reliably has on hand
// when deciding how to invoke the tool. So the help text below should be
// written so an agent that just ran `duvis --help` (and read nothing else)
// can drive the tool end-to-end without surprises.
//
// When editing flag descriptions in this file, optimize for that reader:
//
//   1. State precisely what the flag does, what units it takes, and how it
//      interacts with related flags. Do not assume the reader has seen the
//      README.
//   2. Call out non-obvious behavior explicitly. Examples that have bitten
//      us: --top selects by size REGARDLESS of --sort; --analyze IGNORES
//      --depth/--top/--sort/--reverse; `size` is allocated disk bytes
//      (st_blocks * 512), so a 1-byte file reports ~4096; symlinks appear
//      as leaf entries with the symlink's own disk usage.
//   3. Keep each flag's description in a SINGLE PARAGRAPH (no blank `///`
//      lines anywhere). clap derive otherwise splits doc comments into
//      "short" (-h) and "long" (--help) forms — we want `-h` and `--help`
//      to produce identical, fully detailed output, so we deliberately
//      collapse the two.
//   4. Use clap's `help_heading` to group related flags by purpose
//      (Output Format / Display Options / UI Server Options). Agents and
//      humans both scan faster with sectioned help than with a flat list.
//   5. Keep the `after_help` EXAMPLES block alive. The `--json | jq` recipe
//      in particular is there so an agent has a copy-paste starting point
//      for piping duvis into its own analysis flow. Avoid an example like
//      `duvis .` that produces tens of thousands of lines on a typical
//      project root — agents will copy it verbatim.
//
// Editing checklist — re-verify these whenever you touch a flag:
//   - Does the JSON schema description match the actual JSON output? Run
//     `duvis . --json` and diff field-by-field against what `--help` claims
//     (`name` / `size` / `size_human` / `is_dir` / `category` /
//     `modified_days_ago` / `children`).
//   - Does each flag say which OTHER flags it interacts with or is ignored
//     by (e.g. "ignored by --analyze")?
//   - Does the help describe stderr / non-zero exit behaviors that an agent
//     would otherwise interpret as a bug (skipped paths warning, broken
//     pipe under `| head`, port collision fallback)?
//   - Does any prose drift toward "what an agent could do with this"
//     (vague: "feeding into MCP servers") instead of "what duvis actually
//     emits" (concrete: schema, limits, behaviors)? Trim the former.
//
// Rule of thumb: if a description doesn't tell an agent how to safely
// combine the flag with the others — and how to interpret the bytes that
// come back — it's not detailed enough yet.
// =============================================================================

use crate::entry::SortOrder;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "duvis",
    version,
    about = "duvis (/ˈduːvɪs/) is a fast, read-only disk usage analyzer for both AI agents and humans. \
Default output is a colorized terminal tree; pass --analyze for a per-category summary, --json for \
AI-agent-friendly structured output, or --ui for an interactive browser treemap. duvis is strictly \
read-only — it shows you what's there, never deletes anything, never recommends what to delete.",
    after_help = "EXAMPLES:
  Tree view, depth-limited so output stays scannable
  $ duvis ~/projects --depth 2 --top 10

  Per-category summary (cache / build / log / media / vcs / ide / other)
  $ duvis ~/projects --analyze

  Structured JSON for AI agents and scripts (full subtree, no limits)
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
    /// Target file or directory to scan. When given a directory, duvis
    /// recurses through it; symlinks inside the tree are not followed and
    /// appear as leaf entries reporting the symlink's own disk usage.
    /// Unreadable or vanished paths (permission denied, race conditions)
    /// are silently skipped and a one-line warning is printed to stderr;
    /// duvis still exits 0 and emits output, but reported totals may be
    /// incomplete. Defaults to "." (current directory) when at least one
    /// flag is given. Running `duvis` with no arguments at all prints
    /// --help instead — use `duvis .` to scan the current directory
    /// explicitly.
    #[arg(default_value = ".", value_name = "PATH")]
    pub path: PathBuf,

    // ----- Output Format ----------------------------------------------------
    /// Emit a structured JSON tree to stdout (no other output goes to
    /// stdout in this mode). Each entry has `name`, `size`, `size_human`,
    /// `is_dir`, `category`, and `modified_days_ago`. `size` is allocated
    /// disk bytes (st_blocks * 512 on Unix), so a 1-byte file reports
    /// ~4096 and a sparse file may report 0; `size_human` is the same
    /// value formatted in base-1024 units. Directory entries additionally
    /// carry a `children` array; files never carry it, and directories
    /// trimmed by --depth omit it even when more entries were scanned.
    /// --top likewise drops siblings from `children` without leaving a
    /// marker, but the parent's `size` still reflects the FULL scanned
    /// subtree. Mutually exclusive with --analyze and --ui.
    #[arg(long, help_heading = "Output Format")]
    pub json: bool,

    /// Print a per-category size summary instead of the tree view, with
    /// columns: category label, size, percentage of total, item count.
    /// Categories: cache / build / log / media / vcs / ide / other; rows
    /// are sorted by size descending. `item count` is the number of
    /// top-level entries that fell into each bucket — a single `target/`
    /// directory counts as 1 item under `build`, not the thousands of
    /// files inside it. The display flags (--depth, --top, --sort,
    /// --reverse) are IGNORED in this mode; --analyze always runs against
    /// the full scanned tree. Output is fact-only — duvis does not flag
    /// anything as "safe to delete". Mutually exclusive with --json and
    /// --ui.
    #[arg(long, help_heading = "Output Format")]
    pub analyze: bool,

    /// Open a browser UI with treemap, sunburst, and list views. Starts
    /// an embedded HTTP server (default port 7515; see --port) and
    /// launches your default browser. Cells are color-coded by category;
    /// click to drill in, hover for details. The bundle is embedded in
    /// the binary — no internet access required. The display flags
    /// (--depth, --top, --sort, --reverse) are IGNORED in this mode; the
    /// UI manages depth, sort, and limits interactively. Mutually
    /// exclusive with --json and --analyze.
    #[arg(long, help_heading = "Output Format")]
    pub ui: bool,

    // ----- Display ----------------------------------------------------------
    /// Maximum depth to display in the output (≥ 1). Depth 1 shows the
    /// root and its immediate children, depth 2 adds grandchildren, and
    /// so on. Affects only what is *displayed* — sizes are always summed
    /// from the full scanned subtree. Without --depth, the tree is fully
    /// recursive and routinely produces tens of thousands of lines on a
    /// real project root, so agents should usually pass --depth + --top
    /// or use --json. Ignored by --analyze and --ui.
    #[arg(
        short,
        long,
        value_parser = positive_usize,
        value_name = "N",
        help_heading = "Display Options"
    )]
    pub depth: Option<usize>,

    /// Show only the largest N entries at each level (≥ 1). Selection is
    /// always by size; the displayed order still follows --sort. Combine
    /// with --depth to spot the biggest stuff fast: `duvis ~/projects -d 2 -n 10`.
    /// In --json output, trimmed siblings are not represented by a
    /// marker; the parent's `size` still reflects the full scanned
    /// subtree. Ignored by --analyze and --ui.
    #[arg(
        short = 'n',
        long,
        value_parser = positive_usize,
        value_name = "N",
        help_heading = "Display Options"
    )]
    pub top: Option<usize>,

    /// Sort order for tree / JSON output: `size` (default, largest first)
    /// or `name` (alphabetical). The *selection* of entries under --top
    /// is always by size regardless of this flag; --sort only affects
    /// display order. Ignored by --analyze (categories are always sorted
    /// by size descending) and by --ui (sort is set interactively).
    #[arg(
        long,
        default_value = "size",
        value_name = "size|name",
        help_heading = "Display Options"
    )]
    pub sort: SortOrder,

    /// Reverse the --sort order. With `--sort size`, this puts the
    /// smallest entries first; with `--sort name`, descending
    /// alphabetical. Ignored by --analyze and --ui.
    #[arg(long, help_heading = "Display Options")]
    pub reverse: bool,

    // ----- UI Server --------------------------------------------------------
    /// Port for the --ui HTTP server (default 7515). If the port is
    /// already bound, duvis falls back to a free OS-assigned port
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
