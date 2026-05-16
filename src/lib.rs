pub mod category;
pub mod cli;
pub mod entry;
pub mod output;
pub mod scanner;
pub mod ui;

/// Binary entry. `main.rs` calls this and returns the resulting exit
/// code. All CLI / dispatch / exit-code work lives inside the lib crate
/// so that downstream code (and integration tests) can drive the full
/// pipeline without going through a subprocess.
pub fn run_cli() -> std::process::ExitCode {
    cli::run()
}
