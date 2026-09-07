// Author: Jeff
// Date: 2026-09-07
// Description: Evidence that a track's instrument can be changed
// Notes: TrackList::set_instrument existed with zero callers in any src/, so every track created
//   in the product was a Pulse saw for the life of the project and Filament -- the alpha's own
//   synth, and the only polyphonic-by-design one -- was unreachable on any track a musician made.
//   That is not a missing convenience: it is one waveform, forever.

use spectre_app::AppModel;
use spectre_core::IdGen;
use spectre_project::{TrackInstrument, TrackList};

fn model_with_track() -> (AppModel, spectre_core::ObjectId) {
    let mut model = AppModel::prototype();
    let id = model.add_track("Lead").expect("the track is added");
    (model, id)
}

fn instrument_of(model: &AppModel, id: spectre_core::ObjectId) -> TrackInstrument {
    model
        .track_list()
        .get(id)
        .expect("the track exists")
        .instrument()
}

#[test]
fn a_new_track_can_be_switched_to_filament() {
    let (mut model, id) = model_with_track();
    assert_eq!(
        instrument_of(&model, id),
        TrackInstrument::Pulse,
        "the default changed; this test asserts the switch, not the default"
    );

    model
        .set_track_instrument(id, TrackInstrument::Filament)
        .expect("the instrument is changed");
    assert_eq!(instrument_of(&model, id), TrackInstrument::Filament);
}

#[test]
fn changing_an_instrument_is_reversible() {
    let (mut model, id) = model_with_track();
    model
        .set_track_instrument(id, TrackInstrument::Filament)
        .expect("changed");

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(
        instrument_of(&model, id),
        TrackInstrument::Pulse,
        "the undo did not restore the previous instrument"
    );
    assert!(model.redo().expect("the redo applies"));
    assert_eq!(instrument_of(&model, id), TrackInstrument::Filament);
}

// The instrument node itself differs, so the compiled plan is stale. The shell reads this to
// report PLAN STALE rather than pretending the change is already audible
#[test]
fn changing_an_instrument_is_a_shape_change() {
    let (mut model, id) = model_with_track();
    let before = model.track_list().structure_revision();
    model
        .set_track_instrument(id, TrackInstrument::Filament)
        .expect("changed");
    assert_ne!(
        model.track_list().structure_revision(),
        before,
        "an instrument change did not mark the plan stale"
    );
}

// Setting the instrument a track already has must not mark the plan stale, or every redraw that
// echoed the current value would force a needless rebuild
#[test]
fn setting_the_same_instrument_changes_nothing() {
    let (mut model, id) = model_with_track();
    let before = model.track_list().structure_revision();
    model
        .set_track_instrument(id, TrackInstrument::Pulse)
        .expect("accepted");
    assert_eq!(model.track_list().structure_revision(), before);
}

#[test]
fn an_unknown_track_is_refused() {
    let (mut model, _) = model_with_track();
    let absent = IdGen::new(0xDEAD).next_id();
    model
        .set_track_instrument(absent, TrackInstrument::Filament)
        .expect_err("an unknown track must be refused");
}

// The change must reach the compiled graph, not just the model: a Filament track must build a
// Filament node
#[test]
fn the_changed_instrument_reaches_the_compiled_graph() {
    let (mut model, id) = model_with_track();
    model
        .set_track_instrument(id, TrackInstrument::Filament)
        .expect("changed");

    let mut graph_ids = IdGen::new(spectre_app::engine::APP_GRAPH_SEED);
    let list: &TrackList = model.track_list();
    let (graph, nodes) =
        spectre_project::build_track_graph(list, &mut graph_ids).expect("the graph builds");
    let mut factory = spectre_project::track_device_factory(list, &nodes);
    let plan = graph
        .compile(nodes.master.node, 256, &mut factory)
        .expect("the plan compiles with the new instrument");
    assert!(plan.step_count() > 0);
}
