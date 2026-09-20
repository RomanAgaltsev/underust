//! `underust doctor` -- what can this machine grade, and how do I fix what it cannot?

use std::collections::BTreeMap;
use std::path::Path;

use underust_core::manifest;
use underust_core::requires::{Resolution, resolve};

/// Report the host's capabilities and every actionable gap.
///
/// Always exits successfully. A machine with nothing installed is exactly the machine that
/// needs this output, so failing there would be perverse.
///
/// # Errors
/// Fails only when a manifest is unreadable, which `validate` is the real gate for.
pub fn run(root: &Path) -> anyhow::Result<()> {
    let host = crate::toolchain::probe_host();
    let (linker, linker_remedy) = crate::toolchain::linker_driver();

    println!("host");
    println!(
        "  rustc        {}",
        host.rustc.as_deref().unwrap_or("NOT FOUND")
    );
    println!(
        "  cargo        {}",
        host.cargo.as_deref().unwrap_or("NOT FOUND")
    );
    println!(
        "  rustup       {}",
        host.rustup.as_deref().unwrap_or("NOT FOUND")
    );
    println!(
        "  host target  {}",
        if host.target.is_empty() {
            "unknown"
        } else {
            &host.target
        }
    );
    println!("  os           {}", host.os);
    println!(
        "  linker       {}",
        linker.as_deref().unwrap_or("NOT FOUND")
    );
    println!("  docker       {}", if host.docker { "yes" } else { "no" });
    println!(
        "  toolchains   {}",
        if host.toolchains.is_empty() {
            "none".to_owned()
        } else {
            host.toolchains.join(", ")
        }
    );

    if host.rustc.is_none() {
        println!("\nnothing can be graded here yet.");
        println!("  fix: install rustup from https://rustup.rs, then `rustup default 1.98.1`");
        return Ok(());
    }
    if linker.is_none() {
        println!("\nno C linker driver: cargo can resolve and check, but cannot link.");
        println!("  fix: {linker_remedy}");
    }

    let tasks_dir = crate::repo::tasks_dir(root);
    if !tasks_dir.is_dir() {
        println!("\ntracks\n  no tasks yet");
        return Ok(());
    }
    let tasks = manifest::load_all(&tasks_dir)?;
    if tasks.is_empty() {
        println!("\ntracks\n  no tasks yet");
        return Ok(());
    }

    let mut per_track: BTreeMap<String, (usize, usize, Vec<String>)> = BTreeMap::new();
    for task in &tasks {
        let entry = per_track.entry(task.track.clone()).or_default();
        entry.1 += 1;
        match resolve(&task.requires, &host) {
            Resolution::Satisfied => entry.0 += 1,
            Resolution::Missing(gaps) => {
                for gap in gaps {
                    if !entry.2.contains(&gap.remedy) {
                        entry.2.push(gap.remedy);
                    }
                }
            }
        }
    }

    println!("\ntracks");
    let mut all_clear = true;
    for (track, (ok, total, remedies)) in &per_track {
        println!("  {track:<12} {ok}/{total} gradeable");
        for remedy in remedies {
            all_clear = false;
            println!("      fix: {remedy}");
        }
    }
    if all_clear {
        println!("\nall tracks gradeable on this machine");
    }
    Ok(())
}
