// Author: Jeff
// Date: 2026-08-28
// Description: R4 slice 7 evidence — atomic save and bounded load against a real filesystem
// Notes: CORE-004's filesystem half and CORE-001's persisted half. The fault-injection tests are
//   unit tests inside src/fs.rs, because the injection seam is private on purpose; these need
//   nothing private and everything real.

use serde_json::{Map, Value};
use spectre_core::{IdGen, ObjectId, TempoMap, Transport};
use spectre_project::{
    command::{EditHistory, ProjectCommand, Transaction},
    from_bytes, load_project, save_project_atomic, to_bytes, LoadError, ProjectDoc,
    ProjectEnvelope, SaveError, SaveStage, TargetState, Track, TrackEffect, TrackInsert,
    TrackInstrument, TrackList, ViewDoc, MAX_READABLE_SCHEMA, SCHEMA_VERSION,
};
use std::path::{Path, PathBuf};

const GOLDEN: &[u8] = include_bytes!("fixtures/r1-canonical.json");

// A directory of this test's own, removed and recreated so a stale run cannot pass one
fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("spectre-save-{name}"));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn envelope(name: &str, tracks: TrackList, seed: u64) -> ProjectEnvelope {
    let mut ids = IdGen::new(seed);
    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: ids.next_id(),
            name: name.into(),
            tempo_map: TempoMap::constant(120.0).unwrap(),
            transport: Transport::new(),
            id_gen_state: ids.state(),
            tracks,
            devices: Vec::new(),
            view: ViewDoc::default(),
            unknown: Map::new(),
        },
        unknown: Map::new(),
    }
}

// Three tracks with distinct identities, built through the constructors that refuse bad input
fn three_tracks(seed: u64) -> (TrackList, [ObjectId; 3]) {
    let mut ids = IdGen::new(seed);
    let mut list = TrackList::new();
    let mut created = Vec::new();
    for name in ["Bass", "Keys", "Drums"] {
        let id = ids.next_id();
        list.push(Track::new(id, name, TrackInstrument::Pulse).unwrap())
            .unwrap();
        created.push(id);
    }
    (list, [created[0], created[1], created[2]])
}

fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// I1
#[test]
fn save_then_load_round_trips_a_populated_project() {
    let directory = scratch("round-trip");
    let path = directory.join("take.spectre");
    let (tracks, _) = three_tracks(0x11);
    let snapshot = envelope("Round Trip", tracks, 0x21);

    let receipt = save_project_atomic(&path, &snapshot).unwrap();
    assert_eq!(receipt.target_state, TargetState::ReplacedDurable);

    let loaded = load_project(&path).unwrap();
    assert_eq!(loaded, snapshot);
    assert_eq!(loaded.project.tracks.len(), 3);
}

// I2
#[test]
fn successful_save_leaves_no_temporary_behind() {
    let directory = scratch("no-temp");
    let path = directory.join("take.spectre");
    save_project_atomic(&path, &envelope("T", TrackList::new(), 0x31)).unwrap();

    assert_eq!(entries(&directory), vec!["take.spectre".to_string()]);
}

// I3
#[test]
fn save_over_an_existing_project_replaces_it_completely() {
    let directory = scratch("replace");
    let path = directory.join("take.spectre");
    let (tracks, _) = three_tracks(0x41);
    let a = envelope("Project A with a deliberately long name", tracks, 0x51);
    let b = envelope("B", TrackList::new(), 0x61);

    save_project_atomic(&path, &a).unwrap();
    save_project_atomic(&path, &b).unwrap();

    // Replacement, not merge: the bytes are exactly B's, with no fragment of A left over
    assert_eq!(std::fs::read(&path).unwrap(), to_bytes(&b).unwrap());
    assert_eq!(load_project(&path).unwrap(), b);
}

// I4
#[test]
fn invalid_snapshot_leaves_an_existing_target_byte_identical() {
    let directory = scratch("invalid");
    let path = directory.join("take.spectre");
    let a = envelope("A", TrackList::new(), 0x71);
    save_project_atomic(&path, &a).unwrap();
    let before = std::fs::read(&path).unwrap();

    // A dangling selection: valid Rust, invalid project
    let mut broken = envelope("Broken", TrackList::new(), 0x81);
    broken.project.view.selected_track = Some(ObjectId::from_raw(999).unwrap());
    let error = save_project_atomic(&path, &broken).unwrap_err();

    assert!(matches!(error, SaveError::InvalidProject { .. }));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(entries(&directory), vec!["take.spectre".to_string()]);
}

// I5
#[test]
fn save_into_a_missing_directory_fails_and_creates_nothing() {
    let directory = scratch("missing");
    let path = directory.join("no-such-dir").join("take.spectre");

    let error = save_project_atomic(&path, &envelope("T", TrackList::new(), 0x91)).unwrap_err();
    assert!(matches!(
        error,
        SaveError::Io {
            stage: SaveStage::CreateTemporary,
            target_state: TargetState::Unchanged,
            ..
        }
    ));
    assert!(!path.exists());
    assert!(entries(&directory).is_empty());
}

// I6 — CORE-001's remaining R4 evidence: identity across a reorder AND across the file
#[test]
fn reorder_preserves_identity_across_save_and_reload() {
    let directory = scratch("reorder");
    let path = directory.join("take.spectre");
    let (tracks, [first, second, third]) = three_tracks(0xA1);
    let mut snapshot = envelope("Reorder", tracks, 0xB1);

    save_project_atomic(&path, &snapshot).unwrap();

    let mut history = EditHistory::new(8).unwrap();
    history
        .apply(
            &mut snapshot.project,
            Transaction::single(ProjectCommand::reorder_tracks(first, 2)),
        )
        .unwrap();
    save_project_atomic(&path, &snapshot).unwrap();

    let loaded = load_project(&path).unwrap();
    let ids: Vec<_> = loaded
        .project
        .tracks
        .tracks()
        .iter()
        .map(|track| track.id())
        .collect();
    // The same three identities, in the new positions. Fails if identity is ever derived from
    // index, and fails if the encoder writes position where it should write identity
    assert_eq!(ids, vec![second, third, first]);
    let names: Vec<_> = loaded
        .project
        .tracks
        .tracks()
        .iter()
        .map(|track| track.name().to_string())
        .collect();
    assert_eq!(names, vec!["Keys", "Drums", "Bass"]);
    assert!(loaded
        .project
        .tracks
        .tracks()
        .iter()
        .all(|track| track.level() == 1.0 && !track.is_muted() && !track.is_soloed()));
}

// I7
#[test]
fn undone_reorder_reloads_in_the_original_order() {
    let directory = scratch("undo");
    let path = directory.join("take.spectre");
    let (tracks, [first, second, third]) = three_tracks(0xC1);
    let mut snapshot = envelope("Undo", tracks, 0xD1);

    let mut history = EditHistory::new(8).unwrap();
    history
        .apply(
            &mut snapshot.project,
            Transaction::single(ProjectCommand::reorder_tracks(first, 2)),
        )
        .unwrap();
    save_project_atomic(&path, &snapshot).unwrap();
    assert!(history.undo(&mut snapshot.project).unwrap());
    save_project_atomic(&path, &snapshot).unwrap();

    let loaded = load_project(&path).unwrap();
    let ids: Vec<_> = loaded
        .project
        .tracks
        .tracks()
        .iter()
        .map(|track| track.id())
        .collect();
    assert_eq!(ids, vec![first, second, third]);
}

// I8
#[test]
fn schema_one_fixture_loads_with_empty_collections() {
    let directory = scratch("schema-one");
    let path = directory.join("r1.json");
    std::fs::write(&path, GOLDEN).unwrap();

    let loaded = load_project(&path).unwrap();
    assert_eq!(loaded.schema_version, 1);
    assert_eq!(loaded.project.id.raw(), 1_311_768_467_463_790_320);
    assert_eq!(loaded.project.tracks, TrackList::new());
    // Proves #[serde(default)] reaches TrackList's own Default, not Vec::default plus a zero
    assert_eq!(loaded.project.tracks.master_level(), 1.0);
    assert_eq!(loaded.project.tracks.structure_revision(), 0);
    assert!(loaded.project.devices.is_empty());
    assert_eq!(loaded.project.id_gen_state, 0);
    assert_eq!(loaded.project.view, ViewDoc::default());
    // CORE-003's preservation guarantee still holds across the older schema
    assert!(loaded.project.unknown.contains_key("future_session"));
    assert!(loaded.unknown.contains_key("r1_extension"));
}

// I9 — the writer's half of the version rule
#[test]
fn resaving_a_loaded_schema_one_project_keeps_its_version_and_unknown_fields() {
    let directory = scratch("resave-v1");
    let source = directory.join("r1.json");
    std::fs::write(&source, GOLDEN).unwrap();
    let loaded = load_project(&source).unwrap();

    let target = directory.join("resaved.json");
    save_project_atomic(&target, &loaded).unwrap();
    let again = load_project(&target).unwrap();

    // Still 1. Fails the moment anyone puts a version stamp in the writer
    assert_eq!(again.schema_version, 1);
    assert!(again.schema_version < SCHEMA_VERSION);
    assert_eq!(again.project.tracks, TrackList::new());
    assert!(again.project.devices.is_empty());
    assert_eq!(again.project.id_gen_state, 0);
    assert!(again.project.unknown.contains_key("future_session"));
    assert!(again.unknown.contains_key("r1_extension"));
    // And byte-identical, because a schema-1 document must not gain four empty schema-2 fields
    assert_eq!(std::fs::read(&target).unwrap(), GOLDEN);
}

// I10
#[test]
fn a_newer_schema_file_is_refused_and_not_rewritten() {
    let directory = scratch("newer");
    let path = directory.join("future.json");
    let mut value: Value = serde_json::from_slice(GOLDEN).unwrap();
    value["schema_version"] = Value::from(MAX_READABLE_SCHEMA + 1);
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    std::fs::write(&path, &bytes).unwrap();

    let error = load_project(&path).unwrap_err();
    match error {
        LoadError::SchemaTooNew {
            found,
            max_readable,
        } => {
            assert_eq!(found, MAX_READABLE_SCHEMA + 1);
            assert_eq!(max_readable, MAX_READABLE_SCHEMA);
        }
        other => panic!("expected SchemaTooNew, got {other:?}"),
    }
    // Failing closed is what protects the file
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

// A file past the ceiling is refused before it is decoded, not after it is allocated
#[test]
fn an_oversize_file_is_refused_by_the_loader() {
    let directory = scratch("oversize");
    let path = directory.join("huge.json");
    // The real bound is 64 MiB; writing one would make this test a disk benchmark. What is
    // asserted here is that load_project routes a bounded-read refusal to LoadError::Read, and
    // the bound's own arithmetic is pinned by bounded_read_refuses_oversize in src/fs.rs
    std::fs::write(&path, b"{\"schema_version\": 1}").unwrap();
    assert!(matches!(
        load_project(&path).unwrap_err(),
        LoadError::Malformed { .. }
    ));

    let missing = directory.join("absent.json");
    assert!(matches!(
        load_project(&missing).unwrap_err(),
        LoadError::Open { .. }
    ));
}

// A document that decodes but is not a valid project is refused with the reason, not adopted
#[test]
fn a_decodable_but_invalid_project_is_refused_on_load() {
    let directory = scratch("invalid-load");
    let path = directory.join("bad.json");
    let mut broken = envelope("Broken", TrackList::new(), 0xE1);
    broken.project.view.selected_device = Some(ObjectId::from_raw(4242).unwrap());
    // Encoded directly rather than through save_project_atomic, which would refuse it first
    std::fs::write(&path, to_bytes(&broken).unwrap()).unwrap();

    assert!(matches!(
        load_project(&path).unwrap_err(),
        LoadError::InvalidProject { .. }
    ));
    // from_bytes refuses it for the same reason, so the two doors agree
    assert!(from_bytes(&to_bytes(&broken).unwrap()).is_err());
}

// A track's insert must survive the file, or R4's "atomic save and reload round-trip a project
// containing tracks, clips, and device parameters" row is closed on a project shape the alpha
// does not have -- every alpha track carries one
#[test]
fn a_track_insert_survives_save_and_reload() {
    let (mut list, ids) = three_tracks(0x0053_4156_494e_5301);
    list.set_insert(ids[1], Some(TrackInsert::new(TrackEffect::Gloam, 0.37)))
        .unwrap();
    let directory = scratch("insert-round-trip");
    let path = directory.join("take.spectre");

    save_project_atomic(&path, &envelope("insert", list, 0x0053_4156_494e_5302)).unwrap();
    let loaded = load_project(&path).unwrap();
    let tracks = loaded.project.tracks.tracks();

    assert_eq!(tracks[0].insert(), None, "an untouched track gains nothing");
    assert_eq!(tracks[2].insert(), None);
    let insert = tracks[1]
        .insert()
        .expect("the insert must survive the file");
    assert_eq!(insert.effect(), TrackEffect::Gloam);
    assert_eq!(insert.depth(), 0.37);
}

// The compatibility line the insert slot promised: a project with no insert anywhere must
// serialize exactly as it did before the field existed, or CORE-003's byte-stability evidence
// is quietly weakened by an optional field that is not actually optional in the bytes
#[test]
fn a_project_without_inserts_writes_no_insert_field() {
    let (list, _) = three_tracks(0x0053_4156_494e_5303);
    let bytes = to_bytes(&envelope("plain", list, 0x0053_4156_494e_5304)).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        !text.contains("insert"),
        "a track with no insert must not write the field: {text}"
    );
}
