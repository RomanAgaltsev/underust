//! `underust leak-check` -- CI gate 7.
//!
//! undergo discovered that `go vet` *names the planted defect* in a review drill, and its
//! public CI log would have printed it. `clippy` behaves identically. Gate 2 keeps the
//! linter away from those paths; this gate proves the prevention held.
//!
//! Written as a subcommand rather than a shell script because the Windows CI leg is the
//! only thing standing behind R14, and it has no bash.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use underust_core::seal;

/// The shortest token considered distinctive enough to be worth searching for.
///
/// Short identifiers (`len`, `ptr`, `mid`) appear in ordinary build output constantly and
/// would make the gate cry wolf on every run.
const MIN_TOKEN: usize = 12;

fn collect_seals(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_seals(&path, out);
        } else if path.extension().is_some_and(|e| e == "seal") {
            out.push(path);
        }
    }
}

/// Distinctive tokens from one sealed blob: long identifiers and long source lines.
fn tokens_of(sealed: &seal::Sealed) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();

    let mut harvest = |text: &str| {
        for raw in text.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
            if raw.len() >= MIN_TOKEN {
                tokens.insert(raw.to_owned());
            }
        }
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.len() >= 24 && !trimmed.starts_with("//") && !trimmed.starts_with('#') {
                tokens.insert(trimmed.to_owned());
            }
        }
    };

    harvest(&sealed.explanation);
    for contents in sealed.files.values() {
        harvest(contents);
    }
    tokens
}

/// Search `log` for any sealed text.
///
/// # Errors
/// Fails when the log cannot be read, a seal cannot be opened, or any sealed text appears.
pub fn run(root: &Path, log: &Path) -> anyhow::Result<()> {
    let haystack =
        std::fs::read_to_string(log).with_context(|| format!("reading {}", log.display()))?;

    let mut seals = Vec::new();
    collect_seals(&root.join(".sealed"), &mut seals);
    if seals.is_empty() {
        println!("leak-check: no seals to check");
        return Ok(());
    }

    let mut leaks = Vec::new();
    for path in &seals {
        let blob =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let opened =
            seal::unseal(&blob).with_context(|| format!("unsealing {}", path.display()))?;
        for token in tokens_of(&opened) {
            if haystack.contains(&token) {
                leaks.push(format!("{}: {token}", path.display()));
            }
        }
    }

    if !leaks.is_empty() {
        for leak in leaks.iter().take(20) {
            println!("  LEAK {leak}");
        }
        bail!(
            "leak-check: {} sealed string(s) appear in {}",
            leaks.len(),
            log.display()
        );
    }
    println!(
        "leak-check: {} seal(s) checked, nothing leaked into {}",
        seals.len(),
        log.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sealed() -> seal::Sealed {
        let mut files = BTreeMap::new();
        files.insert(
            "src/lib.rs".to_owned(),
            "pub fn distinctive_helper_name() -> u8 { 42 }\n".to_owned(),
        );
        seal::Sealed {
            explanation: "The mechanism is deterministic reallocation.\n".to_owned(),
            files,
        }
    }

    #[test]
    fn harvests_long_identifiers() {
        let tokens = tokens_of(&sealed());
        assert!(tokens.contains("distinctive_helper_name"));
    }

    #[test]
    fn ignores_short_identifiers_that_appear_everywhere() {
        let tokens = tokens_of(&sealed());
        for noise in ["pub", "fn", "u8"] {
            assert!(
                !tokens.contains(noise),
                "{noise:?} would make the gate cry wolf on every build"
            );
        }
    }

    #[test]
    fn harvests_whole_source_lines() {
        let tokens = tokens_of(&sealed());
        assert!(tokens.contains("pub fn distinctive_helper_name() -> u8 { 42 }"));
    }
}
