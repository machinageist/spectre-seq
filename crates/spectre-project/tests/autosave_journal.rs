// Author: Jeff
// Date: 2026-08-29
// Description: R5-2 evidence — journaled autosave to a sidecar never touches the project
// Notes: Decision 14's model is "journaled autosave to sidecar + atomic rename saves". The
//   sidecar half is what this covers. Every test here is about a boundary rather than a feature:
//   what autosave must NOT do is the reason it is safe to run it while a musician works.

use spectre_core::{IdGen, TempoMap, Transport};
use spectre_project::{
    discard_autosave, journal_path, load_project, read_autosave, save_project_atomic,
    write_autosave, ProjectDoc, ProjectEnvelope, Track, TrackInstrument, TrackList, ViewDoc,
    SCHEMA_VERSION,
};
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("spectre-autosave-{name}"));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch directory");
    path
}

fn envelope(seed: u64, name: &str, tracks: &[&str]) -> ProjectEnvelope {
    let mut ids = IdGen::new(seed);
    let project_id = ids.next_id();
    let mut list = TrackList::new();
    for track in tracks {
        let id = ids.next_id();
        list.push(Track::new(id, track, TrackInstrument::Pulse).expect("valid name"))
            .expect("the track fits");
    }
    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: project_id,
            name: name.into(),
            tempo_map: TempoMap::constant(120.0).expect("a constant tempo is valid"),
            transport: Transport::new(),
            id_gen_state: ids.state(),
            tracks: list,
            devices: Vec::new(),
            view: ViewDoc::default(),
            unknown: serde_json::Map::new(),
        },
        unknown: serde_json::Map::new(),
    }
}

// The property the whole model rests on. An autosave that could damage the saved project would be
// worse than no autosave, so this asserts the project file is byte-identical -- not merely still
// loadable -- across an autosave that carries completely different content
#[test]
fn an_autosave_does_not_touch_the_project_file() {
    let directory = scratch("untouched");
    let path = directory.join("take.spectre");
    let saved = envelope(0x11, "Saved", &["Bass"]);
    save_project_atomic(&path, &saved).expect("the project saves");
    let before = std::fs::read(&path).expect("the project reads");

    let unsaved = envelope(0x11, "Saved", &["Bass", "Keys", "Drums"]);
    write_autosave(&path, &unsaved).expect("the autosave writes");

    let after = std::fs::read(&path).expect("the project still reads");
    assert_eq!(before, after, "the autosave modified the project file");
    assert_eq!(
        load_project(&path)
            .expect("the project still loads")
            .project
            .tracks
            .len(),
        1,
        "the project on disk must still be the SAVED state, not the autosaved one"
    );
}

// The sidecar carries the unsaved work, and reading it back returns exactly what was written
#[test]
fn a_sidecar_round_trips_the_unsaved_state() {
    let directory = scratch("round-trip");
    let path = directory.join("take.spectre");
    let saved = envelope(0x21, "Saved", &["Bass"]);
    save_project_atomic(&path, &saved).expect("the project saves");

    let unsaved = envelope(0x21, "Saved", &["Bass", "Keys"]);
    let sidecar = write_autosave(&path, &unsaved).expect("the autosave writes");
    assert_eq!(sidecar, journal_path(&path));
    assert!(sidecar.exists(), "the sidecar must exist after an autosave");

    let recovered = read_autosave(&path, unsaved.project.id)
        .expect("the sidecar reads")
        .expect("a sidecar for this project exists");
    assert_eq!(recovered, unsaved, "the sidecar did not round trip");
    assert_eq!(recovered.project.tracks.len(), 2);
}

// A sidecar belonging to a different project is not this project's unsaved work. Offering it
// would present another project's tracks as the musician's own
#[test]
fn a_sidecar_from_another_project_is_not_offered() {
    let directory = scratch("identity");
    let path = directory.join("take.spectre");
    let other = envelope(0x31, "Other", &["Strings"]);
    write_autosave(&path, &other).expect("the autosave writes");

    let mine = envelope(0x32, "Mine", &["Bass"]);
    assert_ne!(mine.project.id, other.project.id, "the seeds must differ");

    let found = read_autosave(&path, mine.project.id).expect("the sidecar reads");
    assert!(
        found.is_none(),
        "a sidecar for a different project was offered as this project's unsaved work"
    );
    // And it is reported as absence, not as damage: the file is still there and still valid
    assert!(journal_path(&path).exists());
}

// No sidecar is the ordinary case, not an error
#[test]
fn a_project_with_no_sidecar_reports_absence_rather_than_failure() {
    let directory = scratch("absent");
    let path = directory.join("take.spectre");
    let project = envelope(0x41, "Mine", &["Bass"]);
    assert!(read_autosave(&path, project.project.id)
        .expect("absence is not an error")
        .is_none());
}

// Discarding is idempotent, because every ordinary save will call it and a save must not fail
// because there was nothing to discard
#[test]
fn discarding_a_sidecar_is_idempotent() {
    let directory = scratch("discard");
    let path = directory.join("take.spectre");
    let project = envelope(0x51, "Mine", &["Bass"]);
    write_autosave(&path, &project).expect("the autosave writes");
    assert!(journal_path(&path).exists());

    discard_autosave(&path).expect("the first discard succeeds");
    assert!(!journal_path(&path).exists());
    discard_autosave(&path).expect("discarding nothing is not a failure");
}

// The sidecar's name must not collide with fs.rs's temporaries, which a crash leaves in the same
// directory. If it did, a crash-leftover temp could be read as an autosave
#[test]
fn the_sidecar_name_is_distinct_from_a_save_temporary() {
    let path = PathBuf::from("/tmp/spectre-name/take.spectre");
    let sidecar = journal_path(&path);
    let name = sidecar.file_name().unwrap().to_string_lossy().into_owned();
    assert_eq!(name, "take.spectre.autosave");
    assert!(
        !name.contains("spectre-tmp"),
        "the sidecar name collides with the save temporary suffix"
    );
    assert_ne!(sidecar, path, "the sidecar must not be the project itself");
    assert_eq!(
        sidecar.parent(),
        path.parent(),
        "the sidecar must be a sibling, so recovery finds it beside the project"
    );
}

// An autosave that fails validation must not leave a sidecar. The sidecar goes through the same
// validator the project does, so a broken autosave cannot become a recovery offer.
//
// An earlier version of this test guarded its assertion behind `if result.is_err()`, which made
// it pass whether or not the autosave was refused. The envelope is now invalid by a rule the
// validator actually owns -- an enabled loop with no region -- and the refusal is asserted
#[test]
fn an_invalid_autosave_leaves_no_sidecar() {
    let directory = scratch("invalid");
    let path = directory.join("take.spectre");
    let mut broken = envelope(0x61, "Broken", &["Bass"]);
    broken.project.transport.loop_enabled = true;
    broken.project.transport.loop_region = None;

    let result = write_autosave(&path, &broken);
    assert!(
        result.is_err(),
        "the validator accepted an enabled loop with no region"
    );
    assert!(
        !journal_path(&path).exists(),
        "a refused autosave left a sidecar behind"
    );
}

// A refused autosave must also leave an EXISTING sidecar intact. Replacing good unsaved work with
// nothing because the newest state was momentarily invalid would lose exactly what this protects
#[test]
fn a_refused_autosave_leaves_an_earlier_sidecar_intact() {
    let directory = scratch("refused-keeps");
    let path = directory.join("take.spectre");
    let good = envelope(0x71, "Good", &["Bass", "Keys"]);
    write_autosave(&path, &good).expect("the first autosave writes");
    let before = std::fs::read(journal_path(&path)).expect("the sidecar reads");

    let mut broken = envelope(0x71, "Good", &["Bass", "Keys"]);
    broken.project.transport.loop_enabled = true;
    broken.project.transport.loop_region = None;
    assert!(write_autosave(&path, &broken).is_err());

    let after = std::fs::read(journal_path(&path)).expect("the sidecar still reads");
    assert_eq!(before, after, "a refused autosave damaged the earlier one");
    let recovered = read_autosave(&path, good.project.id)
        .expect("the sidecar reads")
        .expect("the earlier autosave is still offered");
    assert_eq!(recovered, good);
}
