//! Invoking cargo and docker. The only module that spawns build tooling.

use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, bail};
use underust_core::manifest::{Mode, Task};
use underust_core::measure::{Measurement, parse_measurements};

/// The canonical image used by `--docker`.
pub const IMAGE: &str = "rust:1.98.1";

/// What happened when a task's tests ran.
#[derive(Debug)]
pub struct Outcome {
    /// Whether every test passed.
    pub passed: bool,
    /// Captured output, including any `UNDERUST-MEASURE` lines.
    pub output: String,
    /// Measurements the task reported over the line protocol.
    pub measurements: Vec<Measurement>,
}

/// The cargo package name for a task, derived from its id.
#[must_use]
pub fn package_name(task: &Task) -> String {
    format!("task-{}", task.id.replace('/', "-"))
}

/// Run one task's graded tests.
///
/// Passes `--nocapture --test-threads=1`: without `--nocapture` the measurement lines are
/// swallowed, and without single-threading the counting allocator's deltas are unreliable.
/// SOUNDNESS tasks run under `cargo +nightly miri test` instead, because miri's verdict is
/// the grade.
///
/// When `use_work` is set, anything in `work/<id>/` is laid over the task directory for
/// the duration of the run and removed afterwards. That is how R9 is honoured: the solver
/// edits `work/`, the task directory stays pristine, and cargo still sees a normal crate.
/// `prove` passes `false`, because it is grading the sealed solution and must not pick up
/// whatever the local solver happens to have in progress.
///
/// # Errors
/// Fails when the child process cannot be spawned, or a required tool is absent.
pub fn run_task_tests(
    root: &Path,
    task: &Task,
    docker: bool,
    use_work: bool,
) -> anyhow::Result<Outcome> {
    let package = package_name(task);

    // Held for the whole function: dropping it restores the task directory, including
    // when a test fails or this function returns early with an error.
    let _overlay = if use_work {
        let work = crate::repo::work_dir(root, &task.id);
        let files = crate::overlay::read_tree(&work)
            .with_context(|| format!("reading {}", work.display()))?;
        crate::overlay::Overlay::apply(&task.dir, &files)
            .with_context(|| format!("overlaying work onto {}", task.dir.display()))?
    } else {
        crate::overlay::Overlay::none()
    };

    let mut command = if docker {
        let mut docker_cmd = Command::new("docker");
        docker_cmd.args([
            "run",
            "--rm",
            "-v",
            &format!("{}:/w", root.display()),
            "-w",
            "/w",
            "-e",
            "CARGO_TARGET_DIR=/tmp/target",
            IMAGE,
            "cargo",
        ]);
        docker_cmd
    } else {
        Command::new("cargo")
    };

    if task.mode == Mode::Soundness {
        command.args(["+nightly", "miri", "test", "-p", &package]);
    } else {
        if task.requires.toolchain != "stable" {
            command.arg(format!("+{}", task.requires.toolchain));
        }
        command.args(["test", "-p", &package]);
    }
    command.args(["--", "--nocapture", "--test-threads=1"]);
    command.current_dir(root);

    let out = command
        .output()
        .with_context(|| format!("running graded tests for {}", task.id))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    if !out.status.success() && stderr.contains("no such command") {
        bail!("required tool missing while grading {}: {stderr}", task.id);
    }

    Ok(Outcome {
        passed: out.status.success(),
        output: format!("{stdout}{stderr}"),
        measurements: parse_measurements(&stdout),
    })
}
