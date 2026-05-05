use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;

use duvis::cli::{Cli, OutputFormat};
use duvis::output::{self, OutputConfig};
use duvis::scanner;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let path = cli.path.canonicalize().unwrap_or(cli.path.clone());

    if cli.format == OutputFormat::Ui {
        // The UI server runs the scan in a background task so the browser can
        // pop up immediately and show "Scanning..." while we wait.
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(duvis::ui::serve(path, cli.port, cli.sort, cli.reverse))?;
        return Ok(());
    }

    let (mut tree, counts) = scanner::scan(&path)?;
    tree.sort(&cli.sort, cli.reverse);
    let config = OutputConfig {
        depth: cli.depth,
        top: cli.top,
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    output::render(&tree, &config, cli.format, &mut out)?;
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
