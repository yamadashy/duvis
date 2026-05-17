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
//!
//! Module layout:
//! - this facade owns the `Filter` / `FilterInputs` / `EntryType` API
//!   that the cli and renderers consume.
//! - [`parse`] holds the standalone value parsers (`parse_size`,
//!   `parse_duration_days`) so the cli can call them directly without
//!   constructing a `Filter`.
//! - [`subtree`] holds the tree-walk precompute used by tree-form
//!   renderers to skip subtrees with no matching descendants in O(1)
//!   per entry.

mod parse;
mod subtree;

use std::collections::HashSet;

use anyhow::{anyhow, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::classify::Category;
use crate::entry::Entry;

pub use parse::{parse_duration_days, parse_size};
pub use subtree::{precompute_subtree_match, subtree_visible, SubtreeMatch};

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
        // Case-insensitive to match the previous `clap::ValueEnum` behaviour.
        match s.to_ascii_lowercase().as_str() {
            "file" => Ok(EntryType::File),
            "dir" => Ok(EntryType::Dir),
            _ => Err(format!(
                "invalid entry type '{s}' (expected 'file' or 'dir')"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::Category;

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
}
