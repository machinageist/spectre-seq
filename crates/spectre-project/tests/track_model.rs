// Author: Jeff
// Date: 2026-08-24
// Description: R4 slice 4 evidence — track identity, ordering, bounds, and mixer composition
// Notes: CORE-001's reorder half is discharged here; its persistence half waits for R4-7

use spectre_core::IdGen;
use spectre_project::{Track, TrackError, TrackInstrument, TrackList, MAX_TRACKS};

const SEED: u64 = 0x0054_5241_434b_5300;

// Build a list of named tracks with distinct mixer state
fn list_of(names: &[&str]) -> (TrackList, Vec<spectre_core::ObjectId>) {
    let mut ids = IdGen::new(SEED);
    let mut list = TrackList::new();
    let mut created = Vec::new();
    for name in names {
        let id = ids.next_id();
        list.push(Track::new(id, name, TrackInstrument::Pulse).unwrap())
            .unwrap();
        created.push(id);
    }
    (list, created)
}

// 1
#[test]
fn a_track_keeps_its_identity_and_fields_across_a_reorder() {
    let (mut list, ids) = list_of(&["one", "two", "three"]);
    for (index, id) in ids.iter().enumerate() {
        let track = list.get_mut(*id).unwrap();
        track.set_level(0.25 * (index as f32 + 1.0));
        track.set_muted(index == 1);
        track.set_soloed(index == 2);
    }
    let before = list.get(ids[1]).unwrap().clone();
    let others = [
        list.get(ids[0]).unwrap().clone(),
        list.get(ids[2]).unwrap().clone(),
    ];

    list.reorder(ids[1], 0).unwrap();

    assert_eq!(list.index_of(ids[1]), Some(0));
    let after = list.get(ids[1]).unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.name(), before.name());
    assert_eq!(after.level(), before.level());
    assert_eq!(after.is_muted(), before.is_muted());
    assert_eq!(after.is_soloed(), before.is_soloed());
    assert_eq!(after.instrument(), before.instrument());
    // A reorder implemented as remove-then-recreate would mint a new ObjectId and lose these
    assert_eq!(*list.get(ids[0]).unwrap(), others[0]);
    assert_eq!(*list.get(ids[2]).unwrap(), others[1]);
    assert_eq!(list.index_of(ids[0]), Some(1));
    assert_eq!(list.index_of(ids[2]), Some(2));
}

// 2
#[test]
fn the_track_ceiling_refuses_the_overflowing_track_without_mutating_the_list() {
    let mut ids = IdGen::new(SEED);
    let mut list = TrackList::new();
    for index in 0..MAX_TRACKS {
        list.push(Track::new(ids.next_id(), &format!("t{index}"), TrackInstrument::Pulse).unwrap())
            .unwrap();
    }
    let before = list.clone();

    let overflow = Track::new(ids.next_id(), "one too many", TrackInstrument::Pulse).unwrap();
    assert_eq!(
        list.push(overflow),
        Err(TrackError::TrackLimit { limit: MAX_TRACKS })
    );
    assert_eq!(list.len(), MAX_TRACKS);
    assert_eq!(list, before, "a refused push must not mutate the list");
}

// 3
#[test]
fn blank_names_and_unknown_ids_are_refused_without_mutation() {
    let mut ids = IdGen::new(SEED);
    assert_eq!(
        Track::new(ids.next_id(), "   ", TrackInstrument::Pulse).err(),
        Some(TrackError::BlankName)
    );

    let (mut list, created) = list_of(&["one", "two"]);
    let before = list.clone();

    let track = list.get_mut(created[0]).unwrap();
    assert_eq!(track.set_name("\t"), Err(TrackError::BlankName));
    assert_eq!(
        track.name(),
        "one",
        "a refused rename must leave the name alone"
    );

    let stranger = IdGen::new(0xDEAD).next_id();
    assert_eq!(
        list.remove(stranger),
        Err(TrackError::UnknownTrack(stranger))
    );
    assert_eq!(list.len(), 2);

    let len = list.len();
    assert_eq!(
        list.reorder(created[0], len + 5),
        Err(TrackError::IndexOutOfRange {
            index: len + 5,
            len
        })
    );
    assert_eq!(
        list, before,
        "every refusal above must leave the list unchanged"
    );

    // Project-scoped uniqueness, which CORE-001 requires
    let duplicate = Track::new(created[0], "clash", TrackInstrument::Pulse).unwrap();
    assert_eq!(
        list.push(duplicate),
        Err(TrackError::DuplicateId(created[0]))
    );
}

// 4
#[test]
fn effective_gain_composes_level_mute_and_solo_in_that_order() {
    let (mut list, ids) = list_of(&["one", "two", "three"]);
    for (id, level) in ids.iter().zip([1.0_f32, 0.5, 2.0]) {
        list.get_mut(*id).unwrap().set_level(level);
    }

    // Nothing muted or soloed: effective gain is the fader position
    for (id, level) in ids.iter().zip([1.0_f32, 0.5, 2.0]) {
        assert_eq!(list.effective_gain(*id), Some(level));
    }

    list.get_mut(ids[1]).unwrap().set_muted(true);
    assert_eq!(list.effective_gain(ids[1]), Some(0.0));
    assert_eq!(list.effective_gain(ids[0]), Some(1.0));
    assert_eq!(list.effective_gain(ids[2]), Some(2.0));

    list.get_mut(ids[1]).unwrap().set_muted(false);
    list.get_mut(ids[2]).unwrap().set_soloed(true);
    assert!(list.any_soloed());
    assert_eq!(list.effective_gain(ids[2]), Some(2.0));
    assert_eq!(list.effective_gain(ids[0]), Some(0.0));
    assert_eq!(list.effective_gain(ids[1]), Some(0.0));

    // Solo is additive: a second soloed track keeps its own level
    list.get_mut(ids[0]).unwrap().set_soloed(true);
    assert_eq!(list.effective_gain(ids[0]), Some(1.0));
    assert_eq!(list.effective_gain(ids[2]), Some(2.0));
    assert_eq!(list.effective_gain(ids[1]), Some(0.0));

    // Mute dominates solo
    list.get_mut(ids[2]).unwrap().set_muted(true);
    assert_eq!(list.effective_gain(ids[2]), Some(0.0));

    assert_eq!(list.effective_gain(IdGen::new(0xBEEF).next_id()), None);
}

// 5
#[test]
fn structure_revision_advances_only_for_structural_edits() {
    let (mut list, ids) = list_of(&["one", "two"]);
    let base = list.structure_revision();

    // Mixer edits reach a live plan over the parameter lane, so they must NOT rebuild it
    list.get_mut(ids[0]).unwrap().set_level(0.4);
    assert_eq!(list.structure_revision(), base);
    list.get_mut(ids[0]).unwrap().set_muted(true);
    assert_eq!(list.structure_revision(), base);
    list.get_mut(ids[0]).unwrap().set_soloed(true);
    assert_eq!(list.structure_revision(), base);
    list.set_master_level(0.9);
    assert_eq!(list.structure_revision(), base);
    list.get_mut(ids[0]).unwrap().set_name("renamed").unwrap();
    assert_eq!(
        list.structure_revision(),
        base,
        "a rename changes no graph node, so it must not rebuild the plan"
    );

    // Structural edits change the compiled graph's shape and must be observable
    let mut ids_gen = IdGen::new(0xFACE);
    let previous = list.structure_revision();
    list.push(Track::new(ids_gen.next_id(), "three", TrackInstrument::Pulse).unwrap())
        .unwrap();
    assert!(list.structure_revision() > previous);

    let previous = list.structure_revision();
    list.reorder(ids[1], 0).unwrap();
    assert!(list.structure_revision() > previous);

    let previous = list.structure_revision();
    list.remove(ids[0]).unwrap();
    assert!(list.structure_revision() > previous);
}
