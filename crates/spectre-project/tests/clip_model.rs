// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 5 evidence — clip note bounds, placement geometry, and identity survival
// Notes: U-9 of the spec's table is not here. It asks for a clip edit applied through
//   EditHistory, and EditHistory mutates ProjectDoc, which has no clip storage until R4-7 adds
//   the field. A command written now would have nothing to mutate and could not be exercised, so
//   it would be unreachable scaffolding rather than evidence.

use spectre_core::{BeatTicks, IdGen, ObjectId, TICKS_PER_BEAT};
use spectre_project::{
    ClipError, ClipNote, ClipPlacement, MidiClip, TrackClips, MAX_CLIPS_PER_TRACK,
    MAX_CLIP_LENGTH_TICKS, MAX_NOTES_PER_CLIP,
};

const SEED: u64 = 0x0043_4c49_5000_0000;
const BEAT: i64 = TICKS_PER_BEAT;
const BAR: i64 = 4 * TICKS_PER_BEAT;

// A four-beat clip with one note at its start
fn clip_of(ids: &mut IdGen, beats: i64) -> MidiClip {
    MidiClip::new(ids.next_id(), "phrase", BeatTicks::from_beats(beats)).unwrap()
}

// U-1
#[test]
fn a_note_outside_the_clip_is_refused() {
    let mut ids = IdGen::new(SEED);
    let mut clip = clip_of(&mut ids, 4);

    // Half-open [0, length): a note starting exactly at the clip's end is outside it
    let at_end = ClipNote::new(BeatTicks::from_beats(4), BeatTicks(BEAT), 0, 60, 0.8).unwrap();
    assert_eq!(
        clip.insert_note(at_end),
        Err(ClipError::NoteOutsideClip {
            start: 4 * BEAT,
            length: 4 * BEAT
        })
    );
    assert_eq!(clip.note_count(), 0);

    // A note that starts inside but ends past the boundary is equally refused
    let overhang =
        ClipNote::new(BeatTicks::from_beats(3), BeatTicks(2 * BEAT), 0, 60, 0.8).unwrap();
    assert!(clip.insert_note(overhang).is_err());
    assert_eq!(clip.note_count(), 0);

    // The last legal note ends exactly on the boundary
    let flush = ClipNote::new(BeatTicks::from_beats(3), BeatTicks(BEAT), 0, 60, 0.8).unwrap();
    assert_eq!(clip.insert_note(flush), Ok(0));
    assert_eq!(clip.note_count(), 1);
}

// U-2
#[test]
fn a_zero_length_note_is_refused() {
    // A note with no duration cannot produce an Off after its On
    assert_eq!(
        ClipNote::new(BeatTicks(0), BeatTicks(0), 0, 60, 0.8),
        Err(ClipError::NonPositiveLength)
    );
    assert_eq!(
        ClipNote::new(BeatTicks(0), BeatTicks(-1), 0, 60, 0.8),
        Err(ClipError::NonPositiveLength)
    );
    assert!(ClipNote::new(BeatTicks(0), BeatTicks(1), 0, 60, 0.8).is_ok());
}

// U-3
#[test]
fn note_bounds_match_the_event_contract() {
    let start = BeatTicks(0);
    let length = BeatTicks(BEAT);

    assert_eq!(
        ClipNote::new(start, length, 16, 60, 0.8),
        Err(ClipError::InvalidChannel(16))
    );
    assert_eq!(
        ClipNote::new(start, length, 0, 128, 0.8),
        Err(ClipError::InvalidNote(128))
    );
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 1.1] {
        assert!(
            matches!(
                ClipNote::new(start, length, 0, 60, bad),
                Err(ClipError::InvalidVelocity(_))
            ),
            "velocity {bad} must be refused"
        );
    }

    // Exactly the values NoteEvent::is_valid accepts at its own boundaries
    for (channel, note, velocity) in [(15_u8, 127_u8, 1.0_f32), (0, 0, 0.0)] {
        assert!(ClipNote::new(start, length, channel, note, velocity).is_ok());
    }
}

// U-4
#[test]
fn the_note_limit_is_refused_not_truncated() {
    let mut ids = IdGen::new(SEED);
    // Long enough that the limit binds before the geometry does
    let mut clip = MidiClip::new(
        ids.next_id(),
        "dense",
        BeatTicks((MAX_NOTES_PER_CLIP as i64 + 1) * BEAT),
    )
    .unwrap();

    for index in 0..MAX_NOTES_PER_CLIP {
        let note =
            ClipNote::new(BeatTicks(index as i64 * BEAT), BeatTicks(BEAT), 0, 60, 0.5).unwrap();
        clip.insert_note(note).unwrap();
    }
    assert_eq!(clip.note_count(), MAX_NOTES_PER_CLIP);

    let overflow = ClipNote::new(
        BeatTicks(MAX_NOTES_PER_CLIP as i64 * BEAT),
        BeatTicks(BEAT),
        0,
        60,
        0.5,
    )
    .unwrap();
    assert_eq!(
        clip.insert_note(overflow),
        Err(ClipError::NoteLimit {
            limit: MAX_NOTES_PER_CLIP
        })
    );
    assert_eq!(
        clip.note_count(),
        MAX_NOTES_PER_CLIP,
        "a refused insert must not truncate or grow the list"
    );
}

// U-5
#[test]
fn a_clip_longer_than_the_limit_is_refused() {
    let mut ids = IdGen::new(SEED);
    let mut clip = clip_of(&mut ids, 4);

    assert_eq!(
        clip.set_length(BeatTicks(MAX_CLIP_LENGTH_TICKS + 1)),
        Err(ClipError::ClipTooLong {
            ticks: MAX_CLIP_LENGTH_TICKS + 1,
            limit: MAX_CLIP_LENGTH_TICKS
        })
    );
    // The bound is inclusive
    assert_eq!(clip.set_length(BeatTicks(MAX_CLIP_LENGTH_TICKS)), Ok(()));
    assert_eq!(clip.length(), BeatTicks(MAX_CLIP_LENGTH_TICKS));

    assert_eq!(
        MidiClip::new(
            ids.next_id(),
            "too long",
            BeatTicks(MAX_CLIP_LENGTH_TICKS + 1)
        ),
        Err(ClipError::ClipTooLong {
            ticks: MAX_CLIP_LENGTH_TICKS + 1,
            limit: MAX_CLIP_LENGTH_TICKS
        })
    );

    // Shortening past a held note is refused rather than silently dropping it
    let mut holding = clip_of(&mut ids, 4);
    let note = ClipNote::new(BeatTicks::from_beats(3), BeatTicks(BEAT), 0, 60, 0.8).unwrap();
    holding.insert_note(note).unwrap();
    assert!(holding.set_length(BeatTicks::from_beats(2)).is_err());
    assert_eq!(holding.length(), BeatTicks::from_beats(4));
    assert_eq!(holding.note_count(), 1);
}

// U-6
#[test]
fn overlapping_placements_are_refused() {
    let mut ids = IdGen::new(SEED);
    let clip_id = ids.next_id();
    let mut clips = TrackClips::new();
    let four_bars = BeatTicks(4 * BAR);

    let first = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(0)).unwrap();
    clips.insert(first, four_bars).unwrap();

    // Starts inside the first placement's half-open span
    let overlapping = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(2 * BAR)).unwrap();
    assert_eq!(
        clips.insert(overlapping, four_bars),
        Err(ClipError::OverlappingPlacement {
            existing: first.id()
        })
    );
    assert_eq!(clips.len(), 1);

    // Adjacency is legal: [0, 4 bars) and [4 bars, 8 bars) do not overlap
    let adjacent = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(4 * BAR)).unwrap();
    assert!(clips.insert(adjacent, four_bars).is_ok());
    assert_eq!(clips.len(), 2);

    // A placement that ends exactly where an existing one begins is legal too, so the check is
    // symmetric rather than only catching a later start
    let mut symmetric = TrackClips::new();
    let later = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(4 * BAR)).unwrap();
    symmetric.insert(later, four_bars).unwrap();
    let earlier = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(0)).unwrap();
    assert!(symmetric.insert(earlier, four_bars).is_ok());
    // One tick longer and it would reach into the later placement
    let mut colliding = TrackClips::new();
    colliding.insert(later, four_bars).unwrap();
    let overhang = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(0)).unwrap();
    assert_eq!(
        colliding.insert(overhang, BeatTicks(4 * BAR + 1)),
        Err(ClipError::OverlappingPlacement {
            existing: later.id()
        })
    );
}

// U-7
#[test]
fn placements_stay_sorted_by_start() {
    let mut ids = IdGen::new(SEED);
    let clip_id = ids.next_id();
    let mut clips = TrackClips::new();
    let one_bar = BeatTicks(BAR);

    for bar in [9_i64, 1, 5] {
        let placement = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(bar * BAR)).unwrap();
        clips.insert(placement, one_bar).unwrap();
    }

    let starts: Vec<i64> = clips
        .placements()
        .iter()
        .map(|placement| placement.start().0 / BAR)
        .collect();
    // Insert order was 9, 1, 5; bake order is what the schedule walks
    assert_eq!(starts, vec![1, 5, 9]);
    // The stored lengths must travel with their placements, not stay in insert order
    for index in 0..clips.len() {
        assert_eq!(clips.length_at(index), Some(one_bar));
    }
}

// U-8 — CORE-001 reorder evidence on the first persisted collection
#[test]
fn clip_ids_survive_reorder_and_round_trip() {
    let mut ids = IdGen::new(SEED);
    let clip_id = ids.next_id();
    let mut clips = TrackClips::new();
    let one_bar = BeatTicks(BAR);

    // Inserted out of order, so the collection reorders them itself
    let mut created = Vec::new();
    for bar in [8_i64, 0, 4] {
        let placement = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(bar * BAR)).unwrap();
        created.push(placement.id());
        clips.insert(placement, one_bar).unwrap();
    }

    let before: Vec<ObjectId> = clips.placements().iter().map(|p| p.id()).collect();
    let json = serde_json::to_string(&clips).unwrap();
    let after: TrackClips = serde_json::from_str(&json).unwrap();

    assert_eq!(after, clips);
    // Equality is over content: the revision is session bookkeeping and starts over on load
    assert_eq!(after.bake_revision(), 0);
    assert!(clips.bake_revision() > 0);
    let after_ids: Vec<ObjectId> = after.placements().iter().map(|p| p.id()).collect();
    assert_eq!(before, after_ids);
    // Every minted id survived: a round trip that renumbered would still be internally consistent
    for id in &created {
        assert!(after_ids.contains(id), "placement {} was lost", id.raw());
        assert_ne!(id.raw(), 0);
    }
}

// U-10
#[test]
fn bake_revision_advances_only_on_bakeable_edits() {
    let mut ids = IdGen::new(SEED);
    let clip_id = ids.next_id();
    let mut clip = MidiClip::new(clip_id, "phrase", BeatTicks(BAR)).unwrap();
    let mut clips = TrackClips::new();
    let placement = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(0)).unwrap();
    clips.insert(placement, BeatTicks(BAR)).unwrap();
    let base = clips.bake_revision();

    // A rename changes no scheduled event, so it must not force a republish
    clip.set_name("renamed").unwrap();
    assert_eq!(clips.bake_revision(), base);

    // Deactivating one changes what the schedule contains
    clips.get_mut(placement.id()).unwrap().set_active(false);
    assert!(clips.bake_revision() > base);

    let previous = clips.bake_revision();
    let second = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(BAR)).unwrap();
    clips.insert(second, BeatTicks(BAR)).unwrap();
    assert!(clips.bake_revision() > previous);

    let previous = clips.bake_revision();
    clips.remove(second.id()).unwrap();
    assert!(clips.bake_revision() > previous);
}

// The clip ceiling refuses rather than truncating, and leaves the list alone
#[test]
fn the_placement_ceiling_is_refused_without_mutating_the_track() {
    let mut ids = IdGen::new(SEED);
    let clip_id = ids.next_id();
    let mut clips = TrackClips::new();
    let one_bar = BeatTicks(BAR);

    for index in 0..MAX_CLIPS_PER_TRACK {
        let placement =
            ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(index as i64 * BAR)).unwrap();
        clips.insert(placement, one_bar).unwrap();
    }
    let before = clips.clone();

    let overflow = ClipPlacement::new(
        ids.next_id(),
        clip_id,
        BeatTicks(MAX_CLIPS_PER_TRACK as i64 * BAR),
    )
    .unwrap();
    assert_eq!(
        clips.insert(overflow, one_bar),
        Err(ClipError::ClipLimit {
            limit: MAX_CLIPS_PER_TRACK
        })
    );
    assert_eq!(clips, before, "a refused insert must not mutate the track");

    // A duplicate identity is refused too, which is CORE-001's uniqueness half. Checked on a list
    // with room in it, so the ceiling above cannot be what refuses the insert
    let mut room = TrackClips::new();
    let held = ClipPlacement::new(ids.next_id(), clip_id, BeatTicks(0)).unwrap();
    room.insert(held, one_bar).unwrap();
    assert_eq!(
        room.insert(held, one_bar),
        Err(ClipError::DuplicateId(held.id()))
    );
    assert_eq!(room.len(), 1);

    let stranger = ids.next_id();
    assert_eq!(
        room.remove(stranger),
        Err(ClipError::UnknownPlacement(stranger))
    );
}
