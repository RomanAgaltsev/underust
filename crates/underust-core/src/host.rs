//! What the current machine can actually do.
//!
//! The parsers here are pure so they can be tested without a toolchain present. The CLI
//! is responsible for running the commands and handing their output in.

use std::collections::{BTreeMap, BTreeSet};

/// A snapshot of the host's Rust tooling.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Host {
    /// `rustc` version, if `rustc` is on PATH.
    pub rustc: Option<String>,
    /// `cargo` version, if present.
    pub cargo: Option<String>,
    /// `rustup` version, if present.
    pub rustup: Option<String>,
    /// Installed toolchain names, default marker stripped.
    pub toolchains: Vec<String>,
    /// Installed rustup components, keyed by the toolchain they belong to.
    ///
    /// Per-toolchain because that is how rustup stores them: `miri` is installed on
    /// nightly while the repository pins stable, so a single flat set would report miri
    /// missing on a machine that has it.
    pub components: BTreeMap<String, BTreeSet<String>>,
    /// Installed cargo subcommands, e.g. `cargo-expand`.
    pub tools: BTreeSet<String>,
    /// Whether a usable `docker` was found.
    pub docker: bool,
    /// The host target triple, e.g. `x86_64-unknown-linux-gnu`.
    pub target: String,
    /// `linux`, `windows` or `macos`.
    pub os: String,
}

impl Host {
    /// Whether any installed toolchain starts with `name`.
    ///
    /// Toolchain names carry their triple (`nightly-x86_64-unknown-linux-gnu`), so a
    /// prefix match is what callers mean by "do I have nightly".
    #[must_use]
    pub fn has_toolchain(&self, name: &str) -> bool {
        self.toolchains
            .iter()
            .any(|installed| installed.starts_with(name))
    }

    /// Whether `toolchain` has a component whose name starts with `name`.
    ///
    /// `rustup component list --installed` prints `miri-x86_64-unknown-linux-gnu`, so an
    /// exact match would never fire. The toolchain is matched by prefix too, since
    /// installed names carry their triple.
    #[must_use]
    pub fn has_component(&self, toolchain: &str, name: &str) -> bool {
        self.components
            .iter()
            .filter(|(installed_toolchain, _)| installed_toolchain.starts_with(toolchain))
            .any(|(_, components)| components.iter().any(|c| c.starts_with(name)))
    }
}

/// Extract `(version, host_triple)` from `rustc -vV` output.
#[must_use]
pub fn parse_rustc_verbose(output: &str) -> (Option<String>, Option<String>) {
    let field = |key: &str| {
        output
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .map(|rest| rest.trim().to_owned())
    };
    (field("release:"), field("host:"))
}

/// Parse `rustup toolchain list`, stripping the ` (default)` marker.
#[must_use]
pub fn parse_toolchain_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().unwrap_or(line).to_owned())
        .collect()
}

/// The OS this binary was compiled for, in the vocabulary `task.toml` uses.
#[must_use]
pub const fn current_os() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUSTC_VV: &str = "\
rustc 1.98.1 (a1b2c3d4e 2026-09-03)
binary: rustc
commit-hash: a1b2c3d4e5f6
commit-date: 2026-09-03
host: x86_64-unknown-linux-gnu
release: 1.98.1
LLVM version: 20.1.0
";

    const TOOLCHAIN_LIST: &str = "\
1.98.1-x86_64-unknown-linux-gnu (default)
nightly-x86_64-unknown-linux-gnu
";

    #[test]
    fn reads_version_and_host_triple_from_rustc_vv() {
        let (version, triple) = parse_rustc_verbose(RUSTC_VV);
        assert_eq!(version.as_deref(), Some("1.98.1"));
        assert_eq!(triple.as_deref(), Some("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn returns_none_for_unparseable_rustc_output() {
        let (version, triple) = parse_rustc_verbose("command not found");
        assert!(version.is_none());
        assert!(triple.is_none());
    }

    #[test]
    fn strips_the_default_marker_from_toolchain_names() {
        let list = parse_toolchain_list(TOOLCHAIN_LIST);
        assert_eq!(
            list,
            vec![
                "1.98.1-x86_64-unknown-linux-gnu".to_string(),
                "nightly-x86_64-unknown-linux-gnu".to_string(),
            ]
        );
    }

    #[test]
    fn host_knows_whether_it_has_a_named_toolchain() {
        let host = Host {
            toolchains: parse_toolchain_list(TOOLCHAIN_LIST),
            ..Host::default()
        };
        assert!(host.has_toolchain("nightly"));
        assert!(host.has_toolchain("1.98.1"));
        assert!(!host.has_toolchain("1.85.0"));
    }

    #[test]
    fn a_bare_host_has_nothing() {
        let host = Host::default();
        assert!(!host.has_toolchain("stable"));
        assert!(host.rustc.is_none());
        assert!(!host.docker);
    }

    fn host_with_nightly_miri() -> Host {
        let mut components = BTreeMap::new();
        components.insert(
            "nightly-x86_64-unknown-linux-gnu".to_owned(),
            ["miri-x86_64-unknown-linux-gnu".to_owned()]
                .into_iter()
                .collect::<BTreeSet<_>>(),
        );
        components.insert(
            "1.98.1-x86_64-unknown-linux-gnu".to_owned(),
            ["clippy-x86_64-unknown-linux-gnu".to_owned()]
                .into_iter()
                .collect::<BTreeSet<_>>(),
        );
        Host {
            components,
            ..Host::default()
        }
    }

    #[test]
    fn components_match_by_prefix_because_rustup_appends_the_triple() {
        let host = host_with_nightly_miri();
        assert!(host.has_component("nightly", "miri"));
        assert!(!host.has_component("nightly", "rust-src"));
    }

    #[test]
    fn a_component_on_one_toolchain_is_not_on_another() {
        // The bug this replaced: `rustup component list --installed` reports the ACTIVE
        // toolchain, which rust-toolchain.toml pins to stable. miri lives on nightly, so
        // a flat set reported it missing on a machine that had it.
        let host = host_with_nightly_miri();
        assert!(host.has_component("nightly", "miri"));
        assert!(
            !host.has_component("1.98.1", "miri"),
            "miri is installed on nightly, not on the pinned stable"
        );
        assert!(host.has_component("1.98.1", "clippy"));
    }
}
