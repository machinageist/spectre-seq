// Author: Jeff
// Date: 2026-09-06
// Description: R5 slice 6 evidence — the shell's autosave trigger and its save/sidecar ordering
// Notes: Every case here is arranged to be able to fail. The trigger is content-derived, so a
//   test that fed it one buffer would pass against an implementation that ignored its inputs;
//   each case below differs from its neighbour in exactly the field under test.

use spectre_app::project::{
    autosave_action, commit_save, project_envelope, run_autosave, AutosaveAction, SaveOutcome,
};
use spectre_app::AppModel;
use spectre_project::journal::journal_path;
use spectre_project::{load_project, save_project_atomic};
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("spectre-autosave-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("scratch directory");
    directory
}

// ---- the trigger ----

#[test]
fn work_that_differs_from_both_disk_copies_is_written() {
    assert_eq!(
        autosave_action(true, Some(b"edited"), Some(b"saved"), Some(b"older")),
        AutosaveAction::Write
    );
}

#[test]
fn work_already_journaled_is_not_rewritten() {
    assert_eq!(
        autosave_action(true, Some(b"edited"), Some(b"saved"), Some(b"edited")),
        AutosaveAction::Skip
    );
}

// The interesting branch: undoing back to the saved state leaves a sidecar describing work that
// no longer exists, which would be offered on the next launch as though it were unsaved
#[test]
fn a_document_matching_the_saved_file_retires_its_sidecar() {
    assert_eq!(
        autosave_action(true, Some(b"saved"), Some(b"saved"), Some(b"older")),
        AutosaveAction::Discard
    );
}

#[test]
fn a_project_with_no_path_has_nowhere_to_journal() {
    assert_eq!(
        autosave_action(false, Some(b"edited"), None, None),
        AutosaveAction::Skip
    );
}

// is_dirty already reports an unencodable document dirty. There are no bytes to write, so the
// trigger must not claim there are
#[test]
fn a_document_that_did_not_encode_is_not_journaled() {
    assert_eq!(
        autosave_action(true, None, Some(b"saved"), None),
        AutosaveAction::Skip
    );
}

// A first edit before any save has no saved bytes to compare against and must still be protected
#[test]
fn unsaved_work_with_no_file_yet_is_written() {
    assert_eq!(
        autosave_action(true, Some(b"edited"), None, None),
        AutosaveAction::Write
    );
}

// ---- the save/sidecar order ----

#[test]
fn a_durable_save_retires_the_sidecar_it_no_longer_needs() {
    let directory = scratch("commit");
    let path = directory.join("take.spectre");
    let model = AppModel::prototype();
    let envelope = project_envelope(&model, "take");

    run_autosave(&path, &envelope).expect("the sidecar is written");
    assert!(journal_path(&path).exists(), "the sidecar was not written");

    match commit_save(&path, &envelope) {
        SaveOutcome::Saved => {}
        other => panic!("a clean save reported {other:?}"),
    }
    assert!(
        !journal_path(&path).exists(),
        "a committed save left its obsolete sidecar behind"
    );
    assert_eq!(load_project(&path).expect("loads").project.name, "take");
}

// The sidecar must never be retired before the save that replaces it has succeeded. A failing
// save leaves the project untouched, so the sidecar is still the only copy of the work
#[test]
fn a_failed_save_keeps_the_sidecar() {
    let directory = scratch("failed");
    let path = directory.join("take.spectre");
    let model = AppModel::prototype();
    let envelope = project_envelope(&model, "take");
    run_autosave(&path, &envelope).expect("the sidecar is written");

    // The failing save must target THE SAME path the sidecar belongs to, or the ordering rule
    // is not under test at all: journal_path is derived from the project path, so a save that
    // fails somewhere else touches a different sidecar and the assertion cannot fire.
    //
    // A directory standing where the project file goes fails at the rename with the parent
    // still writable -- so a discard-first implementation would successfully remove the sidecar,
    // which is what makes this discriminating
    std::fs::create_dir(&path).expect("a directory stands where the project file would go");
    match commit_save(&path, &envelope) {
        SaveOutcome::Failed(_) => {}
        other => panic!("a save onto a directory reported {other:?}"),
    }
    assert!(
        journal_path(&path).exists(),
        "a failed save discarded the only remaining copy of the work"
    );
}

// A sidecar is written beside the project and the project file itself is never opened by that
// path. R5 slice 2 asserts this in spectre-project; this is the same claim through the shell's
// own boundary, where the envelope comes from the live model
#[test]
fn an_autosave_through_the_shell_does_not_touch_the_project_file() {
    let directory = scratch("untouched");
    let path = directory.join("take.spectre");
    let mut model = AppModel::prototype();
    let saved = project_envelope(&model, "take");
    save_project_atomic(&path, &saved).expect("the project is saved");
    let on_disk = std::fs::read(&path).expect("the project reads back");

    model
        .add_track("Second")
        .expect("a track is added after the save");
    let edited = project_envelope(&model, "take");
    assert_ne!(
        saved.project.tracks.tracks().len(),
        edited.project.tracks.tracks().len(),
        "the edit did not change the document, so this test could not fail"
    );

    run_autosave(&path, &edited).expect("the sidecar is written");
    assert_eq!(
        std::fs::read(&path).expect("the project still reads back"),
        on_disk,
        "an autosave modified the saved project"
    );
    assert!(journal_path(&path).exists());
}
