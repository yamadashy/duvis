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
