//! The `agents-md` gate (ADR 4): deterministic probes on a repository's rules file.
//!
//! - [`audit`] reads the repository once ([`snapshot`](crate) boundary), then runs every
//!   probe as a pure function of that snapshot.
//! - The non-deterministic half of the gate is the `agents-md-review` skill shipped in
//!   `skills/`: it runs this binary first, then reviews the content.
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

mod probes;
mod snapshot;

use std::path::Path;

use vitals_utils::{Error, Report};

pub use probes::{CHARS, GATE, HANDOFF_WINDOW, LINES};

/// Runs every probe of the `agents-md` gate on the repository at `root`.
///
/// # Errors
///
/// Returns an [`Error`] when `root` is not a readable directory or a rules file exists but
/// cannot be read as UTF-8 text.
pub fn audit(root: &Path) -> Result<Report, Error> {
    snapshot::read(root).map(|snapshot| probes::check(&snapshot))
}
