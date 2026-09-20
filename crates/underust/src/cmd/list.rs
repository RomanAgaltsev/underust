//! `underust list`.

use std::path::Path;

use anyhow::Context as _;
use underust_core::manifest::{self, Mode};

/// Print matching tasks, one per line.
///
/// # Errors
/// Fails when a manifest cannot be loaded.
pub fn run(root: &Path, track: Option<&str>, mode: Option<Mode>) -> anyhow::Result<()> {
    let tasks_dir = crate::repo::tasks_dir(root);
    if !tasks_dir.is_dir() {
        println!("no tasks yet");
        return Ok(());
    }
    let tasks = manifest::load_all(&tasks_dir).context("loading task manifests")?;

    for task in tasks {
        if track.is_some_and(|prefix| !task.track.starts_with(prefix)) {
            continue;
        }
        if mode.is_some_and(|wanted| task.mode != wanted) {
            continue;
        }
        println!(
            "{:<34} {:<10} {}",
            task.id,
            format!("{:?}", task.mode).to_lowercase(),
            task.title
        );
    }
    Ok(())
}
