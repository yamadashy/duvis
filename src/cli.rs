// Facade for the cli/ layer. clap is confined to this folder.
//
// Pipeline:
//   main → cli::run() → parse Cli → plan::from_cli → app::run → exit code
//
// Each child has a narrow role:
//   args     — clap-derived Cli struct
//   help     — hand-formatted HELP_TEXT
//   plan     — Cli → RunPlan normalization
//   app      — RunPlan dispatch (scan / ui / explain)
//   signals  — SIGPIPE reset
//   exit     — error → ExitCode mapping

mod app;
mod args;
mod exit;
mod help;
mod plan;
pub mod signals;

pub use args::Cli;

use clap::{CommandFactory, Parser};
use std::process::ExitCode;

/// Binary entry. Owns the full CLI pipeline and returns the process
/// exit code. `main.rs` should call this directly (via `duvis::run_cli`)
/// so the lib crate owns every CLI concern.
pub fn run() -> ExitCode {
    signals::reset_sigpipe();

    // No arguments at all → show help instead of silently scanning the
    // current directory. Once the user passes any flag (e.g. `duvis --ui`)
    // we keep `.` as the default PATH, so power-user flows aren't gated
    // behind typing `.` every time.
    if std::env::args_os().len() == 1 {
        if let Err(e) = print_help() {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    // clap handles its own --help / --version / parse-error exits before
    // returning. Anything past this point is post-parse work.
    let cli = args::Cli::parse();

    let plan = match plan::from_cli(cli) {
        Ok(p) => p,
        Err(e) => return exit::from_error(&e),
    };
    match app::run(plan) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => exit::from_error(&e),
    }
}

fn print_help() -> std::io::Result<()> {
    args::Cli::command().print_help()?;
    println!();
    Ok(())
}
