use serde::Serialize;

use crate::category::Category;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SortOrder {
    Size,
    Name,
}

#[derive(Debug, Serialize, Clone)]
pub struct Entry {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub category: Category,
    /// Days since last modification (None if unavailable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_days_ago: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Entry>>,
}

impl Entry {
    pub fn sort(&mut self, order: &SortOrder, reverse: bool) {
        if let Some(children) = &mut self.children {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(name: &str, size: u64) -> Entry {
        Entry {
            name: name.to_string(),
            size,
            is_dir: false,
            category: Category::Other,
            modified_days_ago: None,
            children: None,
        }
    }

    fn dir(name: &str, children: Vec<Entry>) -> Entry {
        let size = children.iter().map(|c| c.size).sum();
        Entry {
            name: name.to_string(),
            size,
            is_dir: true,
            category: Category::Other,
            modified_days_ago: None,
            children: Some(children),
        }
    }

    #[test]
    fn sort_by_size_descending_by_default() {
        let mut root = dir(
            "root",
            vec![leaf("small", 10), leaf("big", 1000), leaf("medium", 100)],
        );
        root.sort(&SortOrder::Size, false);
        let names: Vec<_> = root
            .children
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["big", "medium", "small"]);
    }

    #[test]
    fn sort_by_name_ascending() {
        let mut root = dir("root", vec![leaf("c", 1), leaf("a", 2), leaf("b", 3)]);
        root.sort(&SortOrder::Name, false);
        let names: Vec<_> = root
            .children
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn sort_reverse_flips_order() {
        let mut root = dir("root", vec![leaf("a", 1), leaf("b", 2), leaf("c", 3)]);
        root.sort(&SortOrder::Size, true);
        let names: Vec<_> = root
            .children
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn sort_recurses_into_children() {
        let mut root = dir("root", vec![dir("inner", vec![leaf("z", 1), leaf("a", 2)])]);
        root.sort(&SortOrder::Name, false);
        let inner = &root.children.as_ref().unwrap()[0];
        let names: Vec<_> = inner
            .children
            .as_ref()
            .unwrap()
            .iter()
            .map(|e| e.name.clone())
            .collect();
        assert_eq!(names, vec!["a", "z"]);
    }
}
