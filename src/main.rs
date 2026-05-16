use std::io::{self, Write};

use anyhow::Result;
use clap::{CommandFactory, Parser};

use duvis::category;
use duvis::cli::signals;
use duvis::cli::Cli;
use duvis::output::filter::{Filter, FilterInputs};
use duvis::output::largest::LargestFormat;
use duvis::output::{self, OutputConfig, OutputMode};
use duvis::scanner;

fn main() -> Result<()> {
    signals::reset_sigpipe();

    // No arguments at all → show help instead of silently scanning the
    // current directory. Once the user passes any flag (e.g. `duvis --ui`)
    // we keep `.` as the default PATH, so power-user flows aren't gated
    // behind typing `.` every time.
    if std::env::args_os().len() == 1 {
        Cli::command().print_help()?;
        println!();
        return Ok(());
    }

    let cli = Cli::parse();

    // Diagnostic mode: print classifier reasoning for a single name and
    // exit. Skips scanning entirely so it stays a millisecond-scale lookup.
    if let Some(name) = &cli.explain_category {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        explain_category(name, cli.json, &mut out)?;
        out.flush()?;
        return Ok(());
    }

    let path = cli.path.canonicalize().unwrap_or(cli.path.clone());

    // Parse filter inputs *before* scanning. A typo in --min-size or
    // --changed-within should fail in milliseconds, not after a multi-minute
    // walk of a huge tree. Also runs before scanner::scan's path-existence
    // check so the user sees the most actionable error first.
    let filter = Filter::from_inputs(FilterInputs {
        categories: cli.category.clone(),
        type_: cli.r#type,
        min_size: cli.min_size.clone(),
        names: cli.name.clone(),
        changed_within: cli.changed_within.clone(),
        changed_before: cli.changed_before.clone(),
    })?;

    if cli.ui {
        // The UI server runs the scan in a background task so the browser can
        // pop up immediately and show "Scanning..." while we wait.
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(duvis::ui::serve(
            path,
            cli.port,
            cli.sort,
            cli.reverse,
            cli.hardlinks,
        ))?;
        return Ok(());
    }

    let (mut tree, counts) = scanner::scan(&path, cli.hardlinks)?;
    tree.sort(&cli.sort, cli.reverse);

    let config = OutputConfig {
        max_depth: cli.max_depth,
        top: cli.top,
        scan_root: &path,
        counts: &counts,
        hardlinks: cli.hardlinks,
        filter: &filter,
    };
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
        OutputMode::Largest { n, format }
    } else if cli.json {
        OutputMode::Json
    } else if cli.ndjson {
        OutputMode::Ndjson
    } else if cli.summary {
        OutputMode::Summary
    } else {
        OutputMode::Tree
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    output::render(&tree, &config, mode, &mut out)?;
    out.flush()?;

    let skipped = counts.skipped();
    if skipped > 0 {
        let plural = if skipped == 1 { "" } else { "s" };
        eprintln!(
            "warning: {skipped} path{plural} could not be read \
             (permission denied or path vanished); reported total may be incomplete"
        );
    }
    Ok(())
}

/// Render `--explain-category <NAME>`. We classify the name both as a
/// directory and as a file (the same string can match different rules in
/// each role — e.g. `node_modules` is `cache` as a dir but `other` as a
/// file) and surface both so callers don't have to guess which role to
/// pass.
fn explain_category(name: &str, json: bool, out: &mut impl Write) -> Result<()> {
    let as_dir = category::explain_dir(name);
    let as_file = category::explain_file(name);
    if json {
        let payload = serde_json::json!({
            "name": name,
            "as_directory": as_dir,
            "as_file": as_file,
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&payload)?)?;
    } else {
        writeln!(out, "{name:?}")?;
        writeln!(
            out,
            "  as directory: {:<12} ({})",
            as_dir.category.label(),
            as_dir.reason.describe()
        )?;
        writeln!(
            out,
            "  as file:      {:<12} ({})",
            as_file.category.label(),
            as_file.reason.describe()
        )?;
    }
    Ok(())
}
