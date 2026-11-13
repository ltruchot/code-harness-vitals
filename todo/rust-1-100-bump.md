# Rust 1.100 bump

Due on or after 2026-11-15, when Rust 1.100.0 (released 2026-11-12) leaves the 3-day
grace period. Pick the newest 1.100.x patch release that is at least 3 days old.

1. `rust-toolchain.toml`: set `channel = "1.100.x"`. If needed, update the gitignored
   `mise.local.toml` too.
2. Pick a nightly at least 3 days old and set `NIGHTLY` in `xtask/src/plan.rs`. Then run
   `cargo xtask setup --toolchain-only`.
3. `Cargo.toml`: set `rust-version = "1.100"`.
4. `.cargo/config.toml`: add the native cooldown, which Cargo 1.99 warns about, so it is
   not committed yet:
   ```toml
   [registry]
   global-min-publish-age = "3 days"
   ```
5. `Cargo.toml`: add `[workspace.lints.cargo]` with `unused_dependencies = "warn"`.
6. Run `cargo xtask check`. Fix new clippy lints from the 1.100 lint set without
   relaxing the configuration.
7. ADR 3:
   - flip the "Native Cargo cooldown" row to Done;
   - drop the interim manual lockfile-age check from Decision and Trade-offs.
