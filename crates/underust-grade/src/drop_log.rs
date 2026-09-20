//! An ordered record of what dropped, and when.
//!
//! Drop order is language-defined, so this grader's answers are invariant across every
//! platform and every toolchain -- which makes it the most honest instrument in the gym.

use std::cell::RefCell;
use std::rc::Rc;

/// A shared, ordered log of drop events.
///
/// Cloning a `DropLog` shares the same underlying log; each clone is a handle, not a copy.
/// Per-log rather than global, so two tests in one binary cannot interleave their entries.
#[derive(Debug, Clone, Default)]
pub struct DropLog(Rc<RefCell<Vec<&'static str>>>);

/// A value whose destruction appends a label to a [`DropLog`].
#[derive(Debug)]
pub struct Token {
    label: &'static str,
    log: DropLog,
}

impl DropLog {
    /// Start an empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mint a value that will append `label` when it drops.
    #[must_use]
    pub fn token(&self, label: &'static str) -> Token {
        Token {
            label,
            log: self.clone(),
        }
    }

    /// The labels appended so far, in the order they were appended.
    #[must_use]
    pub fn entries(&self) -> Vec<&'static str> {
        self.0.borrow().clone()
    }
}

impl Drop for Token {
    fn drop(&mut self) {
        self.log.0.borrow_mut().push(self.label);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locals_drop_in_reverse_declaration_order() {
        let log = DropLog::new();
        {
            let _first = log.token("first");
            let _second = log.token("second");
        }
        assert_eq!(log.entries(), vec!["second", "first"]);
    }

    #[test]
    fn struct_fields_drop_in_declaration_order() {
        struct Pair {
            _a: Token,
            _b: Token,
        }
        let log = DropLog::new();
        {
            let _pair = Pair {
                _a: log.token("a"),
                _b: log.token("b"),
            };
        }
        assert_eq!(log.entries(), vec!["a", "b"]);
    }

    #[test]
    fn forget_suppresses_the_entry_entirely() {
        let log = DropLog::new();
        std::mem::forget(log.token("never"));
        assert!(log.entries().is_empty());
    }

    #[test]
    fn two_logs_do_not_share_entries() {
        let left = DropLog::new();
        let right = DropLog::new();
        drop(left.token("left"));
        drop(right.token("right"));
        assert_eq!(left.entries(), vec!["left"]);
        assert_eq!(right.entries(), vec!["right"]);
    }

    #[test]
    fn an_untouched_log_is_empty() {
        assert!(DropLog::new().entries().is_empty());
    }
}
