# TODO

## Where we stopped
Nothing in progress. The Rust workspace (ADR 3) runs on Rust 1.100.0, with the native
Cargo cooldown (`global-min-publish-age = "3 days"`) on. Official toolchains are exempt
from the grace period (ADR 3, "Grace period: third-party code only").
`cargo xtask check` passes: fmt, clippy `-D warnings`, 33 tests, docs, cargo-deny,
100% branch coverage (gate 90%).
Every tool version lives in `mise.toml` (Rust, cargo-deny, cargo-llvm-cov); CI installs
it with `jdx/mise-action`. Only the coverage nightly comes from `cargo xtask setup`.
Not verified: whether `/context all` in a fresh session lists AGENTS.md under Memory files.

## Later
- Start the check engine: first pillar checks per ADR 1 (instruction files) in a new `pkg/` crate.
