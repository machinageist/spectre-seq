// Author: Jeff
// Date: 2026-09-06
// Description: Evidence that the project tempo can be changed, reversibly, and reaches playback
// Notes: The transport rendered "120.00 BPM" as a string literal and AppModel exposed only a
//   getter, so nothing could be written at any other tempo. That is a hard blocker for making
//   music, not a missing convenience.
//
//   The command carries the WHOLE map rather than a number. A tempo map may hold segments, so an
//   inverse carrying only a bpm could not restore one that did — it would flatten a curve into a
//   constant and call that an undo.

use spectre_app::AppModel;
use spectre_core::{BeatTicks, TempoMap, TempoSegment};
use spectre_project::ClipNote;

const BAR: i64 = 960 * 4;

#[test]
fn the_project_tempo_can_be_changed() {
    let mut model = AppModel::prototype();
    assert_eq!(model.tempo_map().segments()[0].bpm, 120.0);

    model.set_tempo(140.0).expect("140 is a valid tempo");
    assert_eq!(model.tempo_map().segments()[0].bpm, 140.0);
}

#[test]
fn a_tempo_change_is_reversible() {
    let mut model = AppModel::prototype();
    model.set_tempo(174.0).expect("174 is a valid tempo");
    assert_eq!(model.tempo_map().segments()[0].bpm, 174.0);

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(
        model.tempo_map().segments()[0].bpm,
        120.0,
        "undoing the tempo change did not restore the original"
    );
    assert!(model.redo().expect("the redo applies"));
    assert_eq!(model.tempo_map().segments()[0].bpm, 174.0);
}

// The reason the command carries a map: undoing onto a project that had a tempo CURVE must
// restore the curve, not a constant taken from its first segment
#[test]
fn undoing_onto_a_tempo_curve_restores_the_curve() {
    let mut model = AppModel::prototype();
    let curve = TempoMap::new(vec![
        TempoSegment {
            start: BeatTicks(0),
            bpm: 120.0,
        },
        TempoSegment {
            start: BeatTicks(BAR),
            bpm: 150.0,
        },
    ])
    .expect("a two-segment map is valid");
    model
        .set_tempo_map(curve.clone())
        .expect("the curve is accepted");
    assert_eq!(model.tempo_map().segments().len(), 2);

    model.set_tempo(90.0).expect("90 is a valid tempo");
    assert_eq!(model.tempo_map().segments().len(), 1);

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(
        model.tempo_map().segments().len(),
        2,
        "the undo flattened a tempo curve into a constant"
    );
    assert_eq!(model.tempo_map(), &curve);
}

#[test]
fn an_impossible_tempo_is_refused_and_changes_nothing() {
    let mut model = AppModel::prototype();
    model.set_tempo(0.0).expect_err("zero bpm must be refused");
    model
        .set_tempo(f64::NAN)
        .expect_err("a non-finite tempo must be refused");
    assert_eq!(model.tempo_map().segments()[0].bpm, 120.0);
    assert!(
        !model.can_undo(),
        "a refused tempo change entered the history"
    );
}

// Tempo is what converts ticks to samples, so a change must reach the material a musician hears.
// Baking the same clip at two tempos must place its notes at different sample positions
#[test]
fn a_tempo_change_moves_where_notes_land() {
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
    model
        .add_note(
            clip,
            ClipNote::new(BeatTicks(BAR / 2), BeatTicks(480), 0, 60, 0.8).expect("a valid note"),
        )
        .expect("the note is accepted");

    let rate = spectre_core::SampleRate::new(48_000).expect("48 kHz is valid");
    let at_120 = model.tempo_map().ticks_to_samples(BeatTicks(BAR / 2), rate);

    model.set_tempo(240.0).expect("240 is a valid tempo");
    let at_240 = model.tempo_map().ticks_to_samples(BeatTicks(BAR / 2), rate);

    assert!(
        at_240.0 < at_120.0,
        "doubling the tempo did not move the note earlier: {at_120:?} vs {at_240:?}"
    );
}

// Tempo travels in the project document, so it must survive a save and reload
#[test]
fn a_tempo_change_survives_a_round_trip() {
    let mut model = AppModel::prototype();
    model.set_tempo(174.0).expect("174 is a valid tempo");

    let bytes =
        spectre_project::to_bytes(&spectre_app::project::project_envelope(&model, "Session"))
            .expect("encodes");
    let mut reloaded = AppModel::prototype();
    spectre_app::project::adopt(
        &mut reloaded,
        spectre_project::from_bytes(&bytes).expect("decodes"),
    )
    .expect("adopts");

    assert_eq!(
        reloaded.tempo_map().segments()[0].bpm,
        174.0,
        "the tempo did not survive the file"
    );
}
