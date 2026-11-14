//! Filesystem boundary: the only reads gates make, with "absent" kept apart from "unreadable".
#![expect(
    clippy::disallowed_methods,
    reason = "boundary: filesystem reads for every gate"
)]

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::Error;

/// Fails unless `path` is a directory.
///
/// # Errors
///
/// [`Error::Read`] when `path` cannot be inspected (for example, it does not exist), and
/// [`Error::NotADirectory`] when it is a file.
pub fn require_dir(path: &Path) -> Result<(), Error> {
    let metadata = std::fs::metadata(path).map_err(|source| read_error(path, source))?;
    if metadata.is_dir() {
        Ok(())
    } else {
        Err(Error::NotADirectory {
            path: path.to_path_buf(),
        })
    }
}

/// The UTF-8 content of `path`, or `None` when nothing is there. A dangling symbolic link
/// counts as nothing.
///
/// # Errors
///
/// [`Error::Read`] when the path exists but cannot be read as UTF-8 text.
pub fn read_optional(path: &Path) -> Result<Option<String>, Error> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(None),
        Err(source) => Err(read_error(path, source)),
    }
}

/// The target of `path` when it is a symbolic link, as written in the link; `None` when it
/// is not a link or does not exist.
///
/// # Errors
///
/// [`Error::Read`] when `path` cannot be inspected.
pub fn link_target(path: &Path) -> Result<Option<PathBuf>, Error> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => std::fs::read_link(path)
            .map(Some)
            .map_err(|source| read_error(path, source)),
        Ok(_) => Ok(None),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(None),
        Err(source) => Err(read_error(path, source)),
    }
}

fn read_error(path: &Path, source: std::io::Error) -> Error {
    Error::Read {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{link_target, read_optional, require_dir};
    use crate::error::Error;
    use std::fs;

    #[test]
    fn require_dir_accepts_a_directory_only() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("file");
        fs::write(&file, "x").unwrap();
        assert!(require_dir(dir.path()).is_ok());
        assert!(matches!(
            require_dir(&file),
            Err(Error::NotADirectory { .. })
        ));
        assert!(matches!(
            require_dir(&dir.path().join("missing")),
            Err(Error::Read { .. })
        ));
    }

    #[test]
    fn read_optional_tells_absent_from_unreadable() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("AGENTS.md");
        fs::write(&file, "# Rules\n").unwrap();
        assert_eq!(read_optional(&file).unwrap().as_deref(), Some("# Rules\n"));
        assert_eq!(read_optional(&dir.path().join("missing")).unwrap(), None);
        assert!(matches!(read_optional(dir.path()), Err(Error::Read { .. })));
    }

    #[test]
    fn link_target_reads_links_only() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("AGENTS.md");
        fs::write(&file, "x").unwrap();
        assert_eq!(link_target(&file).unwrap(), None);
        assert_eq!(link_target(&dir.path().join("missing")).unwrap(), None);
        // A path through a regular file cannot be inspected at all.
        assert!(matches!(
            link_target(&file.join("child")),
            Err(Error::Read { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn link_target_returns_the_target_as_written() {
        let dir = tempfile::tempdir().unwrap();
        let link = dir.path().join("CLAUDE.md");
        std::os::unix::fs::symlink("AGENTS.md", &link).unwrap();
        assert_eq!(
            link_target(&link).unwrap(),
            Some(std::path::PathBuf::from("AGENTS.md"))
        );
        // Dangling: the target does not exist, so there is no content.
        assert_eq!(read_optional(&link).unwrap(), None);
    }
}
