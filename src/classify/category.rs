//! `Category` enum + the Core / Extended `Tier` split surfaced in the
//! UI legend.
//!
//! `Display` / `FromStr` use the canonical snake_case names (`cache`,
//! `vm_image`, …); clap awareness lives in `cli/args.rs`. Wire-format
//! serialization lives in `wire::category` so domain types stay free of
//! serde derives.

use std::fmt;

/// Whether a category is part of the always-shown core vocabulary or an
/// extended category that only surfaces in the legend / sidebar when at
/// least one entry of that category is actually present in the scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Universal categories. The color and label are reserved in the legend
    /// regardless of whether anything was matched, so the visual vocabulary
    /// stays stable across scans.
    Core,
    /// Niche categories that would be noise on a typical project tree but
    /// are valuable when present (e.g. a 5 GB `model_cache` block on an AI
    /// dev machine, or a 50 GB `vm_image` on a VM user's disk).
    Extended,
}

/// File / directory categorisation surfaced in the legend, in `--summary`,
/// and as the `--category` filter values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    // ----- Core -----
    Cache,
    Build,
    Log,
    Media,
    Vcs,
    Ide,
    Other,
    // ----- Extended -----
    Archive,
    Installer,
    VmImage,
    ModelCache,
    Backup,
}

impl Category {
    /// Every variant in declaration order. Single source of truth for
    /// iteration (`FromStr` error message, round-trip tests, future
    /// listings). Adding a new variant to the enum requires extending
    /// this slice — the round-trip test in this module will catch
    /// drift.
    pub const ALL: &'static [Category] = &[
        Category::Cache,
        Category::Build,
        Category::Log,
        Category::Media,
        Category::Vcs,
        Category::Ide,
        Category::Other,
        Category::Archive,
        Category::Installer,
        Category::VmImage,
        Category::ModelCache,
        Category::Backup,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Category::Cache => "cache",
            Category::Build => "build",
            Category::Log => "log",
            Category::Media => "media",
            Category::Vcs => "vcs",
            Category::Ide => "ide",
            Category::Other => "other",
            Category::Archive => "archive",
            Category::Installer => "installer",
            Category::VmImage => "vm_image",
            Category::ModelCache => "model_cache",
            Category::Backup => "backup",
        }
    }

    pub fn tier(&self) -> Tier {
        match self {
            Category::Cache
            | Category::Build
            | Category::Log
            | Category::Media
            | Category::Vcs
            | Category::Ide
            | Category::Other => Tier::Core,
            Category::Archive
            | Category::Installer
            | Category::VmImage
            | Category::ModelCache
            | Category::Backup => Tier::Extended,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl std::str::FromStr for Category {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case-insensitive to match the previous `clap::ValueEnum`
        // behaviour. Matching and the error message both come from
        // `Category::ALL` + `label()` so adding a new variant only
        // requires extending the enum and `ALL`.
        let lower = s.to_ascii_lowercase();
        Category::ALL
            .iter()
            .copied()
            .find(|c| c.label() == lower)
            .ok_or_else(|| {
                let labels: Vec<&str> = Category::ALL.iter().map(|c| c.label()).collect();
                format!(
                    "invalid category '{s}' (expected one of: {})",
                    labels.join(", ")
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_split_matches_intent() {
        // Core categories.
        for c in [
            Category::Cache,
            Category::Build,
            Category::Log,
            Category::Media,
            Category::Vcs,
            Category::Ide,
            Category::Other,
        ] {
            assert_eq!(c.tier(), Tier::Core, "{c:?} should be Core");
        }
        // Extended categories.
        for c in [
            Category::Archive,
            Category::Installer,
            Category::VmImage,
            Category::ModelCache,
            Category::Backup,
        ] {
            assert_eq!(c.tier(), Tier::Extended, "{c:?} should be Extended");
        }
    }

    #[test]
    fn category_display_round_trips_through_from_str() {
        for &c in Category::ALL {
            let s = c.to_string();
            let parsed: Category = s.parse().expect("label() must parse back via FromStr");
            assert_eq!(parsed, c, "round-trip mismatch for {c:?} via '{s}'");
        }
    }

    #[test]
    fn category_from_str_is_case_insensitive() {
        assert_eq!("CACHE".parse::<Category>().unwrap(), Category::Cache);
        assert_eq!("Vm_Image".parse::<Category>().unwrap(), Category::VmImage);
    }

    #[test]
    fn category_from_str_rejects_unknown_with_full_list() {
        let err = "bogus".parse::<Category>().unwrap_err();
        // Error message should mention every variant so users see the
        // full vocabulary even when they typo.
        for c in Category::ALL {
            assert!(
                err.contains(c.label()),
                "error message missing '{}': {err}",
                c.label()
            );
        }
    }
}
