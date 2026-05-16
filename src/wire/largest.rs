//! Wire DTOs for `--largest N`. Both the JSON form
//! (`{meta, largest: [...]}`) and the NDJSON form (meta line +
//! one entry per line) live here.
//!
//! The meta block is a superset of [`super::tree::WireMeta`] —
//! `largest_requested` and `total_entries` are added so an agent can
//! tell "we saw 10 of 1234" without separate context.

use serde::Serialize;

use crate::classify::Category;
use crate::render::{is_zero_u64, RenderConfig};
use crate::wire::tree::WireMeta;

/// Top-level `--largest --json` envelope: `{meta, largest: [...]}`.
/// No `tree` field — `--largest` is intentionally a flat query, not a
/// hierarchical view.
#[derive(Debug, Serialize)]
pub struct WireLargestRoot<'a> {
    pub meta: WireLargestMeta<'a>,
    pub largest: Vec<WireLargestEntry>,
}

/// Meta block for `--largest`: the shared `WireMeta` flattened in,
/// plus the two largest-specific counters. Flattening keeps the
/// JSON shape identical to the pre-refactor hand-built struct while
/// guaranteeing the base fields can't drift from `--json` / `--ndjson`.
#[derive(Debug, Serialize)]
pub struct WireLargestMeta<'a> {
    #[serde(flatten)]
    pub base: WireMeta<'a>,
    /// Surfaced so an agent can tell whether the list it sees was
    /// truncated by `--largest N` or genuinely smaller than N. Pair
    /// with `total_entries` to compute "we saw 10 of 1234".
    pub largest_requested: usize,
    pub total_entries: u64,
}

impl<'a> WireLargestMeta<'a> {
    pub fn from_config(
        config: &'a RenderConfig<'a>,
        largest_requested: usize,
        total_entries: u64,
    ) -> Self {
        Self {
            base: WireMeta::from_config(config),
            largest_requested,
            total_entries,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WireLargestEntry {
    pub name: String,
    pub relative_path: String,
    pub depth: u32,
    pub size: u64,
    pub size_human: String,
    pub is_dir: bool,
    pub category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_days_ago: Option<u64>,
    pub file_count: u64,
    pub dir_count: u64,
}

/// NDJSON record for `--largest --ndjson`: meta line followed by one
/// entry per line. Schema-compatible with the tree-form NDJSON stream
/// (same `type` discriminator, same per-entry fields including the
/// `truncated_*` pair) so downstream parsers can share code — even
/// though `truncated_*` is always 0 in flat-list mode.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireLargestNdjsonRecord<'a> {
    Meta(WireLargestMeta<'a>),
    Entry(WireLargestNdjsonEntry<'a>),
}

#[derive(Debug, Serialize)]
pub struct WireLargestNdjsonEntry<'a> {
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
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub truncated_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub truncated_size: u64,
}
