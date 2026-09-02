// Author: Jeff
// Date: 2026-08-28
// Description: R4 slice 7 evidence — the model maps to a document and back without losing work
// Notes: Every assertion here is arranged to be able to fail. A prototype model and a fresh
//   prototype model agree on device instance IDs, parameter values, and track levels BY
//   CONSTRUCTION, so comparing two untouched models would pass even if adopt ignored the file
//   entirely. The edits below are what make the comparison discriminating.

use spectre_app::project::{adopt, project_envelope, AdoptError};
use spectre_app::{AppModel, Lens};
use spectre_core::ObjectId;
use spectre_dsp::{GAIN_PARAMETERS, SATURATOR_PARAMETERS};
use spectre_project::{from_bytes, to_bytes, DeviceDoc, ParameterDoc, SCHEMA_VERSION};

fn unknown(key: &str, value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    [(key.to_string(), value)].into_iter().collect()
}

// Round-trip a model through the encoder into a fresh one, the way the shell's save/open does
fn round_trip(model: &AppModel) -> Result<AppModel, AdoptError> {
    let bytes = to_bytes(&project_envelope(model, "Round Trip")).unwrap();
    let envelope = from_bytes(&bytes).unwrap();
    let mut target = AppModel::prototype();
    adopt(&mut target, envelope)?;
    Ok(target)
}

fn value_of(model: &AppModel, device_key: &str, parameter_key: &str) -> f32 {
    model
        .devices()
        .iter()
        .find(|device| device.key == device_key)
        .and_then(|device| {
            device
                .parameters
                .iter()
                .find(|parameter| parameter.descriptor.key.as_str() == parameter_key)
        })
        .map(|parameter| parameter.value)
        .expect("the fixture device and parameter exist")
}

// I11
#[test]
fn model_envelope_round_trip_preserves_identity_order_and_edited_values() {
    let mut model = AppModel::prototype();
    let first = model.tracks()[0].id();
    model.add_track("Keys").unwrap();
    model.add_track("Drums").unwrap();
    model
        .set_device_parameter("saturator", "drive", 6.0)
        .unwrap();
    model.set_track_level(first, 0.5).unwrap();

    // The two edited values must differ from their descriptor defaults, or neither half of this
    // test can fail: an adopt that dropped the devices would leave drive at its default, and one
    // that rebuilt tracks through Track::new would leave level at unity
    assert_ne!(6.0, SATURATOR_PARAMETERS[0].default());
    assert_ne!(0.5, GAIN_PARAMETERS[0].default());

    let target = {
        let fresh = AppModel::prototype();
        assert_eq!(fresh.tracks().len(), 1);
        round_trip(&model).unwrap()
    };

    assert_eq!(target.tracks().len(), 3);
    let source: Vec<_> = model
        .tracks()
        .iter()
        .map(|track| (track.id(), track.name().to_string()))
        .collect();
    let restored: Vec<_> = target
        .tracks()
        .iter()
        .map(|track| (track.id(), track.name().to_string()))
        .collect();
    assert_eq!(restored, source);

    assert_eq!(target.tracks()[0].level(), 0.5);
    assert_eq!(value_of(&target, "saturator", "drive"), 6.0);
    // The generator resumes rather than restarting, so the next mint cannot collide
    let mut advanced = target;
    let minted = advanced.add_track("Fourth").unwrap();
    assert!(
        !source.iter().any(|(id, _)| *id == minted),
        "a reloaded project must not re-mint an ID it already holds"
    );
}

// I12
#[test]
fn adopting_a_project_stops_the_transport() {
    let mut model = AppModel::prototype();
    model.toggle_play();
    assert!(model.is_playing());

    let mut envelope = project_envelope(&model, "Playing");
    // The document really does carry a playing transport, so the stop is doing work
    assert!(envelope.project.transport.state == spectre_core::TransportState::Playing);
    envelope.project.transport.position = spectre_core::SampleTime(48_000);

    let mut target = AppModel::prototype();
    adopt(&mut target, envelope).unwrap();
    // Opening a file never starts playback — and, post R4-1, never starts audio
    assert!(!target.is_playing());
}

// I13
#[test]
fn a_failed_adopt_leaves_the_model_untouched() {
    let mut source = AppModel::prototype();
    source.add_track("Keys").unwrap();
    source.set_device_parameter("gain", "gain", 0.25).unwrap();
    let mut envelope = project_envelope(&source, "Broken");
    envelope.project.devices.push(DeviceDoc {
        id: ObjectId::from_raw(0x7001).unwrap(),
        key: "not-a-device".into(),
        parameters: Vec::new(),
        unknown: serde_json::Map::new(),
    });

    let mut target = AppModel::prototype();
    let before_tracks: Vec<_> = target.tracks().iter().map(|track| track.id()).collect();
    let before_devices = target.devices().to_vec();
    let before_lens = target.lens();
    let before_selected_track = target.selected_track_id();
    let before_selected_device = target.selected_device_id();
    let before_master = target.track_list().master_level();

    let error = adopt(&mut target, envelope).unwrap_err();
    assert!(matches!(error, AdoptError::UnknownDevice { .. }));

    // Nothing moved: the live project stays whole until a load fully succeeds
    let after: Vec<_> = target.tracks().iter().map(|track| track.id()).collect();
    assert_eq!(after, before_tracks);
    assert_eq!(target.devices(), before_devices.as_slice());
    assert_eq!(target.lens(), before_lens);
    assert_eq!(target.selected_track_id(), before_selected_track);
    assert_eq!(target.selected_device_id(), before_selected_device);
    assert_eq!(target.track_list().master_level(), before_master);
}

// I14
#[test]
fn out_of_range_parameter_values_are_refused_not_clamped() {
    let source = AppModel::prototype();
    let mut envelope = project_envelope(&source, "Out of range");
    let device = envelope
        .project
        .devices
        .iter_mut()
        .find(|device| device.key == "saturator")
        .unwrap();
    let over = SATURATOR_PARAMETERS[0].maximum() + 1.0;
    device
        .parameters
        .iter_mut()
        .find(|parameter| parameter.key == "drive")
        .unwrap()
        .value = over;

    let mut target = AppModel::prototype();
    let error = adopt(&mut target, envelope).unwrap_err();
    match error {
        AdoptError::ValueOutOfRange { value, .. } => assert_eq!(value, over),
        other => panic!("expected a refusal, got {other:?}"),
    }
    // Refused, not clamped: the model still holds its own value
    assert_eq!(
        value_of(&target, "saturator", "drive"),
        SATURATOR_PARAMETERS[0].default()
    );
}

// An unknown parameter key on a known device is refused by name
#[test]
fn an_unknown_parameter_key_is_refused_by_name() {
    let source = AppModel::prototype();
    let mut envelope = project_envelope(&source, "Unknown parameter");
    envelope
        .project
        .devices
        .iter_mut()
        .find(|device| device.key == "gain")
        .unwrap()
        .parameters
        .push(ParameterDoc {
            id: ObjectId::from_raw(0x7002).unwrap(),
            key: "wobble".into(),
            value: 0.5,
            unknown: serde_json::Map::new(),
        });

    let mut target = AppModel::prototype();
    match adopt(&mut target, envelope).unwrap_err() {
        AdoptError::UnknownParameter { device_key, key } => {
            assert_eq!(device_key, "gain");
            assert_eq!(key, "wobble");
        }
        other => panic!("expected UnknownParameter, got {other:?}"),
    }
}

// I15 — the version stamp has exactly one owner
#[test]
fn an_app_built_snapshot_carries_the_current_schema_version() {
    let envelope = project_envelope(&AppModel::prototype(), "Stamped");
    assert_eq!(envelope.schema_version, SCHEMA_VERSION);

    let directory = std::env::temp_dir().join("spectre-app-stamp");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("take.spectre");

    spectre_project::save_project_atomic(&path, &envelope).unwrap();
    let loaded = spectre_project::load_project(&path).unwrap();
    assert_eq!(loaded.schema_version, SCHEMA_VERSION);
    // A literal too, so a silent bump fails here rather than agreeing with itself
    assert_eq!(loaded.schema_version, 2);
}

// The restored working context is the musician's own, not the prototype's
#[test]
fn the_view_context_survives_the_round_trip() {
    let mut model = AppModel::prototype();
    let keys = model.add_track("Keys").unwrap();
    model.select_track(keys);
    model.select_lens(Lens::Mix);
    let selected_device = model.selected_device_id();

    let target = round_trip(&model).unwrap();
    assert_eq!(target.lens(), Lens::Mix);
    assert_eq!(target.selected_track_id(), Some(keys));
    assert_eq!(target.selected_device_id(), selected_device);
    // Arrange is the prototype's lens, so Mix is a value the target could not have had already
    assert_ne!(AppModel::prototype().lens(), Lens::Mix);
}

// Identity of the project itself survives, so two saves describe one project rather than two
#[test]
fn the_project_keeps_its_own_identity_across_a_round_trip() {
    let model = AppModel::prototype();
    let id = model.project_id();
    let target = round_trip(&model).unwrap();
    assert_eq!(target.project_id(), id);

    // And a second round trip does not mint a third identity
    assert_eq!(round_trip(&target).unwrap().project_id(), id);
}

// CORE-003 must hold through the product path, not only through a codec-level rewrite.
#[test]
fn open_edit_save_preserves_feasible_unknown_fields_at_every_supported_level() {
    let source = AppModel::prototype();
    let mut envelope = project_envelope(&source, "Forward fields");
    envelope.unknown = unknown("future_envelope", serde_json::json!({"writer": 3}));
    envelope.project.unknown = unknown("future_project", serde_json::json!([1, 2, 3]));

    let device = envelope
        .project
        .devices
        .iter_mut()
        .find(|device| device.key == "gain")
        .expect("the canonical gain device exists");
    device.unknown = unknown("future_device", serde_json::json!({"mode": "linked"}));
    let parameter = device
        .parameters
        .iter_mut()
        .find(|parameter| parameter.key == "gain")
        .expect("the canonical gain parameter exists");
    parameter.unknown = unknown("future_parameter", serde_json::json!({"curve": [0, 1]}));

    let expected_envelope = envelope.unknown.clone();
    let expected_project = envelope.project.unknown.clone();
    let expected_device = device.unknown.clone();
    let expected_parameter = parameter.unknown.clone();

    let mut opened = AppModel::prototype();
    adopt(&mut opened, envelope).expect("known content with unknown fields adopts");
    opened.add_track("An ordinary edit").unwrap();
    opened.set_device_parameter("gain", "gain", 0.25).unwrap();

    let saved = project_envelope(&opened, "Forward fields");
    assert_eq!(saved.unknown, expected_envelope);
    assert_eq!(saved.project.unknown, expected_project);
    let saved_device = saved
        .project
        .devices
        .iter()
        .find(|device| device.key == "gain")
        .unwrap();
    assert_eq!(saved_device.unknown, expected_device);
    assert_eq!(
        saved_device
            .parameters
            .iter()
            .find(|parameter| parameter.key == "gain")
            .unwrap()
            .unknown,
        expected_parameter
    );
}

#[test]
fn opening_schema_one_performs_the_explicit_migration_and_preserves_its_identity() {
    let source = from_bytes(include_bytes!(
        "../../spectre-project/tests/fixtures/r1-canonical.json"
    ))
    .expect("the canonical schema-1 fixture decodes");
    let project_id = source.project.id;
    let meter_map = source
        .project
        .meter_map()
        .expect("the fixture meter map is valid")
        .expect("the fixture carries meter state");
    let envelope_unknown = source.unknown.clone();
    let project_unknown = source.project.unknown.clone();

    let mut opened = AppModel::prototype();
    adopt(&mut opened, source).expect("the product migrates schema 1 while adopting it");
    let saved = project_envelope(&opened, "Migrated");

    assert_eq!(saved.schema_version, SCHEMA_VERSION);
    assert_eq!(saved.project.id, project_id);
    assert_eq!(opened.meter_map(), &meter_map);
    assert_eq!(saved.unknown, envelope_unknown);
    assert_eq!(saved.project.unknown, project_unknown);
    assert_ne!(saved.project.id_gen_state, 0);
}

// The dirty marker is derived, never asserted. These are the two shell decisions R4-1's review
// showed cannot be left in main.rs, where no test can reach them
#[test]
fn a_project_with_no_file_behind_it_is_dirty() {
    use spectre_app::project::is_dirty;

    let model = AppModel::prototype();
    let bytes = to_bytes(&project_envelope(&model, "T")).unwrap();
    // Nothing saved yet: there is unsaved work by definition
    assert!(is_dirty(None, Some(&bytes)));
    // Saved and unchanged
    assert!(!is_dirty(Some(&bytes), Some(&bytes)));

    // One edit and the same comparison says so, without anyone setting a flag
    let mut edited = AppModel::prototype();
    edited.add_track("Keys").unwrap();
    let after = to_bytes(&project_envelope(&edited, "T")).unwrap();
    assert!(is_dirty(Some(&bytes), Some(&after)));

    // A parameter edit alone is enough; content is content
    let mut tweaked = AppModel::prototype();
    tweaked.set_device_parameter("gain", "gain", 0.25).unwrap();
    let tweak = to_bytes(&project_envelope(&tweaked, "T")).unwrap();
    assert!(is_dirty(Some(&bytes), Some(&tweak)));
}

#[test]
fn one_press_never_discards_unsaved_work() {
    use spectre_app::project::{open_gate, OpenGate};

    // Dirty and unarmed: the first press arms, it does not open
    assert_eq!(open_gate(true, false), OpenGate::ArmDiscard);
    // The second press goes through
    assert_eq!(open_gate(true, true), OpenGate::Proceed);
    // A clean project needs no confirm at all
    assert_eq!(open_gate(false, false), OpenGate::Proceed);
    assert_eq!(open_gate(false, true), OpenGate::Proceed);
}

// The whole product path in one test: build a project, write it with the app's own snapshot
// builder through the atomic writer, read it back with the loader, adopt it. This is what the
// musician does; every piece above tests one joint of it
#[test]
fn the_whole_save_quit_relaunch_open_path_returns_the_same_project() {
    let directory = std::env::temp_dir().join("spectre-app-e2e");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("session.spectre");

    let mut model = AppModel::prototype();
    let keys = model.add_track("Keys").unwrap();
    model.add_track("Drums").unwrap();
    model
        .set_device_parameter("saturator", "drive", 6.0)
        .unwrap();
    model.set_track_level(keys, 0.5).unwrap();
    model.select_track(keys);
    model.select_lens(Lens::Mix);
    let order: Vec<_> = model.tracks().iter().map(|track| track.id()).collect();

    spectre_project::save_project_atomic(&path, &project_envelope(&model, "Session")).unwrap();

    // "Quit and relaunch": a brand new prototype model that knows nothing about the above
    let mut relaunched = AppModel::prototype();
    assert_eq!(relaunched.tracks().len(), 1);
    let envelope = spectre_project::load_project(&path).unwrap();
    adopt(&mut relaunched, envelope).unwrap();

    let restored: Vec<_> = relaunched.tracks().iter().map(|track| track.id()).collect();
    assert_eq!(restored, order);
    assert_eq!(relaunched.tracks()[1].level(), 0.5);
    assert_eq!(value_of(&relaunched, "saturator", "drive"), 6.0);
    assert_eq!(relaunched.selected_track_id(), Some(keys));
    assert_eq!(relaunched.lens(), Lens::Mix);
    assert!(!relaunched.is_playing());

    // And saving again produces the same bytes, so a round trip is not a slow drift
    let resaved = directory.join("again.spectre");
    spectre_project::save_project_atomic(&resaved, &project_envelope(&relaunched, "Session"))
        .unwrap();
    assert_eq!(
        std::fs::read(&resaved).unwrap(),
        std::fs::read(&path).unwrap()
    );
    std::fs::remove_dir_all(&directory).unwrap();
}
