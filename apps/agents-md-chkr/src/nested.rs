//! Probes `AM09`–`AM13`: the instruction files below the root, which agents load as they work
//! in subdirectories. Pure functions of a [`Snapshot`].

use std::path::{Path, PathBuf};

use vitals_utils::{Finding, TextSize, Verdict, or_pass};

use crate::cascade::{Kind, classify, scope_dir};
use crate::imports::{Imported, Load};
use crate::probes::{CHARS, LINES, layout};
use crate::snapshot::{Entry, Snapshot};

pub(crate) fn findings(snapshot: &Snapshot) -> Vec<Finding> {
    let nested = &snapshot.nested;
    [
        claude_only(nested),
        budgets(snapshot),
        vec![rules_parity(snapshot)],
        unread(nested),
        vec![inventory(nested)],
    ]
    .concat()
}

fn kind(entry: &Entry) -> Option<Kind> {
    classify(&entry.path)
}

fn shown(dir: &Path) -> PathBuf {
    if dir.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        dir.to_path_buf()
    }
}

/// `AM09`: no nested `CLAUDE.md` or `CLAUDE.local.md` holds rules that `OpenCode` never reads,
/// or that make Claude Code skip the directory's `AGENTS.md`.
fn claude_only(nested: &[Entry]) -> Vec<Finding> {
    let found = nested
        .iter()
        .filter_map(|entry| shadowing(entry, nested))
        .collect();
    or_pass(
        found,
        Finding::new(
            "AM09",
            Verdict::Pass,
            ".",
            "no nested CLAUDE.md holds its own rules",
        ),
    )
}

fn shadowing(entry: &Entry, nested: &[Entry]) -> Option<Finding> {
    let (agents, verdict) = match kind(entry)? {
        Kind::Claude => ("AGENTS.md", Verdict::Fail),
        Kind::DotClaude => ("../AGENTS.md", Verdict::Fail),
        Kind::ClaudeLocal => ("AGENTS.md", Verdict::Warn),
        Kind::Agents | Kind::ClaudeAgents | Kind::Rule | Kind::Unread => return None,
    };
    if verdict == Verdict::Fail && layout(&entry.file, agents).loads_agents() {
        return None;
    }
    let dir = shown(scope_dir(&entry.path));
    let sibling = dir.join("AGENTS.md");
    let message = if nested.iter().any(|other| other.path == sibling) {
        format!(
            "Claude Code loads it and skips {}; OpenCode reads only AGENTS.md",
            sibling.display()
        )
    } else {
        "only Claude Code reads it; OpenCode reads only AGENTS.md".to_owned()
    };
    Some(
        Finding::new("AM09", verdict, &entry.path, message).with_fix(format!(
            "move its rules into {} (create it if needed), then delete it or reduce it to `@{agents}`",
            sibling.display()
        )),
    )
}

/// `AM10`: every nested instruction file, as loaded, is within the line and char budgets.
fn budgets(snapshot: &Snapshot) -> Vec<Finding> {
    let found = snapshot
        .nested
        .iter()
        .filter_map(|entry| over_budget(entry, &snapshot.imported))
        .collect();
    or_pass(
        found,
        Finding::new(
            "AM10",
            Verdict::Pass,
            ".",
            "every nested instruction file is within budget",
        ),
    )
}

fn over_budget(entry: &Entry, imported: &Imported) -> Option<Finding> {
    let text = entry.file.content.as_deref()?;
    let kind = kind(entry);
    if kind == Some(Kind::Unread) {
        return None;
    }
    let load = Load::of(&entry.path, text, imported, kind == Some(Kind::Agents));
    let (lines, _) = load.larger(TextSize::lines);
    let (chars, _) = load.larger(TextSize::chars);
    let verdict = LINES.verdict(lines).max(CHARS.verdict(chars));
    (verdict > Verdict::Pass).then(|| {
        Finding::new(
            "AM10",
            verdict,
            &entry.path,
            format!(
                "{lines} lines, {chars} chars as loaded (warn above {} lines; fail above {} lines \
                 or {} chars)",
                LINES.warn(),
                LINES.fail(),
                CHARS.fail()
            ),
        )
        .with_fix(
            "split it: past the char limit Claude Code sends only the start of a file it loads \
             on demand",
        )
    })
}

/// `AM11`: `.claude/rules/` files, which only Claude Code loads, are also given to `OpenCode`.
fn rules_parity(snapshot: &Snapshot) -> Finding {
    let rules = snapshot
        .nested
        .iter()
        .filter(|entry| kind(entry) == Some(Kind::Rule))
        .count();
    let listed = snapshot
        .opencode_config
        .as_deref()
        .is_some_and(|config| config.contains(".claude/rules"));
    match (rules, listed) {
        (0, _) => Finding::new(
            "AM11",
            Verdict::NotApplicable,
            ".claude/rules",
            "no rules files",
        ),
        (_, true) => Finding::new(
            "AM11",
            Verdict::Pass,
            "opencode.json",
            format!("{rules} .claude/rules files, listed in OpenCode `instructions`"),
        ),
        (_, false) => Finding::new(
            "AM11",
            Verdict::Warn,
            ".claude/rules",
            format!("{rules} .claude/rules files: Claude Code loads them, OpenCode does not"),
        )
        .with_fix(
            "list them in the `instructions` array of opencode.json (OpenCode then loads them in \
             every session, without `paths` scoping), or move the rules into AGENTS.md files",
        ),
    }
}

/// `AM12`: no instruction file that only one agent, or neither, reads.
fn unread(nested: &[Entry]) -> Vec<Finding> {
    let found = nested.iter().filter_map(one_sided).collect();
    or_pass(
        found,
        Finding::new(
            "AM12",
            Verdict::Pass,
            ".",
            "no instruction file that an agent skips",
        ),
    )
}

fn one_sided(entry: &Entry) -> Option<Finding> {
    let message = match kind(entry)? {
        Kind::ClaudeAgents => "only Claude Code reads .claude/AGENTS.md; OpenCode does not",
        Kind::Unread => {
            "neither agent reads this name: both load `AGENTS.md` exactly, and names are \
             case-sensitive on Linux and in CI"
        }
        Kind::Agents | Kind::Claude | Kind::DotClaude | Kind::ClaudeLocal | Kind::Rule => {
            return None;
        }
    };
    let target = shown(scope_dir(&entry.path)).join("AGENTS.md");
    Some(
        Finding::new("AM12", Verdict::Warn, &entry.path, message)
            .with_fix(format!("merge it into {}", target.display())),
    )
}

/// `AM13`: the inventory of nested instruction files, for the review skill to read.
fn inventory(nested: &[Entry]) -> Finding {
    if nested.is_empty() {
        return Finding::new(
            "AM13",
            Verdict::NotApplicable,
            ".",
            "no instruction file below the root",
        );
    }
    let paths: Vec<String> = nested
        .iter()
        .map(|entry| entry.path.display().to_string())
        .collect();
    Finding::new(
        "AM13",
        Verdict::Pass,
        ".",
        format!(
            "{} more instruction files load as agents work: {}",
            paths.len(),
            paths.join(", ")
        ),
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::findings;
    use crate::snapshot::{Entry, RulesFile, Snapshot};
    use std::path::PathBuf;
    use vitals_utils::{Finding, Verdict};

    fn file(path: &str, content: &str) -> Entry {
        Entry {
            path: PathBuf::from(path),
            file: RulesFile {
                content: Some(content.to_owned()),
                link: None,
            },
        }
    }

    fn run(nested: Vec<Entry>) -> Vec<Finding> {
        findings(&Snapshot {
            nested,
            ..Snapshot::default()
        })
    }

    fn of(found: &[Finding], probe: &str) -> Vec<(Verdict, String)> {
        found
            .iter()
            .filter(|finding| finding.probe() == probe)
            .map(|finding| (finding.verdict(), finding.evidence().display().to_string()))
            .collect()
    }

    #[test]
    fn an_empty_tree_passes_or_does_not_apply() {
        let found = run(Vec::new());
        assert!(
            found
                .iter()
                .all(|finding| finding.verdict() <= Verdict::Pass)
        );
        assert_eq!(
            of(&found, "AM13"),
            [(Verdict::NotApplicable, ".".to_owned())]
        );
    }

    #[test]
    fn nested_claude_md_with_rules_fails_and_names_the_sibling() {
        let found = run(vec![
            file("pkg/AGENTS.md", "# pkg\n"),
            file("pkg/CLAUDE.md", "# other\n"),
            file("lib/.claude/CLAUDE.md", "# lib\n"),
        ]);
        let shadows = of(&found, "AM09");
        assert_eq!(
            shadows,
            [
                (Verdict::Fail, "pkg/CLAUDE.md".to_owned()),
                (Verdict::Fail, "lib/.claude/CLAUDE.md".to_owned()),
            ]
        );
        let messages: Vec<&str> = found.iter().map(Finding::message).collect();
        assert!(messages.iter().any(|m| m.contains("skips pkg/AGENTS.md")));
        assert!(messages.iter().any(|m| m.starts_with("only Claude Code")));
    }

    #[test]
    fn nested_bridges_pass_and_local_files_warn() {
        let link = Entry {
            path: PathBuf::from("app/CLAUDE.md"),
            file: RulesFile {
                content: None,
                link: Some(PathBuf::from("AGENTS.md")),
            },
        };
        let found = run(vec![
            file("pkg/CLAUDE.md", "@AGENTS.md\n"),
            file("lib/.claude/CLAUDE.md", "@../AGENTS.md"),
            link,
            file("pkg/CLAUDE.local.md", "# mine\n"),
        ]);
        assert_eq!(
            of(&found, "AM09"),
            [(Verdict::Warn, "pkg/CLAUDE.local.md".to_owned())]
        );
    }

    #[test]
    fn nested_budgets_flag_each_oversized_file() {
        let long = "x\n".repeat(201);
        let huge = "x".repeat(40_001);
        let found = run(vec![
            file("a/AGENTS.md", &long),
            file(".claude/rules/big.md", &huge),
            file("b/agents.md", &huge),
            file("c/AGENTS.md", "short\n"),
        ]);
        assert_eq!(
            of(&found, "AM10"),
            [
                (Verdict::Warn, "a/AGENTS.md".to_owned()),
                (Verdict::Fail, ".claude/rules/big.md".to_owned()),
            ]
        );
    }

    #[test]
    fn rules_warn_unless_opencode_lists_them() {
        let rules = vec![
            file(".claude/rules/a.md", "x"),
            file("pkg/.claude/rules/b.md", "y"),
        ];
        let unlisted = run(rules.clone());
        assert_eq!(
            of(&unlisted, "AM11"),
            [(Verdict::Warn, ".claude/rules".to_owned())]
        );
        let listed = findings(&Snapshot {
            nested: rules,
            opencode_config: Some(r#"{ "instructions": [".claude/rules/*.md"] }"#.to_owned()),
            ..Snapshot::default()
        });
        assert_eq!(
            of(&listed, "AM11"),
            [(Verdict::Pass, "opencode.json".to_owned())]
        );
    }

    #[test]
    fn one_sided_and_unread_files_warn() {
        let found = run(vec![
            file(".claude/AGENTS.md", "x"),
            file("pkg/AGENTS.override.md", "x"),
            file("docs/agents.md", "x"),
        ]);
        assert_eq!(of(&found, "AM12").len(), 3);
        assert!(
            found
                .iter()
                .any(|f| f.fix() == Some("merge it into ./AGENTS.md"))
        );
        assert!(
            found
                .iter()
                .any(|f| f.fix() == Some("merge it into docs/AGENTS.md"))
        );
    }

    #[test]
    fn inventory_lists_every_nested_file() {
        let found = run(vec![
            file("pkg/AGENTS.md", "x"),
            file(".claude/rules/a.md", "y"),
        ]);
        let inventory = found.iter().find(|f| f.probe() == "AM13").unwrap();
        assert_eq!(inventory.verdict(), Verdict::Pass);
        assert!(
            inventory
                .message()
                .ends_with("pkg/AGENTS.md, .claude/rules/a.md")
        );
    }
}
