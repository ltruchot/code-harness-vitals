# TODO

## Where we stopped
Nothing in progress. `main` is 3 commits ahead of `origin/main` (through `565678a`), not
pushed: next action is `git push`, then watch CI run the container step. The `agents-md` gate covers the whole instruction cascade (ADR 4,
"The agents-md gate"): root files `AM01`–`AM08`, nested files `AM09`–`AM13`, and parent
directories in user scope `AM14`–`AM15` (warn only; `--repo-only` skips them).
Tests run at three levels: unit, e2e on `tempfile` trees (`tests/cli.rs`, 24 cases), and
containers (`tests/container.rs`, 6 `FROM scratch` images with a fake `/` and `$HOME`,
run by `cargo xtask e2e`). `cargo xtask check` runs all three and needs docker; branch
coverage is 100%. The musl target is in `mise.toml`.
`cargo agents-md-chkr --repo-only .` passes on this repo. The user-scope run on this
machine was not done: it reads parent directories outside the working directory.
Not verified: the review skill run end to end from an installed copy in `.claude/skills/`.
Not verified: CI with the container step (`jdx/mise-action` installing the musl target,
docker on `ubuntu-latest`).

## Later
- Decide whether to commit `.claude/` (kaizen skill and `qol-mini.lock.json`); CI's `AM08` depends on it.
- Distribute the review skill: a qol-mini bank entry, or an install command in the gate binary.
- `agents-md` probes: `@import` resolution (max four hops), backticked paths and commands that do not resolve, `paths` frontmatter of rules.
- `agents-md`: instruction files inside ignored dependency directories (`node_modules/*/AGENTS.md` loads on read in both agents).
- `agents-md`: user settings that change the cascade (`claudeMdExcludes`, Project instructions `claude-md-and-agents-md`).
- JSON and SARIF output for every gate (`vitals-utils::Report`).
- Next gate: `skills` (pillar 2), per the ADR 4 catalog.
- Scoring engine: 0–10 per gate, risk weights, per-pillar levels (ADR 1 open question).
