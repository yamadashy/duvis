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
pub(crate) struct WireLargestRoot<'a> {
    pub(crate) meta: WireLargestMeta<'a>,
    pub(crate) largest: Vec<WireLargestEntry>,
}

/// Meta block for `--largest`: the shared `WireMeta` flattened in,
/// plus the two largest-specific counters. Flattening keeps the
/// JSON shape identical to the pre-refactor hand-built struct while
/// guaranteeing the base fields can't drift from `--json` / `--ndjson`.
#[derive(Debug, Serialize)]
pub(crate) struct WireLargestMeta<'a> {
    #[serde(flatten)]
    pub(crate) base: WireMeta<'a>,
    /// Surfaced so an agent can tell whether the list it sees was
    /// truncated by `--largest N` or genuinely smaller than N. Pair
    /// with `total_entries` to compute "we saw 10 of 1234".
    pub(crate) largest_requested: usize,
    pub(crate) total_entries: u64,
}

impl<'a> WireLargestMeta<'a> {
    pub(crate) fn from_config(
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
pub(crate) struct WireLargestEntry {
    pub(crate) name: String,
    pub(crate) relative_path: String,
    pub(crate) depth: u32,
    pub(crate) size: u64,
    pub(crate) size_human: String,
    pub(crate) is_dir: bool,
    pub(crate) category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) modified_days_ago: Option<u64>,
    pub(crate) file_count: u64,
    pub(crate) dir_count: u64,
}

/// NDJSON record for `--largest --ndjson`: meta line followed by one
/// entry per line. Schema-compatible with the tree-form NDJSON stream
/// (same `type` discriminator, same per-entry fields including the
/// `truncated_*` pair) so downstream parsers can share code — even
/// though `truncated_*` is always 0 in flat-list mode.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum WireLargestNdjsonRecord<'a> {
    Meta(WireLargestMeta<'a>),
    Entry(WireLargestNdjsonEntry<'a>),
}

#[derive(Debug, Serialize)]
pub(crate) struct WireLargestNdjsonEntry<'a> {
    pub(crate) name: &'a str,
    pub(crate) relative_path: &'a str,
    pub(crate) depth: u32,
    pub(crate) size: u64,
    pub(crate) size_human: String,
    pub(crate) is_dir: bool,
    pub(crate) category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) modified_days_ago: Option<u64>,
    pub(crate) file_count: u64,
    pub(crate) dir_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) truncated_count: u64,
    #[serde(skip_serializing_if = "is_zero_u64")]
    pub(crate) truncated_size: u64,
}
