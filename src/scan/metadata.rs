//! Per-file metadata reads: disk-usage attribution (with hardlink
//! dedup) and "days since modified". Kept as free functions so the
//! walk can call them without needing a metadata struct on hand.

use std::fs;
use std::time::SystemTime;

use super::walk::ScanCtx;
#[cfg(unix)]
use super::HardlinkPolicy;

/// Bytes a regular file actually occupies on disk. On Unix we use
/// `st_blocks * 512` (matching what `du` reports by default) so that sparse
/// files such as VM disk images (OrbStack `data.img.raw`, `Docker.raw`, ...)
/// don't appear hundreds of times bigger than they really are. APFS/NTFS
/// transparent compression is reflected for the same reason. Windows lacks
/// a portable per-file allocated-bytes API, so we fall back to apparent size.
///
/// When `ctx.hardlinks == CountOnce` we dedupe regular files with
/// `nlink > 1`: the first walker to claim a (dev, ino) reports the bytes,
/// later walkers see 0. Only regular files participate — hardlinks to
/// symlinks/FIFOs/sockets are rare and their footprint is negligible, so
/// counting them every time is simpler than risking surprising "0 B"
/// rows for those types.
///
/// Note: because the walk is parallel via rayon, *which* path among
/// several hardlinks ends up holding the bytes is not deterministic
/// across runs. Totals are stable; per-path attribution and per-category
/// attribution (when the same inode lives under directories of different
/// categories) can shuffle. Use `count-each` if you need every path to
/// report its raw size.
#[cfg(unix)]
pub(super) fn file_disk_usage(metadata: &fs::Metadata, ctx: &ScanCtx<'_>) -> u64 {
    use std::os::unix::fs::MetadataExt;
    let bytes = metadata.blocks() * 512;
    if ctx.hardlinks == HardlinkPolicy::CountEach {
        return bytes;
    }
    if !metadata.file_type().is_file() || metadata.nlink() <= 1 {
        return bytes;
    }
    let key = (metadata.dev(), metadata.ino());
    let mut seen = ctx.seen_inodes.lock().unwrap();
    if seen.insert(key) {
        bytes
    } else {
        0
    }
}

#[cfg(not(unix))]
pub(super) fn file_disk_usage(metadata: &fs::Metadata, _ctx: &ScanCtx<'_>) -> u64 {
    metadata.len()
}

pub(super) fn days_since_modified(metadata: &fs::Metadata) -> Option<u64> {
    let modified = metadata.modified().ok()?;
    let duration = SystemTime::now().duration_since(modified).ok()?;
    Some(duration.as_secs() / 86400)
}
