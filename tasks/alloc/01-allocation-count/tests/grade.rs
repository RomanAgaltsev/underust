//! The grader. Graded on allocation count, never on time: allocation *requests* are
//! deterministic and machine-independent, so this needs no warm-up, no statistics and no
//! tolerance.

#[global_allocator]
static ALLOC: underust_grade::alloc::Counting = underust_grade::alloc::Counting;

use task_alloc_01_allocation_count::{Entry, even_names};
use underust_grade::alloc::measure;

/// The most allocations a passing solution may make.
///
/// Calibrated by measurement, not guessed: the shipped version makes **11**, and the
/// reference solution makes **1**. Three is deliberately looser than the reference so a
/// straightforward single-buffer answer passes without having to size it up front.
const BUDGET: u64 = 3;

fn fixture() -> Vec<Entry> {
    ["ada", "grace", "alan", "edsger", "barbara"]
        .iter()
        .enumerate()
        .map(|(i, name)| Entry {
            name: (*name).to_owned(),
            score: u32::try_from(i).expect("fixture is small"),
        })
        .collect()
}

#[test]
fn behaviour_is_unchanged() {
    let entries = fixture();
    assert_eq!(even_names(&entries), "ADA, ALAN, BARBARA");
}

#[test]
fn handles_an_empty_input() {
    assert_eq!(even_names(&[]), "");
}

#[test]
fn handles_a_single_entry_without_a_separator() {
    let one = vec![Entry {
        name: "solo".to_owned(),
        score: 0,
    }];
    assert_eq!(even_names(&one), "SOLO");
}

#[test]
fn stays_within_the_allocation_budget() {
    let entries = fixture();
    // Warm the input's own allocations out of the measurement.
    let _ = even_names(&entries);

    let (_, stats) = measure(|| even_names(&entries));
    underust_grade::emit("allocations", &stats.allocations);
    assert!(
        stats.allocations <= BUDGET,
        "budget is {BUDGET} allocations, this made {}",
        stats.allocations
    );
}
