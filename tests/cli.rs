//! End-to-end CLI tests that invoke the built `duvis` binary against a
//! fixture directory. Sizes vary by filesystem block size, so we redact them
//! before snapshotting.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

/// Build a small directory tree that exercises every classifier branch:
/// node_modules → Cache, target → Build, *.log → Log, .git → VCS, regular
/// files → Other.
///
/// File sizes are deliberately spread across orders of magnitude so the
/// category ranking is the same on Unix (which rounds to FS block size via
/// `st_blocks * 512`) and Windows (apparent size). Otherwise the analyze
/// snapshot order would diverge between platforms.
fn build_fixture() -> TempDir {
    let dir = tempfile::Builder::new()
        .prefix("duvis-test-")
        .tempdir()
        .expect("tempdir");
    let root = dir.path();

    // Other (small text files)
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("readme.md"), "# project\n").unwrap();

    // Log: ~20 KB
    fs::write(root.join("server.log"), "log line\n".repeat(2_000)).unwrap();

    // Build: ~200 KB (largest)
    fs::create_dir_all(root.join("target/debug")).unwrap();
    fs::write(root.join("target/debug/app"), "x".repeat(200_000)).unwrap();

    // Cache: ~80 KB
    fs::create_dir_all(root.join("node_modules/foo")).unwrap();
    fs::write(
        root.join("node_modules/foo/index.js"),
        "console.log(1);\n".repeat(5_000),
    )
    .unwrap();

    // VCS: tiny, smaller than Other so it sorts last
    fs::create_dir_all(root.join(".git/objects")).unwrap();
    fs::write(root.join(".git/HEAD"), "ref: main\n").unwrap();

    dir
}

fn run_duvis(fixture: &Path, args: &[&str]) -> String {
    let output = Command::cargo_bin("duvis")
        .expect("cargo bin")
        .arg(fixture)
        .args(args)
        .output()
        .expect("spawn duvis");
    assert!(
        output.status.success(),
        "duvis exited non-zero: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout).expect("stdout is utf-8")
}

/// Settings that strip sources of cross-machine flakiness:
/// * the random tempdir basename in the first line of tree/analyze output;
/// * size strings (vary by FS block size between Unix and Windows);
/// * percentages (vary because the totals depend on which sizes round up).
///
/// The leading `\s+` on the size and percent patterns swallows the column
/// padding too, so column widths normalize regardless of the original
/// number's character length.
fn redacted_settings(fixture_basename: &str) -> insta::Settings {
    let mut s = insta::Settings::clone_current();
    s.add_filter(fixture_basename, "<FIXTURE>");
    s.add_filter(r"\s*\d+(\.\d+)?\s+(GB|MB|KB|B)", " <SIZE>");
    s.add_filter(r"\s+\d+%", " <PCT>");
    s
}

fn fixture_basename(dir: &Path) -> String {
    dir.file_name().unwrap().to_string_lossy().into_owned()
}

#[test]
fn tree_format_default() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--sort", "name"]);
    redacted_settings(&fixture_basename(fixture.path())).bind(|| {
        insta::assert_snapshot!("tree_default", stdout);
    });
}

#[test]
fn analyze_format() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--analyze"]);
    redacted_settings(&fixture_basename(fixture.path())).bind(|| {
        insta::assert_snapshot!("analyze_default", stdout);
    });
}

#[test]
fn json_format_is_valid_and_classifies() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--json", "--sort", "name"]);

    let value: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    // v0.1.4: top-level wrapper with `meta` + `tree`.
    let meta = value.get("meta").expect("missing meta");
    assert_eq!(meta["wire_version"], 2);
    assert!(meta.get("scan_root").is_some(), "missing meta.scan_root");
    assert_eq!(meta["hardlinks"], "count-once");
    assert!(meta.get("items_scanned").is_some(), "missing items_scanned");

    let tree = value.get("tree").expect("missing tree");
    assert!(tree.get("name").is_some(), "missing name");
    assert!(tree.get("size").is_some(), "missing size");
    assert!(tree.get("category").is_some(), "missing category");
    assert_eq!(tree["relative_path"], ".");
    assert_eq!(tree["depth"], 0);
    assert!(tree["file_count"].as_u64().unwrap() > 0);
    assert!(tree["dir_count"].as_u64().unwrap() > 0);

    let dump = value.to_string();
    // Each fixture branch should appear with the right category somewhere
    // in the tree.
    assert!(dump.contains("\"node_modules\""), "missing node_modules");
    assert!(dump.contains("\"cache\""), "missing cache category");
    assert!(dump.contains("\"build\""), "missing build category");
    assert!(dump.contains("\"vcs\""), "missing vcs category");
    assert!(dump.contains("\"log\""), "missing log category");
}

#[test]
fn ndjson_emits_meta_then_pre_order_entries() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--ndjson", "--sort", "name"]);

    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("each line is valid json"))
        .collect();
    assert!(lines.len() >= 2, "expected meta + at least one entry");
    assert_eq!(lines[0]["type"], "meta");
    assert_eq!(lines[0]["wire_version"], 2);

    // Every other record is an entry; the first entry is root with
    // relative_path ".".
    assert_eq!(lines[1]["type"], "entry");
    assert_eq!(lines[1]["relative_path"], ".");
    assert_eq!(lines[1]["depth"], 0);

    // Pre-order: a parent's relative_path always appears before any of
    // its descendants. Sanity-check by ensuring "node_modules" comes
    // before "node_modules/foo" etc.
    let paths: Vec<&str> = lines[1..]
        .iter()
        .filter_map(|l| l["relative_path"].as_str())
        .collect();
    let nm = paths.iter().position(|p| *p == "node_modules");
    let nm_foo = paths.iter().position(|p| *p == "node_modules/foo");
    assert!(nm.is_some() && nm_foo.is_some());
    assert!(nm.unwrap() < nm_foo.unwrap(), "parent must precede child");
}

#[test]
fn conflicting_format_flags_are_rejected() {
    // --json / --ndjson / --analyze / --ui are exclusive via clap ArgGroup.
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--json", "--analyze", "."])
        .assert()
        .failure();
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--json", "--ndjson", "."])
        .assert()
        .failure();
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--ndjson", "--analyze", "."])
        .assert()
        .failure();
    // --largest conflicts with --analyze and --ui (different views).
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--largest", "5", "--analyze", "."])
        .assert()
        .failure();
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--largest", "5", "--ui", "."])
        .assert()
        .failure();
}

#[test]
fn largest_text_lists_top_n_by_size() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--largest", "3"]);
    // Header announces both the requested count and the total entries
    // visited so an agent can tell whether it saw a truncated view.
    assert!(stdout.contains("Largest"), "missing header: {stdout}");
    assert!(stdout.contains("of "), "missing total count: {stdout}");
    // The build artifact is the biggest in the fixture and must show up
    // in any top-N where N >= 1.
    assert!(
        stdout.contains("target"),
        "expected 'target' in top 3: {stdout}"
    );
}

#[test]
fn largest_json_returns_meta_and_flat_largest_array() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--json", "--largest", "3"]);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    let meta = value.get("meta").expect("missing meta");
    assert_eq!(meta["largest_requested"], 3);
    assert!(meta.get("total_entries").is_some());
    // No tree field — flat list, not hierarchical.
    assert!(value.get("tree").is_none());
    let largest = value
        .get("largest")
        .expect("missing largest")
        .as_array()
        .unwrap();
    assert!(largest.len() <= 3);
    // Sorted by size descending.
    let sizes: Vec<u64> = largest
        .iter()
        .map(|e| e["size"].as_u64().unwrap())
        .collect();
    let mut sorted = sizes.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(sizes, sorted, "largest must be sorted by size desc");
}

#[test]
fn largest_ndjson_streams_meta_then_entries() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--ndjson", "--largest", "2"]);
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(lines.len() >= 2, "want meta + at least one entry");
    assert_eq!(lines[0]["type"], "meta");
    assert_eq!(lines[0]["largest_requested"], 2);
    assert!(lines[1..].iter().all(|l| l["type"] == "entry"));
}

#[test]
fn top_n_limits_children() {
    let fixture = build_fixture();
    let stdout = run_duvis(fixture.path(), &["--top", "1", "--sort", "name"]);
    // "and N more" line should appear because we capped at 1 child.
    assert!(
        stdout.contains("more"),
        "expected overflow line, got:\n{stdout}"
    );
}

// =========================================================================
// Filters (--category / --type / --min-size / --name / mtime)
// =========================================================================

#[test]
fn filter_category_narrows_largest_to_matching_entries() {
    let fixture = build_fixture();
    let stdout = run_duvis(
        fixture.path(),
        &["--ndjson", "--largest", "20", "--category", "cache,build"],
    );
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let entries: Vec<&serde_json::Value> = lines.iter().filter(|l| l["type"] == "entry").collect();
    assert!(!entries.is_empty(), "expected some matches");
    for e in &entries {
        let cat = e["category"].as_str().unwrap();
        assert!(
            cat == "cache" || cat == "build",
            "non-matching category leaked through: {cat}"
        );
    }
}

#[test]
fn filter_type_file_excludes_directories() {
    let fixture = build_fixture();
    let stdout = run_duvis(
        fixture.path(),
        &["--ndjson", "--largest", "20", "--type", "file"],
    );
    let entries: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|l: &serde_json::Value| l["type"] == "entry")
        .collect();
    assert!(!entries.is_empty(), "expected some files");
    for e in &entries {
        assert_eq!(e["is_dir"], false, "dir leaked through --type file");
    }
}

#[test]
fn filter_min_size_drops_small_entries() {
    let fixture = build_fixture();
    let stdout = run_duvis(
        fixture.path(),
        &["--ndjson", "--largest", "20", "--min-size", "50K"],
    );
    let entries: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|l: &serde_json::Value| l["type"] == "entry")
        .collect();
    assert!(!entries.is_empty());
    for e in &entries {
        let size = e["size"].as_u64().unwrap();
        assert!(size >= 50 * 1024, "entry under 50K leaked: {e}");
    }
}

#[test]
fn filter_name_glob_matches_only_log_files() {
    let fixture = build_fixture();
    let stdout = run_duvis(
        fixture.path(),
        &[
            "--ndjson",
            "--largest",
            "20",
            "--name",
            "*.log",
            "--type",
            "file",
        ],
    );
    let entries: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|l: &serde_json::Value| l["type"] == "entry")
        .collect();
    assert!(!entries.is_empty(), "expected at least the *.log fixture");
    for e in &entries {
        let name = e["name"].as_str().unwrap();
        assert!(name.ends_with(".log"), "non-log file leaked: {name}");
    }
}

#[test]
fn filter_invalid_glob_is_rejected_with_error() {
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--name", "[unclosed", "."])
        .assert()
        .failure();
}

#[test]
fn filter_invalid_size_is_rejected_with_error() {
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--min-size", "not-a-size", "."])
        .assert()
        .failure();
}

#[test]
fn filter_invalid_duration_is_rejected_with_error() {
    Command::cargo_bin("duvis")
        .unwrap()
        .args(["--newer-than", "7h", "."])
        .assert()
        .failure();
}
