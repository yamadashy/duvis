// `--help` for AI agents and humans
//
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
// Width convention: hand-wrap at column ~100 with description column at 26,
// matching pdfvision's help layout. `-h` and `--help` produce identical
// output (clap's `override_help` replaces both).
//
// We keep two versions of HELP_TEXT — one with `--ui` / `--port` and one
// without — gated on the `ui` Cargo feature. clap's `override_help`
// needs a `&'static str` const, and the standard `concat!` macro only
// composes literals, so a single template stitched from `cfg`-gated
// fragments isn't possible without a build script or an extra dep
// (`const_format`). The two strings are kept adjacent so drift is
// obvious in review — when you change one, change the other.

#[cfg(feature = "ui")]
pub(super) const HELP_TEXT: &str = "duvis - Disk usage visualizer for both AI agents and humans

Usage:
  duvis [PATH] [options]
  duvis --explain-category <NAME>

duvis is strictly read-only — it never deletes anything and never recommends what to delete.
The default output is a colorized terminal tree. Pass --summary, --json, --toon, --ndjson,
--largest, or --ui to get a different view of the same scan.

Display options
  -d, --max-depth <N>     Maximum depth to display. Affects display only — sizes always sum
                          the full subtree.
  -n, --top <N>           Show only the largest N entries at each level. Selection by size;
                          display order follows --sort.
      --sort <size|name>  Sort order. Default: size.
      --reverse           Reverse the --sort order.
      --hardlinks <count-once|count-each>
                          Attribution for hardlinked files. Default: count-once (matches
                          `du`). count-each inflates totals when many links share an inode
                          (e.g. pnpm stores). Unix only.

Output formats (mutually exclusive; default = colorized terminal tree)
      --json              Structured JSON tree to stdout. Shape: `{meta, tree}`.
      --toon              Same `{meta, tree}` data as --json, encoded in TOON — an
                          indentation-based, tabular format that costs fewer LLM tokens.
                          Combines with --largest.
      --ndjson            Newline-delimited JSON, one entry per line, in DFS pre-order.
                          Designed for jq / streaming agents.
      --summary           Per-category size summary (cache / build / log / media / vcs /
                          ide / other).
      --largest <N>       Flat list of the N largest entries globally, ordered by size.
                          Combines with --json / --toon / --ndjson for structured output.
      --ui                Browser UI with treemap, sunburst, and list views. Starts an
                          embedded HTTP server (default port 7515).
      --port <PORT>       Port for the --ui HTTP server. Default: 7515. Falls back to a
                          free OS-assigned port if busy.

Filters (AND-combined; affect display only, not totals; rejected with --ui)
      --category <CAT>    Restrict to one or more categories. Repeatable / CSV:
                          `--category cache,build`. Categories: cache, build, log, media,
                          vcs, ide, other, archive, installer, vm_image, model_cache, backup.
      --type <file|dir>   Restrict by entry type.
      --min-size <SIZE>   Show only entries at least this size. 1024-based:
                          `100M`, `1.5G`, `50KiB`, `1024` (bare = bytes).
      --name <GLOB>       Restrict to basenames matching one or more globs. Repeatable;
                          multiple are OR-combined: `--name \"*.log\" --name \"*.tmp\"`.
      --changed-within <DURATION>
                          Modified within the past `Nd` / `Nw` / `Nm` / `Ny` (m=30d, y=365d).
      --changed-before <DURATION>
                          Modified more than <DURATION> ago. Combine with --changed-within
                          for a window.

Diagnostics
      --explain-category <NAME>
                          Explain how a name would be classified, without scanning. Prints
                          both interpretations (as-dir / as-file) and the matched rule.
                          Combine with --json for structured output. Skips scanning
                          entirely; PATH is ignored.

  -h, --help              Show this help.
  -V, --version           Show version.

Examples
  duvis ~/projects                                        # tree (default)
  duvis ~/projects --max-depth 2 --top 10                 # depth-limited
  duvis ~/projects --summary                              # category summary
  duvis ~/projects --json | jq '.tree.children[]'         # structured for agents
  duvis ~/projects --ndjson | jq -c 'select(.size>1e8)'   # streaming filter
  duvis ~/projects --largest 10                           # 10 largest globally
  duvis ~/projects --category cache --min-size 100M       # cache > 100MB only
  duvis ~/projects --ui                                   # browser UI
  duvis --explain-category node_modules                   # which rule fires?

Exit codes
  0  Success
  1  Argument error, scan failure, or other error (message on stderr)";

#[cfg(not(feature = "ui"))]
pub(super) const HELP_TEXT: &str = "duvis - Disk usage visualizer for both AI agents and humans

Usage:
  duvis [PATH] [options]
  duvis --explain-category <NAME>

duvis is strictly read-only — it never deletes anything and never recommends what to delete.
The default output is a colorized terminal tree. Pass --summary, --json, --toon, --ndjson,
or --largest to get a different view of the same scan.

Display options
  -d, --max-depth <N>     Maximum depth to display. Affects display only — sizes always sum
                          the full subtree.
  -n, --top <N>           Show only the largest N entries at each level. Selection by size;
                          display order follows --sort.
      --sort <size|name>  Sort order. Default: size.
      --reverse           Reverse the --sort order.
      --hardlinks <count-once|count-each>
                          Attribution for hardlinked files. Default: count-once (matches
                          `du`). count-each inflates totals when many links share an inode
                          (e.g. pnpm stores). Unix only.

Output formats (mutually exclusive; default = colorized terminal tree)
      --json              Structured JSON tree to stdout. Shape: `{meta, tree}`.
      --toon              Same `{meta, tree}` data as --json, encoded in TOON — an
                          indentation-based, tabular format that costs fewer LLM tokens.
                          Combines with --largest.
      --ndjson            Newline-delimited JSON, one entry per line, in DFS pre-order.
                          Designed for jq / streaming agents.
      --summary           Per-category size summary (cache / build / log / media / vcs /
                          ide / other).
      --largest <N>       Flat list of the N largest entries globally, ordered by size.
                          Combines with --json / --toon / --ndjson for structured output.

Filters (AND-combined; affect display only, not totals)
      --category <CAT>    Restrict to one or more categories. Repeatable / CSV:
                          `--category cache,build`. Categories: cache, build, log, media,
                          vcs, ide, other, archive, installer, vm_image, model_cache, backup.
      --type <file|dir>   Restrict by entry type.
      --min-size <SIZE>   Show only entries at least this size. 1024-based:
                          `100M`, `1.5G`, `50KiB`, `1024` (bare = bytes).
      --name <GLOB>       Restrict to basenames matching one or more globs. Repeatable;
                          multiple are OR-combined: `--name \"*.log\" --name \"*.tmp\"`.
      --changed-within <DURATION>
                          Modified within the past `Nd` / `Nw` / `Nm` / `Ny` (m=30d, y=365d).
      --changed-before <DURATION>
                          Modified more than <DURATION> ago. Combine with --changed-within
                          for a window.

Diagnostics
      --explain-category <NAME>
                          Explain how a name would be classified, without scanning. Prints
                          both interpretations (as-dir / as-file) and the matched rule.
                          Combine with --json for structured output. Skips scanning
                          entirely; PATH is ignored.

  -h, --help              Show this help.
  -V, --version           Show version.

Examples
  duvis ~/projects                                        # tree (default)
  duvis ~/projects --max-depth 2 --top 10                 # depth-limited
  duvis ~/projects --summary                              # category summary
  duvis ~/projects --json | jq '.tree.children[]'         # structured for agents
  duvis ~/projects --ndjson | jq -c 'select(.size>1e8)'   # streaming filter
  duvis ~/projects --largest 10                           # 10 largest globally
  duvis ~/projects --category cache --min-size 100M       # cache > 100MB only
  duvis --explain-category node_modules                   # which rule fires?

Exit codes
  0  Success
  1  Argument error, scan failure, or other error (message on stderr)";

#[cfg(test)]
mod tests {
    use super::HELP_TEXT;

    /// Guard against drift: when the `ui` feature is off, the help text
    /// must not advertise flags that the parser rejects; when it's on,
    /// it must.
    #[cfg(feature = "ui")]
    #[test]
    fn ui_help_mentions_ui_flags() {
        assert!(HELP_TEXT.contains("--ui"));
        assert!(HELP_TEXT.contains("--port"));
    }

    #[cfg(not(feature = "ui"))]
    #[test]
    fn no_ui_help_omits_ui_flags() {
        assert!(
            !HELP_TEXT.contains("--ui"),
            "no-ui HELP_TEXT still mentions --ui"
        );
        assert!(
            !HELP_TEXT.contains("--port"),
            "no-ui HELP_TEXT still mentions --port"
        );
    }
}
