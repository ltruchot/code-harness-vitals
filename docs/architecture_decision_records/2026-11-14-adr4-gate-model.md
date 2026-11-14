# Architecture Decision Record 4: Gate Model — Deterministic Probes, Agent Review

**Date**: 2026-11-14
**Status**: Accepted
**Context**: Turns ADR 1's six pillars into runnable gates. Each gate is a Rust binary on
the ADR 3 stack. The first gate, `agents-md`, proves the model.

## Implementation Status

| Component                                   | Status     | Location                                   |
| ------------------------------------------- | ---------- | ------------------------------------------ |
| Gate model (this ADR)                       | ✅ Done    | `docs/architecture_decision_records/`      |
| Shared findings, budgets, markers, file I/O | ✅ Done    | `pkg/vitals-utils/`                        |
| `agents-md` gate: probes                    | ✅ Done    | `apps/agents-md-chkr/`                     |
| `agents-md` gate: review skill              | ✅ Done    | `apps/agents-md-chkr/skills/agents-md-review/` |
| `agents-md` gate: cascade probes            | ✅ Done    | `apps/agents-md-chkr/src/{cascade,nested,ancestors}.rs` |
| Container end-to-end tests                  | ✅ Done    | `apps/agents-md-chkr/tests/container.rs`   |
| Other gates (catalog below)                 | ⏳ Planned | -                                          |

## Problem

A harness is good when Claude Code or OpenCode can work in it with nothing missing.
Part of that can be decided by a pure function of the files on disk: a file exists, a
layout is unambiguous, a budget holds, a marker is present. Part of it cannot: whether
an `AGENTS.md` names the right commands, holds a block that belongs in a skill, or lacks
a rule that every session needs. ADR 1 kept the score deterministic. Without a defined
place for the judgment half, the most useful findings on instruction files cannot be
reported at all.

## Decision

**Every check is a gate with a deterministic part, run in Rust, and, where content must
be judged, an agent part, run as a skill in the user's own Claude Code or OpenCode
session. Only the deterministic part sets the exit code and the score.**

### Gate anatomy

- **Id and pillar**: a kebab-case id (`agents-md`) and the ADR 1 pillar it serves.
- **Probes**: the deterministic part. Each probe has a stable id (`AM01`) and returns a
  verdict — `pass`, `warn`, `fail`, or `n/a` — with the evidence path (relative to the
  repository root), what it observed, and a concrete fix for a `warn` or a `fail`.
  Probes are pure functions of a snapshot read once at the boundary.
- **Exit status**: 0 when no probe fails (warnings included), 1 when one does, 2 when
  the gate cannot run (bad usage, unreadable repository).
- **Review skill**: the agent part, optional per gate. It is a `SKILL.md` with a rubric,
  shipped next to the binary.

### What goes where

A probe decides anything that a pure function of the files can decide: presence, layout,
sizes, explicit markers, paths that resolve. The review gets only what needs reading for
meaning: what is missing, what is misplaced (it belongs in a skill, a path-scoped rule,
the docs, or a code comment), what contradicts something else, and what is noise. When a
review criterion becomes decidable by a marker, it moves into a probe.

### Review contract

A review skill:

1. runs the gate binary first and quotes its findings;
2. reads the judged file in full, and everything it imports or requires;
3. returns at most five items, each an `ADD`, `CUT`, `MOVE`, or `REWORD` block with
   evidence, in the format of the qol-mini kaizen skill, one item per message;
4. edits nothing without the user's yes;
5. labels every item **advisory**: a review never changes the exit code or the score.

It runs inside the user's agent session, so it needs no API key, and it behaves the same
in Claude Code and OpenCode, which read the same `SKILL.md`.

### Layout

- `apps/<gate>-chkr/`: one binary per gate, with its review skill in
  `skills/<gate>-review/`.
- `pkg/vitals-utils/`: only code that two or more gates need — `Finding`, `Verdict`,
  `Report`, `Outcome` (exit contract), `TextSize` and `Budget`, the marker paths agents
  read (`AGENTS.md`, skill roots), and the filesystem boundary (`require_dir`,
  `read_optional`, `link_target`). A helper that a second gate needs moves here; one
  needed by a single gate stays in its app.

### Gate catalog

| Gate | Pillar | Probes (deterministic) | Review (agent, advisory) | Status |
| --- | --- | --- | --- | --- |
| `agents-md` | 1. Instruction files | `AM01`–`AM15`, below | `agents-md-review`: 18-criterion rubric | ✅ |
| `skills` | 2. Skills | `SKILL.md` frontmatter, description length, body budget, referenced files exist | description triggerability, overlap between skills, unsafe script patterns | ⏳ |
| `codebase-memory` | 3. Codebase memory | a code-intelligence MCP server or plugin is wired in for both agents | - | ⏳ |
| `context-limiting` | 4. Context limiting | an output-capping tool or hook is wired in for both agents | - | ⏳ |
| `commit-gates` | 5. Deterministic gates | an active hook runner whose hooks call lint, format, type check, tests, vulnerability and secret scans; CI runs the same list | whether the gate list fits the stack | ⏳ |
| `agent-config` | 6. Agent-config security | permission lists, pinned MCP servers, auto-approving hooks | - | ⏳ |

### The `agents-md` gate

Instruction files cascade, and the cascade changes what each agent reads. Both agents
concatenate every file they load; neither has override semantics, so when two files
disagree the model may follow either one.

| Where | Claude Code | OpenCode 2 |
| --- | --- | --- |
| Working directory and every parent, up to `/` | `CLAUDE.md`, `.claude/CLAUDE.md`, `CLAUDE.local.md` load at launch; `AGENTS.md` and `.claude/AGENTS.md` too, but **only when none of the three `CLAUDE.md` files exists at that level or above** | every `AGENTS.md` from the working directory up to `$HOME` (when the repository is under it) or the project root, nearest first; no `CLAUDE.md` |
| Subdirectories | `CLAUDE.md` and `CLAUDE.local.md` load on demand when a file there is touched; a subdirectory `AGENTS.md` loads on Read only if that subdirectory has none of the three `CLAUDE.md` files | a subdirectory `AGENTS.md` loads when the read tool opens a file below it |
| `.claude/rules/**/*.md` | loaded, at launch or (`paths`) on demand | not read, unless listed in `opencode.json` `instructions` |
| Exempt | `~/.claude/CLAUDE.md` and the managed file load alongside `AGENTS.md` | `~/.config/opencode/AGENTS.md` is the global file |
| Never read | `AGENTS.local.md`, `AGENTS.override.md`, `.agents/` | anything but `AGENTS.md` and configured `instructions` |
| Size | launch files over 4 MiB are skipped; on-demand files over 40,000 chars (200K context) are truncated | no limit found |

A single canonical `AGENTS.md` per directory is therefore the only layout that both agents
read the same way. A `CLAUDE.md` that only imports it (`@AGENTS.md`) or is a symbolic link
to it is an accepted bridge. A `CLAUDE.md` in a parent directory outside the repository —
`~/CLAUDE.md`, `~/workspace/CLAUDE.md`, `/CLAUDE.md` — silently turns off every
`AGENTS.md` in Claude Code while OpenCode keeps reading them: the two agents then follow
different rules with nothing in the repository to show it.

| Id | Probe | Verdict |
| --- | --- | --- |
| `AM01` | `AGENTS.md` at the root | fail when missing; the fix names `CLAUDE.md` to rename when one exists |
| `AM02` | root `CLAUDE.md` does not shadow `AGENTS.md` | pass when absent, a bridge, or a link to `AGENTS.md`; fail when it has its own content |
| `AM03` | `.claude/CLAUDE.md` does not shadow `AGENTS.md` | same as `AM02`, with `@../AGENTS.md` as the bridge |
| `AM04` | no `CLAUDE.local.md` that makes Claude Code skip `AGENTS.md` | warn when present without a bridge |
| `AM05` | line budget | warn above 200 lines (the Claude Code memory docs target), fail above 500 |
| `AM06` | char budget | fail above 40,000 chars, the per-file limit Claude Code applies to instruction files loaded on demand (200K context window) |
| `AM07` | resume after `/clear` or a compaction | warn unless the first 10 lines name `TODO.md` and `TODO.md` exists |
| `AM08` | continuous improvement | warn unless a `kaizen` skill exists in `.claude/skills/`, the only skill root both agents read (Claude Code never reads `.agents/skills/` or `.opencode/skills/`); the fix suggests qol-mini, or a symbolic link into `.claude/skills/` |
| `AM09` | nested `CLAUDE.md`, `.claude/CLAUDE.md`, `CLAUDE.local.md` | fail for each one with its own rules (OpenCode never reads it; Claude Code skips the sibling `AGENTS.md`); warn for a nested `CLAUDE.local.md`; bridges and links pass |
| `AM10` | nested budgets | `AM05`/`AM06` budgets applied to every nested instruction file and every `.claude/rules` file |
| `AM11` | `.claude/rules` parity | warn when rules files exist and `opencode.json[c]` does not list `.claude/rules` |
| `AM12` | names an agent skips | warn for `.claude/AGENTS.md` (Claude Code only), `AGENTS.local.md`, `AGENTS.override.md`, and case variants such as `agents.md` |
| `AM13` | cascade inventory | lists every nested instruction file, for the review to read |
| `AM14` | user scope: `CLAUDE.md`, `.claude/CLAUDE.md`, `CLAUDE.local.md` in a parent directory (except `~/.claude/CLAUDE.md`) | warn: Claude Code loads it and, without a bridge in the repository, skips every `AGENTS.md` |
| `AM15` | user scope: `AGENTS.md`, `.claude/AGENTS.md` in a parent directory | warn: rules that teammates and CI do not have |

The nested walk is the tree an agent sees: hidden files included, `.gitignore` honored,
`.git/` and `.claude/worktrees/` skipped. User-scope probes (`AM14`, `AM15`) read every
parent directory up to `/` and the home directory; they are labeled `(user scope)`, never
fail, and never change the exit status. `--repo-only` skips them for reproducible CI runs.

`AM07` and `AM08` detect one convention by its explicit markers. The review recognizes
equivalents under other names (rubric criteria 12 and 13) and suggests
[qol-mini](https://github.com/ltruchot/qol-mini-4-ai-tui) when there is none.

Not probed yet: `@import` resolution and depth, backticked paths and commands that do
not resolve, `paths` frontmatter of rules, instruction files inside ignored dependency
directories (`node_modules/*/AGENTS.md` loads when an agent reads a file there), sessions
started in a subdirectory instead of the root, the Claude Code **Project instructions**
setting (`claude-md-and-agents-md` loads both families) and `claudeMdExcludes`, which live
in user settings, and JSON/SARIF output.

### Tests

Every gate is tested at three levels, all run by `cargo xtask check`:

- **Unit**: each probe on in-memory snapshots, each classification and budget edge, and
  property tests on the algebra (`TextSize`, `Budget`).
- **End to end on real trees**: the binary on `tempfile` directory trees, one per case:
  every probe, symbolic links, ignore rules, and exit statuses 0, 1, and 2. Repository
  cases run with `--repo-only`; user-scope cases build their parent directories and a
  fake `$HOME` inside the temporary directory.
- **End to end in a container**: a temporary directory cannot hold `/CLAUDE.md` or a real
  `$HOME`. `cargo xtask e2e` builds a static (musl) binary and runs it in `FROM scratch`
  images whose whole filesystem is the fixture (`/CLAUDE.md`, `/home/dev/CLAUDE.md`,
  `/home/dev/.claude/CLAUDE.md`, the repository at `/home/dev/work/repo`), with no network
  and no base image to pull. It needs docker, locally and in CI.

## Alternatives Rejected

**NO gate binary that spawns `claude -p` or `opencode run`**: We reject because it needs
an authenticated agent CLI on the machine and in CI, its output varies between runs,
and the process boundary is hard to test; a skill runs in the session the user already
has.

**NO agent verdict in the exit code or the score**: We reject because two runs can
disagree, so a CI gate would flake; ADR 1's reproducibility holds for the score.

**NO single binary for every gate**: We reject for now because one app per gate keeps
each gate's dependencies, tests, and release independent; shared code lives in
`pkg/vitals-utils`. Revisit when a combined report or score is built.

**NO content heuristics in probes** (keyword lists for "has a commands section", "has
guardrails"): We reject because prose matching yields false positives and false
negatives that a deterministic gate cannot justify; that judgment belongs to the review.

## Trade-offs

### Wins

- The gate's verdict is reproducible in CI, and the judgment half still reaches the
  user, with evidence and a diff.
- One review format (the kaizen block) for every gate, read and answered the same way.
- Probes and review share one catalog, so a criterion moves from review to probe when it
  becomes decidable.

### Costs

- Review quality depends on the model and the session; the same file can get different
  suggestions.
- The review skill must be installed in the agent's skill directory to run; distribution
  is not settled (ADR 1, open question).
- `AM07`/`AM08` recognize one convention by name; other conventions only pass in the
  review.

## References

- [Claude Code — How Claude remembers your project](https://code.claude.com/docs/en/memory)
  — 200-line target, four-hop imports, launch and on-demand loading, when AGENTS.md is
  read, the 4 MiB and 40,000-char limits, conflicts.
- [Using CLAUDE.md files](https://claude.com/resources/articles/using-claude-md-files)
- [OpenCode source](https://github.com/anomalyco/opencode/tree/024a1462f5/packages/core/src)
  at `024a1462f5` (2026-11-14): `config/plugin/instruction.ts` (the `AGENTS.md` walk up
  to `$HOME` or the project root, configured `instructions`), `tool/plugin/read.ts`
  (subdirectory `AGENTS.md` on read). The V1 rules page
  (<https://opencode.ai/docs/rules/>) still describes a `CLAUDE.md` fallback that the V2
  source no longer has.
- [qol-mini](https://github.com/ltruchot/qol-mini-4-ai-tui) — kaizen skill, `TODO.md`
  handoff, ADR 0003 (AGENTS.md as the canonical rules file).
- Rubric sources: `apps/agents-md-chkr/skills/agents-md-review/references/sources.md`.
- ADR 1: `2026-11-13-adr1-main-intent.md` — pillars and principles.
- ADR 3: `2026-11-13-adr3-rust-stack.md` — workspace, lints, coverage.
