//! `underust test`.

use std::path::Path;

use anyhow::{Context as _, bail};
use underust_core::forbids;
use underust_core::manifest::{self, Mode, Task};
use underust_core::progress;
use underust_core::requires::{Resolution, resolve};

/// Exit code used when the host cannot satisfy a task's requirements.
///
/// Distinct from a test failure's 1 so a caller can tell "wrong" from "cannot know".
pub const REFUSED: i32 = 2;

/// Modes whose grading is judgement-based, and whose content lands in M6.
///
/// Returns the message to fail with, or `None` when the mode is gradeable now.
#[must_use]
pub fn deferred_to_m6(mode: Mode) -> Option<&'static str> {
    match mode {
        Mode::Review => {
            Some("review tasks are graded against a rubric by a human; content lands in M6")
        }
        Mode::Design => {
            Some("design tasks are graded against a checklist by a human; content lands in M6")
        }
        _ => None,
    }
}

/// Find one task by id.
///
/// # Errors
/// Fails when no task carries that id.
pub fn find(root: &Path, id: &str) -> anyhow::Result<Task> {
    let tasks_dir = crate::repo::tasks_dir(root);
    if !tasks_dir.is_dir() {
        bail!("no task with id `{id}` -- this repository has no tasks yet");
    }
    let tasks = manifest::load_all(&tasks_dir).context("loading task manifests")?;
    tasks
        .into_iter()
        .find(|task| task.id == id)
        .with_context(|| format!("no task with id `{id}` -- try `underust list`"))
}

/// Grade one task. Returns whether it passed.
///
/// # Errors
/// Fails when the task is unknown, its mode is not gradeable yet, a child process cannot
/// run, or a CONSTRAIN ban was broken. An unsatisfiable requirement is **not** an error:
/// it prints remedies and exits [`REFUSED`].
pub fn run(root: &Path, id: &str, docker: bool) -> anyhow::Result<bool> {
    let task = find(root, id)?;

    if let Some(why) = deferred_to_m6(task.mode) {
        bail!("{id}: {why}");
    }

    if task.mode == Mode::Predict {
        return crate::cmd::predict::grade(root, id, docker);
    }

    if !docker {
        let host = crate::toolchain::probe_host();
        if let Resolution::Missing(gaps) = resolve(&task.requires, &host) {
            println!("refusing to grade {id} -- this host cannot satisfy it:");
            for gap in &gaps {
                println!("  {}", gap.what);
                println!("      fix: {}", gap.remedy);
            }
            println!("\na wrong grade is worse than no grade.");
            std::process::exit(REFUSED);
        }
    }

    let outcome = crate::cargo::run_task_tests(root, &task, docker, true)?;
    print!("{}", outcome.output);

    if task.mode == Mode::Constrain {
        let work = crate::repo::work_dir(root, &task.id);
        if work.is_dir() {
            let violations = forbids::check_dir(&work, &task.forbids)
                .with_context(|| format!("checking forbidden constructs for {id}"))?;
            if !violations.is_empty() {
                for violation in &violations {
                    println!(
                        "forbidden construct `{}` used at line {}",
                        violation.construct, violation.line
                    );
                }
                bail!("{id}: the ban was broken; compiling is not enough for a constrain task");
            }
        }
    }

    if outcome.passed {
        let path = crate::repo::progress_path(root);
        let mut store = progress::load(&path)?;
        store.record_pass(&task.id);
        progress::save(&store, &path)?;
        println!("PASS {id}");
    } else {
        println!("FAIL {id}");
    }
    Ok(outcome.passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_and_design_are_deferred() {
        assert!(deferred_to_m6(Mode::Review).is_some());
        assert!(deferred_to_m6(Mode::Design).is_some());
    }

    #[test]
    fn the_five_instrument_graded_modes_are_not_deferred() {
        for mode in [
            Mode::Build,
            Mode::Predict,
            Mode::Optimize,
            Mode::Constrain,
            Mode::Soundness,
        ] {
            assert!(
                deferred_to_m6(mode).is_none(),
                "{mode:?} must be gradeable in M0.5"
            );
        }
    }

    #[test]
    fn refused_is_distinct_from_a_test_failure() {
        assert_ne!(REFUSED, 0);
        assert_ne!(REFUSED, 1, "a refusal must be tellable from a wrong answer");
    }
}
