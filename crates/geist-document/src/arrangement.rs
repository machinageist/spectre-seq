// =============================================================================
// Author: Jeff
// Date: 2026-07-31
// Description: Canonical stable-ID arrangement entities and ordered membership.
// Notes: Additive model; legacy sample-based Timeline remains unchanged during migration.
//
// File: crates/geist-document/src/arrangement.rs
// Layer: document
// Purpose: Exact clip lookup, ownership, ordering, and atomic structural edits
// Status: Implemented; exact lookup and atomic ordered membership edits.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::BTreeMap;

use spectre_core::time::MusicalTime;

use crate::identity::{ClipId, TrackId};

// Structural edit rejection; every rejected operation leaves state unchanged
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrangementError {
    ZeroDuration,
    DuplicateTrack(TrackId),
    MissingTrack(TrackId),
    DuplicateClip(ClipId),
    MissingClip(ClipId),
    InvalidClipOrder {
        track: TrackId,
        index: usize,
        len: usize,
    },
}

// One visible arrangement region with stable identity and musical placement
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipEntity {
    id: ClipId,
    owner: TrackId,
    start: MusicalTime,
    duration: MusicalTime,
}

impl ClipEntity {
    // Build a region; visible duration must contain at least one tick
    pub fn new(
        id: ClipId,
        owner: TrackId,
        start: MusicalTime,
        duration: MusicalTime,
    ) -> Result<Self, ArrangementError> {
        if duration == MusicalTime::ZERO {
            return Err(ArrangementError::ZeroDuration);
        }
        Ok(Self {
            id,
            owner,
            start,
            duration,
        })
    }

    pub const fn id(&self) -> ClipId {
        self.id
    }

    pub const fn owner(&self) -> TrackId {
        self.owner
    }

    pub const fn start(&self) -> MusicalTime {
        self.start
    }

    pub const fn duration(&self) -> MusicalTime {
        self.duration
    }
}

// Canonical track record; clip order is deterministic arrangement state
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrangementTrack {
    id: TrackId,
    clip_ids: Vec<ClipId>,
}

impl ArrangementTrack {
    pub const fn id(&self) -> TrackId {
        self.id
    }

    pub fn clip_ids(&self) -> &[ClipId] {
        &self.clip_ids
    }
}

// Exact ordered location captured for structural undo
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClipLocation {
    track: TrackId,
    index: usize,
}

impl ClipLocation {
    pub const fn new(track: TrackId, index: usize) -> Self {
        Self { track, index }
    }

    pub const fn track(self) -> TrackId {
        self.track
    }

    pub const fn index(self) -> usize {
        self.index
    }
}

// Removed entity plus its exact former ordered location
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedClip {
    entity: ClipEntity,
    location: ClipLocation,
}

impl RemovedClip {
    pub const fn entity(&self) -> &ClipEntity {
        &self.entity
    }

    pub const fn location(&self) -> ClipLocation {
        self.location
    }

    pub fn into_parts(self) -> (ClipEntity, ClipLocation) {
        (self.entity, self.location)
    }
}

// Canonical app-thread arrangement; legacy Timeline remains during migration
#[derive(Clone, Debug, Default)]
pub struct Arrangement {
    tracks: Vec<ArrangementTrack>,
    clips: BTreeMap<ClipId, ClipEntity>,
}

impl Arrangement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tracks(&self) -> &[ArrangementTrack] {
        &self.tracks
    }

    pub fn track(&self, id: TrackId) -> Option<&ArrangementTrack> {
        self.tracks.iter().find(|track| track.id == id)
    }

    pub fn clip(&self, id: ClipId) -> Option<&ClipEntity> {
        self.clips.get(&id)
    }

    pub fn insert_track(&mut self, id: TrackId) -> Result<(), ArrangementError> {
        if self.track(id).is_some() {
            return Err(ArrangementError::DuplicateTrack(id));
        }
        self.tracks.push(ArrangementTrack {
            id,
            clip_ids: Vec::new(),
        });
        Ok(())
    }

    pub fn insert_clip(&mut self, entity: ClipEntity) -> Result<(), ArrangementError> {
        let owner = entity.owner;
        let index = self
            .track(owner)
            .ok_or(ArrangementError::MissingTrack(owner))?
            .clip_ids
            .len();
        self.insert_clip_at(entity, index)
    }

    pub fn insert_clip_at(
        &mut self,
        entity: ClipEntity,
        index: usize,
    ) -> Result<(), ArrangementError> {
        if self.clips.contains_key(&entity.id) {
            return Err(ArrangementError::DuplicateClip(entity.id));
        }
        let track_index = self
            .track_index(entity.owner)
            .ok_or(ArrangementError::MissingTrack(entity.owner))?;
        let len = self.tracks[track_index].clip_ids.len();
        if index > len {
            return Err(ArrangementError::InvalidClipOrder {
                track: entity.owner,
                index,
                len,
            });
        }

        let id = entity.id;
        self.clips.insert(id, entity);
        self.tracks[track_index].clip_ids.insert(index, id);
        Ok(())
    }

    pub fn remove_clip(&mut self, id: ClipId) -> Result<RemovedClip, ArrangementError> {
        let owner = self
            .clips
            .get(&id)
            .ok_or(ArrangementError::MissingClip(id))?
            .owner;
        let track_index = self
            .track_index(owner)
            .ok_or(ArrangementError::MissingTrack(owner))?;
        let order = self.tracks[track_index]
            .clip_ids
            .iter()
            .position(|candidate| *candidate == id)
            .ok_or(ArrangementError::MissingClip(id))?;

        let entity = self
            .clips
            .remove(&id)
            .ok_or(ArrangementError::MissingClip(id))?;
        self.tracks[track_index].clip_ids.remove(order);
        Ok(RemovedClip {
            entity,
            location: ClipLocation::new(owner, order),
        })
    }

    // Rehome or reorder a clip; index addresses destination order after source removal
    pub fn rehome_clip(
        &mut self,
        id: ClipId,
        target: TrackId,
        index: usize,
    ) -> Result<ClipLocation, ArrangementError> {
        let source = self
            .clips
            .get(&id)
            .ok_or(ArrangementError::MissingClip(id))?
            .owner;
        let source_track = self
            .track_index(source)
            .ok_or(ArrangementError::MissingTrack(source))?;
        let source_order = self.tracks[source_track]
            .clip_ids
            .iter()
            .position(|candidate| *candidate == id)
            .ok_or(ArrangementError::MissingClip(id))?;
        let target_track = self
            .track_index(target)
            .ok_or(ArrangementError::MissingTrack(target))?;

        let target_len = self.tracks[target_track].clip_ids.len();
        let max_index = if source_track == target_track {
            target_len.saturating_sub(1)
        } else {
            target_len
        };
        if index > max_index {
            return Err(ArrangementError::InvalidClipOrder {
                track: target,
                index,
                len: max_index,
            });
        }

        self.clips
            .get_mut(&id)
            .ok_or(ArrangementError::MissingClip(id))?
            .owner = target;
        self.tracks[source_track].clip_ids.remove(source_order);
        self.tracks[target_track].clip_ids.insert(index, id);
        Ok(ClipLocation::new(source, source_order))
    }

    fn track_index(&self, id: TrackId) -> Option<usize> {
        self.tracks.iter().position(|track| track.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(raw: u64) -> TrackId {
        TrackId::new(raw).unwrap()
    }

    fn clip(raw: u64, owner: TrackId, start: u64, duration: u64) -> ClipEntity {
        ClipEntity::new(
            ClipId::new(raw).unwrap(),
            owner,
            MusicalTime::from_ticks(start),
            MusicalTime::from_ticks(duration),
        )
        .unwrap()
    }

    #[test]
    fn zero_duration_is_rejected() {
        assert_eq!(
            ClipEntity::new(
                ClipId::new(1).unwrap(),
                track(1),
                MusicalTime::ZERO,
                MusicalTime::ZERO,
            ),
            Err(ArrangementError::ZeroDuration)
        );
    }

    #[test]
    fn duplicate_tracks_are_rejected_without_mutation() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();

        assert_eq!(
            arrangement.insert_track(track(1)),
            Err(ArrangementError::DuplicateTrack(track(1)))
        );
        assert_eq!(arrangement.tracks().len(), 1);
    }

    #[test]
    fn clips_require_an_existing_owner_track() {
        let mut arrangement = Arrangement::new();
        let entity = clip(1, track(9), 0, 960);

        assert_eq!(
            arrangement.insert_clip(entity),
            Err(ArrangementError::MissingTrack(track(9)))
        );
        assert!(arrangement.clip(ClipId::new(1).unwrap()).is_none());
    }

    #[test]
    fn duplicate_clip_ids_are_rejected_atomically() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();

        assert_eq!(
            arrangement.insert_clip(clip(1, track(1), 1_920, 960)),
            Err(ArrangementError::DuplicateClip(ClipId::new(1).unwrap()))
        );
        let stored = arrangement.clip(ClipId::new(1).unwrap()).unwrap();
        assert_eq!(stored.start(), MusicalTime::ZERO);
        assert_eq!(arrangement.track(track(1)).unwrap().clip_ids().len(), 1);
    }

    #[test]
    fn stable_id_lookup_is_independent_of_track_order() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(20)).unwrap();
        arrangement.insert_track(track(10)).unwrap();
        arrangement
            .insert_clip(clip(50, track(10), 960, 480))
            .unwrap();

        let stored = arrangement.clip(ClipId::new(50).unwrap()).unwrap();
        assert_eq!(stored.owner(), track(10));
        assert_eq!(stored.start(), MusicalTime::from_ticks(960));
        assert_eq!(stored.duration(), MusicalTime::from_ticks(480));
        assert_eq!(arrangement.tracks()[1].id(), track(10));
    }

    #[test]
    fn removal_returns_exact_entity_and_original_order() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(2, track(1), 960, 960))
            .unwrap();
        arrangement
            .insert_clip(clip(3, track(1), 1_920, 960))
            .unwrap();

        let removed = arrangement.remove_clip(ClipId::new(2).unwrap()).unwrap();
        assert_eq!(removed.entity().id(), ClipId::new(2).unwrap());
        assert_eq!(removed.location(), ClipLocation::new(track(1), 1));
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[ClipId::new(1).unwrap(), ClipId::new(3).unwrap()]
        );

        let (entity, location) = removed.into_parts();
        arrangement
            .insert_clip_at(entity, location.index())
            .unwrap();
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[
                ClipId::new(1).unwrap(),
                ClipId::new(2).unwrap(),
                ClipId::new(3).unwrap()
            ]
        );
    }

    #[test]
    fn invalid_restore_order_is_rejected_without_insertion() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        let entity = clip(1, track(1), 0, 960);

        assert_eq!(
            arrangement.insert_clip_at(entity, 1),
            Err(ArrangementError::InvalidClipOrder {
                track: track(1),
                index: 1,
                len: 0,
            })
        );
        assert!(arrangement.clip(ClipId::new(1).unwrap()).is_none());
    }

    #[test]
    fn cross_track_rehome_is_atomic_and_preserves_identity() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_track(track(2)).unwrap();
        arrangement.insert_clip(clip(7, track(1), 0, 960)).unwrap();

        let previous = arrangement
            .rehome_clip(ClipId::new(7).unwrap(), track(2), 0)
            .unwrap();
        assert_eq!(previous, ClipLocation::new(track(1), 0));
        assert_eq!(
            arrangement.clip(ClipId::new(7).unwrap()).unwrap().owner(),
            track(2)
        );
        assert!(arrangement.track(track(1)).unwrap().clip_ids().is_empty());
        assert_eq!(
            arrangement.track(track(2)).unwrap().clip_ids(),
            &[ClipId::new(7).unwrap()]
        );
    }

    #[test]
    fn rejected_rehome_leaves_source_membership_unchanged() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(7, track(1), 0, 960)).unwrap();

        assert_eq!(
            arrangement.rehome_clip(ClipId::new(7).unwrap(), track(9), 0),
            Err(ArrangementError::MissingTrack(track(9)))
        );
        assert_eq!(
            arrangement.clip(ClipId::new(7).unwrap()).unwrap().owner(),
            track(1)
        );
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[ClipId::new(7).unwrap()]
        );
    }

    #[test]
    fn rejected_rehome_order_leaves_source_membership_unchanged() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_track(track(2)).unwrap();
        arrangement.insert_clip(clip(7, track(1), 0, 960)).unwrap();

        assert_eq!(
            arrangement.rehome_clip(ClipId::new(7).unwrap(), track(2), 1),
            Err(ArrangementError::InvalidClipOrder {
                track: track(2),
                index: 1,
                len: 0,
            })
        );
        assert_eq!(
            arrangement.clip(ClipId::new(7).unwrap()).unwrap().owner(),
            track(1)
        );
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[ClipId::new(7).unwrap()]
        );
    }

    #[test]
    fn same_track_rehome_changes_only_order() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(2, track(1), 960, 960))
            .unwrap();
        arrangement
            .insert_clip(clip(3, track(1), 1_920, 960))
            .unwrap();

        let previous = arrangement
            .rehome_clip(ClipId::new(1).unwrap(), track(1), 2)
            .unwrap();
        assert_eq!(previous, ClipLocation::new(track(1), 0));
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[
                ClipId::new(2).unwrap(),
                ClipId::new(3).unwrap(),
                ClipId::new(1).unwrap(),
            ]
        );
        assert_eq!(
            arrangement.clip(ClipId::new(1).unwrap()).unwrap().owner(),
            track(1)
        );
    }

    #[test]
    fn same_track_rehome_moves_backward() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(2, track(1), 960, 960))
            .unwrap();
        arrangement
            .insert_clip(clip(3, track(1), 1_920, 960))
            .unwrap();

        arrangement
            .rehome_clip(ClipId::new(3).unwrap(), track(1), 0)
            .unwrap();
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[
                ClipId::new(3).unwrap(),
                ClipId::new(1).unwrap(),
                ClipId::new(2).unwrap(),
            ]
        );
    }

    #[test]
    fn same_position_rehome_is_a_structural_no_op() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(2, track(1), 960, 960))
            .unwrap();

        let previous = arrangement
            .rehome_clip(ClipId::new(2).unwrap(), track(1), 1)
            .unwrap();
        assert_eq!(previous, ClipLocation::new(track(1), 1));
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[ClipId::new(1).unwrap(), ClipId::new(2).unwrap()]
        );
    }

    #[test]
    fn rejected_same_track_order_is_atomic() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(2, track(1), 960, 960))
            .unwrap();

        assert_eq!(
            arrangement.rehome_clip(ClipId::new(1).unwrap(), track(1), 2),
            Err(ArrangementError::InvalidClipOrder {
                track: track(1),
                index: 2,
                len: 1,
            })
        );
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[ClipId::new(1).unwrap(), ClipId::new(2).unwrap()]
        );
    }

    #[test]
    fn same_track_rehome_round_trips_through_returned_location() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(2, track(1), 960, 960))
            .unwrap();
        arrangement
            .insert_clip(clip(3, track(1), 1_920, 960))
            .unwrap();

        let original = arrangement
            .rehome_clip(ClipId::new(1).unwrap(), track(1), 2)
            .unwrap();
        arrangement
            .rehome_clip(ClipId::new(1).unwrap(), original.track(), original.index())
            .unwrap();
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[
                ClipId::new(1).unwrap(),
                ClipId::new(2).unwrap(),
                ClipId::new(3).unwrap(),
            ]
        );
    }

    #[test]
    fn cross_track_rehome_round_trips_nonempty_destination() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track(1)).unwrap();
        arrangement.insert_track(track(2)).unwrap();
        arrangement.insert_clip(clip(1, track(1), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(2, track(1), 960, 960))
            .unwrap();
        arrangement.insert_clip(clip(3, track(2), 0, 960)).unwrap();
        arrangement
            .insert_clip(clip(4, track(2), 960, 960))
            .unwrap();

        let original = arrangement
            .rehome_clip(ClipId::new(2).unwrap(), track(2), 1)
            .unwrap();
        assert_eq!(
            arrangement.track(track(2)).unwrap().clip_ids(),
            &[
                ClipId::new(3).unwrap(),
                ClipId::new(2).unwrap(),
                ClipId::new(4).unwrap(),
            ]
        );

        arrangement
            .rehome_clip(ClipId::new(2).unwrap(), original.track(), original.index())
            .unwrap();
        assert_eq!(
            arrangement.track(track(1)).unwrap().clip_ids(),
            &[ClipId::new(1).unwrap(), ClipId::new(2).unwrap()]
        );
        assert_eq!(
            arrangement.track(track(2)).unwrap().clip_ids(),
            &[ClipId::new(3).unwrap(), ClipId::new(4).unwrap()]
        );
    }
}
