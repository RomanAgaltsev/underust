//! Locating the repository root and the paths hanging off it.

use std::path::{Path, PathBuf};

use anyhow::Context as _;

/// Walk up from the current directory to the one holding `rust-toolchain.toml`.
///
/// # Errors
/// Fails when run outside an underust checkout.
pub fn root() -> anyhow::Result<PathBuf> {
    let start = std::env::current_dir().context("reading the current directory")?;
    let mut here: &Path = &start;
    loop {
        if here.join("rust-toolchain.toml").is_file() {
            return Ok(here.to_owned());
        }
        here = here
            .parent()
            .with_context(|| format!("{} is not inside an underust checkout", start.display()))?;
    }
}

/// Where task directories live.
#[must_use]
pub fn tasks_dir(root: &Path) -> PathBuf {
    root.join("tasks")
}

/// Where the gitignored solving workspace lives.
#[must_use]
pub fn work_dir(root: &Path, id: &str) -> PathBuf {
    root.join("work").join(id)
}

/// Where local progress is stored.
#[must_use]
pub fn progress_path(root: &Path) -> PathBuf {
    root.join(".underust").join("progress.toml")
}

/// Where sealed solutions live.
#[must_use]
pub fn sealed_path(root: &Path, id: &str) -> PathBuf {
    root.join(".sealed").join(format!("{id}.seal"))
}
