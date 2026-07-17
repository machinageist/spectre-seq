// Author: Jeff
// Date: 2026-07-12
// Description: Interaction-model tests for the launchable Geist prototype
// Notes: Pins behavior independently of egui rendering

use geist_app::{AppModel, Lens};
use geist_dsp::{GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS};

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
