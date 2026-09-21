//! CI gates 3, 4 and 6.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, bail};
use underust_core::manifest::{self, Mode};
use underust_core::requires::{Resolution, resolve};
use underust_core::seal;

/// The outcome of gate 4.
#[derive(Debug, Default)]
pub struct ProveReport {
    /// Seals that were opened and whose solutions passed.
    pub proven: usize,
    /// Tasks deliberately not proven on this host, with a reason printed.
    pub skipped: usize,
    /// Task ids whose sealed solution failed.
    pub failures: Vec<String>,
}

/// Gate 3: every task stub compiles.
///
/// `todo!()` types as `!`, so a stub type-checks without build guards; this gate proves
/// that stays true.
///
/// # Errors
/// Fails when `cargo check` rejects the workspace.
pub fn ci_stubs(root: &Path) -> anyhow::Result<usize> {
    let tasks_dir = crate::repo::tasks_dir(root);
    let tasks = if tasks_dir.is_dir() {
        manifest::load_all(&tasks_dir)?
    } else {
        Vec::new()
    };
    let status = Command::new("cargo")
        .args(["check", "--workspace", "--all-targets"])
        .current_dir(root)
        .status()
        .context("running cargo check")?;
    if !status.success() {
        bail!("ci-stubs: cargo check failed");
    }
    println!("ci-stubs: checked {} tasks", tasks.len());
    Ok(tasks.len())
}

/// Gate 4: every sealed solution unseals and passes its own tests.
///
/// Skips are counted and printed. A gate that reports only its successes is how undergo
/// once claimed 174 proofs while performing three.
///
/// # Errors
/// Fails when any sealed solution fails, or when a seal cannot be opened.
pub fn prove(root: &Path) -> anyhow::Result<ProveReport> {
    let tasks_dir = crate::repo::tasks_dir(root);
    let tasks = if tasks_dir.is_dir() {
        manifest::load_all(&tasks_dir)?
    } else {
        Vec::new()
    };
    let host = crate::toolchain::probe_host();
    let mut report = ProveReport::default();

    for task in &tasks {
        let sealed_path = crate::repo::sealed_path(root, &task.id);
        if !sealed_path.is_file() {
            if task.mode == Mode::Predict {
                println!("  skip {} (predict tasks seal no answer)", task.id);
            } else {
                println!("  skip {} (no seal)", task.id);
            }
            report.skipped += 1;
            continue;
        }

        if let Resolution::Missing(gaps) = resolve(&task.requires, &host) {
            println!("  skip {} ({})", task.id, gaps[0].what);
            report.skipped += 1;
            continue;
        }

        let blob = std::fs::read_to_string(&sealed_path)?;
        let opened = seal::unseal(&blob).with_context(|| format!("unsealing {}", task.id))?;

        let outcome = {
            // The guard restores the task directory when this scope ends, including if
            // run_task_tests returns early -- the old explicit revert() did not.
            let _overlay = crate::overlay::Overlay::apply(&task.dir, &opened.files)
                .with_context(|| format!("overlaying the seal onto {}", task.dir.display()))?;
            crate::cargo::run_task_tests(root, task, false, false)
        };

        match outcome {
            Ok(outcome) if outcome.passed => {
                println!("  ok   {}", task.id);
                report.proven += 1;
            }
            Ok(failed) => {
                println!("  FAIL {}", task.id);
                for line in failed
                    .output
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .rev()
                    .take(12)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    println!("       {line}");
                }
                report.failures.push(task.id.clone());
            }
            Err(e) => {
                println!("  FAIL {} -- could not run: {e:#}", task.id);
                report.failures.push(task.id.clone());
            }
        }
    }

    println!(
        "prove: {} proven, {} skipped",
        report.proven, report.skipped
    );
    if !report.failures.is_empty() {
        bail!(
            "prove: {} sealed solution(s) failed: {:?}",
            report.failures.len(),
            report.failures
        );
    }
    Ok(report)
}

/// Gate 6: every invalidation in the radar names a task that exists.
///
/// # Errors
/// Fails when an invalidation names a task id that is not in the catalogue.
pub fn radar_check(root: &Path) -> anyhow::Result<usize> {
    let tasks_dir = crate::repo::tasks_dir(root);
    let known: BTreeSet<String> = if tasks_dir.is_dir() {
        manifest::load_all(&tasks_dir)?
            .into_iter()
            .map(|task| task.id)
            .collect()
    } else {
        BTreeSet::new()
    };

    let radar = root.join("radar/versions");
    if !radar.is_dir() {
        println!("radar-check: no radar yet");
        return Ok(0);
    }

    let mut checked = 0_usize;
    let mut bad = Vec::new();
    for entry in std::fs::read_dir(&radar)?.flatten() {
        let text = std::fs::read_to_string(entry.path())?;
        for line in text.lines() {
            let Some(rest) = line.split("-> invalidates |").nth(1) else {
                continue;
            };
            checked += 1;
            let id = rest.trim();
            if !known.contains(id) {
                bad.push(format!(
                    "{}: invalidates unknown task `{id}`",
                    entry.path().display()
                ));
            }
        }
    }
    if !bad.is_empty() {
        for problem in &bad {
            println!("  {problem}");
        }
        bail!(
            "radar-check: {} invalidation(s) name tasks that do not exist",
            bad.len()
        );
    }
    println!("radar-check: {checked} invalidation(s) ok");
    Ok(checked)
}
