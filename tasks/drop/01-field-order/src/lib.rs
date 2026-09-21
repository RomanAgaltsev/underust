//! A struct holding three tracked values, and two loose ones beside it.
//!
//! This is a PREDICT task, so the program is complete. You are not asked to write code --
//! you are asked to say, in advance, what it will do.

use underust_grade::drop_log::{DropLog, Token};

/// Three tracked values, dropped as fields.
pub struct Trio {
    /// First declared.
    pub first: Token,
    /// Second declared.
    pub second: Token,
    /// Third declared.
    pub third: Token,
}

/// Build a `Trio` and two loose locals beside it, then let the whole scope end.
///
/// Returns the order in which the five labels were appended.
#[must_use]
pub fn observe() -> Vec<&'static str> {
    let log = DropLog::new();
    {
        let _loose_a = log.token("loose_a");
        let _trio = Trio {
            first: log.token("first"),
            second: log.token("second"),
            third: log.token("third"),
        };
        let _loose_b = log.token("loose_b");
    }
    log.entries()
}
