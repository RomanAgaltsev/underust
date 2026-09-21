//! CI gates 3, 4 and 6.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
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

/// Files an overlay replaced or created, so they can be put back.
struct Overlay {
    replaced: Vec<(PathBuf, String)>,
    created: Vec<PathBuf>,
}

impl Overlay {
    fn apply(task_dir: &Path, files: &std::collections::BTreeMap<String, String>) -> Self {
        let mut overlay = Overlay {
            replaced: Vec::new(),
            created: Vec::new(),
        };
        for (name, contents) in files {
            let dest = task_dir.join(name);
            match std::fs::read_to_string(&dest) {
                Ok(original) => overlay.replaced.push((dest.clone(), original)),
                Err(_) => overlay.created.push(dest.clone()),
            }
            if let Some(parent) = dest.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&dest, contents);
        }
        overlay
    }

    /// Put the task directory back exactly as it was.
    ///
    /// Files the overlay created are deleted, not merely reverted -- otherwise a sealed
    /// solution that adds a file would leave it behind and the next run would grade a
    /// half-solved task.
    fn revert(self) {
        for (path, original) in self.replaced {
            let _ = std::fs::write(path, original);
        }
        for path in self.created {
            let _ = std::fs::remove_file(path);
        }
    }
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

        let overlay = Overlay::apply(&task.dir, &opened.files);
        let outcome = crate::cargo::run_task_tests(root, task, false, false);
        overlay.revert();

        match outcome {
            Ok(outcome) if outcome.passed => {
                println!("  ok   {}", task.id);
                report.proven += 1;
            }
            Ok(_) | Err(_) => {
                println!("  FAIL {}", task.id);
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
