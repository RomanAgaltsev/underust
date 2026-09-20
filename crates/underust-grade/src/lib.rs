//! Graders that task crates link against.
//!
//! Each grader turns a question into a measurement. A task reports what it measured with
//! [`emit`], and the harness reads those lines back -- so the answer is computed by an
//! instrument at run time, never asserted by whoever wrote the task.

pub mod alloc;
pub mod drop_log;
pub mod rc;

use serde::Serialize;

/// Report one measured value to the harness.
///
/// Prints a single line the harness recognises. Run graded tests with `--nocapture`, which
/// the harness does, or nothing is seen.
///
/// # Panics
/// Panics if `value` cannot be serialised as JSON, which is a bug in the task.
pub fn emit<T: Serialize + ?Sized>(name: &str, value: &T) {
    let json = serde_json::to_string(value).expect("a measurement must be serialisable");
    println!("{} {name} {json}", underust_core::measure::MARKER);
}
