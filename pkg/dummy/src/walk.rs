//! Filesystem boundary: walks a directory tree and folds every file into [`Counts`].
#![expect(
    clippy::disallowed_methods,
    reason = "boundary: directory walk and file reads"
)]

use std::ffi::OsStr;
use std::path::Path;

use ignore::{DirEntry, Walk, WalkBuilder};

use crate::counts::Counts;
use crate::error::Error;

/// Which files a scan visits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scope {
    /// Project files: dotfiles included, `.gitignore` and `.ignore` rules honored, `.git/`
    /// skipped.
    #[default]
    Project,
    /// Every file under the root, `.git/` and ignored files included.
    All,
}

/// Walks `root` and counts every regular file in `scope`. Symbolic links are not followed.
///
/// # Errors
///
/// Returns [`Error::Walk`] when the tree cannot be walked (for example, `root` does not
/// exist) and [`Error::Read`] when a file cannot be read. The scan stops at the first error.
pub fn scan(root: &Path, scope: Scope) -> Result<Counts, Error> {
    walk(root, scope).map(count_entry).sum()
}

fn walk(root: &Path, scope: Scope) -> Walk {
    let project = scope == Scope::Project;
    WalkBuilder::new(root)
        .standard_filters(project)
        .hidden(false)
        .require_git(false)
        .filter_entry(move |entry| !project || entry.file_name() != OsStr::new(".git"))
        .build()
}

fn count_entry(entry: Result<DirEntry, ignore::Error>) -> Result<Counts, Error> {
    let entry = entry?;
    if entry.file_type().is_some_and(|kind| kind.is_file()) {
        let bytes = std::fs::read(entry.path()).map_err(|source| Error::Read {
            path: entry.into_path(),
            source,
        })?;
        Ok(Counts::of_bytes(&bytes))
    } else {
        Ok(Counts::default())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Scope, scan};
    use crate::error::Error;
    use std::fs;
    use std::path::Path;

    fn write(root: &Path, relative: &str, content: &[u8]) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn counts_text_and_binary_files_in_nested_directories() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "a.txt", b"one\ntwo\n");
        write(dir.path(), "nested/deeper/b.md", "\u{e9}\n".as_bytes());
        write(dir.path(), "image.bin", &[0xff, 0xfe, 0x00]);

        let counts = scan(dir.path(), Scope::Project).unwrap();
        assert_eq!(
            (counts.files(), counts.text_files(), counts.binary_files()),
            (3, 2, 1)
        );
        assert_eq!((counts.lines(), counts.chars()), (3, 10));
    }

    #[test]
    fn project_scope_honors_gitignore_and_skips_dot_git() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), ".gitignore", b"ignored.txt\n");
        write(dir.path(), "ignored.txt", b"x\n");
        write(dir.path(), "kept.txt", b"y\n");
        write(dir.path(), ".git/HEAD", b"ref: refs/heads/main\n");

        assert_eq!(scan(dir.path(), Scope::Project).unwrap().files(), 2);
        assert_eq!(scan(dir.path(), Scope::All).unwrap().files(), 4);
    }

    #[test]
    fn missing_root_is_a_walk_error() {
        let dir = tempfile::tempdir().unwrap();
        let error = scan(&dir.path().join("missing"), Scope::Project).unwrap_err();
        assert!(matches!(error, Error::Walk(_)), "{error:?}");
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_file_is_a_read_error() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "secret.txt", b"x\n");
        let path = dir.path().join("secret.txt");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        if fs::read(&path).is_ok() {
            // Running with privileges that bypass file modes: nothing to observe.
            return;
        }

        let error = scan(dir.path(), Scope::Project).unwrap_err();
        assert!(
            matches!(error, Error::Read { path: ref failed, .. } if failed.ends_with("secret.txt"))
        );
    }
}
