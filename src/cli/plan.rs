use std::path::PathBuf;

use anyhow::Result;

use crate::entry::SortOrder;
use crate::filter::{Filter, FilterInputs};
use crate::render::{OutputFormat, RenderMode};
use crate::scan::HardlinkPolicy;

use super::args::Cli;

/// Port the browser UI binds to when `--port` isn't given. Falls back to
/// a free OS-assigned port if this one is busy.
#[cfg(feature = "ui")]
const DEFAULT_UI_PORT: u16 = 7515;

/// What the binary should do once `Cli` has been parsed. Each variant
/// carries only the fields its dispatch needs — no further inspection
/// of the raw `Cli` happens past this conversion. Keeps `cli::app`
/// a single `match` and keeps the clap-derived struct out of scan /
/// render / ui paths.
pub(super) enum RunPlan {
    /// `--explain-category <NAME>` short-circuit. Skips scanning.
    ExplainCategory { name: String, json: bool },
    /// `--ui` browser UI. Spins up an async runtime. Only present
    /// when the `ui` feature is enabled — without it, `--ui` / `--port`
    /// aren't parsed in the first place.
    #[cfg(feature = "ui")]
    Ui {
        path: PathBuf,
        port: u16,
        sort: SortOrder,
        reverse: bool,
        hardlinks: HardlinkPolicy,
    },
    /// Default path: walk + render to one of the textual / structured
    /// outputs.
    Scan(ScanPlan),
}

pub(super) struct ScanPlan {
    pub path: PathBuf,
    pub sort: SortOrder,
    pub reverse: bool,
    pub hardlinks: HardlinkPolicy,
    pub max_depth: Option<usize>,
    pub top: Option<usize>,
    pub mode: RenderMode,
    pub filter: Filter,
}

/// Convert a parsed `Cli` into the dispatch-ready `RunPlan`. Filter
/// inputs are parsed here (not lazily) so a typo in `--min-size` or
/// `--changed-within` fails in milliseconds rather than after a
/// multi-minute walk of a huge tree.
pub(super) fn from_cli(cli: Cli) -> Result<RunPlan> {
    if let Some(name) = cli.explain_category {
        return Ok(RunPlan::ExplainCategory {
            name,
            json: cli.json,
        });
    }

    // clap owns the "exactly one of these" rule via the `format` group,
    // so the first match wins without any further exclusivity checking.
    let format = if cli.json {
        OutputFormat::Json
    } else if cli.toon {
        OutputFormat::Toon
    } else if cli.ndjson {
        OutputFormat::Ndjson
    } else {
        OutputFormat::Text
    };

    // PATH carries no clap default so `--explain-category` can reject an
    // explicit one; the default lands here instead.
    let path = cli.path.unwrap_or_else(|| PathBuf::from("."));
    let path = path.canonicalize().unwrap_or(path);

    #[cfg(feature = "ui")]
    if cli.ui {
        return Ok(RunPlan::Ui {
            path,
            // Default lives here rather than in clap so `--port` stays
            // absent-unless-typed, which is what `requires = "ui"` needs.
            port: cli.port.unwrap_or(DEFAULT_UI_PORT),
            sort: cli.sort,
            reverse: cli.reverse,
            hardlinks: cli.hardlinks,
        });
    }

    let filter = Filter::from_inputs(FilterInputs {
        categories: cli.category,
        type_: cli.r#type,
        min_size: cli.min_size,
        names: cli.name,
        changed_within: cli.changed_within,
        changed_before: cli.changed_before,
    })?;

    // The view axis. `--ui` already returned above; the remaining three
    // are mutually exclusive via the `view` group, so this is a straight
    // pick rather than a precedence chain.
    let mode = if let Some(n) = cli.largest {
        RenderMode::Largest { n, format }
    } else if cli.summary {
        RenderMode::Summary { format }
    } else {
        RenderMode::Tree { format }
    };

    Ok(RunPlan::Scan(ScanPlan {
        path,
        sort: cli.sort,
        reverse: cli.reverse,
        hardlinks: cli.hardlinks,
        max_depth: cli.max_depth,
        top: cli.top,
        mode,
        filter,
    }))
}
