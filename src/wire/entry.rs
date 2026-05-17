//! Wire shape for a single [`Entry`] tree node as embedded in the UI
//! `/data.json` response.
//!
//! Mirrors the v0.1.0 wire format: `name`, `size`, `is_dir`,
//! `category`, plus optional `modified_days_ago` and recursive
//! `children`. `EntryKind` is flattened back into an `is_dir` boolean
//! and a sibling `children` array, the way the browser UI has always
//! consumed it.
//!
//! Output renderers (`render/json.rs`, `render/ndjson.rs`,
//! `render/largest.rs`) emit *richer* shapes (relative_path, depth,
//! file_count, size_human, …) defined in [`super::tree`] and
//! [`super::largest`]; this lean shape exists for the UI consumer.

use serde::Serialize;

use crate::entry::{Entry, EntryKind};

/// Lean tree node used by the UI. Keeping it a plain struct (rather
/// than reaching for a custom `Serialize` impl as the domain `Entry`
/// used to) means `#[serde(skip_serializing_if = "Option::is_none")]`
/// handles the optional-field discipline declaratively.
#[derive(Debug, Serialize)]
pub(crate) struct WireEntry {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) is_dir: bool,
    pub(crate) category: crate::classify::Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) modified_days_ago: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) children: Option<Vec<WireEntry>>,
}

impl WireEntry {
    /// Project an [`Entry`] tree onto the wire shape. Files get
    /// `children: None`; directories get `children: Some(_)`, even if
    /// empty — the empty array carries information ("we descended but
    /// found nothing") that distinguishes it from a depth-truncated
    /// view.
    pub(crate) fn from_entry(entry: &Entry) -> Self {
        let children = match &entry.kind {
            EntryKind::Dir(c) => Some(c.iter().map(WireEntry::from_entry).collect()),
            EntryKind::File => None,
        };
        Self {
            name: entry.name.clone(),
            size: entry.size,
            is_dir: entry.is_dir(),
            category: entry.category,
            modified_days_ago: entry.modified_days_ago,
            children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::Category;

    fn leaf(name: &str, size: u64) -> Entry {
        Entry::file(name.to_string(), size, Category::Other, None)
    }

    fn dir_with(name: &str, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), Category::Other, None, children)
    }

    #[test]
    fn wire_shape_matches_v0_1_0_format() {
        let root = dir_with("root", vec![leaf("a.txt", 10)]);
        let wire = WireEntry::from_entry(&root);
        let json = serde_json::to_string(&wire).unwrap();
        assert!(json.contains("\"is_dir\":true"));
        assert!(json.contains("\"children\":["));
        assert!(json.contains("\"is_dir\":false"));
        // `modified_days_ago` skipped on both root and child because
        // the constructors above leave it as `None`.
        assert!(!json.contains("modified_days_ago"));
    }

    #[test]
    fn empty_directory_serializes_with_an_empty_children_array() {
        // An empty dir keeps `children: []` (not omitted) so the UI
        // can distinguish "this dir is empty" from "the leaf is a
        // file".
        let empty_dir = dir_with("empty", vec![]);
        let wire = WireEntry::from_entry(&empty_dir);
        let json = serde_json::to_string(&wire).unwrap();
        assert!(json.contains("\"children\":[]"));
    }

    #[test]
    fn modified_days_ago_is_emitted_when_present() {
        let dated = Entry::file("a.txt".to_string(), 5, Category::Other, Some(7));
        let wire = WireEntry::from_entry(&dated);
        let json = serde_json::to_string(&wire).unwrap();
        assert!(json.contains("\"modified_days_ago\":7"));
    }
}
