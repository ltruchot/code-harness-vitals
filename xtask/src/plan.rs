//! Pure task planning: which commands each task runs, in order. Nothing here spawns a process.

/// Nightly toolchain used only for branch coverage (`--branch` is nightly-only).
/// Pinned to a dated build: nightlies skip the beta and pre-release soak (ADR 3).
pub(crate) const NIGHTLY: &str = "nightly-2026-11-09";

/// Target of the static binary that the container end-to-end tests run on an empty root.
/// Installed from `mise.toml`; `tests/container.rs` reads the binary from its release dir.
pub(crate) const STATIC_TARGET: &str = "x86_64-unknown-linux-musl";

/// Minimum branch coverage, in percent, for the coverage gate.
pub(crate) const MIN_BRANCH_COVERAGE: f64 = 90.0;

/// A task the user can run as `cargo xtask <task>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Task {
    /// Every gate: format, lints, tests, docs, supply chain, container tests, branch coverage.
    Check,
    /// End-to-end tests in containers whose filesystem root holds the fixtures (needs docker).
    E2e,
    /// Branch coverage with reports (`lcov.info`, `coverage/html`) and the gate.
    Coverage,
    /// Install the coverage nightly. Every other tool comes from `mise.toml`.
    Setup,
    /// Point git at the versioned hooks in `.githooks/`.
    InstallHooks,
}

/// What to do with a command once it exits successfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Expect {
    /// Exit status zero is enough.
    Success,
    /// Capture stdout as a cargo-llvm-cov JSON summary and apply the branch gate.
    BranchGate,
}

/// One command of a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Step {
    /// Program to run, resolved through `PATH`.
    pub(crate) program: &'static str,
    /// Arguments, in order.
    pub(crate) args: Vec<String>,
    /// Extra environment variables.
    pub(crate) env: Vec<(&'static str, &'static str)>,
    /// How the step is judged.
    pub(crate) expect: Expect,
}

impl Step {
    fn new(program: &'static str, args: &[&str]) -> Self {
        Self {
            program,
            args: args.iter().map(ToString::to_string).collect(),
            env: Vec::new(),
            expect: Expect::Success,
        }
    }

    fn cargo(args: &[&str]) -> Self {
        Self::new("cargo", args)
    }

    fn nightly_cargo(args: &[&str]) -> Self {
        let toolchain = format!("+{NIGHTLY}");
        let all = core::iter::once(toolchain.as_str()).chain(args.iter().copied());
        Self::new("cargo", &all.collect::<Vec<_>>())
    }

    fn with_env(self, key: &'static str, value: &'static str) -> Self {
        Self {
            env: [self.env, vec![(key, value)]].concat(),
            ..self
        }
    }

    fn gated(self) -> Self {
        Self {
            expect: Expect::BranchGate,
            ..self
        }
    }
}

/// The commands `task` runs, in order. Execution stops at the first failure.
pub(crate) fn steps(task: Task) -> Vec<Step> {
    match task {
        Task::Check => [static_gates(), e2e(), coverage()].concat(),
        Task::E2e => e2e(),
        Task::Coverage => coverage(),
        Task::Setup => vec![nightly_toolchain()],
        Task::InstallHooks => vec![Step::new("git", &["config", "core.hooksPath", ".githooks"])],
    }
}

fn static_gates() -> Vec<Step> {
    vec![
        Step::cargo(&["fmt", "--all", "--check"]),
        Step::cargo(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ]),
        Step::cargo(&["test", "--workspace", "--locked"]),
        Step::cargo(&["doc", "--workspace", "--no-deps", "--locked"])
            .with_env("RUSTDOCFLAGS", "-D warnings"),
        Step::cargo(&["deny", "check"]),
    ]
}

fn e2e() -> Vec<Step> {
    vec![
        Step::cargo(&[
            "build",
            "--release",
            "--locked",
            "--target",
            STATIC_TARGET,
            "-p",
            "agents-md-chkr",
        ]),
        Step::cargo(&[
            "test",
            "--locked",
            "-p",
            "agents-md-chkr",
            "--test",
            "container",
            "--",
            "--ignored",
        ]),
    ]
}

fn coverage() -> Vec<Step> {
    let scope = ["--workspace", "--exclude", "xtask"];
    vec![
        Step::nightly_cargo(&["llvm-cov", "clean", "--workspace"]),
        Step::nightly_cargo(
            &[
                &["llvm-cov", "--branch", "--locked", "--no-report"][..],
                &scope,
            ]
            .concat(),
        ),
        Step::nightly_cargo(&[
            "llvm-cov",
            "report",
            "--branch",
            "--lcov",
            "--output-path",
            "lcov.info",
        ]),
        Step::nightly_cargo(&[
            "llvm-cov",
            "report",
            "--branch",
            "--html",
            "--output-dir",
            "coverage",
        ]),
        Step::nightly_cargo(&["llvm-cov", "report", "--branch", "--summary-only"]),
        Step::nightly_cargo(&["llvm-cov", "report", "--branch", "--json", "--summary-only"])
            .gated(),
    ]
}

fn nightly_toolchain() -> Step {
    Step::new(
        "rustup",
        &[
            "toolchain",
            "install",
            NIGHTLY,
            "--profile",
            "minimal",
            "--component",
            "llvm-tools-preview",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::{Expect, NIGHTLY, STATIC_TARGET, Step, Task, steps};

    fn command_lines(task: Task) -> Vec<String> {
        steps(task)
            .iter()
            .map(|step| format!("{} {}", step.program, step.args.join(" ")))
            .collect()
    }

    #[test]
    fn check_runs_static_gates_before_coverage() {
        let lines = command_lines(Task::Check);
        assert_eq!(
            lines.first().map(String::as_str),
            Some("cargo fmt --all --check")
        );
        assert!(lines.iter().any(|line| line.ends_with("-- -D warnings")));
        assert!(lines.iter().any(|line| line == "cargo deny check"));
        assert_eq!(lines.len(), 13);
    }

    #[test]
    fn e2e_builds_the_static_binary_then_runs_the_ignored_container_tests() {
        let lines = command_lines(Task::E2e);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(&format!("--target {STATIC_TARGET}")));
        assert!(lines[1].ends_with("--test container -- --ignored"));
        let check = command_lines(Task::Check);
        let deny = check
            .iter()
            .position(|line| line == "cargo deny check")
            .unwrap();
        assert_eq!(check[deny + 1], lines[0]);
    }

    #[test]
    fn coverage_ends_with_the_branch_gate_on_the_pinned_nightly() {
        let all = steps(Task::Coverage);
        let gate = all.last().unwrap();
        assert_eq!(gate.expect, Expect::BranchGate);
        assert_eq!(gate.args.first().unwrap(), &format!("+{NIGHTLY}"));
        assert!(
            all.iter()
                .rev()
                .skip(1)
                .all(|step| step.expect == Expect::Success)
        );
        assert!(
            all.iter()
                .any(|step| step.args.contains(&"xtask".to_owned()))
        );
    }

    #[test]
    fn docs_deny_warnings_through_the_environment() {
        let docs: Vec<Step> = steps(Task::Check)
            .into_iter()
            .filter(|step| step.args.first().is_some_and(|arg| arg == "doc"))
            .collect();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].env, vec![("RUSTDOCFLAGS", "-D warnings")]);
    }

    #[test]
    fn setup_installs_only_the_coverage_nightly() {
        let setup = steps(Task::Setup);
        assert_eq!(setup.len(), 1);
        assert_eq!(setup[0].program, "rustup");
        assert!(setup[0].args.contains(&NIGHTLY.to_owned()));
    }

    #[test]
    fn install_hooks_sets_the_hooks_path() {
        assert_eq!(
            command_lines(Task::InstallHooks),
            vec!["git config core.hooksPath .githooks".to_owned()]
        );
    }
}
