//! Splitting a slice into two halves that can be written independently.

/// Split `data` into two mutable halves at `mid`.
///
/// # Panics
/// Panics when `mid` is out of bounds.
#[must_use]
pub fn split_at_mut_ours(data: &mut [u8], mid: usize) -> (&mut [u8], &mut [u8]) {
    assert!(mid <= data.len(), "mid out of bounds");
    let len = data.len();
    let ptr = data.as_mut_ptr();

    // SAFETY: this comment is wrong, and finding out why is the exercise.
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, len),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
