//! `underust validate` -- CI gate 5.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context as _, bail};
use underust_core::manifest;

/// Validate every manifest, returning how many were checked.
///
/// Beyond per-task validation this enforces two repository-wide invariants: ids are unique,
/// and a task's declared id matches the directory it lives in.
///
/// # Errors
/// Fails on the first inconsistency found.
pub fn run(root: &Path) -> anyhow::Result<usize> {
    let tasks_dir = crate::repo::tasks_dir(root);
    if !tasks_dir.is_dir() {
        return Ok(0);
    }
    let tasks = manifest::load_all(&tasks_dir).context("loading task manifests")?;

    let mut seen = BTreeSet::new();
    for task in &tasks {
        if !seen.insert(task.id.clone()) {
            bail!("duplicate task id: {}", task.id);
        }
        let expected = tasks_dir.join(&task.id);
        if task.dir.canonicalize().ok() != expected.canonicalize().ok() {
            bail!(
                "task {} declares an id that does not match its directory {}",
                task.id,
                task.dir.display()
            );
        }
    }
    Ok(tasks.len())
}
