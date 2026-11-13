# TODO

## Where we stopped
Nothing in progress. The Rust workspace (ADR 3) runs on Rust 1.100.0, with the native
Cargo cooldown (`global-min-publish-age = "3 days"`) on. Official toolchains are exempt
from the grace period (ADR 3, "Grace period: third-party code only").
`cargo xtask check` passes: fmt, clippy `-D warnings`, 33 tests, docs, cargo-deny,
100% branch coverage (gate 90%).
Local env note: the mise shims ignore `rust-toolchain.toml`; a gitignored
`mise.local.toml` pins `rust = "1.100.0"`. Update it when the toolchain moves.
Not verified: whether `/context all` in a fresh session lists AGENTS.md under Memory files.

## Later
- Start the check engine: first pillar checks per ADR 1 (instruction files) in a new `pkg/` crate.
