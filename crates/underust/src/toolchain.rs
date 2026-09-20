//! Running the external tools, and turning their absence into data.

use std::collections::BTreeSet;
use std::process::Command;

use underust_core::host::{self, Host};

/// Run a command and return its stdout, or `None` if it could not run or failed.
fn capture(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The C linker driver rustc needs, and how to install it on this platform.
///
/// Not optional and not a C *dependency*: every target needs a platform linker, and on
/// linux-gnu rustc drives `cc` and links crt objects from libc6-dev. Without it the only
/// symptom is a link error that teaches a Rust beginner nothing.
#[must_use]
pub fn linker_driver() -> (Option<String>, &'static str) {
    let found = capture("cc", &["--version"])
        .or_else(|| capture("clang", &["--version"]))
        .and_then(|out| out.lines().next().map(str::to_owned));
    let remedy = if cfg!(target_os = "linux") {
        "sudo apt-get install -y build-essential"
    } else if cfg!(target_os = "macos") {
        "xcode-select --install"
    } else {
        "install the MSVC build tools and the Windows SDK"
    };
    (found, remedy)
}

/// Inspect the machine. Never fails: a missing tool is a `None`, not an error.
#[must_use]
pub fn probe_host() -> Host {
    let rustc_vv = capture("rustc", &["-vV"]).unwrap_or_default();
    let (rustc, target) = host::parse_rustc_verbose(&rustc_vv);

    let toolchains = capture("rustup", &["toolchain", "list"])
        .map(|out| host::parse_toolchain_list(&out))
        .unwrap_or_default();

    let components = capture("rustup", &["component", "list", "--installed"])
        .map(|out| {
            out.lines()
                .map(|line| line.trim().to_owned())
                .filter(|line| !line.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut tools = BTreeSet::new();
    for tool in ["cargo-expand", "cargo-asm", "cargo-bloat", "cargo-nextest"] {
        if capture(tool, &["--version"]).is_some() {
            tools.insert(tool.to_owned());
        }
    }

    Host {
        rustc,
        cargo: capture("cargo", &["--version"]).map(|s| s.trim().to_owned()),
        rustup: capture("rustup", &["--version"]).map(|s| s.trim().to_owned()),
        toolchains,
        components,
        tools,
        docker: capture("docker", &["version", "--format", "{{.Server.Os}}"]).is_some(),
        target: target.unwrap_or_default(),
        os: host::current_os().to_owned(),
    }
}
