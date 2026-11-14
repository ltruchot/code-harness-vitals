//! Errors raised at the filesystem boundary, and their rendering.

use std::path::PathBuf;

/// Why a scan stopped.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The directory tree could not be walked (missing root, unreadable directory, …).
    #[error("cannot walk the directory tree")]
    Walk(#[from] ignore::Error),
    /// A file was listed but its content could not be read.
    #[error("cannot read {}", path.display())]
    Read {
        /// The file that failed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Renders an error and every source below it on one line, outermost first.
///
/// # Examples
///
/// ```
/// let missing = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
/// let error = dummy::Error::Read { path: "a.txt".into(), source: missing };
/// assert_eq!(dummy::report(&error), "cannot read a.txt: no such file");
/// ```
#[must_use]
pub fn report(error: &dyn std::error::Error) -> String {
    core::iter::successors(Some(error), |current| current.source())
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ")
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Error, report};
    use std::io;

    #[test]
    fn report_walks_the_whole_source_chain() {
        let error = Error::Read {
            path: "dir/file.bin".into(),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"),
        };
        assert_eq!(
            report(&error),
            "cannot read dir/file.bin: permission denied"
        );
    }

    #[test]
    fn report_of_a_leaf_error_is_its_message() {
        let error = io::Error::other("leaf");
        assert_eq!(report(&error), "leaf");
    }
}
