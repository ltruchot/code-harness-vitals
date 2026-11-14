# AGENTS.md

> Session handoff: read `TODO.md` first — where the last session stopped, and what is left.
> Open a `todo/` file only for the item you work on. An item done → delete its line and its `todo/` file.

## Project

code-harness-vitals audits whether a repository is ready for agentic coding with
Claude Code and OpenCode: instruction files, skills, codebase memory, context limiting,
deterministic gates, and agent-config security. Product intent and scope live in
`docs/architecture_decision_records/2026-11-13-adr1-main-intent.md`; read it before
proposing features. Every check is a gate (ADR 4): deterministic Rust probes set the exit
code and the score; an advisory review skill judges content. Read ADR 4 before adding one.

## Language (MANDATORY)

- Write everything in US English at native fluency: code, identifiers, comments, docs,
  commit messages, PR descriptions, issues, and CLI output.
- Use idiomatic, expert software-engineering terminology ("deterministic gate",
  "pre-commit hook", "exit code", "false positive"), never literal translations.
- If a request arrives in another language, reply in that language but keep every
  artifact in US English.

## Commits and repository content (MANDATORY)

- Never include personal data: no names, emails, handles, locations, or private context
  in commits, code, docs, fixtures, or test data.
- Never write about the maintainer or developer: no opinions, preferences, conversations,
  relationships, or past experiments. Forbidden shapes include "the dev thinks…",
  "X told me…", "we tried…", "as discussed…".
- State facts an agent needs to work on the code: what changed, why, and any constraint
  it introduces. Nothing else.
- Commit messages: imperative subject line of 72 characters or fewer, blank line, a body
  explaining why when the subject does not. Trailer allowed: `Co-Authored-By` only.
- Before committing, grep the staged diff and the message for names and emails, and
  remove any match.

## Layout

- `mise.toml`: the only place for language and tool versions (Rust, cargo tools). Never
  add `rust-toolchain.toml`, `.tool-versions`, or `cargo install` steps.
- Cargo workspace: binaries in `apps/<name>/`, libraries in `pkg/<name>/`, automation in
  `xtask/`. Lints, versions, and package metadata are inherited from the root `Cargo.toml`.
- `apps/<gate>-chkr/`: one gate, its probes, and its review skill in
  `skills/<gate>-review/`. `pkg/vitals-utils/`: code that two or more gates need.
- `apps/dummy/`: toolchain smoke test (counts files, lines, chars); not a product.
- `README.md`: pitch, pillars, status, related projects.
- `docs/architecture_decision_records/`: one ADR per decision,
  named `YYYY-MM-DD-adrN-kebab-slug.md`.

## Rust (MANDATORY, details in ADR 3)

- Idiomatic Rust 2024 per the Rust API Guidelines; functional style only through Rust
  idioms (iterators, enums, `?`, traits), never patterns borrowed from other languages.
- Side effects only in boundary modules, which open with
  `#![expect(clippy::disallowed_methods, reason = "boundary: …")]`; everything else is pure.
- Exemptions use `#[expect(lint, reason = "…")]`, scoped as narrowly as possible, never
  `allow`. Do not relax `Cargo.toml` or `clippy.toml` to make a gate pass.
- Test modules carry `#[cfg_attr(coverage_nightly, coverage(off))]`; integration test
  files start with `#![cfg(test)]`.
- Before writing a function or adding a crate, search `pkg/vitals-utils/` and the
  crate's own modules for one that exists. A helper that a second gate needs moves to
  `vitals-utils`.
- New crate or version bump: meet the ADR 3 admission criteria and pin `=x.y.z` in
  `[workspace.dependencies]`. Never weaken the 3-day Cargo cooldown in
  `.cargo/config.toml`; official stable toolchains are exempt, nightlies stay date-pinned.

## Architecture Decision Records

- ADR numbers are permanent; never renumber. A merged or retired ADR keeps a tombstone.
- Header: `# Architecture Decision Record N: Title`, then `**Date**`, `**Status**`,
  `**Context**` lines.
- Sections, in order: Problem, Decision, Alternatives Rejected, Trade-offs, Supersedes
  (when applicable), References.
- When a decision changes, amend the ADR in place and move the old decision to
  Supersedes with its date range and the reason it was dropped.
- Competitor data in ADR 2 is a dated snapshot. Re-measure freshness (commits in the last
  14 days, latest release, issues closed/opened in 30 days) before citing a project
  anywhere new.
- Measure it with `gh api graphql` (aliased `repository { history(since:) releases }`
  plus `search(type: ISSUE) { issueCount }`), never a loop over REST `search/issues`:
  that endpoint allows 30 requests/min and fails partway through a batch.

## Commands

- `cargo agents-md-chkr [PATH]`: run the `agents-md` gate (exit 0 pass, 1 fail, 2 error).
- `cargo dummy [PATH] [--all]`: run the dummy app (default: current directory).
- `cargo test`: every unit, property, integration, and doc test on stable.
- `cargo xtask check`: every gate (fmt, clippy `-D warnings`, test, doc, cargo-deny,
  90% branch coverage). It must pass before every commit; the pre-commit hook runs it.
- `cargo xtask cov`: branch coverage only; writes `lcov.info` and `coverage/html/`.
- `mise install`, `cargo xtask setup`, then `cargo xtask install-hooks`: once per clone.
