//! `underust hint`, `underust reveal` and `underust progress`.
//!
//! The seal is obfuscation, not secrecy: anyone can base64-decode the blobs by hand. The
//! gate exists so the easy path is the honest one, not to stop a determined reader.

use std::path::Path;

use anyhow::{Context as _, bail};
use underust_core::manifest::{self, Mode};
use underust_core::{progress, seal};

/// Rung one. Always available, never gated, recorded.
///
/// # Errors
/// Fails when the task id is unknown.
pub fn hint(root: &Path, id: &str) -> anyhow::Result<()> {
    let task = crate::cmd::test::find(root, id)?;
    println!("{}", task.hint.trim());

    let path = crate::repo::progress_path(root);
    let mut store = progress::load(&path)?;
    store.record_hint(id);
    progress::save(&store, &path)?;
    Ok(())
}

/// Rung two. Refuses while the task's tests fail, unless `stuck` is set.
///
/// # Errors
/// Fails when the task is unknown, has no seal, or is unsolved and `stuck` is not set.
pub fn reveal(root: &Path, id: &str, stuck: bool) -> anyhow::Result<()> {
    let task = crate::cmd::test::find(root, id)?;

    let sealed_path = crate::repo::sealed_path(root, id);
    if !sealed_path.is_file() {
        if task.mode == Mode::Predict {
            bail!("{id} is a predict task -- it seals no answer. Run: underust test {id}");
        }
        bail!("no seal at {}", sealed_path.display());
    }

    if !stuck {
        let outcome = crate::cargo::run_task_tests(root, &task, false, true)?;
        if !outcome.passed {
            println!("refusing to reveal {id} -- its tests do not pass yet.");
            println!("  take rung one first:  underust hint {id}");
            println!("  or override and record it:  underust reveal {id} --stuck");
            std::process::exit(1);
        }
    }

    let blob = std::fs::read_to_string(&sealed_path)
        .with_context(|| format!("reading {}", sealed_path.display()))?;
    let opened = seal::unseal(&blob).with_context(|| format!("unsealing {id}"))?;

    println!("=== {id} -- explanation ===\n");
    println!("{}", opened.explanation.trim());
    for (name, contents) in &opened.files {
        println!("\n=== {name} ===\n");
        println!("{contents}");
    }
    if stuck {
        println!("\n(revealed with --stuck; recorded in .underust/progress.toml)");
    }

    let path = crate::repo::progress_path(root);
    let mut store = progress::load(&path)?;
    store.record_reveal(id, stuck);
    progress::save(&store, &path)?;
    Ok(())
}

/// Print what has happened on this machine, task by task.
///
/// A stuck reveal is shown distinctly from an earned one. The record exists to be looked
/// at; hiding the difference would make recording it pointless.
///
/// # Errors
/// Fails when a manifest or the progress file cannot be read.
pub fn show(root: &Path) -> anyhow::Result<()> {
    let tasks_dir = crate::repo::tasks_dir(root);
    let tasks = if tasks_dir.is_dir() {
        manifest::load_all(&tasks_dir)?
    } else {
        Vec::new()
    };
    let store = progress::load(&crate::repo::progress_path(root))?;

    if tasks.is_empty() {
        println!("no tasks yet");
        return Ok(());
    }

    let mut passed = 0_usize;
    let mut points = 0_u32;
    println!("{:<34} {:<8} {:<7} reveal", "task", "status", "hint");
    for task in &tasks {
        let record = store.get(&task.id);
        if record.passed {
            passed += 1;
            points += task.points;
        }
        let reveal = match (record.revealed, record.stuck) {
            (true, true) => "stuck",
            (true, false) => "earned",
            (false, _) => "-",
        };
        println!(
            "{:<34} {:<8} {:<7} {}",
            task.id,
            if record.passed { "passed" } else { "open" },
            if record.hinted { "taken" } else { "-" },
            reveal
        );
    }
    println!("\n{passed}/{} passed, {points} points", tasks.len());
    Ok(())
}
