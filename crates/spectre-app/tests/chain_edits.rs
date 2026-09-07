// Author: Jeff
// Date: 2026-09-06
// Description: M3 evidence — the shell can build a chain in any order, reversibly
// Notes: effect_chain.rs proves the model holds a chain and the graph renders it. This proves
//   the product path reaches it: every edit goes through AppModel, and every one is undoable.
//   A method that mutated the track list directly would pass there and fail here.

use spectre_app::AppModel;
use spectre_project::{TrackEffect, TrackInsert};

fn model_with_track() -> (AppModel, spectre_core::ObjectId) {
    let mut model = AppModel::prototype();
    let id = model.add_track("Chained").expect("the track is added");
    (model, id)
}

fn depths(model: &AppModel, track: spectre_core::ObjectId) -> Vec<f32> {
    model
        .track_list()
        .get(track)
        .expect("the track exists")
        .inserts()
        .iter()
        .map(|insert| insert.depth())
        .collect()
}

#[test]
fn effects_chain_in_the_order_they_are_added() {
    let (mut model, track) = model_with_track();
    for depth in [0.1, 0.2, 0.3] {
        model
            .append_effect(track, TrackInsert::new(TrackEffect::Gloam, depth))
            .expect("the effect is added");
    }
    assert_eq!(depths(&model, track), vec![0.1, 0.2, 0.3]);
}

// "Any permutation" is the requirement, so inserting into the middle is a first-class edit
#[test]
fn an_effect_can_be_added_anywhere_in_the_chain() {
    let (mut model, track) = model_with_track();
    model
        .append_effect(track, TrackInsert::new(TrackEffect::Gloam, 0.1))
        .expect("added");
    model
        .append_effect(track, TrackInsert::new(TrackEffect::Gloam, 0.3))
        .expect("added");
    model
        .add_effect(track, 1, TrackInsert::new(TrackEffect::Gloam, 0.2))
        .expect("inserted in the middle");
    assert_eq!(depths(&model, track), vec![0.1, 0.2, 0.3]);
}

#[test]
fn every_chain_edit_the_shell_makes_is_reversible() {
    let (mut model, track) = model_with_track();
    for depth in [0.1, 0.2, 0.3] {
        model
            .append_effect(track, TrackInsert::new(TrackEffect::Gloam, depth))
            .expect("added");
    }
    let built = depths(&model, track);

    model.move_effect(track, 0, 2).expect("moved");
    assert_eq!(depths(&model, track), vec![0.2, 0.3, 0.1]);
    model.remove_effect(track, 1).expect("removed");
    assert_eq!(depths(&model, track), vec![0.2, 0.1]);
    model.set_effect_depth(track, 0, 0.9).expect("depth set");
    assert_eq!(depths(&model, track), vec![0.9, 0.1]);

    // Exactly the six chain edits: three appends, a move, a removal, a depth. Undoing further
    // would step over add_track, which is in the same history and would take the track with it
    for _ in 0..6 {
        assert!(model.undo().expect("the undo applies"));
    }
    assert!(
        depths(&model, track).is_empty(),
        "undoing every chain edit left effects behind"
    );

    for _ in 0..6 {
        assert!(model.redo().expect("the redo applies"));
    }
    assert_eq!(depths(&model, track), vec![0.9, 0.1]);
    assert_ne!(
        built,
        depths(&model, track),
        "the test could not have failed"
    );
}

// A removed effect must come back whole, not rebuilt from its position
#[test]
fn undoing_a_removal_restores_the_effect_that_was_there() {
    let (mut model, track) = model_with_track();
    model
        .append_effect(track, TrackInsert::new(TrackEffect::Gloam, 0.42))
        .expect("added");
    model.remove_effect(track, 0).expect("removed");
    assert!(depths(&model, track).is_empty());

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(
        depths(&model, track),
        vec![0.42],
        "the restored effect is not the one that was removed"
    );
}

// A move is its own inverse with the ends swapped; that has to hold for a non-adjacent pair
#[test]
fn undoing_a_move_restores_the_original_order() {
    let (mut model, track) = model_with_track();
    for depth in [0.1, 0.2, 0.3, 0.4] {
        model
            .append_effect(track, TrackInsert::new(TrackEffect::Gloam, depth))
            .expect("added");
    }
    model.move_effect(track, 3, 0).expect("moved");
    assert_eq!(depths(&model, track), vec![0.4, 0.1, 0.2, 0.3]);

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(depths(&model, track), vec![0.1, 0.2, 0.3, 0.4]);
}

// Adding a device changes the graph's shape; changing its depth does not. The shell reads this
// to decide whether the running plan is stale, so getting it wrong either strands an edit or
// forces a needless stream restart
#[test]
fn structure_advances_for_shape_edits_and_not_for_depth() {
    let (mut model, track) = model_with_track();
    let before = model.track_list().structure_revision();
    model
        .append_effect(track, TrackInsert::new(TrackEffect::Gloam, 0.5))
        .expect("added");
    let after_add = model.track_list().structure_revision();
    assert_ne!(after_add, before, "adding an effect is a shape change");

    model.set_effect_depth(track, 0, 0.75).expect("depth set");
    assert_eq!(
        model.track_list().structure_revision(),
        after_add,
        "a depth edit rebuilt the plan; it travels the parameter lane"
    );

    model.remove_effect(track, 0).expect("removed");
    assert_ne!(
        model.track_list().structure_revision(),
        after_add,
        "removing an effect is a shape change"
    );
}

#[test]
fn a_chain_edit_on_an_unknown_track_is_refused_and_changes_nothing() {
    let (mut model, track) = model_with_track();
    model
        .append_effect(track, TrackInsert::new(TrackEffect::Gloam, 0.5))
        .expect("added");
    let absent = spectre_core::ObjectId::from_raw(0xDEAD).expect("nonzero");

    model
        .append_effect(absent, TrackInsert::new(TrackEffect::Gloam, 0.5))
        .expect_err("an unknown track must be refused");
    model
        .remove_effect(track, 9)
        .expect_err("a position past the chain must be refused");

    assert_eq!(depths(&model, track), vec![0.5]);
    // A refused edit must not enter the history, or undo would step over nothing
    assert!(model.undo().expect("the undo applies"));
    assert!(depths(&model, track).is_empty());
}

// ---- reachability ----
//
// The chain model was complete, ordered, unbounded and reversible, and a track created in the
// product had no effects and no way to gain one: Track::new starts with an empty chain and the
// only TrackInsert::new in any src/ was in the offline fixture builder. Gloam -- the alpha's own
// effect -- was unreachable on any track a musician made, exactly as Filament was.

#[test]
fn a_new_track_starts_with_no_effects_and_can_gain_one() {
    let (mut model, track) = model_with_track();
    assert!(
        depths(&model, track).is_empty(),
        "a new track already had an effect; this test asserts it can GAIN one"
    );

    model
        .append_effect(track, TrackInsert::new(TrackEffect::Gloam, 0.5))
        .expect("the effect is added");
    assert_eq!(depths(&model, track), vec![0.5]);
}

// The chain reaches the compiled graph, so an added effect is in the signal path rather than
// only in the model
#[test]
fn an_added_effect_reaches_the_compiled_graph() {
    let (mut model, track) = model_with_track();
    model
        .append_effect(track, TrackInsert::new(TrackEffect::Gloam, 0.5))
        .expect("added");

    let mut ids = spectre_core::IdGen::new(spectre_app::engine::APP_GRAPH_SEED);
    let list = model.track_list();
    let (graph, nodes) =
        spectre_project::build_track_graph(list, &mut ids).expect("the graph builds");
    let index = list.index_of(track).expect("the track is present");
    assert_eq!(
        nodes.inserts[index].len(),
        1,
        "the added effect built no node"
    );
    let mut factory = spectre_project::track_device_factory(list, &nodes);
    graph
        .compile(nodes.master.node, 256, &mut factory)
        .expect("the plan compiles with the effect in it");
}

// Every parameter of a chained effect is addressable by POSITION, which is what lets a second
// Gloam be reached -- Shape resolves by key and would only ever find the first
#[test]
fn a_second_effect_in_the_chain_is_addressable() {
    let (mut model, track) = model_with_track();
    for depth in [0.2, 0.7] {
        model
            .append_effect(track, TrackInsert::new(TrackEffect::Gloam, depth))
            .expect("added");
    }
    let list = model.track_list();
    let index = list.index_of(track).expect("present");

    let first = list.insert_parameter_index(index, 0, spectre_dsp::GLOAM_DEPTH);
    let second = list.insert_parameter_index(index, 1, spectre_dsp::GLOAM_DEPTH);
    assert!(first.is_some() && second.is_some());
    assert_ne!(
        first, second,
        "both Gloams resolve to one target, so the second is unreachable"
    );

    // And its depth is edited by position, not by key
    model
        .set_effect_depth(track, 1, 0.9)
        .expect("the second effect's depth is set");
    assert_eq!(depths(&model, track), vec![0.2, 0.9]);
}
