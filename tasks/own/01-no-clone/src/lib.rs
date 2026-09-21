//! Rotating a buffer held behind a mutable reference.

/// A ring of string buffers, rotated in place.
pub struct Ring {
    /// Buffers, oldest first.
    pub slots: Vec<String>,
}

impl Ring {
    /// Build a ring from owned buffers.
    #[must_use]
    pub fn new(slots: Vec<String>) -> Self {
        Self { slots }
    }

    /// Move the oldest buffer to the back, emptied and ready for reuse, and return what
    /// it held.
    ///
    /// The returned `String` must be the **same allocation** the slot held -- not a copy
    /// of it -- and the vacated slot must be left as an empty `String` whose capacity is
    /// reusable.
    ///
    /// # Panics
    /// Panics when the ring is empty. The stub panics until you implement it.
    pub fn rotate(&mut self) -> String {
        todo!("take the oldest buffer out, leave an empty one behind, rotate")
    }
}
