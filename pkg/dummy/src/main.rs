//! Command-line entry point: the only code that reads arguments and writes to the terminal.
#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "boundary: terminal output"
)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use dummy::{Scope, report, scan};

/// Count the files, lines, and chars under a directory.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Directory to scan.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Count every file: ignore rules off, `.git/` included.
    #[arg(long)]
    all: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let scope = if cli.all { Scope::All } else { Scope::Project };
    match scan(&cli.path, scope) {
        Ok(counts) => {
            println!("{counts}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("dummy: {}", report(&error));
            ExitCode::FAILURE
        }
    }
}
