//! Counts the files, lines, and chars under a directory.
//!
//! The crate is a functional core with a single effectful boundary:
//!
//! - [`Counts`] is a pure monoid: per-file counts combine with `+` and [`Sum`](core::iter::Sum).
//! - [`scan`] is the only code that touches the filesystem; it walks a tree and folds every
//!   file into one [`Counts`].
//! - [`report`] renders an error chain for the terminal.
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

mod counts;
mod error;
mod walk;

pub use counts::Counts;
pub use error::{Error, report};
pub use walk::{Scope, scan};
