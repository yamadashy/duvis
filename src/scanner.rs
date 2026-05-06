use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::category::{self, Category};
use crate::entry::Entry;

/// Above this depth, fall back to sequential to avoid task-spawn overhead
/// dominating tiny leaf directories. Empirically, raising it from 3 to a
/// large value cut warm-cache scan time on a deeply-nested ghq tree
/// (lots of node_modules) roughly in half — a directory walk is I/O-bound,
/// so issuing more concurrent metadata calls lets the kernel overlap them.
/// Shallow / wide trees (e.g. ~/Library) see negligible change.
const PARALLEL_DEPTH: usize = 16;

/// Live counters shared between the scanner and any observer. `items_scanned`
/// drives the "12,345 items…" progress UI; `items_skipped` reports filesystem
/// entries we could not read (permission denied, races against deletion, …)
/// so the user can tell that the reported total is incomplete.
#[derive(Default, Debug)]
pub struct ScanCounts {
    pub items_scanned: AtomicU64,
    pub items_skipped: AtomicU64,
}

impl ScanCounts {
    pub fn scanned(&self) -> u64 {
        self.items_scanned.load(Ordering::Relaxed)
    }

    pub fn skipped(&self) -> u64 {
        self.items_skipped.load(Ordering::Relaxed)
    }
}

/// Scan `path` to completion, returning the tree and the final counters
/// (visited / skipped). Callers that need live progress should use
/// [`scan_with_progress`] and observe a shared [`ScanCounts`] from another
/// thread.
pub fn scan(path: &Path) -> Result<(Entry, ScanCounts)> {
    let counts = ScanCounts::default();
    let entry = scan_recursive(path, 0, Category::Other, &counts)?;
    Ok((entry, counts))
}

/// Same as [`scan`] but writes progress into `counts` so a UI thread can poll
/// the running totals while the scan is in flight.
pub fn scan_with_progress(path: &Path, counts: &ScanCounts) -> Result<Entry> {
    scan_recursive(path, 0, Category::Other, counts)
}

/// Walk one entry. The outermost ancestor with an explicit (non-Other) category
/// owns its whole subtree, so we pass that category down via `inherited`.
fn scan_recursive(
    path: &Path,
    depth: usize,
    inherited: Category,
    counts: &ScanCounts,
) -> Result<Entry> {
    counts.items_scanned.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let metadata = fs::symlink_metadata(path)?;
    let modified_days_ago = days_since_modified(&metadata);

    if metadata.is_dir() {
        let own = category::classify_dir(&name);
        let effective = if inherited != Category::Other {
            inherited
        } else {
            own
        };
        let children = scan_children(path, depth, effective, counts);
        Ok(Entry::dir(name, effective, modified_days_ago, children))
    } else {
        let effective = if inherited != Category::Other {
            inherited
        } else {
            category::classify_file(&name)
        };
        Ok(Entry::file(
            name,
            file_disk_usage(&metadata),
            effective,
            modified_days_ago,
        ))
    }
}

/// Bytes a regular file actually occupies on disk. On Unix we use
/// `st_blocks * 512` (matching what `du` reports by default) so that sparse
/// files such as VM disk images (OrbStack `data.img.raw`, `Docker.raw`, ...)
/// don't appear hundreds of times bigger than they really are. APFS/NTFS
/// transparent compression is reflected for the same reason. Windows lacks
/// a portable per-file allocated-bytes API, so we fall back to apparent size.
#[cfg(unix)]
fn file_disk_usage(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.blocks() * 512
}

#[cfg(not(unix))]
fn file_disk_usage(metadata: &fs::Metadata) -> u64 {
    metadata.len()
}

fn scan_children(
    path: &Path,
    depth: usize,
    inherited: Category,
    counts: &ScanCounts,
) -> Vec<Entry> {
    let entries: Vec<_> = match fs::read_dir(path) {
        Ok(rd) => rd
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(_) => {
                    counts.items_skipped.fetch_add(1, Ordering::Relaxed);
                    None
                }
            })
            .collect(),
        Err(_) => {
            // Couldn't open the directory at all (permission denied, vanished,
            // …). Count one skipped entry and bail; the parent dir size will
            // simply be the sum of whatever we managed to read elsewhere.
            counts.items_skipped.fetch_add(1, Ordering::Relaxed);
            return Vec::new();
        }
    };

    if depth < PARALLEL_DEPTH {
        entries
            .par_iter()
            .filter_map(
                |entry| match scan_recursive(&entry.path(), depth + 1, inherited, counts) {
                    Ok(e) => Some(e),
                    Err(_) => {
                        counts.items_skipped.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                },
            )
            .collect()
    } else {
        entries
            .iter()
            .filter_map(
                |entry| match scan_recursive(&entry.path(), depth + 1, inherited, counts) {
                    Ok(e) => Some(e),
                    Err(_) => {
                        counts.items_skipped.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                },
            )
            .collect()
    }
}

fn days_since_modified(metadata: &fs::Metadata) -> Option<u64> {
    let modified = metadata.modified().ok()?;
    let duration = SystemTime::now().duration_since(modified).ok()?;
    Some(duration.as_secs() / 86400)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_tmp(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "duvis_test_{}_{}_{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    fn find<'a>(entry: &'a Entry, name: &str) -> Option<&'a Entry> {
        entry.children()?.iter().find(|c| c.name == name)
    }

    #[test]
    fn scan_walks_a_real_directory() {
        let tmp = unique_tmp("walk");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("sub")).unwrap();
        fs::write(tmp.join("a.txt"), b"hello").unwrap();
        fs::write(tmp.join("sub/b.txt"), b"world!").unwrap();

        let (entry, _counts) = scan(&tmp).unwrap();
        assert!(entry.is_dir());
        // Sizes use disk allocation now (st_blocks * 512 on Unix), so the
        // exact byte count depends on the filesystem block size. We just
        // assert the size is at least the apparent total.
        assert!(entry.size >= 5 + 6);

        let children = entry.children().unwrap();
        assert_eq!(children.len(), 2);

        fs::remove_dir_all(&tmp).unwrap();
    }

    /// Files inside a Cache directory inherit Cache, regardless of their own
    /// extension-based classification.
    #[test]
    fn category_propagates_into_node_modules() {
        let tmp = unique_tmp("nm");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("node_modules/some-pkg")).unwrap();
        fs::write(tmp.join("node_modules/some-pkg/preview.png"), b"x").unwrap();
        fs::write(tmp.join("node_modules/some-pkg/debug.log"), b"x").unwrap();

        let (root, _counts) = scan(&tmp).unwrap();
        let nm = find(&root, "node_modules").unwrap();
        assert_eq!(nm.category, Category::Cache);
        let pkg = find(nm, "some-pkg").unwrap();
        assert_eq!(pkg.category, Category::Cache);
        // .png would normally be Media, but inherits Cache from node_modules.
        let png = find(pkg, "preview.png").unwrap();
        assert_eq!(png.category, Category::Cache);
        // .log would normally be Log, but inherits Cache.
        let log = find(pkg, "debug.log").unwrap();
        assert_eq!(log.category, Category::Cache);

        fs::remove_dir_all(&tmp).unwrap();
    }

    /// The outermost explicit-category ancestor wins; nested classifications
    /// are ignored once we are already inside one.
    #[test]
    fn outermost_ancestor_wins() {
        let tmp = unique_tmp("nest");
        let _ = fs::remove_dir_all(&tmp);
        // node_modules (Cache) > target (would be Build, but stays Cache)
        fs::create_dir_all(tmp.join("node_modules/inner/target")).unwrap();
        fs::write(tmp.join("node_modules/inner/target/main.o"), b"x").unwrap();

        let (root, _counts) = scan(&tmp).unwrap();
        let nm = find(&root, "node_modules").unwrap();
        let inner = find(nm, "inner").unwrap();
        let target = find(inner, "target").unwrap();
        assert_eq!(target.category, Category::Cache);
        assert_eq!(find(target, "main.o").unwrap().category, Category::Cache);

        fs::remove_dir_all(&tmp).unwrap();
    }

    /// Without an explicit-category ancestor, leaves keep their own
    /// classification.
    #[test]
    fn leaves_keep_own_category_outside_named_dirs() {
        let tmp = unique_tmp("free");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("Movies")).unwrap();
        fs::write(tmp.join("Movies/clip.mp4"), b"x").unwrap();
        fs::write(tmp.join("Movies/notes.txt"), b"x").unwrap();

        let (root, _counts) = scan(&tmp).unwrap();
        let movies = find(&root, "Movies").unwrap();
        // The dir name "Movies" is not in classify_dir, so dir is Other.
        assert_eq!(movies.category, Category::Other);
        // Inside Other, the file's own classification stands.
        assert_eq!(find(movies, "clip.mp4").unwrap().category, Category::Media);
        assert_eq!(find(movies, "notes.txt").unwrap().category, Category::Other);

        fs::remove_dir_all(&tmp).unwrap();
    }

    /// A clean scan reports zero skipped entries and a non-zero scanned count.
    #[test]
    fn counts_are_populated_on_clean_scan() {
        let tmp = unique_tmp("counts");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("a")).unwrap();
        fs::write(tmp.join("a/x.txt"), b"x").unwrap();
        fs::write(tmp.join("y.txt"), b"y").unwrap();

        let (_, counts) = scan(&tmp).unwrap();
        // root + "a" + "x.txt" + "y.txt" = 4 visited.
        assert_eq!(counts.scanned(), 4);
        assert_eq!(counts.skipped(), 0);

        fs::remove_dir_all(&tmp).unwrap();
    }

    /// On Unix we can mode 000 a directory and confirm scan_children records a
    /// skip instead of pretending the directory is empty. Skipped on Windows
    /// where chmod semantics differ.
    #[cfg(unix)]
    #[test]
    fn unreadable_directory_is_counted_as_skipped() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = unique_tmp("noperm");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("locked")).unwrap();
        fs::write(tmp.join("locked/secret"), b"x").unwrap();
        fs::write(tmp.join("readable"), b"y").unwrap();

        // Strip read+execute on the inner dir so read_dir refuses.
        let mut perms = fs::metadata(tmp.join("locked")).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(tmp.join("locked"), perms).unwrap();

        let (_, counts) = scan(&tmp).unwrap();
        // We tried to walk into "locked" → +1 skip when read_dir failed.
        assert!(
            counts.skipped() >= 1,
            "expected ≥1 skip, got {}",
            counts.skipped(),
        );

        // Restore perms so the cleanup can proceed.
        let mut perms = fs::metadata(tmp.join("locked")).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(tmp.join("locked"), perms).unwrap();
        fs::remove_dir_all(&tmp).unwrap();
    }
}
