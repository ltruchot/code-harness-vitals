//! Command-line entry point: the only code that reads arguments and writes to the terminal.
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "boundary: terminal output"
)]

use std::path::PathBuf;
use std::process::ExitCode;

use agents_md_chkr::audit;
use clap::Parser;
use vitals_utils::{Outcome, report};

/// Run the AGENTS.md gate's deterministic probes on a repository.
///
/// Exit status: 0 when no probe fails, 1 when one does, 2 when the repository cannot be read.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Repository root.
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match audit(&cli.path) {
        Ok(gate) => {
            println!("{gate}");
            gate.outcome().into()
        }
        Err(error) => {
            eprintln!("agents-md-chkr: {}", report(&error));
            Outcome::Error.into()
        }
    }
}
