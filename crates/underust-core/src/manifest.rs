//! The `task.toml` schema and its loader.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// What kind of exercise a task is, and therefore how it is graded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Implement the thing; graded by its tests passing.
    Build,
    /// Commit a guess, then measure. Stores no answer.
    Predict,
    /// Reduce a deterministic cost under a correctness floor.
    Optimize,
    /// Find the planted defect and write the review.
    Review,
    /// Produce a design under constraints.
    Design,
    /// Make it compile without the escape hatch.
    Constrain,
    /// Find the undefined behaviour; miri decides.
    Soundness,
}

/// Whether a task's answer is guaranteed by the language or merely observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnswerStability {
    /// Guaranteed. May be asserted flatly.
    Invariant,
    /// Toolchain- or platform-dependent. Must pin `requires` and say so.
    Observed,
}

/// What a host must provide before a task may be graded.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Requires {
    /// `stable`, `nightly`, or an exact version such as `1.85.0`.
    pub toolchain: String,
    /// rustup components, e.g. `miri`.
    pub components: Vec<String>,
    /// cargo subcommands, e.g. `cargo-expand`.
    pub tools: Vec<String>,
    /// `any`, `linux`, `windows` or `macos`.
    pub os: String,
    /// `any` or an exact target triple.
    pub target: String,
}

impl Default for Requires {
    fn default() -> Self {
        Self {
            toolchain: "stable".to_owned(),
            components: Vec::new(),
            tools: Vec::new(),
            os: "any".to_owned(),
            target: "any".to_owned(),
        }
    }
}

/// One task, as declared by its `task.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    /// Permanent identifier, `track/NN-slug`. Never changes once committed.
    pub id: String,
    /// The track this task belongs to. Must equal `track_of(&id)`.
    pub track: String,
    /// How the task is graded.
    pub mode: Mode,
    /// One-line human title.
    pub title: String,
    /// Points awarded on a pass.
    pub points: u32,
    /// Host requirements; permissive when absent.
    #[serde(default)]
    pub requires: Requires,
    /// CONSTRAIN only: constructs the solution may not use.
    #[serde(default)]
    pub forbids: Vec<String>,
    /// Whether the answer is guaranteed or observed.
    #[serde(default = "default_stability")]
    pub answer_stability: AnswerStability,
    /// Rung one of the reveal ladder. Always available.
    pub hint: String,
    /// PREDICT only: the measurement names the solver must predict.
    #[serde(default)]
    pub predict: Vec<String>,
    /// Attribution when the idea was borrowed. Code never is.
    #[serde(default)]
    pub inspired_by: String,
    /// SOUNDNESS only: a digest of `tests/grade.rs`, so a solver cannot pass by weakening
    /// the tests. Empty means unchecked.
    #[serde(default)]
    pub test_digest: String,
    /// Directory the manifest was loaded from. Not present in the file.
    #[serde(skip)]
    pub dir: PathBuf,
}

fn default_stability() -> AnswerStability {
    AnswerStability::Invariant
}

/// Everything that can go wrong loading a manifest.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// The file could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The file involved.
        path: PathBuf,
        /// The underlying failure.
        source: std::io::Error,
    },
    /// The file was not valid TOML, or did not match the schema.
    #[error("parsing {path}: {source}")]
    Parse {
        /// The file involved.
        path: PathBuf,
        /// The underlying failure.
        source: toml::de::Error,
    },
    /// The manifest parsed but is internally inconsistent.
    #[error("{id}: {message}")]
    Invalid {
        /// The offending task id.
        id: String,
        /// What is wrong.
        message: String,
    },
}

/// A task id's track is the id minus its final segment.
#[must_use]
pub fn track_of(id: &str) -> &str {
    match id.rfind('/') {
        Some(cut) => &id[..cut],
        None => id,
    }
}

impl Task {
    /// Check the invariants that TOML parsing alone cannot express.
    ///
    /// # Errors
    /// Returns [`ManifestError::Invalid`] when the declared track disagrees with the id,
    /// when a PREDICT task names no measurements, or when a CONSTRAIN task forbids nothing.
    pub fn validate(&self) -> Result<(), ManifestError> {
        let bad = |message: String| ManifestError::Invalid {
            id: self.id.clone(),
            message,
        };

        if self.track != track_of(&self.id) {
            return Err(bad(format!(
                "declared track {:?} but the id implies track {:?}",
                self.track,
                track_of(&self.id)
            )));
        }
        if self.mode == Mode::Predict && self.predict.is_empty() {
            return Err(bad(
                "a predict task must name at least one measurement".to_owned()
            ));
        }
        if self.mode == Mode::Constrain && self.forbids.is_empty() {
            return Err(bad(
                "a constrain task must forbid at least one construct".to_owned()
            ));
        }
        if self.mode == Mode::Soundness && self.test_digest.trim().is_empty() {
            return Err(bad(
                "a soundness task must pin test_digest, or it can be passed by deleting the tests"
                    .to_owned(),
            ));
        }
        if self.hint.trim().is_empty() {
            return Err(bad(
                "hint must not be empty -- it is rung one of the ladder".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Load and validate one `task.toml`.
///
/// # Errors
/// Propagates read, parse and validation failures.
pub fn load(path: &Path) -> Result<Task, ManifestError> {
    let text = std::fs::read_to_string(path).map_err(|source| ManifestError::Io {
        path: path.to_owned(),
        source,
    })?;
    let mut task: Task = toml::from_str(&text).map_err(|source| ManifestError::Parse {
        path: path.to_owned(),
        source,
    })?;
    task.dir = path.parent().unwrap_or(Path::new(".")).to_owned();
    task.validate()?;
    Ok(task)
}

/// Load every `task.toml` beneath `tasks_root`, sorted by id.
///
/// # Errors
/// Propagates the first failure encountered.
pub fn load_all(tasks_root: &Path) -> Result<Vec<Task>, ManifestError> {
    let mut found = Vec::new();
    collect(tasks_root, &mut found)?;
    found.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(found)
}

fn collect(dir: &Path, out: &mut Vec<Task>) -> Result<(), ManifestError> {
    let manifest = dir.join("task.toml");
    if manifest.is_file() {
        out.push(load(&manifest)?);
        return Ok(());
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(source) => {
            return Err(ManifestError::Io {
                path: dir.to_owned(),
                source,
            });
        }
    };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            collect(&entry.path(), out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
id = "drop/01-field-order"
track = "drop"
mode = "predict"
title = "Which field drops first?"
points = 10
forbids = []
answer_stability = "invariant"
hint = "Fields and locals disagree about direction."
predict = ["order"]
inspired_by = ""

[requires]
toolchain = "stable"
components = []
tools = []
os = "any"
target = "any"
"#;

    #[test]
    fn parses_a_predict_task() {
        let task: Task = toml::from_str(SAMPLE).expect("sample must parse");
        assert_eq!(task.id, "drop/01-field-order");
        assert_eq!(task.mode, Mode::Predict);
        assert_eq!(task.answer_stability, AnswerStability::Invariant);
        assert_eq!(task.predict, vec!["order".to_string()]);
    }

    #[test]
    fn track_is_the_id_minus_its_final_segment() {
        assert_eq!(track_of("drop/01-field-order"), "drop");
        assert_eq!(
            track_of("review/concurrency/01-counter"),
            "review/concurrency"
        );
    }

    #[test]
    fn rejects_a_task_whose_track_disagrees_with_its_id() {
        let bad = SAMPLE.replace(r#"track = "drop""#, r#"track = "alloc""#);
        let task: Task = toml::from_str(&bad).unwrap();
        let err = task
            .validate()
            .expect_err("track/id disagreement must be rejected");
        assert!(err.to_string().contains("track"), "got: {err}");
    }

    #[test]
    fn rejects_a_soundness_task_with_no_test_digest() {
        let bad = SAMPLE
            .replace("mode = \"predict\"", "mode = \"soundness\"")
            .replace("predict = [\"order\"]", "predict = []");
        let task: Task = toml::from_str(&bad).unwrap();
        let err = task
            .validate()
            .expect_err("an unpinned soundness task must be rejected");
        assert!(err.to_string().contains("test_digest"), "got: {err}");
    }

    #[test]
    fn rejects_an_unknown_mode() {
        let bad = SAMPLE.replace(r#"mode = "predict""#, r#"mode = "transcend""#);
        assert!(toml::from_str::<Task>(&bad).is_err());
    }

    #[test]
    fn defaults_requires_to_permissive_when_absent() {
        let minimal = r#"
id = "weak/01-break-the-cycle"
track = "weak"
mode = "build"
title = "Break the cycle"
points = 10
hint = "Weak does not keep anything alive."
"#;
        let task: Task = toml::from_str(minimal).expect("requires must be optional");
        assert_eq!(task.requires.toolchain, "stable");
        assert_eq!(task.requires.os, "any");
        assert!(task.forbids.is_empty());
    }
}
