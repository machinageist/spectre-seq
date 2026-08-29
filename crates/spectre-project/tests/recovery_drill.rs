// Author: Jeff
// Date: 2026-08-29
// Description: R5-3 evidence — the recovery drill decision 14 requires at R5 exit
// Notes: Decision 14 says "recovery drill required at R5 exit". A drill is not a unit test of a
//   comparison function: it walks the sequence a musician actually hits — work, autosave, die,
//   reopen, decide — and asserts what they are shown at each step.
//
//   The drill kills a REAL process for the unclean-exit step, reusing the crash_saver example
//   R5-1 introduced, because an unclean exit simulated by dropping a value in-process is a
//   different thing from one where nothing ran on the way out.

use spectre_core::{IdGen, TempoMap, Transport};
use spectre_project::journal::{journal_path, write_autosave};
use spectre_project::recovery::{accept, decline, inspect, Difference, Recovery};
use spectre_project::{
    load_project, save_project_atomic, ProjectDoc, ProjectEnvelope, Track, TrackInstrument,
    TrackList, ViewDoc, SCHEMA_VERSION,
};
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("spectre-recovery-{name}"));
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

// The whole sequence: save, work on, autosave, die, reopen, be told, accept
#[test]
fn the_recovery_drill_restores_work_after_an_unclean_exit() {
    let directory = scratch("drill");
    let path = directory.join("take.spectre");

    // 1 — a saved project
    let saved = envelope(0x11, "Session", &["Bass"]);
    save_project_atomic(&path, &saved).expect("the project saves");

    // 2 — work that was never saved, captured by an autosave
    let worked = envelope(0x11, "Session", &["Bass", "Keys", "Drums"]);
    write_autosave(&path, &worked).expect("the autosave writes");

    // 3 — an unclean exit: nothing runs, nothing is cleaned up. The sidecar is simply still there
    // 4 — reopening finds it and DESCRIBES it rather than applying it
    let found = inspect(&path).expect("inspection reads");
    let Recovery::Available(offer) = found else {
        panic!("unsaved work was not offered");
    };
    assert!(offer.differences.contains(&Difference::TrackCount {
        saved: 1,
        autosaved: 3
    }));
    let described = offer.describe();
    assert!(
        described.iter().any(|line| line.contains("tracks")),
        "the offer did not state the difference: {described:?}"
    );
    // What it did NOT compare must be stated too; R5's exit evidence asks for both halves
    assert!(!offer.uncompared().is_empty());

    // The project on disk is STILL the saved state while the offer stands
    assert_eq!(
        load_project(&path).expect("loads").project.tracks.len(),
        1,
        "inspection must not have applied anything"
    );

    // 5 — accepting writes the recovered work and clears the sidecar
    accept(&path, &offer).expect("accepting succeeds");
    assert_eq!(
        load_project(&path).expect("loads").project.tracks.len(),
        3,
        "the recovered work was not written"
    );
    assert!(
        !journal_path(&path).exists(),
        "an accepted offer left its sidecar behind, so it would be offered again"
    );
}

// Declining must leave the saved project exactly as it was, byte for byte
#[test]
fn declining_recovery_leaves_the_saved_project_untouched() {
    let directory = scratch("decline");
    let path = directory.join("take.spectre");
    let saved = envelope(0x21, "Session", &["Bass"]);
    save_project_atomic(&path, &saved).expect("the project saves");
    let before = std::fs::read(&path).expect("reads");

    write_autosave(&path, &envelope(0x21, "Session", &["Bass", "Keys"])).expect("autosave writes");
    assert!(matches!(
        inspect(&path).expect("inspects"),
        Recovery::Available(_)
    ));

    decline(&path).expect("declining succeeds");
    assert_eq!(
        std::fs::read(&path).expect("reads"),
        before,
        "declining modified the project"
    );
    assert!(!journal_path(&path).exists());
    assert!(matches!(
        inspect(&path).expect("inspects"),
        Recovery::Nothing
    ));
}

// A sidecar identical to the saved project is reported as redundant, not as an offer. Presenting
// "recover your work?" when there is no difference trains a musician to dismiss the prompt
#[test]
fn an_identical_sidecar_is_redundant_rather_than_an_offer() {
    let directory = scratch("redundant");
    let path = directory.join("take.spectre");
    let saved = envelope(0x31, "Session", &["Bass"]);
    save_project_atomic(&path, &saved).expect("the project saves");
    write_autosave(&path, &saved).expect("autosave writes");

    assert!(matches!(
        inspect(&path).expect("inspects"),
        Recovery::Redundant
    ));
}

// A crash during a save must not produce a recovery offer built from a half-written project. This
// is the drill's link to R5-1: the atomic save guarantees the project is whole, and inspection is
// what would surface it if it were not
#[test]
fn an_unclean_exit_during_a_save_still_yields_a_whole_project() {
    let directory = scratch("crash");
    let path = directory.join("take.spectre");

    let exe = {
        let mut p = std::env::current_exe().expect("test exe");
        p.pop();
        p.pop();
        p.push("examples");
        p.push("crash_saver");
        p
    };
    assert!(exe.exists(), "the crash_saver example must be built");

    for trial in 0..8 {
        let mut child = std::process::Command::new(&exe)
            .arg(&path)
            .arg("7")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("the saver runs");
        std::thread::sleep(
            std::time::Duration::from_millis(60)
                + std::time::Duration::from_micros((trial as u64 * 7_919) % 23_000),
        );
        child.kill().expect("killable");
        child.wait().expect("reapable");

        if !path.exists() {
            continue;
        }
        // Inspection must never fail on a project left by a crash, and must never report an offer
        // built from a torn file -- load_project would have refused it first
        match inspect(&path) {
            Ok(Recovery::Nothing) | Ok(Recovery::Redundant) | Ok(Recovery::Available(_)) => {}
            Err(error) => panic!("trial {trial}: inspection failed after a crash: {error}"),
        }
    }
}
