//! Factual filters for the display layer.
//!
//! Filters are AND-combined and applied *after* scanning, so totals
//! (parent dir sizes, scan counts in `meta`) always reflect the full
//! scanned tree. Only what's *shown* is filtered. This keeps the
//! "we measured 1.2 TB but you asked to see only the .log files"
//! mental model honest.
//!
//! Per-mode behavior:
//! - tree / json (tree-form): find-style. A directory is shown when
//!   the dir itself matches OR any of its descendants does. A file is
//!   shown when it matches. The pre-pass `precompute_subtree_match`
//!   records `subtree_has_match` once so renderers can lookup O(1).
//! - summary: only matching entries enter the per-category aggregate.
//! - ndjson: only matching entries are emitted.
//! - largest: filter rows before the partial sort + truncate.
//!
//! Per-path attribution under `--hardlinks count-once` is already
//! rayon-order-dependent; filters that reference per-path attributes
//! (e.g. `--min-size`) inherit that nondeterminism. Stability holds at
//! the unfiltered scan-total level, not at the per-path level.

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::category::Category;
use crate::entry::Entry;

/// `--type` filter: file or dir. Anything else (symlink etc.) is
/// reduced to one of these by `Entry::is_dir()`.
///
/// `Display` / `FromStr` are the canonical string forms used by the CLI
/// (`--type file|dir`). clap awareness lives in `cli/args.rs`; the core
/// type stays clap-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Dir,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            EntryType::File => "file",
            EntryType::Dir => "dir",
        })
    }
}

impl std::str::FromStr for EntryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(EntryType::File),
            "dir" => Ok(EntryType::Dir),
            other => Err(format!(
                "invalid entry type '{other}' (expected 'file' or 'dir')"
            )),
        }
    }
}

#[derive(Debug, Default)]
pub struct Filter {
    /// Empty = no category filter; non-empty = AND requires
    /// `entry.category ∈ this set`.
    categories: HashSet<Category>,
    type_: Option<EntryType>,
    min_size: Option<u64>,
    /// `None` = no name filter; `Some(set)` = AND requires
    /// `entry.name` to match at least one glob in `set` (OR within the
    /// set, AND with the rest of the filters).
    names: Option<GlobSet>,
    changed_within_days: Option<u64>,
    changed_before_days: Option<u64>,
}

/// Inputs collected from clap. Kept as a separate struct so `Filter`
/// itself can stay decoupled from the CLI surface and be unit-testable.
pub struct FilterInputs {
    pub categories: Vec<Category>,
    pub type_: Option<EntryType>,
    pub min_size: Option<String>,
    pub names: Vec<String>,
    pub changed_within: Option<String>,
    pub changed_before: Option<String>,
}

impl Filter {
    pub fn from_inputs(inputs: FilterInputs) -> Result<Self> {
        let categories: HashSet<Category> = inputs.categories.into_iter().collect();

        let names = if inputs.names.is_empty() {
            None
        } else {
            let mut builder = GlobSetBuilder::new();
            for pat in &inputs.names {
                let glob = Glob::new(pat)
                    .map_err(|e| anyhow!("--name '{pat}' is not a valid glob: {e}"))?;
                builder.add(glob);
            }
            Some(builder.build()?)
        };

        let min_size = inputs.min_size.as_deref().map(parse_size).transpose()?;
        let changed_within_days = inputs
            .changed_within
            .as_deref()
            .map(parse_duration_days)
            .transpose()?;
        let changed_before_days = inputs
            .changed_before
            .as_deref()
            .map(parse_duration_days)
            .transpose()?;

        Ok(Self {
            categories,
            type_: inputs.type_,
            min_size,
            names,
            changed_within_days,
            changed_before_days,
        })
    }

    /// `true` when no filter has been set — renderers can fast-path
    /// skip the per-entry match call.
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
            && self.type_.is_none()
            && self.min_size.is_none()
            && self.names.is_none()
            && self.changed_within_days.is_none()
            && self.changed_before_days.is_none()
    }

    /// Test a single entry against every active sub-filter (AND).
    pub fn matches(&self, entry: &Entry) -> bool {
        if !self.categories.is_empty() && !self.categories.contains(&entry.category) {
            return false;
        }
        if let Some(t) = self.type_ {
            match t {
                EntryType::File if entry.is_dir() => return false,
                EntryType::Dir if !entry.is_dir() => return false,
                _ => {}
            }
        }
        if let Some(min) = self.min_size {
            if entry.size < min {
                return false;
            }
        }
        if let Some(globs) = &self.names {
            if !globs.is_match(&entry.name) {
                return false;
            }
        }
        // mtime filters: entries with no `modified_days_ago` are
        // excluded conservatively. In practice the scanner always sets
        // it, so this is just defense against future Entry constructors.
        // `--changed-within` keeps recent entries (mtime ≤ threshold).
        if let Some(threshold) = self.changed_within_days {
            match entry.modified_days_ago {
                Some(d) if d <= threshold => {}
                _ => return false,
            }
        }
        // `--changed-before` keeps stale entries (mtime > threshold).
        if let Some(threshold) = self.changed_before_days {
            match entry.modified_days_ago {
                Some(d) if d > threshold => {}
                _ => return false,
            }
        }
        true
    }
}

/// `subtree_has_match[entry] = self.matches(entry) ∨ ∃ child : subtree_has_match[child]`.
/// Used by tree / json renderers so they can skip subtrees with no
/// matching descendants in O(1) per entry.
pub type SubtreeMatch = HashMap<*const Entry, bool>;

pub fn precompute_subtree_match(entry: &Entry, filter: &Filter) -> SubtreeMatch {
    let mut map: SubtreeMatch = HashMap::new();
    walk_match(entry, filter, &mut map);
    map
}

fn walk_match(entry: &Entry, filter: &Filter, map: &mut SubtreeMatch) -> bool {
    let mut any = filter.matches(entry);
    if let Some(children) = entry.children() {
        for child in children {
            // Always recurse — a non-matching dir might still hold a
            // matching descendant, and we need that bit recorded for
            // the renderer.
            let child_has = walk_match(child, filter, map);
            any = any || child_has;
        }
    }
    map.insert(entry as *const Entry, any);
    any
}

/// Look up "should this entry contribute to the rendered view?" given
/// a filter precompute result. Returns whether `entry` itself or any
/// descendant matched according to the precomputed `SubtreeMatch`.
/// Returns `false` for entries not present in the map (e.g. when
/// `precompute_subtree_match` was rooted elsewhere). Renderers are
/// responsible for emitting the scan-root header / `meta` block
/// regardless of this value, so an "empty match" scan still produces a
/// coherent (zero-result) document.
pub fn subtree_visible(entry: &Entry, map: &SubtreeMatch) -> bool {
    map.get(&(entry as *const Entry)).copied().unwrap_or(false)
}

// =========================================================================
// Parsers
// =========================================================================

/// Parse a human-friendly byte count: bare integer = bytes, suffix
/// `B / K / KB / KiB / M / MB / MiB / G / GB / GiB / T / TB / TiB`
/// (case-insensitive). All multipliers are 1024-based to match
/// `format_size` and `du -h`. Floating-point is accepted: `1.5G`.
pub fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow!("empty size"));
    }

    // Split numeric prefix from suffix. We accept floats so we can't
    // just take the longest digit run.
    let split = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (num_str, unit) = s.split_at(split);
    let num: f64 = num_str
        .parse()
        .map_err(|_| anyhow!("not a number: '{num_str}' in '{s}'"))?;
    if !num.is_finite() {
        return Err(anyhow!("not a finite number: '{s}'"));
    }
    if num.is_sign_negative() {
        return Err(anyhow!("negative size: '{s}'"));
    }

    let mult = match unit.trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1u64,
        "K" | "KB" | "KIB" => 1024,
        "M" | "MB" | "MIB" => 1024 * 1024,
        "G" | "GB" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TB" | "TIB" => 1024u64.pow(4),
        other => return Err(anyhow!("unknown size unit: '{other}' in '{s}'")),
    };

    // Range check before the cast: `f64 as u64` saturates silently to
    // u64::MAX on overflow, which would let `--min-size 99999P` quietly
    // become an effectively unmatchable filter instead of an error.
    // Use `>=` because `u64::MAX as f64` rounds *up* to 2^64 (u64::MAX
    // itself isn't representable in f64), so `> u64::MAX as f64` would
    // miss values that round-trip to exactly 2^64 and then saturate.
    let bytes = (num * mult as f64).round();
    if bytes >= u64::MAX as f64 {
        return Err(anyhow!("size out of range (exceeds u64): '{s}'"));
    }
    Ok(bytes as u64)
}

/// Parse a relative duration: `<integer><suffix>` where suffix is
/// `d` (days, also default), `w` (7d), `m` (30d), `y` (365d).
/// Returns the duration in *days*. Calendar-correctness is not the
/// goal here — agents asking "files older than 30 days" don't want
/// a date library, they want a fast cutoff.
pub fn parse_duration_days(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow!("empty duration"));
    }

    let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (num_str, unit) = s.split_at(split);
    let num: u64 = num_str
        .parse()
        .map_err(|_| anyhow!("not an integer: '{num_str}' in '{s}'"))?;

    let days_per_unit = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "d" => 1,
        "w" => 7,
        "m" => 30,
        "y" => 365,
        other => {
            return Err(anyhow!(
                "unknown duration unit: '{other}' in '{s}' (use d/w/m/y)"
            ))
        }
    };

    num.checked_mul(days_per_unit)
        .ok_or_else(|| anyhow!("duration overflow: '{s}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str, cat: Category, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), cat, None, children)
    }
    fn file(name: &str, size: u64, cat: Category, days_ago: Option<u64>) -> Entry {
        Entry::file(name.to_string(), size, cat, days_ago)
    }

    fn empty_inputs() -> FilterInputs {
        FilterInputs {
            categories: vec![],
            type_: None,
            min_size: None,
            names: vec![],
            changed_within: None,
            changed_before: None,
        }
    }

    #[test]
    fn parse_size_1024_based() {
        assert_eq!(parse_size("0").unwrap(), 0);
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1KiB").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1.5M").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_size("1g").unwrap(), 1024u64.pow(3));
        assert_eq!(parse_size("1T").unwrap(), 1024u64.pow(4));
    }

    #[test]
    fn parse_size_rejects_garbage() {
        assert!(parse_size("").is_err());
        assert!(parse_size("xyz").is_err());
        assert!(parse_size("1XB").is_err());
        assert!(parse_size("-1M").is_err());
    }

    #[test]
    fn parse_size_rejects_non_finite_and_overflow() {
        // f64::parse accepts "inf"/"nan" — we must reject them, not let
        // them silently cast to u64::MAX / 0.
        assert!(parse_size("inf").is_err());
        assert!(parse_size("nan").is_err());
        // 1e20 bytes overflows u64 (max ≈ 1.8e19).
        assert!(parse_size("1e20").is_err());
        // Same magnitude with a unit multiplier.
        assert!(parse_size("99999999999T").is_err());
        // Boundary: u64::MAX is not representable in f64 (rounds up to 2^64),
        // so the literal value must also be rejected — otherwise it would
        // silently saturate via `as u64`.
        assert!(parse_size("18446744073709551615").is_err());
    }

    #[test]
    fn parse_duration_days_works() {
        assert_eq!(parse_duration_days("7").unwrap(), 7);
        assert_eq!(parse_duration_days("7d").unwrap(), 7);
        assert_eq!(parse_duration_days("2w").unwrap(), 14);
        assert_eq!(parse_duration_days("3m").unwrap(), 90);
        assert_eq!(parse_duration_days("1y").unwrap(), 365);
        assert_eq!(parse_duration_days("1Y").unwrap(), 365);
    }

    #[test]
    fn parse_duration_rejects_unknown_suffix() {
        assert!(parse_duration_days("7h").is_err());
        assert!(parse_duration_days("").is_err());
        assert!(parse_duration_days("abc").is_err());
    }

    #[test]
    fn empty_filter_matches_everything() {
        let f = Filter::from_inputs(empty_inputs()).unwrap();
        assert!(f.is_empty());
        let e = file("a.txt", 10, Category::Other, Some(0));
        assert!(f.matches(&e));
    }

    #[test]
    fn category_and_type_filters_combine_with_and() {
        let inputs = FilterInputs {
            categories: vec![Category::Cache],
            type_: Some(EntryType::File),
            ..empty_inputs()
        };
        let f = Filter::from_inputs(inputs).unwrap();

        assert!(f.matches(&file("a", 0, Category::Cache, None)));
        assert!(!f.matches(&file("a", 0, Category::Build, None))); // wrong category
        assert!(!f.matches(&dir("a", Category::Cache, vec![]))); // wrong type
    }

    #[test]
    fn min_size_filter() {
        let inputs = FilterInputs {
            min_size: Some("1K".into()),
            ..empty_inputs()
        };
        let f = Filter::from_inputs(inputs).unwrap();
        assert!(f.matches(&file("a", 1024, Category::Other, None)));
        assert!(f.matches(&file("a", 2048, Category::Other, None)));
        assert!(!f.matches(&file("a", 1023, Category::Other, None)));
    }

    #[test]
    fn name_filter_combines_globs_with_or() {
        let inputs = FilterInputs {
            names: vec!["*.log".into(), "*.tmp".into()],
            ..empty_inputs()
        };
        let f = Filter::from_inputs(inputs).unwrap();
        assert!(f.matches(&file("server.log", 0, Category::Log, None)));
        assert!(f.matches(&file("scratch.tmp", 0, Category::Other, None)));
        assert!(!f.matches(&file("README.md", 0, Category::Other, None)));
    }

    #[test]
    fn mtime_filters() {
        let within = Filter::from_inputs(FilterInputs {
            changed_within: Some("7d".into()),
            ..empty_inputs()
        })
        .unwrap();
        let before = Filter::from_inputs(FilterInputs {
            changed_before: Some("30d".into()),
            ..empty_inputs()
        })
        .unwrap();

        let recent = file("a", 0, Category::Other, Some(3));
        let mid = file("a", 0, Category::Other, Some(15));
        let stale = file("a", 0, Category::Other, Some(60));

        assert!(within.matches(&recent));
        assert!(!within.matches(&mid));
        assert!(!within.matches(&stale));

        assert!(!before.matches(&recent));
        assert!(!before.matches(&mid));
        assert!(before.matches(&stale));
    }

    #[test]
    fn precompute_subtree_match_keeps_ancestors_of_matches() {
        // root/
        //   src/
        //     main.rs        (matches: *.rs)
        //   target/
        //     debug/
        //       app          (no match)
        //   notes.txt        (no match)
        let tree = dir(
            "root",
            Category::Other,
            vec![
                dir(
                    "src",
                    Category::Other,
                    vec![file("main.rs", 10, Category::Other, None)],
                ),
                dir(
                    "target",
                    Category::Build,
                    vec![dir(
                        "debug",
                        Category::Build,
                        vec![file("app", 100, Category::Build, None)],
                    )],
                ),
                file("notes.txt", 5, Category::Other, None),
            ],
        );
        let f = Filter::from_inputs(FilterInputs {
            names: vec!["*.rs".into()],
            ..empty_inputs()
        })
        .unwrap();
        let map = precompute_subtree_match(&tree, &f);

        // root and src kept (have a matching descendant).
        let root_ptr = &tree as *const Entry;
        assert!(*map.get(&root_ptr).unwrap());
        let src = tree
            .children()
            .unwrap()
            .iter()
            .find(|c| c.name == "src")
            .unwrap();
        assert!(*map.get(&(src as *const Entry)).unwrap());
        let main_rs = &src.children().unwrap()[0];
        assert!(*map.get(&(main_rs as *const Entry)).unwrap());

        // target and notes.txt: no matching descendant.
        let target = tree
            .children()
            .unwrap()
            .iter()
            .find(|c| c.name == "target")
            .unwrap();
        assert!(!*map.get(&(target as *const Entry)).unwrap());
        let notes = tree
            .children()
            .unwrap()
            .iter()
            .find(|c| c.name == "notes.txt")
            .unwrap();
        assert!(!*map.get(&(notes as *const Entry)).unwrap());
    }
}
