// Author: Jeff
// Date: 2026-09-06
// Description: R5 slice 7 evidence — the shell's own edits are reversible through the model
// Notes: These drive AppModel's public mutators, not the command layer underneath. That is the
//   point: track_commands.rs proves the vocabulary reverses, and this proves the product path
//   actually goes through it. A mutator that bypassed the history would pass every test there
//   and fail every test here.

use spectre_app::project::{adopt, project_envelope};
use spectre_app::AppModel;
use spectre_project::{from_bytes, to_bytes, Track};

fn round_tripped(model: &AppModel) -> AppModel {
    let bytes = to_bytes(&project_envelope(model, "Round Trip")).expect("encodes");
    let mut target = AppModel::prototype();
    adopt(&mut target, from_bytes(&bytes).expect("decodes")).expect("adopts");
    target
}

#[test]
fn a_fresh_model_has_nothing_to_undo() {
    let model = AppModel::prototype();
    assert!(!model.can_undo());
    assert!(!model.can_redo());
}

#[test]
fn adding_a_track_through_the_shell_is_reversible() {
    let mut model = AppModel::prototype();
    let before = model.tracks().len();
    let id = model.add_track("Perc").expect("the track is added");

    assert_eq!(model.tracks().len(), before + 1);
    assert!(model.can_undo(), "the add did not reach the history");

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(model.tracks().len(), before);
    assert!(model.track_list().get(id).is_none());
    assert!(model.can_redo());

    assert!(model.redo().expect("the redo applies"));
    assert_eq!(model.track_list().index_of(id), Some(before));
}

// The case that makes undo worth having: a delete takes clips and mixer state with it
#[test]
fn undoing_a_delete_restores_the_track_whole_and_reselects_it() {
    let mut model = AppModel::prototype();
    let id = model.add_track("Doomed").expect("the track is added");
    model.set_track_level(id, 0.375).expect("the level is set");
    model.set_track_muted(id, true).expect("the mute is set");
    model.select_track(id);

    model.remove_track(id).expect("the track is removed");
    assert!(model.track_list().get(id).is_none());
    assert_ne!(
        model.selected_track_id(),
        Some(id),
        "selection still points at a removed track"
    );

    assert!(model.undo().expect("the undo applies"));
    let restored = model.track_list().get(id).expect("the track is back");
    assert_eq!(restored.name(), "Doomed");
    assert_eq!(restored.level(), 0.375, "mixer state was not restored");
    assert!(restored.is_muted(), "mute state was not restored");
}

// Undoing past a delete must not leave the shell pointing at a track that no longer exists
#[test]
fn undoing_an_add_moves_selection_off_the_vanished_track() {
    let mut model = AppModel::prototype();
    let id = model.add_track("Temporary").expect("the track is added");
    assert_eq!(model.selected_track_id(), Some(id));

    assert!(model.undo().expect("the undo applies"));
    let selected = model.selected_track_id().expect("something is selected");
    assert_ne!(selected, id, "selection survived onto a removed track");
    assert!(
        model.track_list().get(selected).is_some(),
        "selection points at a track the list does not hold"
    );
}

#[test]
fn every_mixer_edit_the_shell_makes_is_reversible() {
    let mut model = AppModel::prototype();
    let id = model
        .selected_track_id()
        .expect("the prototype selects a track");
    let before = {
        let track = model.track_list().get(id).expect("the track exists");
        (
            track.level(),
            track.is_muted(),
            track.is_soloed(),
            track.instrument_level(),
        )
    };
    let master_before = model.track_list().master_level();

    model.set_track_level(id, 0.25).expect("level");
    model.set_track_muted(id, true).expect("mute");
    model.set_track_soloed(id, true).expect("solo");
    model
        .set_track_instrument_level(id, 0.5)
        .expect("instrument level");
    model.set_master_level(0.125);
    model.rename_track(id, "Renamed").expect("rename");

    let after = {
        let track = model.track_list().get(id).expect("the track exists");
        (
            track.level(),
            track.is_muted(),
            track.is_soloed(),
            track.instrument_level(),
        )
    };
    assert_ne!(after, before, "nothing changed, so the undos cannot fail");

    while model.undo().expect("the undo applies") {}
    let restored = {
        let track = model.track_list().get(id).expect("the track exists");
        (
            track.level(),
            track.is_muted(),
            track.is_soloed(),
            track.instrument_level(),
        )
    };
    assert_eq!(restored, before);
    assert_eq!(model.track_list().master_level(), master_before);
    assert_ne!(
        model.track_list().get(id).expect("the track exists").name(),
        "Renamed",
        "the rename was not reversed"
    );
}

// Opening a project must not leave edits that address the previous project's identities
#[test]
fn opening_a_project_clears_the_history_of_the_previous_one() {
    let mut model = AppModel::prototype();
    model
        .add_track("From the old project")
        .expect("the track is added");
    assert!(model.can_undo());

    let mut other = AppModel::prototype();
    other
        .add_track("From the new project")
        .expect("the track is added");
    let opened = round_tripped(&other);

    let bytes = to_bytes(&project_envelope(&opened, "Other")).expect("encodes");
    adopt(&mut model, from_bytes(&bytes).expect("decodes")).expect("adopts");

    assert!(
        !model.can_undo(),
        "an undo from the previous project survived the open"
    );
    assert!(!model.undo().expect("no edit is available"));
    // Identity cannot be the discriminator here: IdGen is deterministically seeded (decision 4),
    // so two fresh prototypes mint the same ObjectIds and the old track's id is also the new
    // track's. The name is what tells the two projects apart
    let names: Vec<_> = model.tracks().iter().map(|track| track.name()).collect();
    assert!(
        names.contains(&"From the new project"),
        "the opened project's own track is missing: {names:?}"
    );
    assert!(
        !names.contains(&"From the old project"),
        "undoing resurrected work from the previous project: {names:?}"
    );
}

// Undo history is session state, not project state: it must not travel through a save
#[test]
fn edits_survive_a_save_and_reload_while_the_history_does_not() {
    let mut model = AppModel::prototype();
    let id = model.add_track("Kept").expect("the track is added");
    model.set_track_level(id, 0.5).expect("the level is set");

    let reloaded = round_tripped(&model);
    let track = reloaded
        .track_list()
        .get(id)
        .expect("the edit survived the file");
    assert_eq!(track.name(), "Kept");
    assert_eq!(track.level(), 0.5);
    assert!(
        !reloaded.can_undo(),
        "a reloaded project offered to undo an edit made before it was saved"
    );
    assert_eq!(
        reloaded.tracks().iter().map(Track::id).collect::<Vec<_>>(),
        model.tracks().iter().map(Track::id).collect::<Vec<_>>(),
        "identity did not survive the round trip"
    );
}
