//! The probes of the `agents-md` gate: pure functions of a [`Snapshot`], one per probe id.

use std::path::{Component, Path};

use vitals_utils::markers::AGENTS_MD;
use vitals_utils::{Budget, Finding, Report, TextSize, Verdict};

use crate::snapshot::{Shadow, Snapshot};

/// Gate id.
pub const GATE: &str = "agents-md";

/// Line budget: warn above 200, fail above 500.
///
/// Claude Code memory docs: "target under 200 lines per CLAUDE.md file". 500 lines is the
/// cap the kaizen skill sets for an always-loaded rules file.
pub const LINES: Budget = Budget::new(200, 500);

/// Char budget, fail only.
///
/// The per-file limit Claude Code applies to instruction files loaded on demand with a
/// 200K context window (memory docs, "My CLAUDE.md is too large"); past it, Claude
/// receives only the start of the file.
pub const CHARS: Budget = Budget::hard(40_000);

/// How many leading lines of `AGENTS.md` must name the handoff file.
pub const HANDOFF_WINDOW: usize = 10;

const QOL_MINI: &str = "https://github.com/ltruchot/qol-mini-4-ai-tui";

/// A file Claude Code reads instead of `AGENTS.md` when it exists.
struct Location {
    probe: &'static str,
    path: &'static str,
    /// `AGENTS.md` as seen from the file's directory, for imports and link targets.
    agents: &'static str,
}

const ROOT_CLAUDE_MD: Location = Location {
    probe: "AM02",
    path: "CLAUDE.md",
    agents: "AGENTS.md",
};

const DOT_CLAUDE_MD: Location = Location {
    probe: "AM03",
    path: ".claude/CLAUDE.md",
    agents: "../AGENTS.md",
};

/// How a shadowing file relates to `AGENTS.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    Absent,
    Symlink,
    Bridge,
    Own,
}

impl Layout {
    const fn loads_agents(self) -> bool {
        matches!(self, Self::Symlink | Self::Bridge)
    }
}

pub(crate) fn check(snapshot: &Snapshot) -> Report {
    Report::new(
        GATE,
        vec![
            presence(snapshot),
            shadow(&ROOT_CLAUDE_MD, &snapshot.claude_md, snapshot),
            shadow(&DOT_CLAUDE_MD, &snapshot.dot_claude_md, snapshot),
            local_file(snapshot),
            lines(snapshot),
            chars(snapshot),
            handoff(snapshot),
            kaizen(snapshot),
        ],
    )
}

fn not_applicable(probe: &'static str, evidence: &str) -> Finding {
    Finding::new(
        probe,
        Verdict::NotApplicable,
        evidence,
        "no AGENTS.md to judge",
    )
}

/// `AM01`: `AGENTS.md` is at the root.
fn presence(snapshot: &Snapshot) -> Finding {
    match (&snapshot.agents_md, &snapshot.claude_md.content) {
        (Some(_), _) => Finding::new("AM01", Verdict::Pass, AGENTS_MD, "AGENTS.md is at the root"),
        (None, Some(_)) => Finding::new(
            "AM01",
            Verdict::Fail,
            AGENTS_MD,
            "no AGENTS.md: the rules live in CLAUDE.md, which OpenCode reads only as a fallback",
        )
        .with_fix("rename CLAUDE.md to AGENTS.md; Claude Code 2.1.277 and later reads it"),
        (None, None) => Finding::new("AM01", Verdict::Fail, AGENTS_MD, "no AGENTS.md at the root")
            .with_fix(
                "create AGENTS.md: project summary, build/test/lint commands, and the \
                 conventions an agent cannot infer",
            ),
    }
}

fn layout(file: &Shadow, agents: &str) -> Layout {
    if let Some(target) = &file.link {
        let same = target
            .components()
            .filter(|part| *part != Component::CurDir)
            .eq(Path::new(agents).components());
        return if same { Layout::Symlink } else { Layout::Own };
    }
    match file.content.as_deref() {
        None => Layout::Absent,
        Some(text) if text.trim().strip_prefix('@') == Some(agents) => Layout::Bridge,
        Some(_) => Layout::Own,
    }
}

/// `AM02`, `AM03`: no `CLAUDE.md` shadows `AGENTS.md`, unless it imports or links to it.
fn shadow(at: &Location, file: &Shadow, snapshot: &Snapshot) -> Finding {
    if snapshot.agents_md.is_none() {
        return not_applicable(at.probe, at.path);
    }
    let (verdict, message) = match layout(file, at.agents) {
        Layout::Absent => (Verdict::Pass, "absent: nothing shadows AGENTS.md"),
        Layout::Symlink => (Verdict::Pass, "a symbolic link to AGENTS.md"),
        Layout::Bridge => (Verdict::Pass, "a bridge that only imports AGENTS.md"),
        Layout::Own => (
            Verdict::Fail,
            "Claude Code reads this file and skips AGENTS.md, so the two agents follow \
             different rules",
        ),
    };
    let finding = Finding::new(at.probe, verdict, at.path, message);
    if verdict == Verdict::Fail {
        finding.with_fix(format!(
            "merge it into AGENTS.md, then delete it or reduce it to the single line `@{}`",
            at.agents
        ))
    } else {
        finding
    }
}

/// `AM04`: no `CLAUDE.local.md` makes Claude Code skip `AGENTS.md` on one machine.
fn local_file(snapshot: &Snapshot) -> Finding {
    const PATH: &str = "CLAUDE.local.md";
    if snapshot.agents_md.is_none() {
        return not_applicable("AM04", PATH);
    }
    let bridged = layout(&snapshot.claude_md, ROOT_CLAUDE_MD.agents).loads_agents()
        || layout(&snapshot.dot_claude_md, DOT_CLAUDE_MD.agents).loads_agents();
    match (snapshot.claude_local_md, bridged) {
        (false, _) => Finding::new("AM04", Verdict::Pass, PATH, "absent"),
        (true, true) => Finding::new(
            "AM04",
            Verdict::Pass,
            PATH,
            "present, and a bridge CLAUDE.md still loads AGENTS.md",
        ),
        (true, false) => Finding::new(
            "AM04",
            Verdict::Warn,
            PATH,
            "present: Claude Code reads it and skips AGENTS.md on this machine",
        )
        .with_fix(
            "add a CLAUDE.md holding only `@AGENTS.md`, or move the local rules to \
             ~/.claude/CLAUDE.md",
        ),
    }
}

fn budget(probe: &'static str, measure: usize, unit: &str, limit: Budget) -> Finding {
    let verdict = limit.verdict(measure);
    let message = if limit.warn() < limit.fail() {
        format!(
            "{measure} {unit} (warn above {}, fail above {})",
            limit.warn(),
            limit.fail()
        )
    } else {
        format!("{measure} {unit} (fail above {})", limit.fail())
    };
    let finding = Finding::new(probe, verdict, AGENTS_MD, message);
    if verdict > Verdict::Pass {
        finding.with_fix(
            "keep what every session needs; move workflow-specific sections into skills and \
             detail into docs that AGENTS.md links to",
        )
    } else {
        finding
    }
}

/// `AM05`: `AGENTS.md` is within the [`LINES`] budget.
fn lines(snapshot: &Snapshot) -> Finding {
    snapshot.agents_md.as_deref().map_or_else(
        || not_applicable("AM05", AGENTS_MD),
        |text| budget("AM05", TextSize::of(text).lines(), "lines", LINES),
    )
}

/// `AM06`: `AGENTS.md` is within the [`CHARS`] budget.
fn chars(snapshot: &Snapshot) -> Finding {
    snapshot.agents_md.as_deref().map_or_else(
        || not_applicable("AM06", AGENTS_MD),
        |text| budget("AM06", TextSize::of(text).chars(), "chars", CHARS),
    )
}

/// `AM07`: a session that starts after `/clear` or a compaction is sent to `TODO.md` first.
fn handoff(snapshot: &Snapshot) -> Finding {
    let Some(text) = snapshot.agents_md.as_deref() else {
        return not_applicable("AM07", AGENTS_MD);
    };
    let named = text
        .lines()
        .take(HANDOFF_WINDOW)
        .any(|line| line.contains("TODO.md"));
    let finding = match (named, snapshot.todo_md) {
        (true, true) => {
            return Finding::new(
                "AM07",
                Verdict::Pass,
                AGENTS_MD,
                "opens by sending each session to TODO.md",
            );
        }
        (true, false) => Finding::new(
            "AM07",
            Verdict::Warn,
            "TODO.md",
            "AGENTS.md sends each session to TODO.md, which does not exist",
        ),
        (false, _) => Finding::new(
            "AM07",
            Verdict::Warn,
            AGENTS_MD,
            format!(
                "names no TODO.md handoff in its first {HANDOFF_WINDOW} lines: a session \
                 after /clear or a compaction starts cold"
            ),
        ),
    };
    finding.with_fix(format!(
        "open AGENTS.md with a line that sends each session to TODO.md (where the last \
         session stopped, what is left) and keep TODO.md current; the qol-mini kaizen skill \
         sets both up: {QOL_MINI}"
    ))
}

/// `AM08`: a continuous-improvement (kaizen) skill turns session friction into rule edits.
fn kaizen(snapshot: &Snapshot) -> Finding {
    snapshot.kaizen_skill.as_deref().map_or_else(
        || {
            Finding::new(
                "AM08",
                Verdict::Warn,
                ".claude/skills",
                "no kaizen skill in .claude/skills, .opencode/skills, or .agents/skills: \
                 nothing turns session friction into rule edits",
            )
            .with_fix(format!(
                "`qol-mini skills install kaizen <repo>` ({QOL_MINI}); the agents-md-review \
                 skill also recognizes an equivalent under another name"
            ))
        },
        |path| Finding::new("AM08", Verdict::Pass, path, "kaizen skill present"),
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{CHARS, LINES, check};
    use crate::snapshot::{Shadow, Snapshot};
    use std::path::PathBuf;
    use vitals_utils::{Outcome, Verdict};

    const HANDOFF: &str = "> Session handoff: read `TODO.md` first.\n\n# Rules\n";

    fn healthy() -> Snapshot {
        Snapshot {
            agents_md: Some(HANDOFF.to_owned()),
            todo_md: true,
            kaizen_skill: Some(PathBuf::from(".claude/skills/kaizen/SKILL.md")),
            ..Snapshot::default()
        }
    }

    fn content(text: &str) -> Shadow {
        Shadow {
            content: Some(text.to_owned()),
            link: None,
        }
    }

    fn link(target: &str) -> Shadow {
        Shadow {
            content: Some(HANDOFF.to_owned()),
            link: Some(PathBuf::from(target)),
        }
    }

    fn verdict(snapshot: &Snapshot, probe: &str) -> Verdict {
        check(snapshot)
            .findings()
            .iter()
            .find(|finding| finding.probe() == probe)
            .unwrap()
            .verdict()
    }

    fn sized(lines: usize, width: usize) -> Snapshot {
        let line = format!("{}\n", "x".repeat(width.saturating_sub(1)));
        Snapshot {
            agents_md: Some(format!("{HANDOFF}{}", line.repeat(lines.saturating_sub(3)))),
            ..healthy()
        }
    }

    #[test]
    fn a_healthy_repository_passes_every_probe() {
        let report = check(&healthy());
        assert_eq!(report.findings().len(), 8);
        assert_eq!(report.worst(), Verdict::Pass, "{report}");
    }

    #[test]
    fn a_missing_agents_md_fails_once_and_skips_the_rest() {
        let report = check(&Snapshot::default());
        assert_eq!(verdict(&Snapshot::default(), "AM01"), Verdict::Fail);
        assert_eq!(report.outcome(), Outcome::Failed);
        let skipped = ["AM02", "AM03", "AM04", "AM05", "AM06", "AM07"];
        assert!(
            skipped
                .iter()
                .all(|probe| verdict(&Snapshot::default(), probe) == Verdict::NotApplicable)
        );
    }

    #[test]
    fn a_claude_md_alone_is_named_in_the_fix() {
        let snapshot = Snapshot {
            claude_md: content("# Rules\n"),
            ..Snapshot::default()
        };
        let report = check(&snapshot);
        let presence = report.findings().first().unwrap();
        assert_eq!(presence.verdict(), Verdict::Fail);
        assert!(presence.fix().unwrap().starts_with("rename CLAUDE.md"));
    }

    #[test]
    fn root_claude_md_passes_as_bridge_or_symlink_and_fails_with_own_content() {
        let with = |claude_md| Snapshot {
            claude_md,
            ..healthy()
        };
        assert_eq!(
            verdict(&with(content("@AGENTS.md\n")), "AM02"),
            Verdict::Pass
        );
        assert_eq!(verdict(&with(link("AGENTS.md")), "AM02"), Verdict::Pass);
        assert_eq!(verdict(&with(link("./AGENTS.md")), "AM02"), Verdict::Pass);
        assert_eq!(
            verdict(&with(link("docs/AGENTS.md")), "AM02"),
            Verdict::Fail
        );
        assert_eq!(
            verdict(&with(content("# Other rules\n")), "AM02"),
            Verdict::Fail
        );
        assert_eq!(
            verdict(&with(content("@AGENTS.md\n- more\n")), "AM02"),
            Verdict::Fail
        );
    }

    #[test]
    fn dot_claude_md_bridges_through_the_parent_directory() {
        let with = |dot_claude_md| Snapshot {
            dot_claude_md,
            ..healthy()
        };
        assert_eq!(
            verdict(&with(content("@../AGENTS.md")), "AM03"),
            Verdict::Pass
        );
        assert_eq!(verdict(&with(link("../AGENTS.md")), "AM03"), Verdict::Pass);
        assert_eq!(verdict(&with(content("@AGENTS.md")), "AM03"), Verdict::Fail);
    }

    #[test]
    fn a_local_file_warns_unless_a_bridge_loads_agents_md() {
        let local = Snapshot {
            claude_local_md: true,
            ..healthy()
        };
        assert_eq!(verdict(&local, "AM04"), Verdict::Warn);
        let bridged = Snapshot {
            claude_md: content("@AGENTS.md"),
            ..local.clone()
        };
        assert_eq!(verdict(&bridged, "AM04"), Verdict::Pass);
        let dot_bridged = Snapshot {
            dot_claude_md: link("../AGENTS.md"),
            ..local
        };
        assert_eq!(verdict(&dot_bridged, "AM04"), Verdict::Pass);
    }

    #[test]
    fn line_budget_edges() {
        assert_eq!(verdict(&sized(LINES.warn(), 1), "AM05"), Verdict::Pass);
        assert_eq!(verdict(&sized(LINES.warn() + 1, 1), "AM05"), Verdict::Warn);
        assert_eq!(verdict(&sized(LINES.fail(), 1), "AM05"), Verdict::Warn);
        assert_eq!(verdict(&sized(LINES.fail() + 1, 1), "AM05"), Verdict::Fail);
    }

    #[test]
    fn char_budget_edges() {
        let at_limit = Snapshot {
            agents_md: Some(format!(
                "{HANDOFF}{}",
                "x".repeat(CHARS.fail() - HANDOFF.len())
            )),
            ..healthy()
        };
        assert_eq!(verdict(&at_limit, "AM06"), Verdict::Pass);
        assert_eq!(verdict(&sized(100, 500), "AM06"), Verdict::Fail);
        let report = check(&sized(100, 500));
        assert!(report.findings()[5].fix().unwrap().contains("skills"));
    }

    #[test]
    fn handoff_must_name_an_existing_todo_md_early() {
        let no_todo = Snapshot {
            todo_md: false,
            ..healthy()
        };
        assert_eq!(verdict(&no_todo, "AM07"), Verdict::Warn);
        let late = Snapshot {
            agents_md: Some(format!("{}{HANDOFF}", "line\n".repeat(10))),
            ..healthy()
        };
        assert_eq!(verdict(&late, "AM07"), Verdict::Warn);
        let unnamed = Snapshot {
            agents_md: Some("# Rules\n".to_owned()),
            ..healthy()
        };
        assert_eq!(verdict(&unnamed, "AM07"), Verdict::Warn);
    }

    #[test]
    fn a_missing_kaizen_skill_points_to_qol_mini() {
        let snapshot = Snapshot {
            kaizen_skill: None,
            ..healthy()
        };
        let report = check(&snapshot);
        let kaizen = report.findings().last().unwrap();
        assert_eq!(kaizen.verdict(), Verdict::Warn);
        assert!(
            kaizen
                .fix()
                .unwrap()
                .contains("qol-mini skills install kaizen")
        );
        assert_eq!(report.outcome(), Outcome::Passed);
    }
}
