---
name: add-gate
description: Add a gate to code-harness-vitals end to end, on its own branch and pull request — source both agents' real behavior, specify the gate in ADR 4, build the deterministic Rust probes app (apps/GATE-chkr) with unit, real-tree, and container tests, write its advisory review skill, update the docs, and open a draft PR for code review. Use whenever the task is to implement, start, or extend a gate or a pillar check (skills, codebase-memory, context-limiting, commit-gates, agent-config, or a new one), or to add probes to an existing gate.
---

# Add a gate

A gate has two halves (ADR 4): deterministic probes that alone set the exit status, and an
advisory review skill. The `agents-md` gate is the **live template**: copy the shape of
`apps/agents-md-chkr/` and of the ADR 4 section "The agents-md gate", never a stale
copy of them.

**One layer, one commit**, on branch `gate/<id>`: a reviewer reads the PR commit by
commit. A fix that a later layer forces on an earlier one (the ADR, `vitals-utils`) goes
in the commit that found it, and its message says so. The pre-commit hook runs the full
`cargo xtask check` (minutes, docker included) on the working tree, not the index: build
each layer completely, iterate with the fast loop in [TESTS.md](TESTS.md), and commit once.

## 1. Frame

Read `TODO.md`, the pillar in ADR 1, and in ADR 4 the gate anatomy, the split rule, the
review contract, and the catalog row. Pick the gate id and a probe prefix no gate uses.
The PR's base must hold the live template: when `git log --oneline origin/main..main`
lists commits, ask the user to push `main` first, or the PR carries them. Then
`git switch -c gate/<id> main`.

The first PR ships the gate's own probes. Integrating an external scanner (ADR 1:
integrate, do not reimplement) is a later PR with its own ADR amendment.

**Done when:** the branch exists on a base that holds `apps/agents-md-chkr/`, and you can
name the gate's pillar, id, and prefix.

## 2. Source the behavior

Every probe encodes a claim about how Claude Code or OpenCode behaves, and the tests
encode the same claim, so only a source catches a wrong one. Map the **cascade** of every
file the gate judges, for both agents: where it loads from (root, subdirectories, parent
directories, home, global config), what shadows what, name variants, and size limits.
How to reach each source: [RESEARCH.md](RESEARCH.md).

A fact that contradicts an existing gate (a marker, a probe, a fix text) is a bug in that
gate: fix it in the shared layer (step 4) and name it in the PR.

**Done when:** a table lists each fact, the agent, the source (docs URL, OpenCode commit
and file, or an empirical run), and whether it is confirmed. An unconfirmed fact can feed
the review, never a probe.

## 3. Specify in ADR 4 — commit

Design the probes with the ADR 4 split rule. Give each probe an id, a scope (repository,
or user: warn at most, skipped by `--repo-only`), its verdicts, its evidence path, and a
fix for every warn and fail. List the review rubric, each criterion with its source.
Write all of it into ADR 4 the way the `agents-md` section reads: the catalog row set to
✅, a `### The <id> gate` section (the step 2 facts table with its Source column, the
probe table, not probed yet), and the Implementation Status rows and References. A decision that ADR 4 does not cover (an
external scanner, a new crate, an output format) amends the ADR that owns it.

**Done when:** a reviewer could implement the probes from the ADR alone. Commit
`Specify the <id> gate in ADR 4`.

## 4. Share before you write — commit if anything moved

Read `pkg/vitals-utils/src/` and the `agents-md-chkr` modules before writing any
function. Product code is never duplicated:

- A helper the new gate needs that lives in `agents-md-chkr` moves to `vitals-utils`, with
  its tests, and `agents-md-chkr` imports it from there.
- Every filesystem primitive lives in `vitals_utils::fs`, even one a single gate uses: it
  is the one boundary.
- A helper found during step 5 moves in the step 5 commit.

Test helpers are the exception: integration test files cannot import each other, so they
are copied.

**Done when:** no product function in the new gate duplicates one elsewhere. Commit
`Move <helper> to vitals-utils` when something moved.

## 5. Build the probes — commit

Mirror `apps/agents-md-chkr/`: `audit(root, Reach)` in `lib.rs`; `main.rs` with `[PATH]`
and `--repo-only`; all I/O in `snapshot.rs` through `vitals_utils::fs`; one pure function
per probe. Add the `<id>-chkr` alias to `.cargo/config.toml`. Test at all three levels
per [TESTS.md](TESTS.md).

**Done when:** `cargo xtask check` passes, and `cargo <id>-chkr --repo-only .` runs on
this repository. Commit `Add the <id> gate probes`.

## 6. Write the review skill — commit

Create `apps/<id>-chkr/skills/<id>-review/SKILL.md` and `references/sources.md`, in the
shape of `agents-md-review`: run the probes and quote them, read what they point to,
judge with a numbered rubric (present, `ADD`, `MOVE`, `CUT`, `REWORD`), report at most
five kaizen blocks labeled advisory, and apply only what the user accepts.

**Done when:** the skill, run by hand, quotes the probes and returns at most five blocks:
on this repository, or on a scratch repository built from the gate's test fixtures when
this one has nothing for the gate to judge. Commit `Add the <id>-review skill`.

## 7. Document — commit

Update the README (the Gates row, a `### <id>` section with the command and the probe
table, the Develop block), the Implementation Status row in ADR 1, the alias list and
commands table in ADR 3, the "Commands" section of AGENTS.md, and `TODO.md` ("Where we
stopped", and the not-probed-yet items under "Later").

**Done when:** `git grep` finds no stale probe range or path, and AGENTS.md is under 200
lines. Commit `Document the <id> gate`.

## 8. Open the pull request

Check `git log --oneline main..` for one commit per layer, and grep
`git diff main..` for names and emails. Then follow [PR.md](PR.md): confirm before
pushing, open a draft PR, call the Skill tool with `code-review` on it, and fix the
confirmed findings in new commits.

**Done when:** the draft PR exists, its review is clean, and CI is green. Then
`gh pr ready`, and call the Skill tool with `kaizen`.
