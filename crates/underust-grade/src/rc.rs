//! Strong and weak counts, sampled at labelled points.
//!
//! `Rc::strong_count` is pure library semantics, so these answers are invariant.

use std::rc::Rc;

/// One sample of a reference count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Point {
    /// Where in the program this sample was taken.
    pub label: &'static str,
    /// Strong references outstanding.
    pub strong: usize,
    /// Weak references outstanding.
    pub weak: usize,
}

/// An ordered series of reference-count samples.
#[derive(Debug, Clone, Default)]
pub struct RcProbe {
    points: Vec<Point>,
}

impl RcProbe {
    /// Start an empty probe.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sample `rc`'s counts and label the sample.
    pub fn record<T>(&mut self, label: &'static str, rc: &Rc<T>) {
        self.points.push(Point {
            label,
            strong: Rc::strong_count(rc),
            weak: Rc::weak_count(rc),
        });
    }

    /// Every sample taken so far, in order.
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_strong_and_weak_counts_at_labelled_points() {
        let value = Rc::new(7_u8);
        let mut probe = RcProbe::new();
        probe.record("start", &value);
        let clone = Rc::clone(&value);
        probe.record("after clone", &value);
        drop(clone);
        probe.record("after drop", &value);

        assert_eq!(probe.points().len(), 3);
        assert_eq!(probe.points()[0].strong, 1);
        assert_eq!(probe.points()[1].strong, 2);
        assert_eq!(probe.points()[2].strong, 1);
    }

    #[test]
    fn a_weak_handle_does_not_raise_the_strong_count() {
        let value = Rc::new(7_u8);
        let _weak = Rc::downgrade(&value);
        let mut probe = RcProbe::new();
        probe.record("with a weak outstanding", &value);
        assert_eq!(probe.points()[0].strong, 1);
        assert_eq!(probe.points()[0].weak, 1);
    }

    #[test]
    fn an_untouched_probe_has_no_points() {
        assert!(RcProbe::new().points().is_empty());
    }
}
