//! Properties of the sealed blobs themselves.
//!
//! These read `.sealed/` directly rather than driving the CLI, so they are fast and have
//! no side effects on local progress.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn seals(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            seals(&path, out);
        } else if path.extension().is_some_and(|e| e == "seal") {
            out.push(path);
        }
    }
}

fn all_seals() -> Vec<PathBuf> {
    let mut found = Vec::new();
    seals(&repo_root().join(".sealed"), &mut found);
    found
}

#[test]
fn every_seal_unseals_and_carries_an_explanation() {
    for path in all_seals() {
        let blob = std::fs::read_to_string(&path).expect("read seal");
        let opened = underust_core::seal::unseal(&blob)
            .unwrap_or_else(|e| panic!("{} did not unseal: {e}", path.display()));
        assert!(
            !opened.explanation.trim().is_empty(),
            "{} has no explanation -- reveal would show a solution with no mechanism",
            path.display()
        );
        assert!(
            !opened.files.is_empty(),
            "{} seals no files",
            path.display()
        );
    }
}

#[test]
fn no_seal_is_readable_as_plaintext() {
    // The seal is obfuscation, not secrecy -- but it must at least survive being scrolled
    // past. If Rust source shows through, the encoding has silently stopped working.
    for path in all_seals() {
        let blob = std::fs::read_to_string(&path).expect("read seal");
        for giveaway in ["pub fn", "impl ", "use std::", "Weak", "RefCell"] {
            assert!(
                !blob.contains(giveaway),
                "{} leaks {giveaway:?} in plaintext",
                path.display()
            );
        }
    }
}

#[test]
fn seals_are_stored_one_per_task_id() {
    for path in all_seals() {
        let name = path.file_name().expect("file name").to_string_lossy();
        assert!(
            name.ends_with(".seal"),
            "unexpected file in .sealed/: {}",
            path.display()
        );
    }
}
