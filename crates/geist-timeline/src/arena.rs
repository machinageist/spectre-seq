// =============================================================================
// File: crates/geist-timeline/src/arena.rs
// Layer: timeline
// Purpose: Arena<T> allocator for clips
// Status: Implemented; generational arena with free-list slot reuse.
// Notes: Indices carry a generation so a handle to a removed-then-reused slot is
//        rejected. Separates clip data from the tracks that position it.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Stable handle into an arena; the generation guards against stale reuse
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Index {
    slot: u32,
    generation: u32,
}

impl Index {
    // Raw slot, for diagnostics and stable ordering
    pub fn slot(&self) -> u32 {
        self.slot
    }
}

// One arena slot: occupied with a value, or free with the next free slot
#[derive(Debug)]
enum Entry<T> {
    Occupied { generation: u32, value: T },
    Free { next: Option<u32> },
}

// Generational arena: stable handles, O(1) insert/remove, slot reuse
#[derive(Debug)]
pub struct Arena<T> {
    entries: Vec<Entry<T>>,
    free_head: Option<u32>,
    len: usize,
    generation: u32,
}

// Manual Default avoids the derive's spurious `T: Default` bound
impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arena<T> {
    // Build an empty arena
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_head: None,
            len: 0,
            generation: 0,
        }
    }

    // Number of live values
    pub fn len(&self) -> usize {
        self.len
    }

    // Whether the arena holds no live values
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // Insert a value, returning a handle that stays valid until removal
    pub fn insert(&mut self, value: T) -> Index {
        self.len += 1;
        match self.free_head.take() {
            Some(slot) => {
                let next_free = match &self.entries[slot as usize] {
                    Entry::Free { next } => *next,
                    Entry::Occupied { .. } => unreachable!("free list pointed at occupied slot"),
                };
                self.free_head = next_free;
                let generation = self.generation;
                self.entries[slot as usize] = Entry::Occupied { generation, value };
                Index { slot, generation }
            }
            None => {
                let slot = self.entries.len() as u32;
                let generation = self.generation;
                self.entries.push(Entry::Occupied { generation, value });
                Index { slot, generation }
            }
        }
    }

    // Borrow a value if the handle is still valid
    pub fn get(&self, index: Index) -> Option<&T> {
        match self.entries.get(index.slot as usize) {
            Some(Entry::Occupied { generation, value }) if *generation == index.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    // Borrow a value mutably if the handle is still valid
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.entries.get_mut(index.slot as usize) {
            Some(Entry::Occupied { generation, value }) if *generation == index.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    // Whether the handle currently resolves to a live value
    pub fn contains(&self, index: Index) -> bool {
        self.get(index).is_some()
    }

    // Remove and return a value; later handles to this slot are rejected
    pub fn remove(&mut self, index: Index) -> Option<T> {
        let slot = index.slot as usize;
        let is_match = matches!(
            self.entries.get(slot),
            Some(Entry::Occupied { generation, .. }) if *generation == index.generation
        );
        if !is_match {
            return None;
        }
        // Bumping the generation invalidates every outstanding handle to the slot
        self.generation = self.generation.wrapping_add(1);
        let freed = std::mem::replace(
            &mut self.entries[slot],
            Entry::Free {
                next: self.free_head,
            },
        );
        self.free_head = Some(index.slot);
        self.len -= 1;
        match freed {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Free { .. } => unreachable!("matched slot was free"),
        }
    }

    // Iterate live values in slot order
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter().filter_map(|e| match e {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Free { .. } => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get_round_trips() {
        let mut arena = Arena::new();
        let a = arena.insert("a");
        let b = arena.insert("b");
        assert_eq!(arena.get(a), Some(&"a"));
        assert_eq!(arena.get(b), Some(&"b"));
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn remove_returns_value_and_invalidates_handle() {
        let mut arena = Arena::new();
        let a = arena.insert(10);
        assert_eq!(arena.remove(a), Some(10));
        assert_eq!(arena.get(a), None);
        assert!(!arena.contains(a));
        assert_eq!(arena.remove(a), None); // double remove is a no-op
        assert!(arena.is_empty());
    }

    #[test]
    fn reused_slot_rejects_stale_handle() {
        let mut arena = Arena::new();
        let first = arena.insert("first");
        arena.remove(first);
        // The next insert reuses the freed slot but with a new generation
        let second = arena.insert("second");
        assert_eq!(first.slot(), second.slot(), "slot should be reused");
        assert_eq!(arena.get(second), Some(&"second"));
        assert_eq!(arena.get(first), None, "stale handle must not resolve");
    }

    #[test]
    fn get_mut_edits_in_place() {
        let mut arena = Arena::new();
        let a = arena.insert(1);
        *arena.get_mut(a).unwrap() += 41;
        assert_eq!(arena.get(a), Some(&42));
    }

    #[test]
    fn free_list_reuses_before_growing() {
        let mut arena = Arena::new();
        let a = arena.insert(1);
        let b = arena.insert(2);
        let c = arena.insert(3);
        arena.remove(b);
        arena.remove(a);
        // Two slots are free; two inserts must not grow the backing store
        let _ = arena.insert(4);
        let _ = arena.insert(5);
        assert_eq!(arena.len(), 3);
        // c is untouched throughout
        assert_eq!(arena.get(c), Some(&3));
    }

    #[test]
    fn iter_visits_live_values_only() {
        let mut arena = Arena::new();
        let a = arena.insert(1);
        let _b = arena.insert(2);
        let c = arena.insert(3);
        arena.remove(a);
        arena.remove(c);
        let live: Vec<i32> = arena.iter().copied().collect();
        assert_eq!(live, vec![2]);
    }
}
