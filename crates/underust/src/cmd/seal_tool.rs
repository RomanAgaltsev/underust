//! `underust seal` -- author-side only. Turns a solution directory into a sealed blob.
//!
//! The source directory lives outside the repository and is deleted afterwards; `.sealed/`
//! holds the only copy. Nothing here is needed to *solve* a task.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context as _, bail};
use underust_core::seal;

/// Seal `files/` plus `EXPLANATION.md` from `source` into `.sealed/<id>.seal`.
///
/// # Errors
/// Fails when the source directory, its explanation, or its files are missing.
pub fn run(root: &Path, id: &str, source: &Path) -> anyhow::Result<()> {
    let explanation_path = source.join("EXPLANATION.md");
    let explanation = std::fs::read_to_string(&explanation_path)
        .with_context(|| format!("reading {}", explanation_path.display()))?;

    let files_root = source.join("files");
    if !files_root.is_dir() {
        bail!("{} must contain a files/ directory", source.display());
    }

    let mut files = BTreeMap::new();
    let mut stack = vec![files_root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path
                    .strip_prefix(&files_root)?
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(rel, std::fs::read_to_string(&path)?);
            }
        }
    }
    if files.is_empty() {
        bail!("{} contains no files to seal", files_root.display());
    }

    let blob = seal::seal(&seal::Sealed { explanation, files })?;
    let dest = crate::repo::sealed_path(root, id);
    std::fs::create_dir_all(dest.parent().expect("sealed path has a parent"))?;
    std::fs::write(&dest, format!("{blob}\n"))?;
    println!("sealed {id} -> {}", dest.display());
    Ok(())
}
