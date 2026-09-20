//! CLI behaviour that holds regardless of how many tasks exist.
//!
//! Task-specific assertions (that `drop/01-field-order` is listed, that `validate` counts
//! five manifests) arrive in Task 17, when there are tasks to assert about. Committing
//! tests that are known to be red until a later task would normalise a red suite, which is
//! the habit this repository exists to break.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_underust"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("run underust")
}

#[test]
fn list_succeeds_and_says_something() {
    let out = run(&["list"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!String::from_utf8_lossy(&out.stdout).trim().is_empty());
}

#[test]
fn validate_succeeds_on_the_repo_as_committed() {
    let out = run(&["validate"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("manifests ok"));
}

#[test]
fn an_unknown_subcommand_exits_nonzero() {
    let out = run(&["transcend"]);
    assert!(!out.status.success());
}

#[test]
fn an_unknown_mode_filter_is_rejected() {
    let out = run(&["list", "--mode", "transcend"]);
    assert!(
        !out.status.success(),
        "clap must reject a mode that does not exist"
    );
}

#[test]
fn every_real_mode_is_accepted_by_the_filter() {
    for mode in [
        "build",
        "predict",
        "optimize",
        "review",
        "design",
        "constrain",
        "soundness",
    ] {
        let out = run(&["list", "--mode", mode]);
        assert!(out.status.success(), "mode {mode} must be a valid filter");
    }
}

#[test]
fn running_outside_a_checkout_is_a_clear_error() {
    let out = Command::new(bin())
        .args(["list"])
        .current_dir(std::env::temp_dir())
        .output()
        .expect("run underust");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("underust checkout"), "got: {stderr}");
}
