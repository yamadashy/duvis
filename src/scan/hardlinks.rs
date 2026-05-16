//! `HardlinkPolicy` — how to attribute bytes when the same inode is
//! reachable via multiple hardlinked paths. CLI surface enum; the
//! actual dedup happens in `super::metadata::file_disk_usage`.

use std::fmt;
use std::str::FromStr;

/// How to attribute bytes when the same inode is reachable via multiple
/// hardlinked paths. Default matches `du` — each inode counts once. Unix
/// only; on Windows this knob has no effect because the std `Metadata`
/// API can't surface a portable file id.
///
/// `Display` / `FromStr` are the canonical string forms used by the CLI
/// (`--hardlinks count-once|count-each`). clap awareness lives in
/// `cli/args.rs`; the core type stays clap-free.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HardlinkPolicy {
    /// Each (dev, inode) is counted once. Subsequent links to the same
    /// inode contribute 0 bytes. Matches `du`'s default behavior.
    #[default]
    CountOnce,
    /// Each link contributes its full disk usage, even when the underlying
    /// inode is shared. Inflates totals on trees with many hardlinks
    /// (e.g. content-addressable stores like pnpm).
    CountEach,
}

impl fmt::Display for HardlinkPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HardlinkPolicy::CountOnce => "count-once",
            HardlinkPolicy::CountEach => "count-each",
        })
    }
}

impl FromStr for HardlinkPolicy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case-insensitive to match the previous `clap::ValueEnum` behaviour.
        match s.to_ascii_lowercase().as_str() {
            "count-once" => Ok(HardlinkPolicy::CountOnce),
            "count-each" => Ok(HardlinkPolicy::CountEach),
            _ => Err(format!(
                "invalid hardlink policy '{s}' (expected 'count-once' or 'count-each')"
            )),
        }
    }
}
