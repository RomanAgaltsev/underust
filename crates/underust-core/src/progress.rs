//! Local, gitignored progress. Never reaches the etalon repository.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// What has happened to one task on this machine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Record {
    /// Rung one was taken.
    pub hinted: bool,
    /// Rung two was taken.
    pub revealed: bool,
    /// The reveal gate was overridden with `--stuck`.
    pub stuck: bool,
    /// The task's tests have passed at least once.
    pub passed: bool,
}

/// Every record on this machine, keyed by task id.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    #[serde(default)]
    records: BTreeMap<String, Record>,
}

/// Everything that can go wrong reading or writing progress.
#[derive(Debug, thiserror::Error)]
pub enum ProgressError {
    /// The file could not be read or written.
    #[error("progress file {path}: {source}")]
    Io {
        /// The file involved.
        path: PathBuf,
        /// The underlying failure.
        source: std::io::Error,
    },
    /// The file was not valid TOML.
    #[error("parsing progress: {0}")]
    Parse(#[from] toml::de::Error),
    /// The progress could not be serialised.
    #[error("writing progress: {0}")]
    Serialize(#[from] toml::ser::Error),
}

impl Progress {
    /// The record for `id`, defaulting to an all-false record.
    #[must_use]
    pub fn get(&self, id: &str) -> Record {
        self.records.get(id).copied().unwrap_or_default()
    }

    /// Note that rung one was taken.
    pub fn record_hint(&mut self, id: &str) {
        self.records.entry(id.to_owned()).or_default().hinted = true;
    }

    /// Note that rung two was taken, and whether the gate was overridden.
    pub fn record_reveal(&mut self, id: &str, stuck: bool) {
        let record = self.records.entry(id.to_owned()).or_default();
        record.revealed = true;
        record.stuck = stuck;
    }

    /// Note that the task's tests passed.
    pub fn record_pass(&mut self, id: &str) {
        self.records.entry(id.to_owned()).or_default().passed = true;
    }
}

/// Read progress from disk. A missing file yields an empty [`Progress`].
///
/// # Errors
/// Propagates read and parse failures other than "not found".
pub fn load(path: &Path) -> Result<Progress, ProgressError> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(toml::from_str(&text)?),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(Progress::default()),
        Err(source) => Err(ProgressError::Io {
            path: path.to_owned(),
            source,
        }),
    }
}

/// Write progress to disk, creating parent directories as needed.
///
/// # Errors
/// Propagates serialisation and write failures.
pub fn save(progress: &Progress, path: &Path) -> Result<(), ProgressError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ProgressError::Io {
            path: parent.to_owned(),
            source,
        })?;
    }
    let text = toml::to_string_pretty(progress)?;
    std::fs::write(path, text).map_err(|source| ProgressError::Io {
        path: path.to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("underust-progress-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("progress.toml")
    }

    #[test]
    fn a_missing_file_is_an_empty_progress_not_an_error() {
        let progress = load(&scratch("missing")).expect("missing file must be ok");
        assert_eq!(progress.get("drop/01-field-order"), Record::default());
    }

    #[test]
    fn records_survive_a_round_trip() {
        let path = scratch("roundtrip");
        let mut progress = Progress::default();
        progress.record_hint("drop/01-field-order");
        progress.record_pass("weak/01-break-the-cycle");
        save(&progress, &path).expect("save");

        let back = load(&path).expect("load");
        assert!(back.get("drop/01-field-order").hinted);
        assert!(back.get("weak/01-break-the-cycle").passed);
        assert!(!back.get("weak/01-break-the-cycle").hinted);
    }

    #[test]
    fn a_stuck_reveal_is_recorded_as_stuck() {
        let mut progress = Progress::default();
        progress.record_reveal("own/01-no-clone", true);
        let record = progress.get("own/01-no-clone");
        assert!(record.revealed);
        assert!(
            record.stuck,
            "--stuck must leave a mark; that is its whole point"
        );
    }

    #[test]
    fn an_earned_reveal_is_not_marked_stuck() {
        let mut progress = Progress::default();
        progress.record_reveal("own/01-no-clone", false);
        assert!(progress.get("own/01-no-clone").revealed);
        assert!(!progress.get("own/01-no-clone").stuck);
    }

    #[test]
    fn recording_a_hint_twice_does_not_clear_a_pass() {
        let mut progress = Progress::default();
        progress.record_pass("alloc/01-allocation-count");
        progress.record_hint("alloc/01-allocation-count");
        assert!(progress.get("alloc/01-allocation-count").passed);
        assert!(progress.get("alloc/01-allocation-count").hinted);
    }
}
