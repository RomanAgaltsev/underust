//! Resolving a task's `requires` against a host.
//!
//! The harness refuses to grade rather than grading wrong. Refusing is the design, not a
//! failure mode to be minimised -- a wrong grade is worse than no grade.

use crate::host::Host;
use crate::manifest::Requires;

/// One unmet requirement, and the command that closes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    /// What is missing, in human words.
    pub what: String,
    /// The exact command to run, or the flag to pass.
    pub remedy: String,
}

/// Whether a host can grade a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Everything the task needs is present.
    Satisfied,
    /// One entry per unmet requirement. Never truncated to the first.
    Missing(Vec<Gap>),
}

/// Compare a task's requirements against what the host provides.
///
/// Every unmet requirement is reported, not just the first: reporting one at a time forces
/// a solver through a round-trip per gap.
#[must_use]
pub fn resolve(requires: &Requires, host: &Host) -> Resolution {
    let mut gaps = Vec::new();

    if host.rustc.is_none() {
        gaps.push(Gap {
            what: "rustc is not on PATH".to_owned(),
            remedy: "install rustup from https://rustup.rs, then `rustup default 1.98.1`"
                .to_owned(),
        });
    }

    if requires.toolchain != "stable" && !host.has_toolchain(&requires.toolchain) {
        gaps.push(Gap {
            what: format!("toolchain `{}` is not installed", requires.toolchain),
            remedy: format!("rustup toolchain install {}", requires.toolchain),
        });
    }

    for component in &requires.components {
        if !host.has_component(&requires.toolchain, component) {
            gaps.push(Gap {
                what: format!("component `{component}` is not installed"),
                remedy: format!("rustup +{} component add {component}", requires.toolchain),
            });
        }
    }

    for tool in &requires.tools {
        if !host.tools.contains(tool) {
            gaps.push(Gap {
                what: format!("`{tool}` is not installed"),
                remedy: format!("cargo install {tool}"),
            });
        }
    }

    if requires.os != "any" && requires.os != host.os {
        gaps.push(Gap {
            what: format!(
                "this task requires os `{}`, host is `{}`",
                requires.os, host.os
            ),
            remedy: "re-run with `--docker` to use the canonical linux image".to_owned(),
        });
    }

    if requires.target != "any" && requires.target != host.target {
        gaps.push(Gap {
            what: format!(
                "this task requires target `{}`, host is `{}`",
                requires.target, host.target
            ),
            remedy: "re-run with `--docker` to use the canonical linux image".to_owned(),
        });
    }

    if gaps.is_empty() {
        Resolution::Satisfied
    } else {
        Resolution::Missing(gaps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Host;
    use crate::manifest::Requires;

    fn linux_host_with_stable() -> Host {
        Host {
            rustc: Some("1.98.1".to_owned()),
            cargo: Some("1.98.1".to_owned()),
            rustup: Some("1.29.1".to_owned()),
            toolchains: vec!["1.98.1-x86_64-unknown-linux-gnu".to_owned()],
            target: "x86_64-unknown-linux-gnu".to_owned(),
            os: "linux".to_owned(),
            ..Host::default()
        }
    }

    fn gaps_of(requires: &Requires, host: &Host) -> Vec<Gap> {
        match resolve(requires, host) {
            Resolution::Missing(gaps) => gaps,
            Resolution::Satisfied => panic!("expected this fixture to be unsatisfiable"),
        }
    }

    #[test]
    fn a_permissive_task_is_satisfied_by_a_plain_stable_host() {
        assert_eq!(
            resolve(&Requires::default(), &linux_host_with_stable()),
            Resolution::Satisfied
        );
    }

    #[test]
    fn a_host_with_no_rust_at_all_reports_that_first() {
        let gaps = gaps_of(&Requires::default(), &Host::default());
        assert!(gaps.iter().any(|gap| gap.what.contains("rustc")));
        assert!(gaps.iter().any(|gap| gap.remedy.contains("rustup")));
    }

    #[test]
    fn missing_nightly_names_the_install_command() {
        let requires = Requires {
            toolchain: "nightly".to_owned(),
            ..Requires::default()
        };
        let gaps = gaps_of(&requires, &linux_host_with_stable());
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].remedy, "rustup toolchain install nightly");
    }

    #[test]
    fn missing_miri_names_the_component_command() {
        let requires = Requires {
            toolchain: "nightly".to_owned(),
            components: vec!["miri".to_owned()],
            ..Requires::default()
        };
        let mut host = linux_host_with_stable();
        host.toolchains
            .push("nightly-x86_64-unknown-linux-gnu".to_owned());
        let gaps = gaps_of(&requires, &host);
        assert_eq!(gaps[0].remedy, "rustup +nightly component add miri");
    }

    #[test]
    fn an_installed_component_satisfies_despite_its_triple_suffix() {
        let requires = Requires {
            toolchain: "nightly".to_owned(),
            components: vec!["miri".to_owned()],
            ..Requires::default()
        };
        let mut host = linux_host_with_stable();
        host.toolchains
            .push("nightly-x86_64-unknown-linux-gnu".to_owned());
        host.components.insert(
            "nightly-x86_64-unknown-linux-gnu".to_owned(),
            ["miri-x86_64-unknown-linux-gnu".to_owned()]
                .into_iter()
                .collect(),
        );
        assert_eq!(resolve(&requires, &host), Resolution::Satisfied);
    }

    #[test]
    fn missing_cargo_subcommand_names_the_install_command() {
        let requires = Requires {
            tools: vec!["cargo-expand".to_owned()],
            ..Requires::default()
        };
        let gaps = gaps_of(&requires, &linux_host_with_stable());
        assert_eq!(gaps[0].remedy, "cargo install cargo-expand");
    }

    #[test]
    fn a_wrong_os_is_a_gap_docker_can_close() {
        let requires = Requires {
            os: "linux".to_owned(),
            ..Requires::default()
        };
        let mut host = linux_host_with_stable();
        host.os = "windows".to_owned();
        let gaps = gaps_of(&requires, &host);
        assert!(gaps[0].remedy.contains("--docker"));
    }

    #[test]
    fn several_gaps_are_all_reported_not_just_the_first() {
        let requires = Requires {
            toolchain: "nightly".to_owned(),
            tools: vec!["cargo-expand".to_owned(), "cargo-asm".to_owned()],
            ..Requires::default()
        };
        let gaps = gaps_of(&requires, &linux_host_with_stable());
        assert_eq!(gaps.len(), 3, "one gap per unmet requirement: {gaps:?}");
    }
}
