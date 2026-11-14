//! End-to-end tests of the `dummy` binary.
#![cfg(test)]
#![expect(
    clippy::disallowed_methods,
    reason = "tests build fixture trees on disk"
)]

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::str::starts_with;

fn dummy() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("dummy"))
}

fn write(root: &Path, relative: &str, content: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn prints_totals_for_a_directory() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "a.txt", b"one\ntwo\n");
    write(dir.path(), "image.bin", &[0xff, 0x00]);

    dummy()
        .arg(dir.path())
        .assert()
        .success()
        .stdout("files: 2 (text: 1, binary: 1)\nlines: 2\nchars: 8\n");
}

#[test]
fn all_flag_includes_ignored_files_and_dot_git() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), ".gitignore", b"build/\n");
    write(dir.path(), "build/out.txt", b"generated\n");
    write(dir.path(), ".git/HEAD", b"ref: refs/heads/main\n");

    dummy()
        .arg(dir.path())
        .assert()
        .success()
        .stdout(starts_with("files: 1 "));
    dummy()
        .arg(dir.path())
        .arg("--all")
        .assert()
        .success()
        .stdout(starts_with("files: 3 "));
}

#[test]
fn defaults_to_the_current_directory() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "only.txt", b"x");

    dummy()
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout("files: 1 (text: 1, binary: 0)\nlines: 1\nchars: 1\n");
}

#[test]
fn missing_directory_fails_with_the_error_chain() {
    let dir = tempfile::tempdir().unwrap();

    dummy()
        .arg(dir.path().join("missing"))
        .assert()
        .failure()
        .stdout("")
        .stderr(starts_with("dummy: cannot walk the directory tree: "));
}
