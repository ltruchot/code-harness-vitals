# TODO

## Where we stopped
Nothing in progress. ADR 3 (Rust stack) is done and pushed: `main` at `caa6494`, CI green.
A Cargo workspace with `pkg/dummy` and `xtask`; `cargo xtask check` passes locally and in
CI: fmt, clippy `-D warnings`, 33 tests, docs, cargo-deny, 100% branch coverage (gate 90%).
Verified: each gate fails when broken on purpose (unwrap → clippy, `fs::read` in a pure
module → disallowed method, dropped test → coverage below 90%).
Local env note: the mise shims ignore `rust-toolchain.toml`; a gitignored
`mise.local.toml` pins `rust = "1.99.1"`.
Not verified: whether `/context all` in a fresh session lists AGENTS.md under Memory files.

## Later
- On or after 2026-11-15: bump to Rust 1.100.x and a matching nightly → todo/rust-1-100-bump.md
- Start the check engine: first pillar checks per ADR 1 (instruction files) in a new `pkg/` crate.
