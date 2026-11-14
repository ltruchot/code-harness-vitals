# code-harness-vitals

**Is your repository ready for coding agents?** code-harness-vitals audits the harness
that Claude Code and OpenCode work inside — instructions, skills, memory, context budget,
deterministic gates, and agent-config security — so that either agent works in it with
nothing missing.

Every check is a **gate** with two parts:

- **Deterministic probes, in Rust.** Pure functions of the files on disk: presence,
  layout, budgets, markers. Same input, same verdict. They alone set the exit code and the
  score, so the gate is safe to run in CI.
- **An agent review, as a skill.** Where content needs judgment — what is missing, what
  belongs in a skill, what is noise — a skill runs in your own Claude Code or OpenCode
  session. It runs the probes first, then proposes at most five `ADD` / `CUT` / `MOVE` /
  `REWORD` edits, labeled advisory. Nothing is edited without your yes.

> **Status: pre-alpha.** One gate (`agents-md`) is implemented; the others are specified.
> See the [intent ADR](docs/architecture_decision_records/2026-11-13-adr1-main-intent.md)
> and the [gate model ADR](docs/architecture_decision_records/2026-11-14-adr4-gate-model.md).

## What it checks

| Pillar                     | Questions it answers                                                                                     |
| -------------------------- | -------------------------------------------------------------------------------------------------------- |
| 1. Instruction files       | Is there an `AGENTS.md`? Is it under budget, structured, consistent, with real build and test commands? |
| 2. Skills                  | Do project skills exist? Is `SKILL.md` frontmatter valid, the description triggerable, the code safe?   |
| 3. Codebase memory         | Is a code-intelligence MCP server or plugin wired in for both agents?                                    |
| 4. Context limiting        | Does a tool or hook keep raw tool output out of the context window?                                      |
| 5. Deterministic gates     | Do commit hooks actually run lint, format, type checks, tests, vulnerability and secret scans? Does CI?  |
| 6. Agent-config security   | Are permission lists tight, MCP servers pinned, auto-approving hooks flagged?                            |

Design principles: deterministic score and exit code (agent review is advisory), Claude
Code and OpenCode parity, repository and user-scope findings reported separately, Scorecard-style weighting
with per-pillar levels, stable exit codes and JSON/SARIF output for CI.

## Gates

| Gate | Pillar | Deterministic probes (Rust) | Agent review (skill, advisory) | Status |
| ---- | ------ | --------------------------- | ------------------------------ | ------ |
| [`agents-md`](apps/agents-md-chkr/) | 1 | `AM01`–`AM15`, below | [`agents-md-review`](apps/agents-md-chkr/skills/agents-md-review/SKILL.md): 18-criterion rubric | ✅ |
| `skills` | 2 | frontmatter, description length, body budget, referenced files exist | triggerability, overlap, unsafe scripts | ⏳ |
| `codebase-memory` | 3 | a code-intelligence MCP server or plugin, wired in for both agents | - | ⏳ |
| `context-limiting` | 4 | an output-capping tool or hook, wired in for both agents | - | ⏳ |
| `commit-gates` | 5 | an active hook runner calling lint, format, types, tests, vulnerability and secret scans; CI runs the same | fit of the gate list to the stack | ⏳ |
| `agent-config` | 6 | permission lists, pinned MCP servers, auto-approving hooks | - | ⏳ |

### `agents-md`

```sh
cargo agents-md-chkr /path/to/repo              # exit 0: no fail · 1: a probe failed · 2: cannot read
cargo agents-md-chkr --repo-only /path/to/repo  # CI: skip the parent directories and home
```

Instruction files cascade: Claude Code loads every `CLAUDE.md` from the working directory
up to `/`, and one of them anywhere on that path turns off every `AGENTS.md`; OpenCode
loads every `AGENTS.md` up to `$HOME` and never reads `CLAUDE.md`. Both load
subdirectory files as they work there. The gate checks the whole cascade, in the
repository and, as user-scope findings that never fail the gate, above it
([ADR 4](docs/architecture_decision_records/2026-11-14-adr4-gate-model.md#the-agents-md-gate)).

| Probe | Checks | Fails or warns when |
| ----- | ------ | ------------------- |
| `AM01` | `AGENTS.md` at the root | fail: missing (the fix names a `CLAUDE.md` to rename) |
| `AM02` | root `CLAUDE.md` | fail: it has its own content, so Claude Code skips `AGENTS.md`; a bridge (`@AGENTS.md`) or a symlink passes |
| `AM03` | `.claude/CLAUDE.md` | fail: same as `AM02` (bridge: `@../AGENTS.md`) |
| `AM04` | `CLAUDE.local.md` | warn: present with no bridge, so Claude Code skips `AGENTS.md` on that machine |
| `AM05` | lines | warn above 200, fail above 500 |
| `AM06` | chars | fail above 40,000 |
| `AM07` | resume after `/clear` | warn: the first 10 lines do not name `TODO.md`, or it does not exist |
| `AM08` | continuous improvement | warn: no `kaizen` skill; the fix points to [qol-mini](https://github.com/ltruchot/qol-mini-4-ai-tui) |
| `AM09` | nested `CLAUDE.md`, `.claude/CLAUDE.md`, `CLAUDE.local.md` | fail: own rules that OpenCode never reads and that hide the sibling `AGENTS.md` from Claude Code; warn: nested `CLAUDE.local.md` |
| `AM10` | nested budgets | `AM05`/`AM06` on every nested instruction file and `.claude/rules` file |
| `AM11` | `.claude/rules` parity | warn: rules files that `opencode.json` `instructions` does not list |
| `AM12` | names an agent skips | warn: `.claude/AGENTS.md`, `AGENTS.local.md`, `AGENTS.override.md`, `agents.md` |
| `AM13` | cascade inventory | lists every nested instruction file |
| `AM14` | user scope: `CLAUDE.md` above the repository | warn: Claude Code loads it and skips every `AGENTS.md` (`~/.claude/CLAUDE.md` exempt) |
| `AM15` | user scope: `AGENTS.md` above the repository | warn: rules that teammates and CI do not have |

The review skill checks what probes cannot: real commands, conventions, guardrails
(think before coding, simplicity, surgical changes, verify in a loop), reuse before
create, ADR discipline, rules hygiene, handoff and kaizen equivalents, blocks that belong
in a skill, contradictions across the cascade, and rules only one agent sees. To use it, copy `apps/agents-md-chkr/skills/agents-md-review/` into
`.claude/skills/` (Claude Code and OpenCode both read it), then ask your agent to review
`AGENTS.md`.

## Develop

Rust workspace; every tool version is pinned in `mise.toml`. Details in
[ADR 3](docs/architecture_decision_records/2026-11-13-adr3-rust-stack.md).

```sh
mise install               # once: Rust, cargo-deny, cargo-llvm-cov
cargo xtask setup          # once: coverage nightly
cargo xtask install-hooks  # once: pre-commit runs every gate
cargo agents-md-chkr       # run the agents-md gate on the current directory
cargo dummy                # count files, lines, and chars in the current directory
cargo test                 # run the tests
cargo xtask e2e            # container end-to-end tests: a fake filesystem root (needs docker)
cargo xtask check          # every gate: fmt, clippy, tests, docs, cargo-deny, e2e, 90% branch coverage
```

## Related projects

The ten most relevant actively maintained projects (1000+ stars, commits in the last two
weeks, a release in the last three months), as of 2026-11-13. Full analysis in
[ADR 2](docs/architecture_decision_records/2026-11-13-adr2-competitors.md).

| Project                                                                        | What it is                                                                                                               |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| [ECC](https://github.com/affaan-m/ECC)                                         | Large agent harness kit; its `/harness-audit` command scores 12 categories, and AgentShield scans agent configs.         |
| [SkillSpector](https://github.com/NVIDIA/SkillSpector)                         | Security scanner for agent skills: static analyzers, YARA, OSV lookups, SARIF, a fully local `--no-llm` mode.            |
| [agent-scan](https://github.com/snyk/agent-scan)                               | Discovers agents, MCP servers, and skills on a machine and scans them for risks via Snyk's cloud API.                    |
| [claude-code-templates](https://github.com/davila7/claude-code-templates)      | Registry and installer of Claude Code components, with a `--health-check` command and a local analytics dashboard.      |
| [OpenSSF Scorecard](https://github.com/ossf/scorecard)                         | Risk-weighted 0–10 security health score for repositories; the scoring model this project follows.                       |
| [rulesync](https://github.com/dyoshikawa/rulesync)                             | Generates rules, skills, MCP, hooks, and permissions for about 65 agents from one source; `--check` detects drift.       |
| [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp)         | Local code knowledge graph exposed over MCP, with first-class Claude Code and OpenCode integration.                      |
| [context-mode](https://github.com/mksglu/context-mode)                         | Hooks and an MCP sandbox that keep large tool output out of the context window, indexed in SQLite FTS5.                  |
| [rtk](https://github.com/rtk-ai/rtk)                                           | Rust CLI proxy that rewrites shell commands to compress their output before it reaches the agent.                        |
| [MegaLinter](https://github.com/oxsecurity/megalinter)                         | Aggregates 100+ linters, formatters, secret and vulnerability scanners behind one config, with SARIF output.             |

Watch list (closest analogs, under 1000 stars):
[harness-score](https://github.com/paladini/harness-score), a deterministic harness
maturity scorer, and [agnix](https://github.com/agent-sh/agnix), a linter and LSP for
agent configuration files.

## Architecture Decision Records

- [ADR 1 — Main intent](docs/architecture_decision_records/2026-11-13-adr1-main-intent.md)
- [ADR 2 — Competitors and pillar references](docs/architecture_decision_records/2026-11-13-adr2-competitors.md)
- [ADR 3 — Rust stack](docs/architecture_decision_records/2026-11-13-adr3-rust-stack.md)
- [ADR 4 — Gate model](docs/architecture_decision_records/2026-11-14-adr4-gate-model.md)

## License

[MIT](LICENSE)
