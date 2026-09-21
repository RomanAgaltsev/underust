//! Temporarily overlaying files onto a task directory, and putting them back.
//!
//! Task directories stay pristine (R9): solving happens in the gitignored `work/`, and a
//! sealed reference solution lives in `.sealed/`. Neither is where `cargo` looks, so both
//! have to be laid over the task directory for the duration of a run and removed again.
//!
//! Restoration is a `Drop` impl rather than a call at the end of the happy path, because
//! the interesting cases are the ones that do not reach the end: a failing test, a panic,
//! an interrupted run. A task directory left holding someone's half-finished solution
//! would make the next grade a lie.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Files laid over a task directory, restored when this value drops.
#[derive(Debug)]
pub struct Overlay {
    /// For each touched path: what was there before, or `None` if nothing was.
    saved: Vec<(PathBuf, Option<String>)>,
}

impl Overlay {
    /// Lay `files` (paths relative to `task_dir`) over the task directory.
    ///
    /// # Errors
    /// Fails when a file cannot be read or written.
    pub fn apply(task_dir: &Path, files: &BTreeMap<String, String>) -> std::io::Result<Self> {
        let mut saved = Vec::new();
        for (relative, contents) in files {
            let dest = task_dir.join(relative);
            let previous = match std::fs::read_to_string(&dest) {
                Ok(text) => Some(text),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e),
            };
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&dest, contents)?;
            saved.push((dest, previous));
        }
        Ok(Self { saved })
    }

    /// Nothing to lay over: an empty overlay that restores nothing.
    #[must_use]
    pub fn none() -> Self {
        Self { saved: Vec::new() }
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        for (path, previous) in &self.saved {
            let restored = match previous {
                Some(text) => std::fs::write(path, text),
                None => std::fs::remove_file(path),
            };
            if let Err(e) = restored {
                // Deliberately loud. A task directory left dirty makes every later grade
                // untrustworthy, and silence here would hide that.
                eprintln!(
                    "WARNING: could not restore {} after grading: {e}",
                    path.display()
                );
            }
        }
    }
}

/// Read every file under `dir` into a map keyed by path relative to `dir`.
///
/// Returns an empty map when `dir` does not exist, so callers can treat "no work yet" and
/// "nothing to overlay" alike.
///
/// # Errors
/// Fails when a file cannot be read.
pub fn read_tree(dir: &Path) -> std::io::Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    let mut stack = vec![dir.to_owned()];
    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs" || e == "toml") {
                let relative = path
                    .strip_prefix(dir)
                    .expect("walked from dir")
                    .to_string_lossy()
                    .replace('\\', "/");
                // A prediction is input to the harness, not source to compile.
                if relative == "prediction.toml" {
                    continue;
                }
                files.insert(relative, std::fs::read_to_string(&path)?);
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("underust-overlay-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn restores_the_original_contents_on_drop() {
        let dir = scratch("restore");
        std::fs::write(dir.join("a.rs"), "original").expect("write");

        {
            let files = [("a.rs".to_owned(), "overlaid".to_owned())]
                .into_iter()
                .collect();
            let _guard = Overlay::apply(&dir, &files).expect("apply");
            assert_eq!(
                std::fs::read_to_string(dir.join("a.rs")).expect("read"),
                "overlaid"
            );
        }

        assert_eq!(
            std::fs::read_to_string(dir.join("a.rs")).expect("read"),
            "original",
            "the task directory must be pristine again"
        );
    }

    #[test]
    fn removes_files_that_did_not_exist_before() {
        let dir = scratch("remove");
        {
            let files = [("new.rs".to_owned(), "added".to_owned())]
                .into_iter()
                .collect();
            let _guard = Overlay::apply(&dir, &files).expect("apply");
            assert!(dir.join("new.rs").exists());
        }
        assert!(
            !dir.join("new.rs").exists(),
            "an overlay must not leave files behind"
        );
    }

    #[test]
    fn restores_even_when_the_scope_unwinds() {
        let dir = scratch("panic");
        std::fs::write(dir.join("a.rs"), "original").expect("write");

        let result = std::panic::catch_unwind(|| {
            let files = [("a.rs".to_owned(), "overlaid".to_owned())]
                .into_iter()
                .collect();
            let _guard = Overlay::apply(&dir, &files).expect("apply");
            panic!("a failing test unwinds exactly like this");
        });

        assert!(result.is_err());
        assert_eq!(
            std::fs::read_to_string(dir.join("a.rs")).expect("read"),
            "original",
            "restoration must survive a panic, which is the common case"
        );
    }

    #[test]
    fn read_tree_is_empty_for_a_missing_directory() {
        let missing = std::env::temp_dir().join("underust-overlay-nope");
        let _ = std::fs::remove_dir_all(&missing);
        assert!(read_tree(&missing).expect("read_tree").is_empty());
    }

    #[test]
    fn read_tree_skips_a_prediction_file() {
        let dir = scratch("prediction");
        std::fs::write(dir.join("prediction.toml"), "order = []").expect("write");
        std::fs::write(dir.join("lib.rs"), "fn main() {}").expect("write");
        let files = read_tree(&dir).expect("read_tree");
        assert!(files.contains_key("lib.rs"));
        assert!(
            !files.contains_key("prediction.toml"),
            "a prediction is harness input, not source to compile"
        );
    }
}
