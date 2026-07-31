// =============================================================================
// Author: Jeff
// Date: 2026-07-31
// Description: Stable canonical track and clip identity.
// Notes: IDs are never zero or reused; allocation is app-thread-owned.
//
// File: crates/geist-timeline/src/identity.rs
// Layer: timeline
// Purpose: Nonzero stable IDs and checked app-thread allocation
// Status: Implemented; nonzero IDs and independent checked sequences.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub use geist_core::ids::{ClipId, TrackId};

// One monotonic nonzero sequence; zero marks permanent exhaustion
#[derive(Debug)]
struct IdSequence {
    next: u64,
}

impl IdSequence {
    const fn new() -> Self {
        Self { next: 1 }
    }

    fn allocate(&mut self) -> Option<u64> {
        if self.next == 0 {
            return None;
        }
        let id = self.next;
        self.next = self.next.checked_add(1).unwrap_or(0);
        Some(id)
    }

    fn observe(&mut self, id: u64) {
        if self.next != 0 && id >= self.next {
            self.next = id.checked_add(1).unwrap_or(0);
        }
    }
}

impl Default for IdSequence {
    fn default() -> Self {
        Self::new()
    }
}

// App-thread allocator for independent track and clip identity domains
#[derive(Debug, Default)]
pub struct IdentityAllocator {
    tracks: IdSequence,
    clips: IdSequence,
}

impl IdentityAllocator {
    // Build independent sequences beginning at one
    pub const fn new() -> Self {
        Self {
            tracks: IdSequence::new(),
            clips: IdSequence::new(),
        }
    }

    // Allocate the next track ID or report exhaustion
    pub fn allocate_track(&mut self) -> Option<TrackId> {
        self.tracks.allocate().and_then(TrackId::new)
    }

    // Allocate the next clip ID or report exhaustion
    pub fn allocate_clip(&mut self) -> Option<ClipId> {
        self.clips.allocate().and_then(ClipId::new)
    }

    // Advance track allocation beyond an ID restored from durable state
    pub fn observe_track(&mut self, id: TrackId) {
        self.tracks.observe(id.raw());
    }

    // Advance clip allocation beyond an ID restored from durable state
    pub fn observe_clip(&mut self, id: ClipId) {
        self.clips.observe(id.raw());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_not_a_canonical_id() {
        assert_eq!(TrackId::new(0), None);
        assert_eq!(ClipId::new(0), None);
    }

    #[test]
    fn valid_raw_ids_round_trip() {
        assert_eq!(TrackId::new(41).unwrap().raw(), 41);
        assert_eq!(ClipId::new(73).unwrap().raw(), 73);
    }

    #[test]
    fn track_and_clip_sequences_start_at_one_and_advance_independently() {
        let mut ids = IdentityAllocator::new();
        assert_eq!(ids.allocate_track().unwrap().raw(), 1);
        assert_eq!(ids.allocate_track().unwrap().raw(), 2);
        assert_eq!(ids.allocate_clip().unwrap().raw(), 1);
        assert_eq!(ids.allocate_clip().unwrap().raw(), 2);
    }

    #[test]
    fn observed_ids_advance_without_moving_backward() {
        let mut ids = IdentityAllocator::new();
        ids.observe_track(TrackId::new(80).unwrap());
        ids.observe_track(TrackId::new(12).unwrap());
        ids.observe_clip(ClipId::new(120).unwrap());
        ids.observe_clip(ClipId::new(20).unwrap());

        assert_eq!(ids.allocate_track().unwrap().raw(), 81);
        assert_eq!(ids.allocate_clip().unwrap().raw(), 121);
    }

    #[test]
    fn allocated_ids_are_not_reused() {
        let mut ids = IdentityAllocator::new();
        let deleted = ids.allocate_clip().unwrap();
        let next = ids.allocate_clip().unwrap();
        assert_ne!(deleted, next);
        assert_eq!(ids.allocate_clip().unwrap().raw(), 3);
    }

    #[test]
    fn exhaustion_returns_none_without_wrapping_to_zero() {
        let mut ids = IdentityAllocator::new();
        ids.clips.next = u64::MAX;

        assert_eq!(ids.allocate_clip().unwrap().raw(), u64::MAX);
        assert_eq!(ids.allocate_clip(), None);
        assert_eq!(ids.allocate_clip(), None);
    }

    #[test]
    fn observing_maximum_id_exhausts_only_its_sequence() {
        let mut ids = IdentityAllocator::new();
        ids.observe_clip(ClipId::new(u64::MAX).unwrap());

        assert_eq!(ids.allocate_clip(), None);
        assert_eq!(ids.allocate_track().unwrap().raw(), 1);
    }

    #[test]
    fn observing_an_older_id_does_not_revive_an_exhausted_sequence() {
        let mut ids = IdentityAllocator::new();
        ids.clips.next = u64::MAX;
        assert_eq!(ids.allocate_clip().unwrap().raw(), u64::MAX);

        ids.observe_clip(ClipId::new(12).unwrap());
        assert_eq!(ids.allocate_clip(), None);
    }
}
