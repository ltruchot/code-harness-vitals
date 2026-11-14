# TODO

## Where we stopped
The `skills` gate (pillar 2) is built on branch `gate/skills`, one commit per layer, not
pushed and with no pull request yet: next action is push, open the draft PR, and run the
code review on it. ADR 4, "The skills gate", holds the cascade of both agents (Claude Code
skills docs; OpenCode source at `d759c548aa`), probes `SK01`–`SK13` in repository scope and
`SK14`–`SK15` in user scope, and the 12-criterion `skills-review` rubric.
Tests run at three levels: unit, e2e on `tempfile` trees (`apps/skills-chkr/tests/cli.rs`,
23 cases), and containers (`tests/container.rs`, 6 images); `cargo xtask e2e` runs the
container tests of every gate in `CONTAINER_GATES`. Branch coverage is 96.76%.
`cargo skills-chkr --repo-only .` runs on this repo: `SK01` warns, since `.claude/` is not
committed. The user-scope run on this machine was not done: it reads parent directories
outside the working directory.
Not verified: the `skills-review` skill run from an installed copy in a live session of each
agent; Claude Code's handling of deeper layouts (`.claude/skills/<group>/<name>/SKILL.md`)
and of unclosed frontmatter.
Not verified: CI with two container gates (`jdx/mise-action`, docker on `ubuntu-latest`).

## Later
- Decide whether to commit `.claude/` (kaizen skill and `qol-mini.lock.json`); CI's `AM08` and `SK01` depend on it.
- Distribute the review skills: a qol-mini bank entry, or an install command in the gate binaries.
- `agents-md` probes: `@import` resolution (max four hops), backticked paths and commands that do not resolve, `paths` frontmatter of rules.
- `agents-md`: instruction files inside ignored dependency directories (`node_modules/*/AGENTS.md` loads on read in both agents).
- `agents-md`: user settings that change the cascade (`claudeMdExcludes`, Project instructions `claude-md-and-agents-md`).
- `skills`: run SkillSpector (`--no-llm --format json`) or agnix when installed, for script safety and per-agent lint (amends ADR 4).
- `skills`: `.claude/commands/` files, `opencode.json` `skills` entries, `--add-dir`, enterprise and plugin skills, `XDG_CONFIG_HOME` for OpenCode's global directory.
- `skills`: the order between OpenCode's `.opencode` and `.claude`/`.agents` sources for one id (plugin activation order not found at `d759c548aa`).
- JSON and SARIF output for every gate (`vitals-utils::Report`).
- Next gate: `codebase-memory` (pillar 3), per the ADR 4 catalog.
- Scoring engine: 0–10 per gate, risk weights, per-pillar levels (ADR 1 open question).
