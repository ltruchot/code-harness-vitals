//! End-to-end tests of the `agents-md-chkr` binary on real directory trees.
//!
//! Repository cases run with `--repo-only`, so files in the parent directories of the
//! temporary directory cannot leak in. User-scope cases build the parent directories and a
//! fake home inside the temporary directory, and assert only on those paths. The real
//! filesystem root is exercised in `tests/container.rs`.
#![cfg(test)]
#![expect(
    clippy::disallowed_methods,
    reason = "tests build fixture trees on disk"
)]

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

const HANDOFF: &str = "> Session handoff: read `TODO.md` first.\n\n# Rules\n";

fn chkr() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("agents-md-chkr"))
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// A repository that passes every probe, plus `extra` files.
fn repo(extra: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "AGENTS.md", HANDOFF);
    write(dir.path(), "TODO.md", "# TODO\n");
    write(
        dir.path(),
        ".claude/skills/kaizen/SKILL.md",
        "---\nname: kaizen\n---\n",
    );
    for (path, content) in extra {
        write(dir.path(), path, content);
    }
    dir
}

/// Runs the gate on `dir` in repository scope.
fn gate(dir: &TempDir) -> assert_cmd::assert::Assert {
    chkr().arg("--repo-only").arg(dir.path()).assert()
}

#[test]
fn a_healthy_repository_prints_every_probe_and_exits_zero() {
    gate(&repo(&[])).code(0).stdout(
        "[pass] AM01 AGENTS.md: AGENTS.md is at the root\n\
         [pass] AM02 CLAUDE.md: absent: nothing shadows AGENTS.md\n\
         [pass] AM03 .claude/CLAUDE.md: absent: nothing shadows AGENTS.md\n\
         [pass] AM04 CLAUDE.local.md: absent\n\
         [pass] AM05 AGENTS.md: 3 lines (warn above 200, fail above 500)\n\
         [pass] AM06 AGENTS.md: 50 chars (fail above 40000)\n\
         [pass] AM07 AGENTS.md: opens by sending each session to TODO.md\n\
         [pass] AM08 .claude/skills/kaizen/SKILL.md: kaizen skill present\n\
         [pass] AM09 .: no nested CLAUDE.md holds its own rules\n\
         [pass] AM10 .: every nested instruction file is within budget\n\
         [n/a] AM11 .claude/rules: no rules files\n\
         [pass] AM12 .: no instruction file that an agent skips\n\
         [n/a] AM13 .: no instruction file below the root\n\
         [n/a] AM14 (user scope) ..: skipped: --repo-only\n\
         [n/a] AM15 (user scope) ..: skipped: --repo-only\n\
         agents-md gate passed: 0 fail, 0 warn, 11 pass, 4 n/a\n",
    );
}

#[test]
fn a_repository_with_only_claude_md_fails_with_a_rename_fix() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "CLAUDE.md", "# Rules\n");
    chkr()
        .args(["--repo-only"])
        .arg(dir.path())
        .assert()
        .code(1)
        .stdout(contains(
            "[fail] AM01 AGENTS.md: no AGENTS.md: the rules live in CLAUDE.md",
        ))
        .stdout(contains("fix: rename CLAUDE.md to AGENTS.md"))
        .stdout(contains("[n/a] AM02"));
}

#[test]
fn an_empty_directory_fails_on_presence() {
    let dir = tempfile::tempdir().unwrap();
    chkr()
        .args(["--repo-only"])
        .arg(dir.path())
        .assert()
        .code(1)
        .stdout(contains("[fail] AM01 AGENTS.md: no AGENTS.md at the root"));
}

#[test]
fn root_claude_md_with_its_own_rules_fails() {
    gate(&repo(&[("CLAUDE.md", "# Other rules\n")]))
        .code(1)
        .stdout(contains("[fail] AM02 CLAUDE.md"))
        .stdout(contains(
            "fix: merge it into AGENTS.md, then delete it or reduce it to the single line `@AGENTS.md`",
        ));
}

#[test]
fn root_claude_md_bridge_passes() {
    gate(&repo(&[("CLAUDE.md", "@AGENTS.md\n")]))
        .code(0)
        .stdout(contains(
            "[pass] AM02 CLAUDE.md: a bridge that only imports AGENTS.md",
        ));
}

#[cfg(unix)]
#[test]
fn root_claude_md_symlinks_pass_to_agents_md_only() {
    let dir = repo(&[]);
    std::os::unix::fs::symlink("AGENTS.md", dir.path().join("CLAUDE.md")).unwrap();
    gate(&dir).code(0).stdout(contains(
        "[pass] AM02 CLAUDE.md: a symbolic link to AGENTS.md",
    ));

    let elsewhere = repo(&[("docs/RULES.md", "# rules\n")]);
    std::os::unix::fs::symlink("docs/RULES.md", elsewhere.path().join("CLAUDE.md")).unwrap();
    gate(&elsewhere)
        .code(1)
        .stdout(contains("[fail] AM02 CLAUDE.md"));
}

#[test]
fn dot_claude_claude_md_shadows_unless_it_bridges_through_the_parent() {
    gate(&repo(&[(".claude/CLAUDE.md", "# rules\n")]))
        .code(1)
        .stdout(contains("[fail] AM03 .claude/CLAUDE.md"))
        .stdout(contains("`@../AGENTS.md`"));
    gate(&repo(&[(".claude/CLAUDE.md", "@../AGENTS.md\n")]))
        .code(0)
        .stdout(contains("[pass] AM03 .claude/CLAUDE.md: a bridge"));
}

#[test]
fn a_local_file_warns_without_failing() {
    gate(&repo(&[("CLAUDE.local.md", "# mine\n")]))
        .code(0)
        .stdout(contains("[warn] AM04 CLAUDE.local.md"));
    gate(&repo(&[
        ("CLAUDE.local.md", "# mine\n"),
        ("CLAUDE.md", "@AGENTS.md"),
    ]))
    .code(0)
    .stdout(contains(
        "[pass] AM04 CLAUDE.local.md: present, and a bridge",
    ));
}

#[test]
fn line_and_char_budgets_apply_to_the_root_file() {
    let lines = |count: usize| format!("{HANDOFF}{}", "x\n".repeat(count - 3));
    gate(&repo(&[("AGENTS.md", &lines(201))]))
        .code(0)
        .stdout(contains("[warn] AM05 AGENTS.md: 201 lines"));
    gate(&repo(&[("AGENTS.md", &lines(501))]))
        .code(1)
        .stdout(contains("[fail] AM05 AGENTS.md: 501 lines"));
    gate(&repo(&[(
        "AGENTS.md",
        &format!("{HANDOFF}{}", "x".repeat(40_000)),
    )]))
    .code(1)
    .stdout(contains("[fail] AM06 AGENTS.md: 40050 chars"));
}

#[test]
fn handoff_and_kaizen_warn_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "AGENTS.md", "# Rules\n");
    write(dir.path(), ".opencode/skills/kaizen/SKILL.md", "x");
    chkr()
        .args(["--repo-only"])
        .arg(dir.path())
        .assert()
        .code(0)
        .stdout(contains("[warn] AM07 AGENTS.md: names no TODO.md handoff"))
        .stdout(contains("[pass] AM08 .opencode/skills/kaizen/SKILL.md"));
    let bare = tempfile::tempdir().unwrap();
    write(bare.path(), "AGENTS.md", HANDOFF);
    chkr()
        .args(["--repo-only"])
        .arg(bare.path())
        .assert()
        .code(0)
        .stdout(contains("[warn] AM07 TODO.md"))
        .stdout(contains("[warn] AM08 .claude/skills"))
        .stdout(contains("qol-mini skills install kaizen"));
}

#[test]
fn a_nested_claude_md_fails_and_names_the_agents_md_it_hides() {
    gate(&repo(&[
        ("pkg/api/AGENTS.md", "# api\n"),
        ("pkg/api/CLAUDE.md", "# api, Claude only\n"),
        ("pkg/web/.claude/CLAUDE.md", "# web\n"),
    ]))
    .code(1)
    .stdout(contains(
        "[fail] AM09 pkg/api/CLAUDE.md: Claude Code loads it and skips pkg/api/AGENTS.md",
    ))
    .stdout(contains(
        "[fail] AM09 pkg/web/.claude/CLAUDE.md: only Claude Code reads it",
    ))
    .stdout(contains("fix: move its rules into pkg/web/AGENTS.md"));
}

#[cfg(unix)]
#[test]
fn nested_bridges_pass_and_nested_local_files_warn() {
    let dir = repo(&[
        ("pkg/AGENTS.md", "# pkg\n"),
        ("lib/AGENTS.md", "# lib\n"),
        ("lib/CLAUDE.md", "@AGENTS.md\n"),
        ("app/CLAUDE.local.md", "# mine\n"),
    ]);
    std::os::unix::fs::symlink("AGENTS.md", dir.path().join("pkg/CLAUDE.md")).unwrap();
    gate(&dir)
        .code(0)
        .stdout(contains("[warn] AM09 app/CLAUDE.local.md"))
        .stdout(contains("AM09 pkg/CLAUDE.md").not())
        .stdout(contains("AM09 lib/CLAUDE.md").not());
}

#[test]
fn ignored_and_worktree_directories_are_not_part_of_the_cascade() {
    gate(&repo(&[
        (".gitignore", "vendor/\n"),
        ("vendor/lib/CLAUDE.md", "# third party\n"),
        (".claude/worktrees/feature/CLAUDE.md", "# copy\n"),
        (".git/CLAUDE.md", "# not a tree file\n"),
    ]))
    .code(0)
    .stdout(contains("[n/a] AM13 .: no instruction file below the root"));
}

#[test]
fn nested_budgets_apply_to_every_cascade_file() {
    gate(&repo(&[
        ("pkg/AGENTS.md", &"x\n".repeat(250)),
        (".claude/rules/big.md", &"x".repeat(40_001)),
    ]))
    .code(1)
    .stdout(contains("[warn] AM10 pkg/AGENTS.md: 250 lines"))
    .stdout(contains(
        "[fail] AM10 .claude/rules/big.md: 1 lines, 40001 chars",
    ));
}

#[test]
fn claude_rules_warn_until_opencode_lists_them() {
    gate(&repo(&[("pkg/.claude/rules/style.md", "# style\n")]))
        .code(0)
        .stdout(contains(
            "[warn] AM11 .claude/rules: 1 .claude/rules files: Claude Code loads them, OpenCode does not",
        ));
    gate(&repo(&[
        (".claude/rules/style.md", "# style\n"),
        (
            "opencode.jsonc",
            "{\n  // shared rules\n  \"instructions\": [\".claude/rules/*.md\"]\n}\n",
        ),
    ]))
    .code(0)
    .stdout(contains("[pass] AM11 opencode.json"));
}

#[test]
fn names_an_agent_skips_warn() {
    gate(&repo(&[
        ("AGENTS.override.md", "x"),
        ("docs/agents.md", "x"),
        (".claude/AGENTS.md", "x"),
    ]))
    .code(0)
    .stdout(contains(
        "[warn] AM12 AGENTS.override.md: neither agent reads this name",
    ))
    .stdout(contains("[warn] AM12 docs/agents.md"))
    .stdout(contains(
        "[warn] AM12 .claude/AGENTS.md: only Claude Code reads",
    ));
}

#[test]
fn the_inventory_lists_the_whole_cascade() {
    gate(&repo(&[
        ("pkg/AGENTS.md", "# pkg\n"),
        (".claude/rules/a.md", "# a\n"),
    ]))
    .code(0)
    .stdout(contains(
        "[pass] AM13 .: 2 more instruction files load as agents work:",
    ))
    .stdout(contains("pkg/AGENTS.md"))
    .stdout(contains(".claude/rules/a.md"));
}

/// `<tmp>/home` is `$HOME`; the repository is `<tmp>/home/work/repo`.
fn user_tree() -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    write(&home, ".claude/CLAUDE.md", "# user memory\n");
    write(&home, "CLAUDE.md", "# stray rules in home\n");
    write(&home, "work/AGENTS.md", "# workspace rules\n");
    write(
        &home,
        "work/.claude/AGENTS.md",
        "# workspace, Claude only\n",
    );
    let repo = home.join("work/repo");
    write(&repo, "AGENTS.md", HANDOFF);
    write(&repo, "TODO.md", "# TODO\n");
    (dir, repo)
}

#[test]
fn parent_directories_are_reported_in_user_scope_without_failing() {
    let (dir, repo) = user_tree();
    let home = dir.path().join("home").canonicalize().unwrap();
    let shown = |relative: &str| home.join(relative).display().to_string();
    chkr()
        .env("HOME", &home)
        .arg(&repo)
        .assert()
        .code(0)
        .stdout(contains(format!(
            "[warn] AM14 (user scope) {}: outside the repository: Claude Code loads it and, by default, skips every AGENTS.md",
            shown("CLAUDE.md")
        )))
        .stdout(contains(format!("AM14 (user scope) {}", shown(".claude/CLAUDE.md"))).not())
        .stdout(contains(format!(
            "[warn] AM15 (user scope) {}: outside the repository: Claude Code loads it in every session here, and OpenCode too",
            shown("work/AGENTS.md")
        )))
        .stdout(contains(format!(
            "[warn] AM15 (user scope) {}",
            shown("work/.claude/AGENTS.md")
        )));
}

#[test]
fn a_repository_bridge_keeps_agents_md_loaded_under_a_parent_claude_md() {
    let (dir, repo) = user_tree();
    write(&repo, "CLAUDE.md", "@AGENTS.md\n");
    chkr()
        .env("HOME", dir.path().join("home").canonicalize().unwrap())
        .arg(&repo)
        .assert()
        .code(0)
        .stdout(contains("on top of the repository's rules"));
}

#[test]
fn repo_only_skips_the_parent_directories() {
    let (dir, repo) = user_tree();
    chkr()
        .env("HOME", dir.path().join("home"))
        .arg("--repo-only")
        .arg(&repo)
        .assert()
        .code(0)
        .stdout(contains("[n/a] AM14 (user scope) ..: skipped: --repo-only"))
        .stdout(contains("[warn] AM14").not());
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
fn a_file_as_root_exits_two() {
    let dir = repo(&[]);
    chkr()
        .arg(dir.path().join("AGENTS.md"))
        .assert()
        .code(2)
        .stderr(contains("is not a directory"));
}

#[test]
fn an_unreadable_rules_file_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), [0xff, 0xfe]).unwrap();
    chkr().arg("--repo-only").arg(dir.path()).assert().code(2);
}

#[test]
fn defaults_to_the_current_directory() {
    let dir = repo(&[]);
    chkr()
        .arg("--repo-only")
        .current_dir(dir.path())
        .assert()
        .code(0)
        .stdout(contains("agents-md gate passed"));
}
