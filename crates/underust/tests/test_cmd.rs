//! `underust test` behaviour that holds with zero tasks.
//!
//! The refusal path (exit 2 when `requires` is unmet) and the CONSTRAIN ban need real
//! tasks to exercise; Tasks 20 and 21 add those assertions.

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
fn an_unknown_task_id_is_an_error_that_names_the_id() {
    let out = run(&["test", "no/such-task"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no/such-task"),
        "the error must name the id: {stderr}"
    );
}

#[test]
fn an_unknown_task_exits_one_not_two() {
    // Exit 2 means "cannot grade"; exit 1 means "did not pass". A typo in an id is
    // neither -- it is a plain error, and conflating them would make the refusal signal
    // useless to a script.
    let out = run(&["test", "no/such-task"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn the_harness_never_panics_on_a_bad_id() {
    let out = run(&["test", "///"]);
    assert_ne!(out.status.code(), Some(101), "101 is a Rust panic");
}
