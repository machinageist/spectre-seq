// Author: Jeff
// Date: 2026-07-12
// Description: Interaction-model tests for the launchable Geist prototype
// Notes: Pins behavior independently of egui rendering

use geist_app::{AppModel, Lens};
use geist_dsp::{DeviceParameterKey, GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS};

#[test]
fn transport_toggle_is_deterministic() {
    let mut model = AppModel::prototype();
    assert!(!model.is_playing());
    model.toggle_play();
    assert!(model.is_playing());
    model.toggle_play();
    assert!(!model.is_playing());
}

#[test]
fn lens_selection_preserves_track_selection() {
    let mut model = AppModel::prototype();
    let selected = model.selected_track_id();
    model.select_lens(Lens::Build);
    assert_eq!(model.lens(), Lens::Build);
    assert_eq!(model.selected_track_id(), selected);
}

#[test]
fn adding_and_selecting_tracks_uses_stable_unique_ids() {
    let mut model = AppModel::prototype();
    let first = model.selected_track_id().unwrap();
    let second = model.add_track("Atmosphere").unwrap();
    assert_ne!(first, second);
    assert_eq!(model.selected_track_id(), Some(second));
    assert_eq!(model.tracks().len(), 2);
}

#[test]
fn blank_track_names_are_rejected_without_mutation() {
    let mut model = AppModel::prototype();
    let before = model.tracks().to_vec();
    assert!(model.add_track("   ").is_err());
    assert_eq!(model.tracks(), before);
}

#[test]
fn feedback_report_captures_visible_state_and_user_notes() {
    let mut model = AppModel::prototype();
    model.select_lens(Lens::Mix);
    model.set_feedback("Mixer needs larger meters");
    let report = model.feedback_report();

    assert!(report.contains("lens: Mix"));
    assert!(report.contains("transport: stopped"));
    assert!(report.contains("tracks: 1"));
    assert!(report.contains("Mixer needs larger meters"));
}

#[test]
fn prototype_device_controls_derive_from_backend_descriptors() {
    let model = AppModel::prototype();
    let devices = model.devices();
    assert_eq!(devices.len(), 3);
    assert_eq!(devices[0].parameters[0].descriptor, PULSE_PARAMETERS[0]);
    assert_eq!(devices[1].parameters[0].descriptor, GAIN_PARAMETERS[0]);
    assert_eq!(devices[2].parameters[0].descriptor, SATURATOR_PARAMETERS[0]);
    assert_eq!(devices[2].parameters[1].descriptor, SATURATOR_PARAMETERS[1]);
}

#[test]
fn device_parameter_edits_use_backend_clamping() {
    let mut model = AppModel::prototype();
    model
        .set_device_parameter("saturator", "drive", 100.0)
        .unwrap();
    let drive = model
        .devices()
        .iter()
        .find(|device| device.key == "saturator")
        .unwrap()
        .parameters
        .iter()
        .find(|parameter| parameter.descriptor.key.as_str() == "drive")
        .unwrap();
    assert_eq!(drive.value, drive.descriptor.maximum());
}

#[test]
fn device_parameter_snapshot_is_owned_stable_and_descriptor_identified() {
    let mut model = AppModel::prototype();
    let before = model.device_parameter_snapshot().unwrap();

    assert_eq!(before, model.device_parameter_snapshot().unwrap());
    let pulse_parameter = PULSE_PARAMETERS[0];
    let pulse_value = before
        .iter()
        .find(|parameter| {
            parameter.device_key() == "pulse" && parameter.parameter_key() == pulse_parameter.key
        })
        .unwrap();
    let _: DeviceParameterKey = pulse_value.parameter_key();
    assert_eq!(pulse_value.value(), pulse_parameter.default());

    model
        .set_device_parameter(
            "pulse",
            pulse_parameter.key.as_str(),
            pulse_parameter.maximum(),
        )
        .unwrap();
    assert_eq!(pulse_value.value(), pulse_parameter.default());
}

#[test]
fn device_parameter_snapshot_attributes_values_by_stable_identity() {
    let mut model = AppModel::prototype();
    for (device, parameter, value) in [
        ("pulse", "level", 0.61),
        ("gain", "gain", 1.37),
        ("saturator", "drive", 4.25),
        ("saturator", "mix", 0.83),
    ] {
        model
            .set_device_parameter(device, parameter, value)
            .unwrap();
    }
    let snapshot = model.device_parameter_snapshot().unwrap();
    for (device, parameter, value) in [
        ("pulse", "level", 0.61),
        ("gain", "gain", 1.37),
        ("saturator", "drive", 4.25),
        ("saturator", "mix", 0.83),
    ] {
        let entry = snapshot
            .iter()
            .find(|entry| {
                entry.device_key() == device && entry.parameter_key().as_str() == parameter
            })
            .unwrap();
        assert_eq!(entry.value(), value, "{device}.{parameter}");
    }
}

#[test]
fn device_parameter_snapshot_clamps_edits_from_backend_descriptors() {
    let mut model = AppModel::prototype();
    model
        .set_device_parameter("saturator", "drive", 100.0)
        .unwrap();

    let snapshot = model.device_parameter_snapshot().unwrap();
    let drive = snapshot
        .iter()
        .find(|parameter| {
            parameter.device_key() == "saturator" && parameter.parameter_key().as_str() == "drive"
        })
        .unwrap();

    assert_eq!(drive.value(), SATURATOR_PARAMETERS[0].maximum());
}

#[test]
fn device_parameter_snapshot_contains_non_finite_plain_values() {
    let mut model = AppModel::prototype();
    model
        .set_device_parameter("gain", "gain", f32::NAN)
        .unwrap();

    let snapshot = model.device_parameter_snapshot().unwrap();
    let published_gain = snapshot
        .iter()
        .find(|parameter| parameter.device_key() == "gain")
        .unwrap();

    assert!(published_gain.value().is_finite());
    assert_eq!(published_gain.value(), GAIN_PARAMETERS[0].default());
    assert!(published_gain.value() >= GAIN_PARAMETERS[0].minimum());
    assert!(published_gain.value() <= GAIN_PARAMETERS[0].maximum());
}

#[test]
fn device_parameter_snapshot_uses_canonical_identity_and_range() {
    let mut model = AppModel::prototype();
    model.set_device_parameter("gain", "gain", 100.0).unwrap();

    let snapshot = model.device_parameter_snapshot().unwrap();
    let published_gain = snapshot
        .iter()
        .find(|entry| entry.device_key() == "gain")
        .unwrap();

    assert_eq!(published_gain.parameter_key(), GAIN_PARAMETERS[0].key);
    assert_eq!(published_gain.value(), GAIN_PARAMETERS[0].maximum());
}
