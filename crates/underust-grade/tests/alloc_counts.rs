//! Integration test: an integration test file is its own binary root, which is the only
//! place a `#[global_allocator]` may be installed.

#[global_allocator]
static ALLOC: underust_grade::alloc::Counting = underust_grade::alloc::Counting;

use underust_grade::alloc::measure;

#[test]
fn counts_a_single_vec_allocation() {
    let (value, stats) = measure(|| {
        let v: Vec<u8> = Vec::with_capacity(64);
        v.capacity()
    });
    assert_eq!(value, 64);
    assert_eq!(
        stats.allocations, 1,
        "with_capacity allocates exactly once: {stats:?}"
    );
    assert!(stats.bytes_allocated >= 64);
}

#[test]
fn counts_zero_for_a_body_that_does_not_allocate() {
    let (value, stats) = measure(|| 2_u32 + 2);
    assert_eq!(value, 4);
    assert_eq!(stats.allocations, 0);
}

#[test]
fn counts_regrowth_as_further_allocations() {
    let (_, stats) = measure(|| {
        let mut v = Vec::new();
        for i in 0..64_u8 {
            v.push(i);
        }
        v.len()
    });
    assert!(
        stats.allocations > 1,
        "growing from empty reallocates: {stats:?}"
    );
}

#[test]
fn deltas_do_not_accumulate_between_measurements() {
    let (_, first) = measure(|| Vec::<u8>::with_capacity(16).capacity());
    let (_, second) = measure(|| Vec::<u8>::with_capacity(16).capacity());
    assert_eq!(
        first.allocations, second.allocations,
        "measure must return a delta"
    );
}
