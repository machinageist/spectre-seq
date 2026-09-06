// Author: Jeff
// Date: 2026-09-06
// Description: R5 slice 7 evidence — the track edit vocabulary is exactly reversible
// Notes: Every case restores state that DIFFERS from the default, so an inverse that did nothing
//   at all would fail rather than coincide with a fresh list. The delete case is the one that
//   matters most: a track carries clips and mixer state, and an undo that rebuilt a track from
//   its name alone would look right in a list and be wrong in the project.

use spectre_core::{BeatTicks, IdGen, ObjectId, TempoMap, Transport};
use spectre_project::command::{EditHistory, ProjectCommand, Transaction};
use spectre_project::{
    ClipNote, ClipPlacement, MidiClip, ProjectDoc, Track, TrackInstrument, TrackList,
};

fn document() -> (ProjectDoc, Vec<ObjectId>) {
    let mut ids = IdGen::new(0x5EED);
    let mut tracks = TrackList::new();
    let mut identities = Vec::new();
    for name in ["Bass", "Lead", "Pad"] {
        let id = ids.next_id();
        tracks
            .push(Track::new(id, name, TrackInstrument::Filament).expect("a valid track"))
            .expect("the track is accepted");
        identities.push(id);
    }
    let document = ProjectDoc {
        id: ids.next_id(),
        name: "Session".into(),
        tempo_map: TempoMap::constant(120.0).expect("a valid tempo"),
        transport: Transport::default(),
        id_gen_state: ids.state(),
        tracks,
        devices: Vec::new(),
        view: Default::default(),
        unknown: Default::default(),
    };
    (document, identities)
}

fn history() -> EditHistory {
    EditHistory::new(16).expect("a bounded history")
}

fn apply(history: &mut EditHistory, document: &mut ProjectDoc, command: ProjectCommand) {
    history
        .apply(document, Transaction::single(command))
        .expect("the edit applies");
}

// The load-bearing case: a deleted track carries clips and mixer state, and an undo must bring
// back the track itself rather than one that merely shares its name
#[test]
fn undoing_a_delete_restores_the_whole_track_not_a_lookalike() {
    let (mut document, ids) = document();
    let mut ids_gen = IdGen::new(0xC1);
    let target = ids[1];

    // Give the doomed track state a rebuilt track would not have
    {
        let clip_id = ids_gen.next_id();
        let mut clip = MidiClip::new(clip_id, "Riff", BeatTicks(960 * 4)).expect("a valid clip");
        clip.insert_note(
            ClipNote::new(BeatTicks(0), BeatTicks(480), 0, 60, 0.8).expect("a valid note"),
        )
        .expect("the note is accepted");
        document
            .tracks
            .add_clip(clip)
            .expect("the clip is accepted");
        let track = document.tracks.get_mut(target).expect("the track exists");
        track.set_level(0.375);
        track.set_muted(true);
        track
            .clips_mut()
            .insert(
                ClipPlacement::new(ids_gen.next_id(), clip_id, BeatTicks(0))
                    .expect("a valid placement"),
                BeatTicks(960 * 4),
            )
            .expect("the placement is accepted");
    }
    let before = document
        .tracks
        .get(target)
        .expect("the track exists")
        .clone();

    let mut history = history();
    apply(
        &mut history,
        &mut document,
        ProjectCommand::remove_track(target),
    );
    assert!(
        document.tracks.get(target).is_none(),
        "the delete did not happen, so the undo below could not fail"
    );
    assert_eq!(document.tracks.len(), 2);

    assert!(history.undo(&mut document).expect("the undo applies"));
    let after = document.tracks.get(target).expect("the track is back");
    assert_eq!(after.id(), before.id(), "identity was not restored");
    assert_eq!(after.name(), before.name());
    assert_eq!(
        after.level(),
        before.level(),
        "mixer state was not restored"
    );
    assert!(after.is_muted(), "mute state was not restored");
    assert_eq!(
        after.clips().placements().len(),
        1,
        "the track came back with no clips, so it is a lookalike"
    );
    assert_eq!(
        document.tracks.index_of(target),
        Some(1),
        "the track came back in the wrong position"
    );
}

#[test]
fn undoing_an_add_removes_exactly_the_added_track() {
    let (mut document, ids) = document();
    let mut history = history();
    let new_id = IdGen::new(0xADD).next_id();
    let track = Track::new(new_id, "Perc", TrackInstrument::Filament).expect("a valid track");

    apply(
        &mut history,
        &mut document,
        ProjectCommand::insert_track(1, track),
    );
    assert_eq!(document.tracks.index_of(new_id), Some(1));
    assert_eq!(document.tracks.len(), 4);

    assert!(history.undo(&mut document).expect("the undo applies"));
    assert!(document.tracks.get(new_id).is_none());
    assert_eq!(document.tracks.len(), 3);
    assert_eq!(
        document
            .tracks
            .tracks()
            .iter()
            .map(Track::id)
            .collect::<Vec<_>>(),
        ids,
        "undoing the add disturbed the tracks it did not touch"
    );
}

// A level inverse must carry the STORED value, which is already clamped. Capturing the requested
// value instead would make undo restore a number the track never held
#[test]
fn undoing_an_out_of_range_level_restores_what_was_stored() {
    let (mut document, ids) = document();
    let target = ids[0];
    document
        .tracks
        .get_mut(target)
        .expect("the track exists")
        .set_level(1.5);
    let stored = document
        .tracks
        .get(target)
        .expect("the track exists")
        .level();

    let mut history = history();
    apply(
        &mut history,
        &mut document,
        ProjectCommand::set_track_level(target, 99.0),
    );
    let clamped = document
        .tracks
        .get(target)
        .expect("the track exists")
        .level();
    assert_ne!(
        clamped, 99.0,
        "the level was not clamped, so this is not the case under test"
    );

    assert!(history.undo(&mut document).expect("the undo applies"));
    assert_eq!(
        document
            .tracks
            .get(target)
            .expect("the track exists")
            .level(),
        stored
    );
}

#[test]
fn every_scalar_edit_round_trips_through_undo_and_redo() {
    let (mut document, ids) = document();
    let target = ids[2];
    let mut history = history();

    let before = {
        let track = document.tracks.get(target).expect("the track exists");
        (
            track.name().to_string(),
            track.level(),
            track.instrument_level(),
            track.is_muted(),
            track.is_soloed(),
        )
    };
    let master_before = document.tracks.master_level();

    for command in [
        ProjectCommand::set_track_name(target, "Renamed"),
        ProjectCommand::set_track_level(target, 0.25),
        ProjectCommand::set_track_instrument_level(target, 0.5),
        ProjectCommand::set_track_muted(target, true),
        ProjectCommand::set_track_soloed(target, true),
        ProjectCommand::set_master_level(0.125),
    ] {
        apply(&mut history, &mut document, command);
    }

    let edited = {
        let track = document.tracks.get(target).expect("the track exists");
        (
            track.name().to_string(),
            track.level(),
            track.instrument_level(),
            track.is_muted(),
            track.is_soloed(),
        )
    };
    assert_ne!(
        edited, before,
        "nothing changed, so the undos below cannot fail"
    );

    while history.undo(&mut document).expect("the undo applies") {}
    let restored = {
        let track = document.tracks.get(target).expect("the track exists");
        (
            track.name().to_string(),
            track.level(),
            track.instrument_level(),
            track.is_muted(),
            track.is_soloed(),
        )
    };
    assert_eq!(
        restored, before,
        "undoing every edit did not restore the start state"
    );
    assert_eq!(document.tracks.master_level(), master_before);

    while history.redo(&mut document).expect("the redo applies") {}
    let redone = {
        let track = document.tracks.get(target).expect("the track exists");
        (
            track.name().to_string(),
            track.level(),
            track.instrument_level(),
            track.is_muted(),
            track.is_soloed(),
        )
    };
    assert_eq!(
        redone, edited,
        "redoing every edit did not reach the edited state"
    );
}

// A transaction is all-or-nothing. A group whose last command is refused must leave the document
// exactly as it was, including the commands that had already succeeded
#[test]
fn a_refused_command_rolls_back_the_whole_group() {
    let (mut document, ids) = document();
    let before: Vec<_> = document.tracks.tracks().iter().map(Track::id).collect();
    let mut history = history();

    let group = Transaction::new(vec![
        ProjectCommand::set_track_name(ids[0], "Applied"),
        ProjectCommand::remove_track(ids[1]),
        // Addresses a track that does not exist, so the group must be refused
        ProjectCommand::set_track_level(ObjectId::from_raw(0xDEAD).expect("nonzero"), 0.5),
    ])
    .expect("a non-empty group");

    history
        .apply(&mut document, group)
        .expect_err("a group naming an absent track must be refused");

    assert_eq!(
        document
            .tracks
            .tracks()
            .iter()
            .map(Track::id)
            .collect::<Vec<_>>(),
        before,
        "the refused group left a track removed"
    );
    assert_eq!(
        document
            .tracks
            .get(ids[0])
            .expect("the track exists")
            .name(),
        "Bass",
        "the refused group left an applied rename behind"
    );
    assert!(
        !history.undo(&mut document).expect("no edit is available"),
        "a refused group was pushed onto the undo stack"
    );
}
