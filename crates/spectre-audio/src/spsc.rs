// Author: Jeff
// Date: 2026-08-09
// Description: Bounded wait-free single-producer/single-consumer queue for RT-002
// Notes: Both halves are wait-free: push and pop each perform a bounded number of steps with
//   no loops, no locks, and no allocation after construction. Capacity is rounded to a power
//   of two so the index wrap is a mask. Splitting into Producer/Consumer is what enforces the
//   single-producer and single-consumer requirement the memory ordering depends on.
//   One slot is intentionally left empty so a full ring is distinguishable from an empty one.

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Smallest ring the queue will build, keeping the mask arithmetic meaningful
pub const MIN_CAPACITY: usize = 2;

// Shared ring; only Producer touches tail, only Consumer touches head
struct Ring<T> {
    slots: Box<[UnsafeCell<MaybeUninit<T>>]>,
    mask: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

// Safe because the Producer/Consumer split guarantees exactly one writer and one reader,
// and every slot handoff is published with Release and observed with Acquire
unsafe impl<T: Send> Send for Ring<T> {}
unsafe impl<T: Send> Sync for Ring<T> {}

impl<T> Drop for Ring<T> {
    // Drop the elements still queued at teardown
    fn drop(&mut self) {
        let mut head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        while head != tail {
            // Each live slot was initialized by a push that was never popped
            unsafe {
                (*self.slots[head].get()).assume_init_drop();
            }
            head = (head + 1) & self.mask;
        }
    }
}

// Producing half; lives on the app thread
pub struct Producer<T> {
    ring: Arc<Ring<T>>,
}

// Consuming half; lives on the audio thread
pub struct Consumer<T> {
    ring: Arc<Ring<T>>,
}

// Build a bounded queue whose usable capacity is at least the request
pub fn bounded<T>(capacity: usize) -> (Producer<T>, Consumer<T>) {
    // One slot stays empty, so the ring holds capacity + 1 to honor the request
    let requested = capacity.max(MIN_CAPACITY) + 1;
    let slots_len = requested.next_power_of_two();
    let mut slots = Vec::with_capacity(slots_len);
    slots.resize_with(slots_len, || UnsafeCell::new(MaybeUninit::uninit()));
    let ring = Arc::new(Ring {
        slots: slots.into_boxed_slice(),
        mask: slots_len - 1,
        head: AtomicUsize::new(0),
        tail: AtomicUsize::new(0),
    });
    (
        Producer {
            ring: Arc::clone(&ring),
        },
        Consumer { ring },
    )
}

impl<T> Producer<T> {
    // Enqueue one value, handing it back when full rather than blocking, growing, or dropping it.
    // Returning the value matters on the audio thread, where dropping it could deallocate.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        let tail = self.ring.tail.load(Ordering::Relaxed);
        let next = (tail + 1) & self.ring.mask;
        // Acquire pairs with the consumer's Release store of head
        if next == self.ring.head.load(Ordering::Acquire) {
            return Err(value);
        }
        // Sole writer of this slot until the tail store publishes it
        unsafe {
            (*self.ring.slots[tail].get()).write(value);
        }
        self.ring.tail.store(next, Ordering::Release);
        Ok(())
    }

    // Usable capacity, excluding the always-empty slot
    pub fn capacity(&self) -> usize {
        self.ring.mask
    }
}

impl<T> Consumer<T> {
    // Dequeue one value if the producer has published any
    pub fn pop(&mut self) -> Option<T> {
        let head = self.ring.head.load(Ordering::Relaxed);
        // Acquire pairs with the producer's Release store of tail
        if head == self.ring.tail.load(Ordering::Acquire) {
            return None;
        }
        // The publishing Release store guarantees this slot is initialized
        let value = unsafe { (*self.ring.slots[head].get()).assume_init_read() };
        self.ring
            .head
            .store((head + 1) & self.ring.mask, Ordering::Release);
        Some(value)
    }

    // Report whether the producer has published anything unread
    pub fn is_empty(&self) -> bool {
        self.ring.head.load(Ordering::Relaxed) == self.ring.tail.load(Ordering::Acquire)
    }

    // Usable capacity, excluding the always-empty slot
    pub fn capacity(&self) -> usize {
        self.ring.mask
    }
}
