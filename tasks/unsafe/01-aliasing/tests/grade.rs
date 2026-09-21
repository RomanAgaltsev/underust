//! The grader. These tests are pinned by `test_digest` in task.toml: a soundness task is
//! passed by removing the undefined behaviour, not by removing the assertions.

use task_unsafe_01_aliasing::split_at_mut_ours;

#[test]
fn both_halves_have_the_right_contents() {
    let mut data = [1_u8, 2, 3, 4, 5, 6];
    let (left, right) = split_at_mut_ours(&mut data, 3);
    assert_eq!(&left[..3], &[1, 2, 3]);
    assert_eq!(right, &[4, 5, 6]);
}

#[test]
fn the_halves_have_the_right_lengths() {
    let mut data = [0_u8; 10];
    let (left, right) = split_at_mut_ours(&mut data, 4);
    assert_eq!(left.len(), 4, "the left half must be exactly mid long");
    assert_eq!(right.len(), 6);
}

#[test]
fn writes_to_each_half_are_independent() {
    let mut data = [0_u8; 6];
    {
        let (left, right) = split_at_mut_ours(&mut data, 3);
        left[0] = 11;
        right[0] = 22;
    }
    assert_eq!(data, [11, 0, 0, 22, 0, 0]);
}

#[test]
fn a_zero_split_yields_an_empty_left_half() {
    let mut data = [1_u8, 2, 3];
    let (left, right) = split_at_mut_ours(&mut data, 0);
    assert!(left.is_empty());
    assert_eq!(right, &[1, 2, 3]);
}
