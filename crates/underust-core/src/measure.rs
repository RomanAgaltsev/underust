//! The one-line protocol a graded task uses to report reality.
//!
//! A task prints `UNDERUST-MEASURE <name> <json>` for each thing it measured. The harness
//! reads those lines out of `cargo test -- --nocapture` output. Nothing here parses Rust.

use std::collections::BTreeMap;

use serde_json::Value;

/// The line prefix that marks a measurement.
pub const MARKER: &str = "UNDERUST-MEASURE";

/// One value a task measured at run time.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    /// The measurement's name, matching an entry in the task's `predict` list.
    pub name: String,
    /// The measured value.
    pub value: Value,
}

/// One disagreement between a prediction and reality.
#[derive(Debug, Clone, PartialEq)]
pub struct Mismatch {
    /// The measurement's name.
    pub name: String,
    /// What the solver said, if they said anything.
    pub predicted: Option<Value>,
    /// What actually happened, if it was measured.
    pub actual: Option<Value>,
}

/// Extract every well-formed measurement from captured stdout.
///
/// A marked line whose payload is not valid JSON is dropped rather than failing the run:
/// a malformed measurement is a task bug, and the task's own tests are what catch it.
#[must_use]
pub fn parse_measurements(stdout: &str) -> Vec<Measurement> {
    stdout
        .lines()
        // The marker is searched for anywhere in the line, not just at its start.
        // libtest prints `test NAME ... ` without a newline and only then flushes the
        // test's own output under --nocapture, so a measurement routinely lands mid-line:
        //     test report_the_drop_order ... UNDERUST-MEASURE order ["a","b"]
        .filter_map(|line| line.find(MARKER).map(|at| &line[at + MARKER.len()..]))
        .filter_map(|rest| {
            let rest = rest.trim_start();
            let (name, payload) = rest.split_once(char::is_whitespace)?;
            let value = serde_json::from_str(payload.trim()).ok()?;
            Some(Measurement {
                name: name.to_owned(),
                value,
            })
        })
        .collect()
}

/// Compare a prediction against what was measured.
///
/// Every name on either side is considered, so an unpredicted measurement and an
/// unmeasured prediction are both reported.
#[must_use]
pub fn diff(predicted: &BTreeMap<String, Value>, actual: &[Measurement]) -> Vec<Mismatch> {
    let measured: BTreeMap<&str, &Value> =
        actual.iter().map(|m| (m.name.as_str(), &m.value)).collect();

    let mut names: Vec<&str> = predicted.keys().map(String::as_str).collect();
    names.extend(measured.keys().copied());
    names.sort_unstable();
    names.dedup();

    names
        .into_iter()
        .filter_map(|name| {
            let left = predicted.get(name);
            let right = measured.get(name).copied();
            if left == right {
                return None;
            }
            Some(Mismatch {
                name: name.to_owned(),
                predicted: left.cloned(),
                actual: right.cloned(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const OUTPUT: &str = r#"
running 1 test
UNDERUST-MEASURE order ["b","a","c"]
UNDERUST-MEASURE allocations 3
test grade::drop_order ... ok
"#;

    #[test]
    fn takes_only_marked_lines() {
        let found = parse_measurements(OUTPUT);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "order");
        assert_eq!(found[0].value, json!(["b", "a", "c"]));
        assert_eq!(found[1].value, json!(3));
    }

    #[test]
    fn finds_a_measurement_libtest_appended_to_its_own_status_line() {
        // Real captured output. libtest writes "test NAME ... " with no newline, so the
        // marker is not at column 0. Six unit tests passed on idealised input while the
        // real harness silently found nothing.
        const REAL: &str = "\
running 1 test
test report_the_drop_order ... UNDERUST-MEASURE order [\"loose_b\",\"first\"]
ok
";
        let found = parse_measurements(REAL);
        assert_eq!(
            found.len(),
            1,
            "the marker is not always at the start of a line"
        );
        assert_eq!(found[0].name, "order");
        assert_eq!(found[0].value, json!(["loose_b", "first"]));
    }

    #[test]
    fn ignores_a_marked_line_with_unparseable_json() {
        let found = parse_measurements("UNDERUST-MEASURE broken {not json}\n");
        assert!(
            found.is_empty(),
            "a malformed measurement is dropped, not panicked on"
        );
    }

    #[test]
    fn an_exact_match_produces_no_mismatches() {
        let predicted = [("order".to_owned(), json!(["b", "a", "c"]))]
            .into_iter()
            .collect();
        assert!(diff(&predicted, &parse_measurements(OUTPUT)[..1]).is_empty());
    }

    #[test]
    fn a_wrong_prediction_reports_both_sides() {
        let predicted = [("order".to_owned(), json!(["a", "b", "c"]))]
            .into_iter()
            .collect();
        let mismatches = diff(&predicted, &parse_measurements(OUTPUT)[..1]);
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].predicted, Some(json!(["a", "b", "c"])));
        assert_eq!(mismatches[0].actual, Some(json!(["b", "a", "c"])));
    }

    #[test]
    fn a_measurement_the_solver_did_not_predict_is_a_mismatch() {
        let predicted = std::collections::BTreeMap::new();
        let mismatches = diff(&predicted, &parse_measurements(OUTPUT));
        assert_eq!(mismatches.len(), 2);
        assert!(mismatches.iter().all(|m| m.predicted.is_none()));
    }

    #[test]
    fn a_prediction_with_no_measurement_is_a_mismatch() {
        let predicted = [("ghost".to_owned(), json!(1))].into_iter().collect();
        let mismatches = diff(&predicted, &[]);
        assert_eq!(mismatches.len(), 1);
        assert!(mismatches[0].actual.is_none());
    }
}
