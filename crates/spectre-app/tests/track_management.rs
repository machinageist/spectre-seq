// Author: Jeff
// Date: 2026-09-07
// Description: Evidence that a track list is manageable — delete, rename, reorder, master level
// Notes: Every one of these existed on AppModel, reversible, with no caller in any src/. A track
//   added by mistake could never be removed; a track kept whatever name it was created with; the
//   output level could not be changed at all. Found by auditing for public API with no caller
//   rather than by working down a feature list.

use spectre_app::AppModel;
use spectre_core::ObjectId;
use spectre_project::Track;

fn names(model: &AppModel) -> Vec<&str> {
    model.tracks().iter().map(Track::name).collect()
}

fn three_tracks() -> (AppModel, Vec<ObjectId>) {
    let mut model = AppModel::prototype();
    let ids = ["Bass", "Lead", "Pad"]
        .iter()
        .map(|name| model.add_track(*name).expect("the track is added"))
        .collect();
    (model, ids)
}

#[test]
fn a_track_can_be_deleted_and_the_delete_undone() {
    let (mut model, ids) = three_tracks();
    let before = model.tracks().len();

    model.remove_track(ids[1]).expect("the track is removed");
    assert_eq!(model.tracks().len(), before - 1);
    assert!(model.track_list().get(ids[1]).is_none());

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(model.tracks().len(), before);
    assert_eq!(
        model.track_list().index_of(ids[1]),
        Some(2),
        "the restored track came back in the wrong position"
    );
}

#[test]
fn a_track_can_be_renamed_and_the_rename_undone() {
    let (mut model, ids) = three_tracks();
    model
        .rename_track(ids[0], "Sub")
        .expect("the rename applies");
    assert!(names(&model).contains(&"Sub"));

    assert!(model.undo().expect("the undo applies"));
    assert!(
        names(&model).contains(&"Bass"),
        "the undo did not restore the previous name"
    );
}

#[test]
fn a_blank_rename_is_refused_and_changes_nothing() {
    let (mut model, ids) = three_tracks();
    model
        .rename_track(ids[0], "   ")
        .expect_err("a blank name must be refused");
    assert!(names(&model).contains(&"Bass"));
}

// Order is the mixer's order and the sum's bus order, so moving a track is a real edit
#[test]
fn a_track_can_be_reordered_and_the_move_undone() {
    let (mut model, ids) = three_tracks();
    let before: Vec<_> = model.tracks().iter().map(Track::id).collect();

    model.reorder_track(ids[2], 1).expect("the move applies");
    assert_eq!(model.track_list().index_of(ids[2]), Some(1));

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(
        model.tracks().iter().map(Track::id).collect::<Vec<_>>(),
        before,
        "the undo did not restore the original order"
    );
}

// Deleting the selected track must leave selection on something that exists
#[test]
fn deleting_the_selected_track_moves_selection() {
    let (mut model, ids) = three_tracks();
    model.select_track(ids[2]);
    model.remove_track(ids[2]).expect("the track is removed");

    let selected = model.selected_track_id().expect("something is selected");
    assert_ne!(selected, ids[2]);
    assert!(
        model.track_list().get(selected).is_some(),
        "selection points at a track the list does not hold"
    );
}

#[test]
fn the_master_level_can_be_changed() {
    let mut model = AppModel::prototype();
    let before = model.track_list().master_level();
    model.set_master_level(0.25);
    assert_eq!(model.track_list().master_level(), 0.25);
    assert_ne!(before, 0.25, "the fixture already sat at the test value");

    assert!(model.undo().expect("the undo applies"));
    assert_eq!(model.track_list().master_level(), before);
}

// Deleting a track takes its clips with it, and an undo must bring them back
#[test]
fn deleting_a_track_and_undoing_restores_its_clips() {
    let mut model = AppModel::prototype();
    let track = model.add_track("Keys").expect("the track is added");
    model
        .create_clip(
            track,
            "Riff",
            spectre_core::BeatTicks(960 * 4),
            spectre_core::BeatTicks(0),
        )
        .expect("the clip is created");

    model.remove_track(track).expect("the track is removed");
    assert!(model.track_list().get(track).is_none());

    assert!(model.undo().expect("the undo applies"));
    let restored = model.track_list().get(track).expect("the track is back");
    assert_eq!(
        restored.clips().placements().len(),
        1,
        "the restored track came back without its clips"
    );
}
