//! Wire DTOs for `--explain-category --json`.
//!
//! Mirrors the shape used to live on the domain `Classification` /
//! `ClassificationReason` types directly. Moving it here means
//! `classify::rules` no longer needs `serde` at all, so a future tweak
//! to the classification model (renaming an enum variant, say) won't
//! silently shift the JSON contract; the wire shape only changes when
//! someone edits this file.

use serde::Serialize;

use crate::classify::{Category, Classification, ClassificationReason};

/// Top-level payload emitted by `--explain-category <NAME> --json`.
/// The same `name` is classified both as a directory and as a file
/// because the rules differ between the two roles (e.g. `node_modules`
/// is `cache` as a dir but `other` as a file).
#[derive(Debug, Serialize)]
pub(crate) struct WireExplain<'a> {
    pub(crate) name: &'a str,
    pub(crate) as_directory: WireClassification,
    pub(crate) as_file: WireClassification,
}

#[derive(Debug, Serialize)]
pub(crate) struct WireClassification {
    pub(crate) category: Category,
    pub(crate) reason: WireClassificationReason,
}

/// Tagged-union mirror of `ClassificationReason`. Keeping the
/// `#[serde(tag = "kind")]` attribute here (and not on the domain enum)
/// is the whole point of the wire/ split: the on-wire discriminator
/// name is a wire concern, not a domain concern.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum WireClassificationReason {
    DirNameExact { needle: &'static str },
    DirNameContainsCache,
    FileNameSuffix { needle: &'static str },
    FileExtension { needle: &'static str },
    Default,
}

impl<'a> WireExplain<'a> {
    pub(crate) fn new(name: &'a str, as_dir: &Classification, as_file: &Classification) -> Self {
        Self {
            name,
            as_directory: WireClassification::from(as_dir),
            as_file: WireClassification::from(as_file),
        }
    }
}

impl From<&Classification> for WireClassification {
    fn from(c: &Classification) -> Self {
        Self {
            category: c.category,
            reason: WireClassificationReason::from(&c.reason),
        }
    }
}

impl From<&ClassificationReason> for WireClassificationReason {
    fn from(r: &ClassificationReason) -> Self {
        match *r {
            ClassificationReason::DirNameExact { needle } => Self::DirNameExact { needle },
            ClassificationReason::DirNameContainsCache => Self::DirNameContainsCache,
            ClassificationReason::FileNameSuffix { needle } => Self::FileNameSuffix { needle },
            ClassificationReason::FileExtension { needle } => Self::FileExtension { needle },
            ClassificationReason::Default => Self::Default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::{explain_dir, explain_file};

    #[test]
    fn wire_shape_matches_v0_1_x_explain_format() {
        // `node_modules` is `cache` as a dir (exact match) but `other`
        // as a file (no extension rule fires) — exercises both the
        // tagged DirNameExact reason and the Default fallback.
        let as_dir = explain_dir("node_modules");
        let as_file = explain_file("node_modules");
        let payload = WireExplain::new("node_modules", &as_dir, &as_file);
        let v: serde_json::Value = serde_json::to_value(&payload).unwrap();
        assert_eq!(v["name"], "node_modules");
        assert_eq!(v["as_directory"]["category"], "cache");
        assert_eq!(v["as_directory"]["reason"]["kind"], "dir_name_exact");
        assert_eq!(v["as_directory"]["reason"]["needle"], "node_modules");
        assert_eq!(v["as_file"]["category"], "other");
        assert_eq!(v["as_file"]["reason"]["kind"], "default");
    }
}
