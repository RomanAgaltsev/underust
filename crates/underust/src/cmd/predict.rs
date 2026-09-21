//! `underust predict` -- scaffolding and grading a prediction.
//!
//! Nothing is sealed. The solver writes `prediction.toml`; the task computes reality at
//! run time and emits it; the harness diffs. The author of a PREDICT task never has to
//! know the answer, which is why these tasks can be written before oxide is complete.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use serde_json::Value;
use underust_core::manifest::Mode;
use underust_core::measure::diff;

/// Convert a parsed TOML table into the JSON values measurements use.
///
/// Measurements arrive as JSON over the line protocol, predictions as TOML on disk, so
/// one side has to be converted before they can be compared.
fn toml_table_to_json(table: &toml::Table) -> BTreeMap<String, Value> {
    table
        .iter()
        .map(|(key, value)| {
            let json = serde_json::to_value(value).unwrap_or(Value::Null);
            (key.clone(), json)
        })
        .collect()
}

/// Write a blank `prediction.toml` with one key per declared measurement.
///
/// # Errors
/// Fails when the task is not a PREDICT task, or when a prediction already exists.
pub fn scaffold(root: &Path, id: &str) -> anyhow::Result<PathBuf> {
    let task = crate::cmd::test::find(root, id)?;
    if task.mode != Mode::Predict {
        bail!("{id} is a {:?} task, not a predict task", task.mode);
    }

    let dir = crate::repo::work_dir(root, id);
    let path = dir.join("prediction.toml");
    if path.exists() {
        bail!(
            "{} already exists -- edit it, or delete it to start over",
            path.display()
        );
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    let mut text = String::from(
        "# Predict what the program will do, THEN run `underust test`.\n\
         # Values are TOML. Replace each placeholder with your guess.\n\n",
    );
    for name in &task.predict {
        text.push_str(&format!("{name} = [] # your prediction here\n"));
    }
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Read a prediction from disk.
///
/// # Errors
/// Fails when the file is missing or is not a TOML table.
fn read_prediction(path: &Path) -> anyhow::Result<BTreeMap<String, Value>> {
    let text = std::fs::read_to_string(path).with_context(|| {
        format!(
            "{} not found -- run `underust predict <id>` first",
            path.display()
        )
    })?;
    let parsed: toml::Table =
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    Ok(toml_table_to_json(&parsed))
}

fn render(value: Option<&Value>) -> String {
    value.map_or_else(|| "(nothing)".to_owned(), ToString::to_string)
}

/// Grade a prediction against reality.
///
/// # Errors
/// Fails when the task is unknown, not a PREDICT task, or has no prediction on disk.
pub fn grade(root: &Path, id: &str, docker: bool) -> anyhow::Result<bool> {
    let task = crate::cmd::test::find(root, id)?;
    let predicted = read_prediction(&crate::repo::work_dir(root, id).join("prediction.toml"))?;
    let outcome = crate::cargo::run_task_tests(root, &task, docker, true)?;

    if !outcome.passed {
        print!("{}", outcome.output);
        bail!("{id}: the task's own program failed to run; this is a task bug, not a wrong guess");
    }

    let mismatches = diff(&predicted, &outcome.measurements);
    if mismatches.is_empty() {
        // Recorded here rather than in `cmd::test::run`: predict returns before that
        // function's record_pass, so without this a correct prediction stayed "open" in
        // `progress` and scored nothing.
        let path = crate::repo::progress_path(root);
        let mut store = underust_core::progress::load(&path)?;
        store.record_pass(id);
        underust_core::progress::save(&store, &path)?;

        println!("PASS {id} -- prediction matched reality");
        return Ok(true);
    }
    for mismatch in &mismatches {
        println!("  {}", mismatch.name);
        println!("    you said : {}", render(mismatch.predicted.as_ref()));
        println!("    reality  : {}", render(mismatch.actual.as_ref()));
    }
    println!("FAIL {id} -- {} measurement(s) disagreed", mismatches.len());
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn converts_a_toml_array_into_a_json_array() {
        let table: toml::Table = toml::from_str(r#"order = ["b", "a"]"#).expect("parse");
        let converted = toml_table_to_json(&table);
        assert_eq!(converted["order"], json!(["b", "a"]));
    }

    #[test]
    fn converts_a_toml_integer_into_a_json_number() {
        let table: toml::Table = toml::from_str("allocations = 3").expect("parse");
        assert_eq!(toml_table_to_json(&table)["allocations"], json!(3));
    }

    #[test]
    fn a_missing_prediction_names_the_command_that_creates_one() {
        let err = read_prediction(Path::new("/nonexistent/prediction.toml"))
            .expect_err("a missing prediction must fail");
        let text = format!("{err:#}");
        assert!(text.contains("underust predict"), "got: {text}");
    }

    #[test]
    fn renders_a_missing_side_readably() {
        assert_eq!(render(None), "(nothing)");
        assert_eq!(render(Some(&json!(3))), "3");
    }
}
