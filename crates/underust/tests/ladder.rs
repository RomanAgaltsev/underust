//! Ladder behaviour that holds with zero tasks.
//!
//! The gate itself -- reveal refusing while tests fail, and --stuck overriding it -- needs
//! a real task with a real seal; Task 18 adds those assertions.

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
fn progress_succeeds_and_says_something() {
    let out = run(&["progress"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!String::from_utf8_lossy(&out.stdout).trim().is_empty());
}

#[test]
fn hint_on_an_unknown_task_names_the_id() {
    let out = run(&["hint", "no/such-task"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("no/such-task"));
}

#[test]
fn reveal_on_an_unknown_task_names_the_id() {
    let out = run(&["reveal", "no/such-task"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("no/such-task"));
}

#[test]
fn reveal_accepts_the_stuck_flag() {
    // The flag must exist before Task 18 can prove it overrides the gate. A typo here
    // would otherwise surface as "unexpected argument" much later.
    let out = run(&["reveal", "no/such-task", "--stuck"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("unexpected argument"), "got: {stderr}");
}

#[test]
fn the_ladder_verbs_are_all_reachable() {
    let help = String::from_utf8_lossy(&run(&["--help"]).stdout).into_owned();
    for verb in ["hint", "reveal", "progress"] {
        assert!(help.contains(verb), "{verb} must appear in --help: {help}");
    }
}
