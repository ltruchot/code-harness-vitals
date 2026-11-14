//! Building blocks shared by every code-harness-vitals gate (ADR 4).
//!
//! Only code that two or more gates need lives here. Everything is pure except [`fs`],
//! the filesystem boundary.
//!
//! - [`Finding`], [`Verdict`], [`Report`], [`Outcome`]: what probes conclude and how a gate
//!   binary reports it and exits.
//! - [`TextSize`], [`Budget`]: line and char budgets for instruction files and skills.
//! - [`markers`]: the paths that agents read, shared across gates.
//! - [`fs`]: optional file reads, symlink targets, the project walk, and the home directory.
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

mod error;
mod finding;
pub mod fs;
pub mod markers;
mod text;

pub use error::{Error, report};
pub use finding::{Finding, Outcome, Report, Scope, Verdict};
pub use text::{Budget, TextSize};
