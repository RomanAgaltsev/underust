//! A global allocator that counts requests.
//!
//! It counts *calls into the allocator*, not bytes the operating system returned, so its
//! numbers are identical on Windows and on Linux. That invariance is why the `alloc`
//! track can assert its answers flatly.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

/// An allocator that forwards to [`System`] and counts what passes through.
///
/// Install it in a task's test binary:
///
/// ```ignore
/// #[global_allocator]
/// static ALLOC: underust_grade::alloc::Counting = underust_grade::alloc::Counting;
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Counting;

// SAFETY: every method forwards unchanged to `System`, which upholds the `GlobalAlloc`
// contract. The counters are plain atomics and never allocate, so no re-entrancy occurs.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        // SAFETY: forwarding an unchanged layout to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: `ptr` came from `System.alloc` with this same layout.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        // SAFETY: `ptr` came from `System` with `layout`, and `new_size` is valid.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// Allocation counts at one instant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct AllocStats {
    /// Calls to `alloc` plus `realloc`.
    pub allocations: u64,
    /// Calls to `dealloc`.
    pub deallocations: u64,
    /// Bytes requested, summed.
    pub bytes_allocated: u64,
}

/// Read the running totals.
#[must_use]
pub fn snapshot() -> AllocStats {
    AllocStats {
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        deallocations: DEALLOCATIONS.load(Ordering::Relaxed),
        bytes_allocated: BYTES.load(Ordering::Relaxed),
    }
}

/// Run `body` and report how much it allocated.
///
/// The returned stats are a **delta**, so repeated measurements are comparable. Correct
/// only when no other thread allocates concurrently: graded tests run with
/// `--test-threads=1`, which the harness passes.
pub fn measure<T>(body: impl FnOnce() -> T) -> (T, AllocStats) {
    let before = snapshot();
    let value = body();
    let after = snapshot();
    (
        value,
        AllocStats {
            allocations: after.allocations - before.allocations,
            deallocations: after.deallocations - before.deallocations,
            bytes_allocated: after.bytes_allocated - before.bytes_allocated,
        },
    )
}
