use std::process::ExitCode;

use anyhow::Error;

/// Maps a post-parse runtime error into a process exit code. clap's
/// own parse-error path exits with code 2 before this is reached, so
/// this only sees scan / filter / IO failures.
///
/// Conservative for now — every post-parse failure maps to 1. Phase 1b
/// introduces typed errors that will let us split usage (2) vs runtime
/// (1) more finely.
pub(super) fn from_error(err: &Error) -> ExitCode {
    eprintln!("error: {err:#}");
    ExitCode::FAILURE
}
