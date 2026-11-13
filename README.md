# code-harness-vitals

**Is your repository ready for coding agents?** code-harness-vitals audits the harness
that Claude Code and OpenCode work inside — instructions, skills, memory, context budget,
deterministic gates, and agent-config security — and scores it, deterministically.

> **Status: pre-alpha.** Design documents only; no code yet. See the
> [intent ADR](docs/architecture_decision_records/2026-11-13-adr1-main-intent.md).

## What it checks

| Pillar                     | Questions it answers                                                                                     |
| -------------------------- | -------------------------------------------------------------------------------------------------------- |
| 1. Instruction files       | Is there an `AGENTS.md`? Is it under budget, structured, consistent, with real build and test commands? |
| 2. Skills                  | Do project skills exist? Is `SKILL.md` frontmatter valid, the description triggerable, the code safe?   |
| 3. Codebase memory         | Is a code-intelligence MCP server or plugin wired in for both agents?                                    |
| 4. Context limiting        | Does a tool or hook keep raw tool output out of the context window?                                      |
| 5. Deterministic gates     | Do commit hooks actually run lint, format, type checks, tests, vulnerability and secret scans? Does CI?  |
| 6. Agent-config security   | Are permission lists tight, MCP servers pinned, auto-approving hooks flagged?                            |

Design principles: deterministic scores (no LLM in the score), Claude Code and OpenCode
parity, repository and user-scope findings reported separately, Scorecard-style weighting
with per-pillar levels, stable exit codes and JSON/SARIF output for CI.

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

## License

[MIT](LICENSE)
