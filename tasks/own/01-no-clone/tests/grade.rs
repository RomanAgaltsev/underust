//! The grader. The pointer test is belt and braces alongside the `forbids` ban: cloning
//! would fail these tests even if the ban were lifted, and the ban stops a solver reaching
//! the same place by some other copy-shaped route.

use task_own_01_no_clone::Ring;

fn ring() -> Ring {
    Ring::new(vec![
        "oldest".to_owned(),
        "middle".to_owned(),
        "newest".to_owned(),
    ])
}

#[test]
fn returns_the_oldest_buffer() {
    let mut ring = ring();
    assert_eq!(ring.rotate(), "oldest");
}

#[test]
fn the_vacated_slot_is_left_empty_and_moves_to_the_back() {
    let mut ring = ring();
    let _ = ring.rotate();
    assert_eq!(ring.slots.len(), 3);
    assert_eq!(ring.slots[0], "middle");
    assert_eq!(ring.slots[1], "newest");
    assert_eq!(
        ring.slots[2], "",
        "the vacated slot must come back empty, not be removed"
    );
}

#[test]
fn the_returned_string_is_the_original_allocation() {
    let mut original = String::with_capacity(512);
    original.push_str("oldest");
    let address = original.as_ptr();

    let mut ring = Ring::new(vec![original, "middle".to_owned()]);
    let taken = ring.rotate();

    assert_eq!(
        taken.as_ptr(),
        address,
        "the buffer was copied, not moved -- the allocation must survive intact"
    );
    assert!(taken.capacity() >= 512);
}

#[test]
fn rotating_twice_cycles_correctly() {
    let mut ring = ring();
    assert_eq!(ring.rotate(), "oldest");
    assert_eq!(ring.rotate(), "middle");
    assert_eq!(ring.slots[0], "newest");
}
