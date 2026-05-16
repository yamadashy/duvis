// Facade for the cli/ layer. clap is confined to this folder — `args`
// holds the Parser-derived struct, `signals` resets SIGPIPE before any
// stdout writes, `help` carries the hand-formatted --help text.

mod args;
mod help;
pub mod signals;

pub use args::Cli;
