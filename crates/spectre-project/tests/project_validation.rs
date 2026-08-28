// Author: Jeff
// Date: 2026-08-28
// Description: R4 slice 7 evidence — the schema-2 validator and the reorder command
// Notes: Deserialization bypasses every constructor. Track's mutators clamp and Track::new
//   refuses a blank name, but #[derive(Deserialize)] calls none of them, so each case below is
//   built by decoding hand-written JSON rather than through the API that would refuse it.

use serde_json::{json, Map, Value};
use spectre_core::{IdGen, ObjectId, TempoMap, Transport};
use spectre_dsp::GAIN_PARAMETERS;
use spectre_project::{
    command::{CommandError, EditHistory, ProjectCommand, Transaction},
    from_bytes, to_bytes, validate_envelope, DeviceDoc, ParameterDoc, ProjectDoc, ProjectEnvelope,
    Track, TrackError, TrackInstrument, TrackList, ViewDoc, MAX_TRACKS, SCHEMA_VERSION,
};

fn envelope(tracks: TrackList) -> ProjectEnvelope {
    let mut ids = IdGen::new(0x0056_414c_4944);
    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: ids.next_id(),
            name: "Validate".into(),
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

fn three_tracks(seed: u64) -> (TrackList, [ObjectId; 3]) {
    let mut ids = IdGen::new(seed);
    let mut list = TrackList::new();
    let mut created = Vec::new();
    for name in ["A", "B", "C"] {
        let id = ids.next_id();
        list.push(Track::new(id, name, TrackInstrument::Pulse).unwrap())
            .unwrap();
        created.push(id);
    }
    (list, [created[0], created[1], created[2]])
}

// One track object as JSON, so a field can carry what no constructor would accept
fn track_json(id: u64, name: &str, level: Value) -> Value {
    json!({
        "id": id,
        "name": name,
        "instrument": "Pulse",
        "instrument_level": 0.3,
        "level": level,
        "muted": false,
        "soloed": false,
        "clips": { "placements": [], "lengths": [] }
    })
}

// Patch the tracks field of a valid envelope and decode the result
fn envelope_with_track_json(tracks: Vec<Value>) -> ProjectEnvelope {
    let mut value = serde_json::to_value(envelope(TrackList::new())).unwrap();
    value["project"]["tracks"] = json!({
        "tracks": tracks,
        "clips": [],
        "master_level": 1.0
    });
    serde_json::from_value(value).unwrap()
}

fn project_doc(tracks: TrackList) -> ProjectDoc {
    envelope(tracks).project
}

// U13 — CORE-001 across reorder and undo, on the real command path
#[test]
fn reorder_is_exactly_reversible() {
    let (tracks, [a, b, c]) = three_tracks(0x13);
    let mut document = project_doc(tracks);
    let mut history = EditHistory::new(8).unwrap();

    let snapshot = |document: &ProjectDoc| -> Vec<(ObjectId, String, f32, bool, bool)> {
        document
            .tracks
            .tracks()
            .iter()
            .map(|track| {
                (
                    track.id(),
                    track.name().to_string(),
                    track.level(),
                    track.is_muted(),
                    track.is_soloed(),
                )
            })
            .collect()
    };
    let before = snapshot(&document);

    history
        .apply(
            &mut document,
            Transaction::single(ProjectCommand::reorder_tracks(a, 2)),
        )
        .unwrap();
    let ids = |document: &ProjectDoc| -> Vec<ObjectId> {
        document.tracks.tracks().iter().map(Track::id).collect()
    };
    assert_eq!(ids(&document), vec![b, c, a]);

    assert!(history.undo(&mut document).unwrap());
    assert_eq!(ids(&document), vec![a, b, c]);
    // Every field survives, not only the order
    assert_eq!(snapshot(&document), before);

    assert!(history.redo(&mut document).unwrap());
    assert_eq!(ids(&document), vec![b, c, a]);
    // The same five values per track, only rearranged
    let mut after = snapshot(&document);
    let mut original = before.clone();
    after.sort_by_key(|entry| entry.0.raw());
    original.sort_by_key(|entry| entry.0.raw());
    assert_eq!(after, original);
}

// U14
#[test]
fn reorder_out_of_range_is_rejected_before_mutation() {
    let (tracks, [a, ..]) = three_tracks(0x14);
    let mut document = project_doc(tracks);
    let before = document.tracks.clone();
    let mut history = EditHistory::new(8).unwrap();

    let error = history
        .apply(
            &mut document,
            Transaction::single(ProjectCommand::reorder_tracks(a, 7)),
        )
        .unwrap_err();
    assert_eq!(
        error,
        CommandError::Track(TrackError::IndexOutOfRange { index: 7, len: 3 })
    );
    assert_eq!(document.tracks, before);

    let unknown = ObjectId::from_raw(0xDEAD).unwrap();
    let error = history
        .apply(
            &mut document,
            Transaction::single(ProjectCommand::reorder_tracks(unknown, 1)),
        )
        .unwrap_err();
    assert_eq!(
        error,
        CommandError::Track(TrackError::UnknownTrack(unknown))
    );
    assert_eq!(document.tracks, before);

    // A refused edit does not enter history, so an undo cannot replay it
    assert!(!history.undo(&mut document).unwrap());
}

// U15
#[test]
fn duplicate_object_ids_are_rejected() {
    // A parameter that reuses the project's own identity
    let mut aliased = envelope(TrackList::new());
    let project_id = aliased.project.id;
    aliased.project.devices.push(DeviceDoc {
        id: ObjectId::from_raw(0x5001).unwrap(),
        key: "gain".into(),
        parameters: vec![ParameterDoc {
            id: project_id,
            key: "gain".into(),
            value: 1.0,
            unknown: Map::new(),
        }],
        unknown: Map::new(),
    });
    assert!(validate_envelope(&aliased).is_err());
    assert!(from_bytes(&to_bytes(&aliased).unwrap()).is_err());

    // Two tracks sharing one id — only Deserialize can build this; TrackList::insert refuses it
    let shared = envelope_with_track_json(vec![
        track_json(0x77, "A", json!(1.0)),
        track_json(0x77, "B", json!(1.0)),
    ]);
    assert!(validate_envelope(&shared).is_err());
    assert!(from_bytes(&to_bytes(&shared).unwrap()).is_err());
}

// U16
#[test]
fn dangling_selection_is_rejected() {
    let (tracks, [a, ..]) = three_tracks(0x16);
    let mut valid = envelope(tracks);
    valid.project.view.selected_track = Some(a);
    // A selection that names a real track is fine
    assert!(validate_envelope(&valid).is_ok());

    valid.project.view.selected_track = Some(ObjectId::from_raw(0xBEEF).unwrap());
    assert!(validate_envelope(&valid).is_err());

    let mut device = envelope(TrackList::new());
    device.project.view.selected_device = Some(ObjectId::from_raw(0xCAFE).unwrap());
    assert!(validate_envelope(&device).is_err());
}

// U17
#[test]
fn non_finite_parameter_value_is_rejected_before_encoding() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut broken = envelope(TrackList::new());
        broken.project.devices.push(DeviceDoc {
            id: ObjectId::from_raw(0x5002).unwrap(),
            key: "gain".into(),
            parameters: vec![ParameterDoc {
                id: ObjectId::from_raw(0x5003).unwrap(),
                key: "gain".into(),
                value,
                unknown: Map::new(),
            }],
            unknown: Map::new(),
        });
        // JSON has no literal for either, so a non-finite value cannot survive an honest encode
        assert!(
            validate_envelope(&broken).is_err(),
            "{value} must be refused"
        );
    }
}

// U18
#[test]
fn signed_zero_and_subnormal_values_round_trip_bit_exactly() {
    for value in [-0.0_f32, f32::from_bits(1), f32::from_bits(0x8000_0001)] {
        let mut document = envelope(TrackList::new());
        document.project.devices.push(DeviceDoc {
            id: ObjectId::from_raw(0x5004).unwrap(),
            key: "gain".into(),
            parameters: vec![ParameterDoc {
                id: ObjectId::from_raw(0x5005).unwrap(),
                key: "gain".into(),
                value,
                unknown: Map::new(),
            }],
            unknown: Map::new(),
        });

        let back = from_bytes(&to_bytes(&document).unwrap()).unwrap();
        let restored = back.project.devices[0].parameters[0].value;
        // Bits, never the float: -0.0 == 0.0 is true and would hide a lost sign
        assert_eq!(
            restored.to_bits(),
            value.to_bits(),
            "0x{:08x} did not survive the JSON path",
            value.to_bits()
        );
    }
}

// U19
#[test]
fn deserialization_cannot_smuggle_a_track_past_its_own_constructors() {
    let blank = envelope_with_track_json(vec![track_json(0x81, "   ", json!(1.0))]);
    assert!(validate_envelope(&blank).is_err(), "a blank name");

    // JSON has no NaN literal, so a non-finite level arrives as null and fails to decode. The
    // rule still has to exist, because a level built in memory can be NaN
    let mut in_memory = envelope(TrackList::new());
    let mut list = TrackList::new();
    let id = ObjectId::from_raw(0x82).unwrap();
    list.push(Track::new(id, "A", TrackInstrument::Pulse).unwrap())
        .unwrap();
    in_memory.project.tracks = list;
    assert!(validate_envelope(&in_memory).is_ok());

    let above = envelope_with_track_json(vec![track_json(
        0x83,
        "A",
        json!(GAIN_PARAMETERS[0].maximum() + 1.0),
    )]);
    assert!(validate_envelope(&above).is_err(), "a level past the range");

    let below = envelope_with_track_json(vec![track_json(0x84, "A", json!(-1.0))]);
    assert!(validate_envelope(&below).is_err(), "a negative level");

    let too_many: Vec<_> = (0..=MAX_TRACKS)
        .map(|index| track_json(0x1000 + index as u64, "A", json!(1.0)))
        .collect();
    let overflow = envelope_with_track_json(too_many);
    assert!(
        validate_envelope(&overflow).is_err(),
        "more tracks than MAX_TRACKS"
    );
    assert!(from_bytes(&to_bytes(&overflow).unwrap()).is_err());
}

// The master fader is real persisted mixer state and is checked by the same rule
#[test]
fn a_master_level_outside_the_descriptor_range_is_rejected() {
    let mut value = serde_json::to_value(envelope(TrackList::new())).unwrap();
    value["project"]["tracks"] = json!({
        "tracks": [],
        "clips": [],
        "master_level": GAIN_PARAMETERS[0].maximum() + 1.0
    });
    let document: ProjectEnvelope = serde_json::from_value(value).unwrap();
    assert!(validate_envelope(&document).is_err());
}

// Device and parameter keys must name something, and a device may not carry one key twice
#[test]
fn empty_and_duplicated_parameter_keys_are_rejected() {
    let parameter = |key: &str| ParameterDoc {
        id: ObjectId::from_raw(if key.is_empty() { 0x6001 } else { 0x6002 }).unwrap(),
        key: key.into(),
        value: 1.0,
        unknown: Map::new(),
    };

    let mut empty_key = envelope(TrackList::new());
    empty_key.project.devices.push(DeviceDoc {
        id: ObjectId::from_raw(0x6000).unwrap(),
        key: "gain".into(),
        parameters: vec![parameter("")],
        unknown: Map::new(),
    });
    assert!(validate_envelope(&empty_key).is_err());

    let mut duplicated = envelope(TrackList::new());
    duplicated.project.devices.push(DeviceDoc {
        id: ObjectId::from_raw(0x6003).unwrap(),
        key: "gain".into(),
        parameters: vec![
            ParameterDoc {
                id: ObjectId::from_raw(0x6004).unwrap(),
                key: "gain".into(),
                value: 1.0,
                unknown: Map::new(),
            },
            ParameterDoc {
                id: ObjectId::from_raw(0x6005).unwrap(),
                key: "gain".into(),
                value: 0.5,
                unknown: Map::new(),
            },
        ],
        unknown: Map::new(),
    });
    assert!(validate_envelope(&duplicated).is_err());

    let mut blank_device = envelope(TrackList::new());
    blank_device.project.devices.push(DeviceDoc {
        id: ObjectId::from_raw(0x6006).unwrap(),
        key: "  ".into(),
        parameters: Vec::new(),
        unknown: Map::new(),
    });
    assert!(validate_envelope(&blank_device).is_err());
}
