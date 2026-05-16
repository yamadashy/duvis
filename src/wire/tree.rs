//! Wire DTOs for the tree-form structured outputs (`--json` and
//! `--ndjson`). Both share the same per-entry fields and the same
//! `meta` block — the only real difference is hierarchical (`children`
//! nested) vs. flat (one record per line, parent before children).
//!
//! Construction logic lives in `render/json.rs` and `render/ndjson.rs`;
//! this module owns the on-wire shape only, so the JSON contract is
//! defined in exactly one place per format.

use serde::Serialize;

use crate::classify::Category;
use crate::render::{hardlinks_label, is_zero_u64, scan_root_for_wire, RenderConfig, WIRE_VERSION};

/// Common `meta` block emitted by both `--json` (as `meta`) and
/// `--ndjson` (as the first record, tagged `type: "meta"`). Identical
/// fields, identical contents — built from a shared constructor so the
/// two outputs can't drift.
#[derive(Debug, Serialize)]
pub struct WireMeta<'a> {
    pub wire_version: u32,
    pub duvis_version: &'static str,
    pub scan_root: String,
    pub hardlinks: &'a str,
    pub items_scanned: u64,
    pub items_skipped: u64,
}

impl<'a> WireMeta<'a> {
    /// Build the meta block from the active output config. Centralised
    /// so a future field addition (or a `meta.scan_root` formatting
    /// tweak) touches one place.
    pub fn from_config(config: &'a RenderConfig<'a>) -> Self {
        Self {
            wire_version: WIRE_VERSION,
            duvis_version: env!("CARGO_PKG_VERSION"),
            scan_root: scan_root_for_wire(config.scan_root),
            hardlinks: hardlinks_label(config.hardlinks),
            items_scanned: config.counts.scanned(),
            items_skipped: config.counts.skipped(),
        }
    }
}

/// Top-level `--json` envelope: `{meta, tree}`.
#[derive(Debug, Serialize)]
pub struct WireTreeRoot<'a> {
    pub meta: WireMeta<'a>,
    pub tree: WireTreeNode,
}

/// One node in the `--json` tree. Recursive: directories carry their
/// rendered children inline (subject to `--max-depth` / `--top`
/// clipping).
#[derive(Debug, Serialize)]
pub struct WireTreeNode {
    pub name: String,
    /// Path from the scan root, `/`-separated. Root itself is `"."`.
    pub relative_path: String,
    /// 0 at scan root, +1 per directory level.
    pub depth: u32,
    pub size: u64,
    pub size_human: String,
    pub is_dir: bool,
    pub category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_days_ago: Option<u64>,
    /// Total regular files in this subtree (recursive). Constant for
    /// a given Entry — does *not* change with `--top` / `--max-depth`
    /// since those only affect what we emit, not what was measured.
    pub file_count: u64,
    /// Total directories in this subtree, excluding self.
    pub dir_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<WireTreeNode>>,
    /// Direct children dropped at this level by `--top`. Zero
    /// (omitted) when no top filter applied. Distinct from
    /// depth-truncation: that shows up as a non-zero
    /// `file_count`/`dir_count` with no `children` array, and is
    /// implicit.
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub truncated_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub truncated_size: u64,
}

/// NDJSON record. The first line of an `--ndjson` stream is a `Meta`
/// record; every subsequent line is an `Entry`. A `type` discriminator
/// lets consumers route lines without out-of-band parsing.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireStreamRecord<'a> {
    Meta(WireMeta<'a>),
    Entry(WireStreamEntry<'a>),
}

/// One NDJSON entry line. Same per-entry fields as [`WireTreeNode`]
/// minus `children` (NDJSON is intentionally flat). Borrows both
/// `name` and `relative_path` so the hot recursion path doesn't
/// allocate per line — matches the same lifetime story used by
/// [`super::largest::WireLargestNdjsonEntry`].
#[derive(Debug, Serialize)]
pub struct WireStreamEntry<'a> {
    pub name: &'a str,
    pub relative_path: &'a str,
    pub depth: u32,
    pub size: u64,
    pub size_human: String,
    pub is_dir: bool,
    pub category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_days_ago: Option<u64>,
    pub file_count: u64,
    pub dir_count: u64,
    /// Direct children dropped at this level by `--top`. Zero
    /// (omitted) when no top filter applied.
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub truncated_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub truncated_size: u64,
}
