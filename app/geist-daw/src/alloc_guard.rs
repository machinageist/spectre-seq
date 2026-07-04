// Author: Jeff
// Date: 2026-07-03
// Description: Test-build global allocator that counts (de)allocations inside
//              forbidden regions, enforcing the audio-thread no-alloc contract
// Notes: Test binary only (cfg(test) in main.rs). The allocator never panics —
//        it counts per-thread hits; tests assert the count is zero afterwards.
//        See docs/realtime_rules.md for the contract this enforces.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    // Whether the current thread is inside a no-alloc region
    static FORBID: Cell<bool> = const { Cell::new(false) };
    // Allocator hits observed while forbidden, on this thread
    static HITS: Cell<u64> = const { Cell::new(0) };
}

// System allocator wrapper that records hits inside forbidden regions
pub struct CountingAlloc;

// Record one allocator hit if this thread is inside a forbidden region
fn note_hit() {
    FORBID.with(|forbid| {
        if forbid.get() {
            HITS.with(|hits| hits.set(hits.get() + 1));
        }
    });
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note_hit();
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        note_hit();
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        note_hit();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

// Run `f` with heap use forbidden on this thread; returns f's result and the
// number of allocator hits observed (0 = realtime-clean)
pub fn assert_no_alloc_scope<R>(f: impl FnOnce() -> R) -> (R, u64) {
    HITS.with(|hits| hits.set(0));
    FORBID.with(|forbid| forbid.set(true));
    let result = f();
    FORBID.with(|forbid| forbid.set(false));
    let observed = HITS.with(|hits| hits.get());
    (result, observed)
}
