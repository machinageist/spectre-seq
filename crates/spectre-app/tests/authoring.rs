// Author: Jeff
// Date: 2026-09-06
// Description: Evidence that a musician can author MIDI material through the model
// Notes: This is the API a piano roll is a view over, and none of it existed. `create_clip` had
//   zero callers outside tests and `MidiClip::insert_note` had none outside the offline fixture
//   builder, so nothing in the product could author a single note.
//
//   R4-5 recorded that clip undo was impossible "because EditHistory mutates ProjectDoc and
//   ProjectDoc has no clip field until slice 7". Slice 7 landed and clips travel inside
//   TrackList, which EditScope carries, so these are the first reversible clip edits.

use spectre_app::AppModel;
use spectre_core::{BeatTicks, ObjectId};
use spectre_project::ClipNote;

// ClipNote::new is the one validating constructor; these tests go through it exactly as a piano
// roll would
fn note_at(start: i64, length: i64, pitch: u8, velocity: f32) -> ClipNote {
    ClipNote::new(BeatTicks(start), BeatTicks(length), 0, pitch, velocity)
        .expect("a valid note for this fixture")
}

const BAR: i64 = 960 * 4;

fn session() -> (AppModel, ObjectId, ObjectId, ObjectId) {
    let mut model = AppModel::prototype();
    let track = model.add_track("Keys").expect("the track is added");
    let placement = model
        .create_clip(track, "Riff", BeatTicks(BAR), BeatTicks(0))
        .expect("the clip is created");
    let clip = model
        .track_list()
        .get(track)
        .expect("the track exists")
        .clips()
        .get(placement)
        .expect("the placement exists")
        .clip();
    (model, track, placement, clip)
}

fn notes(model: &AppModel, clip: ObjectId) -> Vec<(i64, u8, f32)> {
    model
        .clip_notes(clip)
        .expect("the clip exists")
        .iter()
        .map(|note| (note.start().0, note.note(), note.velocity()))
        .collect()
}

#[test]
fn a_note_can_be_written_into_a_clip() {
    let (mut model, _, _, clip) = session();
    assert!(notes(&model, clip).is_empty());

    model
        .add_note(clip, note_at(0, 480, 60, 0.8))
        .expect("the note is accepted");
    assert_eq!(notes(&model, clip), vec![(0, 60, 0.8)]);
}

// A chord: three notes on one tick, which the model has always been able to represent and the
// instruments can now actually sound
#[test]
fn a_chord_can_be_written_on_one_tick() {
    let (mut model, _, _, clip) = session();
    for pitch in [60, 64, 67] {
        model
            .add_note(clip, note_at(0, 480, pitch, 0.8))
            .expect("the note is accepted");
    }
    assert_eq!(model.clip_notes(clip).expect("the clip exists").len(), 3);
}

#[test]
fn every_authoring_edit_is_reversible() {
    let (mut model, _, _, clip) = session();
    model
        .add_note(clip, note_at(0, 480, 60, 0.8))
        .expect("added");
    model
        .add_note(clip, note_at(480, 480, 64, 0.6))
        .expect("added");
    let written = notes(&model, clip);
    assert_eq!(written.len(), 2);

    model.remove_note(clip, 0).expect("removed");
    assert_eq!(notes(&model, clip).len(), 1);

    // Exactly the three note edits; undoing further would step over create_clip and add_track
    for _ in 0..3 {
        assert!(model.undo().expect("the undo applies"));
    }
    assert!(notes(&model, clip).is_empty());

    for _ in 0..3 {
        assert!(model.redo().expect("the redo applies"));
    }
    assert_eq!(notes(&model, clip).len(), 1);
}

// A drag, a resize or a velocity change. Notes are kept sorted, so the edit moves the index --
// which is why this is one atomic group rather than a mutation in place
#[test]
fn replacing_a_note_moves_it_and_undoes_as_one_step() {
    let (mut model, _, _, clip) = session();
    model
        .add_note(clip, note_at(0, 480, 60, 0.8))
        .expect("added");
    model
        .add_note(clip, note_at(1920, 480, 72, 0.5))
        .expect("added");
    let before = notes(&model, clip);

    // Drag the first note past the second, so its index really does change
    model
        .replace_note(clip, 0, note_at(2880, 480, 60, 0.8))
        .expect("the replacement applies");
    let after = notes(&model, clip);
    assert_eq!(after.len(), 2);
    assert_eq!(after[1], (2880, 60, 0.8), "the note did not move");
    assert_eq!(after[0], (1920, 72, 0.5), "the untouched note moved");

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(
        notes(&model, clip),
        before,
        "a replacement undid as two steps or restored the wrong note"
    );
}

// Creating a clip is one undo step, not two, even though it writes material and a placement
#[test]
fn creating_a_clip_undoes_as_one_step() {
    let mut model = AppModel::prototype();
    let track = model.add_track("Keys").expect("the track is added");
    let placement = model
        .create_clip(track, "Riff", BeatTicks(BAR), BeatTicks(0))
        .expect("created");
    assert_eq!(model.track_list().clips().len(), 1);

    assert!(model.undo().expect("the undo applies"));
    assert!(
        model.track_list().clips().is_empty(),
        "undoing a clip left its material behind"
    );
    assert!(model.clip_track(placement).is_none());
}

#[test]
fn deleting_a_clip_restores_its_notes_on_undo() {
    let (mut model, _, placement, clip) = session();
    for pitch in [60, 64] {
        model
            .add_note(clip, note_at(0, 480, pitch, 0.7))
            .expect("added");
    }
    model.delete_clip(placement).expect("deleted");
    assert!(model.track_list().clips().is_empty());

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(
        model.clip_notes(clip).expect("the clip is back").len(),
        2,
        "the restored clip came back empty"
    );
}

// A placement can be dragged along the timeline, and a refused move changes nothing
#[test]
fn a_clip_can_be_moved_and_an_overlapping_move_is_refused() {
    let (mut model, track, placement, _) = session();
    let second = model
        .create_clip(track, "Second", BeatTicks(BAR), BeatTicks(BAR * 2))
        .expect("created");

    model
        .move_clip(placement, BeatTicks(BAR * 4))
        .expect("the move applies");
    assert_eq!(
        model
            .track_list()
            .get(track)
            .expect("the track exists")
            .clips()
            .get(placement)
            .expect("the placement exists")
            .start(),
        BeatTicks(BAR * 4)
    );

    // Straight onto the second clip
    model
        .move_clip(placement, BeatTicks(BAR * 2))
        .expect_err("an overlapping move must be refused");
    assert_eq!(
        model
            .track_list()
            .get(track)
            .expect("the track exists")
            .clips()
            .get(placement)
            .expect("the placement survived the refusal")
            .start(),
        BeatTicks(BAR * 4),
        "a refused move left the placement somewhere new"
    );
    assert!(model.clip_track(second).is_some());
}

// A note outside its clip is refused rather than clamped, the way every other load-path value is
#[test]
fn a_note_outside_the_clip_is_refused() {
    let (mut model, _, _, clip) = session();
    // Past the clip's end: the note itself is valid, so the CLIP is what refuses it
    model
        .add_note(clip, note_at(BAR * 2, 480, 60, 0.8))
        .expect_err("a note past the clip's end must be refused");
    // Past MIDI's range: the note cannot be constructed at all, which is the earlier refusal
    ClipNote::new(BeatTicks(0), BeatTicks(480), 0, 200, 0.8)
        .expect_err("a note number past 127 must be refused");
    assert!(notes(&model, clip).is_empty());
}
