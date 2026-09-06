// Author: Jeff
// Date: 2026-09-06
// Description: R5 slice 6 evidence — recovered work lands as unsaved and writes neither file
// Notes: The central assertion is that the dirty marker reads UNSAVED after a recovery. That is
//   the one direction the defect arrives from looking like success: a shell that adopted the
//   recovery and then set its saved snapshot from the same envelope would show "Saved" over work
//   that exists in no project file.

use spectre_app::project::{adopt_recovered, is_dirty, project_envelope};
use spectre_app::AppModel;
use spectre_project::journal::{journal_path, write_autosave};
use spectre_project::recovery::{inspect, Recovery, RecoveryOffer};
use spectre_project::{save_project_atomic, to_bytes};
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("spectre-recovery-bridge-{name}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("scratch directory");
    directory
}

// A saved project of one track, and a sidecar holding the same project with a second track added
// the way the shell's own edit path would add it
fn crashed_session(directory: &std::path::Path) -> (PathBuf, AppModel) {
    let path = directory.join("take.spectre");
    let mut model = AppModel::prototype();
    save_project_atomic(&path, &project_envelope(&model, "take")).expect("the project saves");

    model.add_track("Recovered").expect("a track is added");
    write_autosave(&path, &project_envelope(&model, "take")).expect("the sidecar is written");

    // The live model returns to the saved state, which is what an open does after a crash
    (path, AppModel::prototype())
}

fn offer_for(path: &std::path::Path) -> Box<RecoveryOffer> {
    match inspect(path).expect("inspection succeeds") {
        Recovery::Available(offer) => offer,
        other => panic!("unsaved work was not offered: {other:?}"),
    }
}

#[test]
fn recovered_work_arrives_unsaved() {
    let directory = scratch("unsaved");
    let (path, mut model) = crashed_session(&directory);
    let offer = offer_for(&path);

    let saved_snapshot = adopt_recovered(&mut model, &offer).expect("the recovery adopts");

    assert_eq!(
        model.tracks().len(),
        2,
        "the recovered work did not reach the model"
    );
    let current = to_bytes(&project_envelope(&model, "take")).ok();
    assert!(
        is_dirty(saved_snapshot.as_deref(), current.as_deref()),
        "the shell would have reported recovered work as already saved"
    );
}

// The saved snapshot must be the project file's own bytes. Asserting the flag alone would pass
// against an implementation that returned None, which is dirty for the wrong reason
#[test]
fn the_returned_snapshot_is_the_project_file_not_the_recovery() {
    let directory = scratch("snapshot");
    let (path, mut model) = crashed_session(&directory);
    let offer = offer_for(&path);
    let recovered_bytes = to_bytes(&offer.autosaved).ok();

    let saved_snapshot = adopt_recovered(&mut model, &offer).expect("the recovery adopts");

    assert_eq!(
        saved_snapshot,
        std::fs::read(&path).ok(),
        "the snapshot did not match the bytes the project file holds"
    );
    assert_ne!(
        saved_snapshot, recovered_bytes,
        "the snapshot came from the recovery, so the marker would read saved"
    );
}

// Decision 14: accepting changes neither disk version. The sidecar survives until a manual Save
#[test]
fn recovering_writes_neither_file() {
    let directory = scratch("untouched");
    let (path, mut model) = crashed_session(&directory);
    let project_before = std::fs::read(&path).expect("the project reads");
    let sidecar_before = std::fs::read(journal_path(&path)).expect("the sidecar reads");

    let offer = offer_for(&path);
    adopt_recovered(&mut model, &offer).expect("the recovery adopts");

    assert_eq!(
        std::fs::read(&path).expect("the project still reads"),
        project_before,
        "recovering overwrote the saved project"
    );
    assert_eq!(
        std::fs::read(journal_path(&path)).expect("the sidecar still reads"),
        sidecar_before,
        "recovering consumed the sidecar before a manual Save"
    );
}

// A refused adoption must leave the offer usable, so the shell can present it again rather than
// dropping the only handle on the work
#[test]
fn a_refused_adoption_leaves_the_offer_intact() {
    let directory = scratch("refused");
    let (path, mut model) = crashed_session(&directory);
    let mut offer = offer_for(&path);
    // A device this build has no catalogue entry for. adopt refuses rather than repairing
    offer.autosaved.project.devices[0].key = "not-a-device".into();

    let before = offer.autosaved.project.devices[0].key.clone();
    adopt_recovered(&mut model, &offer).expect_err("an unknown device must be refused");
    assert_eq!(
        offer.autosaved.project.devices[0].key, before,
        "the refused adoption consumed the offer"
    );
    assert_eq!(
        model.tracks().len(),
        1,
        "a refused adoption left the model half-changed"
    );
}
