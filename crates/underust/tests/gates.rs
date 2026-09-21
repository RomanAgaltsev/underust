//! Gates 3, 4 and 6.
//!
//! Gate 4's real work -- unsealing a solution, overlaying it, running it, reverting --
//! needs a sealed task; Task 18 adds that. What is asserted here is the property that
//! made undergo's equivalent gate vacuous: it must report its skips.

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
fn ci_stubs_reports_how_many_tasks_it_checked() {
    let out = run(&["ci-stubs"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("checked"));
}

#[test]
fn prove_reports_skips_as_well_as_proofs() {
    // undergo shipped a gate announcing "all 174 reference solutions proven" while it had
    // silently skipped 171. A gate that prints only its successes is not a gate.
    let out = run(&["prove"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("proven"), "got: {stdout}");
    assert!(
        stdout.contains("skipped"),
        "a gate that hides its skips is vacuous: {stdout}"
    );
}

#[test]
fn radar_check_passes_on_the_repo_as_committed() {
    let out = run(&["radar-check"]);
    assert!(
        out.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn radar_check_fails_when_an_invalidation_names_a_missing_task() {
    let path = repo_root().join("radar/versions/zz-test-dangling.md");
    std::fs::write(&path, "- x\n  -> invalidates | no/such-task\n").expect("write");
    let out = run(&["radar-check"]);
    let _ = std::fs::remove_file(&path);
    assert!(
        !out.status.success(),
        "an invalidation naming a ghost task must fail the gate"
    );
}
