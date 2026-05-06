use std::io::{self, Write};

use anyhow::Result;
use clap::{CommandFactory, Parser};

use duvis::cli::Cli;
use duvis::output::{self, OutputConfig};
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
    output::render(&tree, &config, cli.json, cli.analyze, &mut out)?;
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
