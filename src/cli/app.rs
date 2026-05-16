use std::io::{self, Write};
#[cfg(feature = "ui")]
use std::path::PathBuf;

use anyhow::Result;

use crate::classify;
#[cfg(feature = "ui")]
use crate::entry::SortOrder;
use crate::render::{self, RenderConfig};
#[cfg(feature = "ui")]
use crate::scan::HardlinkPolicy;
use crate::scan::{self};
use crate::wire::explain::WireExplain;

use super::plan::{RunPlan, ScanPlan};

/// Dispatch the parsed plan. Each variant handler owns its own IO
/// (stdout lock, async runtime, etc.) so the dispatcher stays a single
/// match.
pub fn run(plan: RunPlan) -> Result<()> {
    match plan {
        RunPlan::ExplainCategory { name, json } => run_explain(&name, json),
        #[cfg(feature = "ui")]
        RunPlan::Ui {
            path,
            port,
            sort,
            reverse,
            hardlinks,
        } => run_ui(path, port, sort, reverse, hardlinks),
        RunPlan::Scan(scan) => run_scan(scan),
    }
}

fn run_explain(name: &str, json: bool) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    explain_category(name, json, &mut out)?;
    out.flush()?;
    Ok(())
}

#[cfg(feature = "ui")]
fn run_ui(
    path: PathBuf,
    port: u16,
    sort: SortOrder,
    reverse: bool,
    hardlinks: HardlinkPolicy,
) -> Result<()> {
    // The UI server runs the scan in a background task so the browser can
    // pop up immediately and show "Scanning..." while we wait.
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::ui::serve(path, port, sort, reverse, hardlinks))?;
    Ok(())
}

fn run_scan(plan: ScanPlan) -> Result<()> {
    let (mut tree, counts) = scan::scan(&plan.path, plan.hardlinks)?;
    tree.sort(&plan.sort, plan.reverse);

    let config = RenderConfig {
        max_depth: plan.max_depth,
        top: plan.top,
        scan_root: &plan.path,
        counts: &counts,
        hardlinks: plan.hardlinks,
        filter: &plan.filter,
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    render::write(&tree, &config, plan.mode, &mut out)?;
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
    let as_dir = classify::explain_dir(name);
    let as_file = classify::explain_file(name);
    if json {
        let payload = WireExplain::new(name, &as_dir, &as_file);
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
