use std::path::PathBuf;

use anyhow::Result;

use crate::entry::SortOrder;
use crate::filter::{Filter, FilterInputs};
use crate::render::largest::LargestFormat;
use crate::render::RenderMode;
use crate::scan::HardlinkPolicy;

use super::args::Cli;

/// What the binary should do once `Cli` has been parsed. Each variant
/// carries only the fields its dispatch needs — no further inspection
/// of the raw `Cli` happens past this conversion. Keeps `cli::app`
/// a single `match` and keeps the clap-derived struct out of scan /
/// render / ui paths.
pub enum RunPlan {
    /// `--explain-category <NAME>` short-circuit. Skips scanning.
    ExplainCategory { name: String, json: bool },
    /// `--ui` browser UI. Spins up an async runtime.
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

pub struct ScanPlan {
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
pub fn from_cli(cli: Cli) -> Result<RunPlan> {
    if let Some(name) = cli.explain_category {
        return Ok(RunPlan::ExplainCategory {
            name,
            json: cli.json,
        });
    }

    let path = cli.path.canonicalize().unwrap_or(cli.path.clone());

    if cli.ui {
        return Ok(RunPlan::Ui {
            path,
            port: cli.port,
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

    let mode = if let Some(n) = cli.largest {
        // --largest is a view, mutually exclusive with --summary and --ui
        // (clap enforces). Format follows the (orthogonal) format flag.
        let format = if cli.json {
            LargestFormat::Json
        } else if cli.ndjson {
            LargestFormat::Ndjson
        } else {
            LargestFormat::Text
        };
        RenderMode::Largest { n, format }
    } else if cli.json {
        RenderMode::Json
    } else if cli.ndjson {
        RenderMode::Ndjson
    } else if cli.summary {
        RenderMode::Summary
    } else {
        RenderMode::Tree
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
