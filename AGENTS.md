# AGENTS.md

## Project

code-harness-vitals audits whether a repository is ready for agentic coding with
Claude Code and OpenCode: instruction files, skills, codebase memory, context limiting,
deterministic gates, and agent-config security. Product intent and scope live in
`docs/architecture_decision_records/2026-11-13-adr1-main-intent.md`; read it before
proposing features.

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

- `README.md`: pitch, pillars, status, related projects.
- `docs/architecture_decision_records/`: one ADR per decision,
  named `YYYY-MM-DD-adrN-kebab-slug.md`.

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

## Commands

- No stack is chosen yet (open question in ADR 1), so there are no build, lint, or test
  commands. Do not invent any; add them here when the first code lands.
