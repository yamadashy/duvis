use std::io::{self, Write};

use anyhow::Result;
use clap::{CommandFactory, Parser};

use duvis::cli::Cli;
use duvis::output::filter::{Filter, FilterInputs};
use duvis::output::largest::LargestFormat;
use duvis::output::{self, OutputConfig, OutputMode};
use duvis::scanner;

fn main() -> Result<()> {
    // Restore SIGPIPE's default disposition so `duvis ... | head` exits
    // silently (process killed by SIGPIPE) instead of surfacing
    // `Error: Broken pipe (os error 32)`. Rust runtime ignores SIGPIPE by
    // default, which is the wrong behavior for a Unix CLI that streams to
    // stdout. Same approach as ripgrep, fd, etc.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

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

    let path = cli.path.canonicalize().unwrap_or(cli.path.clone());

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

    let filter = Filter::from_inputs(FilterInputs {
        categories: cli.category.clone(),
        type_: cli.r#type,
        min_size: cli.min_size.clone(),
        names: cli.name.clone(),
        newer_than: cli.newer_than.clone(),
        older_than: cli.older_than.clone(),
    })?;

    let config = OutputConfig {
        depth: cli.depth,
        top: cli.top,
        scan_root: &path,
        counts: &counts,
        hardlinks: cli.hardlinks,
        filter: &filter,
    };
    let mode = if let Some(n) = cli.largest {
        // --largest is a view, mutually exclusive with --analyze and --ui
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
    } else if cli.analyze {
        OutputMode::Analyze
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
