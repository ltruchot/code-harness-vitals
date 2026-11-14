//! End-to-end tests of the `agents-md-chkr` binary.
#![cfg(test)]
#![expect(
    clippy::disallowed_methods,
    reason = "tests build fixture trees on disk"
)]

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::str::contains;

fn chkr() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("agents-md-chkr"))
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn healthy(root: &Path) {
    write(
        root,
        "AGENTS.md",
        "> Session handoff: read `TODO.md` first.\n\n# Rules\n",
    );
    write(root, "TODO.md", "# TODO\n");
    write(
        root,
        ".claude/skills/kaizen/SKILL.md",
        "---\nname: kaizen\n---\n",
    );
}

#[test]
fn a_healthy_repository_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    healthy(dir.path());
    chkr()
        .arg(dir.path())
        .assert()
        .code(0)
        .stdout(contains(
            "[pass] AM05 AGENTS.md: 3 lines (warn above 200, fail above 500)",
        ))
        .stdout(contains(
            "[pass] AM06 AGENTS.md: 50 chars (fail above 40000)",
        ))
        .stdout(contains("[pass] AM08 .claude/skills/kaizen/SKILL.md"))
        .stdout(contains(
            "agents-md gate passed: 0 fail, 0 warn, 8 pass, 0 n/a",
        ));
}

#[test]
fn a_claude_md_with_its_own_rules_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    healthy(dir.path());
    write(dir.path(), "CLAUDE.md", "# Other rules\n");
    chkr()
        .arg(dir.path())
        .assert()
        .code(1)
        .stdout(contains("[fail] AM02 CLAUDE.md"))
        .stdout(contains("fix: merge it into AGENTS.md"));
}

#[test]
fn warnings_alone_exit_zero() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "AGENTS.md", "# Rules\n");
    chkr()
        .arg(dir.path())
        .assert()
        .code(0)
        .stdout(contains("[warn] AM07"))
        .stdout(contains("[warn] AM08"));
}

#[test]
fn a_kaizen_skill_is_found_under_any_skill_root() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "AGENTS.md", "# Rules\n");
    write(dir.path(), ".agents/skills/kaizen/SKILL.md", "x");
    chkr()
        .arg(dir.path())
        .assert()
        .stdout(contains("[pass] AM08 .agents/skills/kaizen/SKILL.md"));
}

#[test]
fn a_missing_directory_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    chkr()
        .arg(dir.path().join("missing"))
        .assert()
        .code(2)
        .stderr(contains("agents-md-chkr: cannot read"));
}

#[test]
fn an_unreadable_rules_file_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), [0xff, 0xfe]).unwrap();
    chkr().arg(dir.path()).assert().code(2);
}

#[test]
fn defaults_to_the_current_directory() {
    let dir = tempfile::tempdir().unwrap();
    healthy(dir.path());
    chkr().current_dir(dir.path()).assert().code(0);
}
