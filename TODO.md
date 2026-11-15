# TODO

## Where we stopped
Branch `fix/agents-md-sources`, not pushed: every `agents-md` probe now cites a fact (`L`,
`A`, `G` in ADR 4) or says **convention**; the cascade was checked with `claude -p` on
Claude Code 2.1.325 ("Empirical checks"). `AM05` warns only, `AM06` checks the 4 MiB launch
limit, `AM10` applies 40,000 chars to on-demand files only, `AM20` matches `paths` as the
docs' table says. `.claude/` is committed (home path written `$HOME`).
Next: push, open the PR, `code-review`, merge.
Not verified: a user-scope run on this machine; either review skill run from an installed
copy; whether qol-mini accepts the kaizen files with `$HOME` written in (its hashes predate it).

## Later
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
