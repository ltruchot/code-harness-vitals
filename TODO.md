# TODO

## Where we stopped
Nothing in progress. The gate model (ADR 4) and the first gate, `agents-md`, are done:
`apps/agents-md-chkr/` (probes `AM01`–`AM08`, review skill in `skills/agents-md-review/`)
on top of `pkg/vitals-utils/` (findings, report, exit contract, budgets, markers, fs).
`pkg/dummy` moved to `apps/dummy`. ADR 1 and ADR 3 are amended, with Supersedes entries.
`cargo xtask check` passes, with 100% branch coverage (gate: 90%).
`cargo agents-md-chkr .` passes 8/8 on this repo; `AM08` passes only because the
untracked `.claude/skills/kaizen/` is on disk.
Not verified: the review skill run end to end from an installed copy in `.claude/skills/`.
Not verified: whether `/context all` in a fresh session lists AGENTS.md under Memory files.

## Later
- Decide whether to commit `.claude/` (kaizen skill and `qol-mini.lock.json`); CI's `AM08` depends on it.
- Distribute the review skill: a qol-mini bank entry, or an install command in the gate binary.
- `agents-md` probes: `@import` resolution (max four hops), backticked paths and commands that do not resolve, nested and `.claude/rules/` files.
- `agents-md`: user-scope findings (a `CLAUDE.md` above the repo root makes Claude Code skip `AGENTS.md`).
- JSON and SARIF output for every gate (`vitals-utils::Report`).
- Next gate: `skills` (pillar 2), per the ADR 4 catalog.
- Scoring engine: 0–10 per gate, risk weights, per-pillar levels (ADR 1 open question).
