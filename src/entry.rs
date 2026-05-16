use std::fmt;
use std::str::FromStr;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::category::Category;

/// How siblings are ordered for display. `Size` (default) is largest-first
/// before `--reverse` flips it; `Name` is alphabetical.
///
/// `Display` / `FromStr` are the canonical string forms used by the CLI
/// (`--sort size|name`) and any future programmatic caller. The core type
/// deliberately doesn't derive `clap::ValueEnum` — clap awareness lives
/// in `cli/args.rs`, which wires this via its `value_parser`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Size,
    Name,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SortOrder::Size => "size",
            SortOrder::Name => "name",
        })
    }
}

impl FromStr for SortOrder {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case-insensitive to match the previous `clap::ValueEnum` behaviour
        // (clap's PossibleValue matching is case-insensitive by default).
        match s.to_ascii_lowercase().as_str() {
            "size" => Ok(SortOrder::Size),
            "name" => Ok(SortOrder::Name),
            _ => Err(format!(
                "invalid sort order '{s}' (expected 'size' or 'name')"
            )),
        }
    }
}

/// File or directory variant. Holding children inside the `Dir` variant
/// makes "is this a directory?" and "does it have children?" the same
/// question at the type level — earlier the two were modeled as separate
/// fields (`is_dir: bool` + `children: Option<Vec<Entry>>`) and could in
/// principle disagree.
#[derive(Debug, Clone)]
pub enum EntryKind {
    File,
    Dir(Vec<Entry>),
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub size: u64,
    pub category: Category,
    /// Days since last modification (None if unavailable)
    pub modified_days_ago: Option<u64>,
    pub kind: EntryKind,
}

impl Entry {
    /// File constructor. Use this from the scanner so the (kind, size) pair
    /// stays consistent at construction time.
    pub fn file(
        name: String,
        size: u64,
        category: Category,
        modified_days_ago: Option<u64>,
    ) -> Self {
        Self {
            name,
            size,
            category,
            modified_days_ago,
            kind: EntryKind::File,
        }
    }

    /// Directory constructor. The directory's `size` is computed as the sum
    /// of its children's sizes — callers don't need to pre-aggregate.
    pub fn dir(
        name: String,
        category: Category,
        modified_days_ago: Option<u64>,
        children: Vec<Entry>,
    ) -> Self {
        let size = children.iter().map(|c| c.size).sum();
        Self {
            name,
            size,
            category,
            modified_days_ago,
            kind: EntryKind::Dir(children),
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, EntryKind::Dir(_))
    }

    pub fn children(&self) -> Option<&[Entry]> {
        match &self.kind {
            EntryKind::Dir(c) => Some(c),
            EntryKind::File => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<Entry>> {
        match &mut self.kind {
            EntryKind::Dir(c) => Some(c),
            EntryKind::File => None,
        }
    }

    pub fn sort(&mut self, order: &SortOrder, reverse: bool) {
        if let Some(children) = self.children_mut() {
            for child in children.iter_mut() {
                child.sort(order, reverse);
            }
            children.sort_by(|a, b| {
                let cmp = match order {
                    SortOrder::Size => b.size.cmp(&a.size),
                    SortOrder::Name => a.name.cmp(&b.name),
                };
                if reverse {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
        }
    }
}

/// Custom Serialize impl that flattens `EntryKind` back into the v0.1.0
/// wire format (`is_dir: bool` + optional `children: [Entry]`). The
/// browser UI and any AI agent consuming `/data.json` rely on this shape.
impl Serialize for Entry {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        // Field count = name, size, is_dir, category, [modified_days_ago],
        // [children]. Worth being a bit loose — the count is a hint, not a
        // contract for serde_json.
        let mut s = ser.serialize_struct("Entry", 6)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("size", &self.size)?;
        s.serialize_field("is_dir", &self.is_dir())?;
        s.serialize_field("category", &self.category)?;
        if let Some(d) = self.modified_days_ago {
            s.serialize_field("modified_days_ago", &d)?;
        }
        if let EntryKind::Dir(children) = &self.kind {
            s.serialize_field("children", children)?;
        }
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(name: &str, size: u64) -> Entry {
        Entry::file(name.to_string(), size, Category::Other, None)
    }

    fn dir_with(name: &str, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), Category::Other, None, children)
    }

    #[test]
    fn sort_by_size_descending_by_default() {
        let mut root = dir_with(
            "root",
            vec![leaf("small", 10), leaf("big", 1000), leaf("medium", 100)],
        );
        root.sort(&SortOrder::Size, false);
        let names: Vec<_> = root
            .children()
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["big", "medium", "small"]);
    }

    #[test]
    fn sort_by_name_ascending() {
        let mut root = dir_with("root", vec![leaf("c", 1), leaf("a", 2), leaf("b", 3)]);
        root.sort(&SortOrder::Name, false);
        let names: Vec<_> = root
            .children()
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn sort_reverse_flips_order() {
        let mut root = dir_with("root", vec![leaf("a", 1), leaf("b", 2), leaf("c", 3)]);
        root.sort(&SortOrder::Size, true);
        let names: Vec<_> = root
            .children()
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn sort_recurses_into_children() {
        let mut root = dir_with(
            "root",
            vec![dir_with("inner", vec![leaf("z", 1), leaf("a", 2)])],
        );
        root.sort(&SortOrder::Name, false);
        let inner = &root.children().unwrap()[0];
        let names: Vec<_> = inner
            .children()
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["a", "z"]);
    }

    #[test]
    fn dir_size_is_sum_of_children() {
        let d = dir_with("root", vec![leaf("a", 10), leaf("b", 20), leaf("c", 30)]);
        assert_eq!(d.size, 60);
        assert!(d.is_dir());
    }

    #[test]
    fn file_has_no_children() {
        let f = leaf("a.txt", 42);
        assert!(!f.is_dir());
        assert!(f.children().is_none());
    }

    #[test]
    fn json_round_trip_preserves_wire_shape() {
        // Sanity-check that the custom Serialize matches the v0.1.0 wire
        // format: is_dir + children present, modified_days_ago skipped when
        // None.
        let root = dir_with("root", vec![leaf("a.txt", 10)]);
        let json = serde_json::to_string(&root).unwrap();
        assert!(json.contains("\"is_dir\":true"));
        assert!(json.contains("\"children\":["));
        assert!(json.contains("\"is_dir\":false"));
        assert!(!json.contains("modified_days_ago"));
    }
}
