# TODO

## Where we stopped
Nothing in progress. Both gates are on `main`: `agents-md` (`AM01`–`AM21`, PR #2) and
`skills` (`SK01`–`SK15`, PR #1), each reviewed with `code-review high` and fixed.
Next: pick an item below; the next gate is `codebase-memory` (`.claude/skills/add-gate/`).
Verified: `cargo agents-md-chkr --repo-only` on a 393-line `CLAUDE.md`-only project reports
`AM01` fail and `AM05`, `AM08`, `AM17`, `AM18`, `AM19` warn.
Not verified: a user-scope run on this machine (it reads parent directories outside the
working directory); either review skill run from an installed copy.

## Later
- Decide whether to commit `.claude/` (kaizen skill and `qol-mini.lock.json`); CI's `AM08` and `SK01` depend on it.
- `agents-md`: source the open AGENTS.md format (agents.md, read by most coding agents) in ADR 4 and the review rubric; today only Claude Code and OpenCode behavior is sourced.
- Distribute the review skills: a qol-mini bank entry, or an install command in the gate binaries.
- `agents-md`: commands that do not exist (`/doctor prompt-audit` covers them in Claude Code), imports outside the repository, the `claudeMd` key of managed settings.
- `agents-md`: instruction files inside ignored dependency directories (`node_modules/*/AGENTS.md` loads on read in both agents).
- `agents-md`: user settings that change the cascade (`claudeMdExcludes`, Project instructions `claude-md-and-agents-md`).
- `skills`: run SkillSpector (`--no-llm --format json`) or agnix when installed, for script safety and per-agent lint (amends ADR 4).
- `skills`: `.claude/commands/` files, `opencode.json` `skills` entries, `--add-dir`, enterprise and plugin skills, `XDG_CONFIG_HOME` for OpenCode's global directory.
- `skills`: the order between OpenCode's `.opencode` and `.claude`/`.agents` sources for one id (plugin activation order not found at `d759c548aa`).
- JSON and SARIF output for every gate (`vitals-utils::Report`).
- Next gate: `codebase-memory` (pillar 3), per the ADR 4 catalog.
- Scoring engine: 0–10 per gate, risk weights, per-pillar levels (ADR 1 open question).
