//! TOON (Token-Oriented Object Notation) output: same data as `--json`,
//! encoded in a compact, indentation-based format designed to cost fewer
//! tokens when fed to an LLM. Uniform arrays collapse to a tabular form
//! (one header naming the fields, then comma-separated value rows), and
//! object braces / quotes / key repetition are dropped.
//!
//! This module owns no tree-shaping logic of its own: it builds the very
//! same `{meta, tree}` DTO as `render/json.rs` (via [`super::json::build_root`])
//! and only swaps `serde_json` for the TOON encoder, so the two formats
//! can't drift in what they report — only in how they spell it.

use std::io::Write;

use anyhow::Result;

use super::{json, RenderConfig};
use crate::entry::Entry;

pub(crate) fn write(entry: &Entry, config: &RenderConfig, out: &mut impl Write) -> Result<()> {
    let root = json::build_root(entry, config);
    let encoded = toon_format::encode_default(&root)?;
    out.write_all(encoded.as_bytes())?;
    writeln!(out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::Category;
    use crate::filter::Filter;
    use crate::scan::HardlinkPolicy;
    use std::path::PathBuf;

    fn dir(name: &str, children: Vec<Entry>) -> Entry {
        Entry::dir(name.to_string(), Category::Other, None, children)
    }

    fn file(name: &str, size: u64) -> Entry {
        Entry::file(name.to_string(), size, Category::Other, None)
    }

    fn cfg<'a>(
        scan_root: &'a PathBuf,
        counts: &'a crate::scan::ScanCounts,
        filter: &'a Filter,
    ) -> RenderConfig<'a> {
        RenderConfig {
            max_depth: None,
            top: None,
            scan_root,
            counts,
            hardlinks: HardlinkPolicy::CountOnce,
            filter,
        }
    }

    #[test]
    fn emits_indentation_based_meta_and_tree_without_json_braces() {
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scan::ScanCounts::default();
        let filter = Filter::default();
        let cfg = cfg(&scan_root, &counts, &filter);
        let tree = dir(
            "proj",
            vec![dir("src", vec![file("main.rs", 5)]), file("readme.md", 3)],
        );
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // TOON spells keys bare (`key: value`) rather than JSON's `"key":`.
        assert!(out.contains("scan_root: /tmp/proj"));
        assert!(out.contains("relative_path:") || out.contains("relative_path "));
        // No JSON object braces in the encoded form.
        assert!(
            !out.contains('{') || out.contains("}:"),
            "unexpected raw JSON braces: {out}"
        );
    }

    #[test]
    fn quotes_names_containing_the_field_delimiter() {
        // A filename with a comma must not be read as two tabular columns.
        let scan_root = PathBuf::from("/tmp/proj");
        let counts = crate::scan::ScanCounts::default();
        let filter = Filter::default();
        let cfg = cfg(&scan_root, &counts, &filter);
        let tree = dir("proj", vec![file("a,b.txt", 1)]);
        let mut buf = Vec::new();
        write(&tree, &cfg, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("\"a,b.txt\""), "comma name not quoted: {out}");
    }
}
