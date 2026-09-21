//! The grader. The second test is the one with teeth: a reference cycle passes the other
//! two and fails only that one.

use std::rc::Rc;
use task_weak_01_break_the_cycle::{build, first_child, parent_value_of};
use underust_grade::rc::RcProbe;

#[test]
fn the_child_can_reach_its_parent() {
    let parent = build();
    let child = first_child(&parent);
    assert_eq!(parent_value_of(&child), Some(parent.value));
}

#[test]
fn the_parent_is_not_kept_alive_by_its_child() {
    let parent = build();
    let child = first_child(&parent);

    let mut probe = RcProbe::new();
    probe.record("before dropping the parent handle", &parent);
    let weak_to_parent = Rc::downgrade(&parent);
    drop(parent);

    assert!(
        weak_to_parent.upgrade().is_none(),
        "the parent is still alive after its only strong handle was dropped -- that is the cycle"
    );
    assert_eq!(
        parent_value_of(&child),
        None,
        "the upward link must go dead with the parent"
    );
    underust_grade::emit("strong_before_drop", &probe.points()[0].strong);
}

#[test]
fn the_parent_owns_its_child() {
    let parent = build();
    let child = first_child(&parent);
    assert_eq!(
        Rc::strong_count(&child),
        2,
        "the tree holds one, this test holds one"
    );
}
