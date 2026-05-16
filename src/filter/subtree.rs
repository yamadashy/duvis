//! Pre-computed "is this entry or any descendant a filter match?" map.
//!
//! Tree-form renderers (tree / json) need to decide whether a directory
//! should be shown even when the directory itself doesn't match the
//! filter — because one of its descendants does. Computing that per
//! visit would be O(N²) on deep trees; the precompute walks the tree
//! once bottom-up and records the bit for every entry so renderers can
//! lookup in O(1).
//!
//! Pointer keys are safe because the tree is owned by `main` for the
//! duration of the render call and never moved.

use std::collections::HashMap;

use crate::entry::Entry;

use super::Filter;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::Category;
    use crate::filter::FilterInputs;

    fn dir(name: &str, cat: Category, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), cat, None, children)
    }
    fn file(name: &str, size: u64, cat: Category, days_ago: Option<u64>) -> Entry {
        Entry::file(name.to_string(), size, cat, days_ago)
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
            categories: vec![],
            type_: None,
            min_size: None,
            names: vec!["*.rs".into()],
            changed_within: None,
            changed_before: None,
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
