# TODO

## Where we stopped
- PR #1 (`gate/skills`, the skills gate) is merged into `main` by rebase, its review
  findings fixed.
- `gate/agents-md-content`, rebased on `main`, not pushed, no PR: the `AM05`–`AM07` fix,
  the content probes `AM16`–`AM21`, ADR 4 facts `A1`–`A15`, the 22-criterion rubric, the
  README TL;DR format, and the frontmatter and link scanner fixes from the PR #1 review.
  `cargo xtask check` passes.
Next actions, in order:
1. Push `gate/agents-md-content`, open its draft PR (`.claude/skills/add-gate/PR.md`),
   `code-review`, fix in new commits, `gh pr ready`, merge by rebase.
2. `git branch -d gate/skills` locally and `git push origin --delete gate/skills`.
3. The main checkout holds an older, uncommitted `TODO.md`: `git checkout TODO.md` there
   before pulling; then `git worktree remove .claude/worktrees/agent-af7db5ec9d62615ce`.
Verified: `cargo agents-md-chkr --repo-only` on a 393-line `CLAUDE.md`-only project reports
`AM01` fail and `AM05`, `AM08`, `AM17`, `AM18`, `AM19` warn.
Not verified: a user-scope run on this machine (it reads parent directories outside the
working directory); either review skill run from an installed copy.

## Later
- Decide whether to commit `.claude/` (kaizen skill and `qol-mini.lock.json`); CI's `AM08` and `SK01` depend on it.
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
