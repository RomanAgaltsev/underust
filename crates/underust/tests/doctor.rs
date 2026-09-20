//! `doctor` reports; it never fails. A machine with nothing installed is exactly the
//! machine that needs its output.

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

fn doctor() -> String {
    let out = Command::new(bin())
        .args(["doctor"])
        .current_dir(repo_root())
        .output()
        .expect("run underust doctor");
    assert!(
        out.status.success(),
        "doctor must succeed even when things are missing"
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn reports_the_toolchain_it_is_running_under() {
    let stdout = doctor();
    assert!(stdout.contains("rustc"), "got: {stdout}");
    assert!(stdout.contains("host target"), "got: {stdout}");
}

#[test]
fn reports_whether_a_linker_driver_is_present() {
    // Spec 9.4: "no cc" is a link error whose text explains nothing to someone learning
    // Rust, so doctor must name it explicitly rather than leave it to cargo.
    assert!(doctor().contains("linker"));
}

#[test]
fn reports_a_tracks_section() {
    assert!(doctor().contains("tracks"));
}

#[test]
fn exits_zero_on_a_machine_missing_everything() {
    let out = Command::new(bin())
        .args(["doctor"])
        .env("PATH", "")
        .current_dir(repo_root())
        .output()
        .expect("run underust doctor with an empty PATH");
    assert!(
        out.status.success(),
        "doctor must not fail on the machine that needs it most"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("NOT FOUND"), "got: {stdout}");
    assert!(
        stdout.contains("rustup.rs"),
        "it must say how to fix it: {stdout}"
    );
}
