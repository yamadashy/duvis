//! Wire DTOs for `--summary`. Both the JSON form
//! (`{meta, summary: [...]}`) and the NDJSON form (meta line + one
//! category per line) live here.
//!
//! The meta block is a superset of [`super::tree::WireMeta`] — `total`
//! and `total_human` are added because the summary view's total is the
//! sum of what survived the filter, which is not derivable from the
//! unfiltered scan counters already in `WireMeta`.

use serde::Serialize;

use crate::classify::Category;
use crate::render::RenderConfig;
use crate::wire::tree::WireMeta;

/// Top-level `--summary --json` envelope: `{meta, summary: [...]}`.
/// No `tree` field — the summary is a rollup, not a hierarchical view.
#[derive(Debug, Serialize)]
pub(crate) struct WireSummaryRoot<'a> {
    pub(crate) meta: WireSummaryMeta<'a>,
    pub(crate) summary: Vec<WireSummaryCategory>,
}

#[derive(Debug, Serialize)]
pub(crate) struct WireSummaryMeta<'a> {
    #[serde(flatten)]
    pub(crate) base: WireMeta<'a>,
    /// Sum of every bucket below. Reflects the filtered subtree, matching
    /// the `Total:` line of the text output.
    pub(crate) total: u64,
    pub(crate) total_human: String,
}

impl<'a> WireSummaryMeta<'a> {
    pub(crate) fn from_config(
        config: &'a RenderConfig<'a>,
        total: u64,
        total_human: String,
    ) -> Self {
        Self {
            base: WireMeta::from_config(config),
            total,
            total_human,
        }
    }
}

/// One category bucket. `percent` is integer-truncated against the same
/// total the text view prints, so the rendered numbers agree across
/// formats rather than each doing its own rounding.
#[derive(Debug, Serialize)]
pub(crate) struct WireSummaryCategory {
    pub(crate) category: Category,
    pub(crate) size: u64,
    pub(crate) size_human: String,
    pub(crate) percent: u64,
    pub(crate) count: u64,
}

/// NDJSON record for `--summary --ndjson`: meta line followed by one
/// category per line. Same `type` discriminator convention as the tree
/// and largest streams.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum WireSummaryNdjsonRecord<'a> {
    Meta(WireSummaryMeta<'a>),
    Category(WireSummaryCategory),
}
