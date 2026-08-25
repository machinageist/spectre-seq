// Author: Jeff
// Date: 2026-07-12
// Description: Interaction-model tests for the launchable Spectre prototype
// Notes: Pins behavior independently of egui rendering

use spectre_app::{
    open_device_in_shape_from_ui, set_device_parameter_from_ui, AppModel, Lens, ShapePresentation,
    CLIP_INSPECTOR_EMPTY_MESSAGE, CLIP_LANE_EMPTY_MESSAGE, OPEN_IN_SHAPE_ACTION_LABEL,
    SHAPE_EMPTY_MESSAGE,
};
use spectre_core::{BeatTicks, ObjectId};
use spectre_dsp::{
    DeviceParameterKey, FILAMENT_PARAMETERS, GAIN_PARAMETERS, GLOAM_PARAMETERS, PULSE_PARAMETERS,
    SATURATOR_PARAMETERS,
};
use spectre_project::ClipError;
use std::collections::HashSet;

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
    assert!(report.contains("selected device: Pulse (pulse)"));
    assert!(report.contains("Mixer needs larger meters"));
}

#[test]
fn prototype_selects_pulse_device_by_project_instance_identity() {
    let model = AppModel::prototype();
    let pulse = &model.devices()[0];

    assert_eq!(pulse.key, "pulse");
    assert_eq!(model.selected_device_id(), Some(pulse.instance_id));
    assert_eq!(model.selected_device(), Some(pulse));
}

#[test]
fn build_presentation_has_one_correct_action_per_existing_device() {
    let model = AppModel::prototype();
    let presentation = model.build_presentation();
    let cards: Vec<_> = presentation.cards().collect();

    assert_eq!(cards.len(), 5);
    assert_eq!(cards.len(), model.devices().len());
    for (card, device) in cards.iter().zip(model.devices()) {
        assert!(std::ptr::eq(card.device(), device));
        assert_eq!(card.device().instance_id, device.instance_id);
        assert_eq!(card.action().label(), OPEN_IN_SHAPE_ACTION_LABEL);
        assert_eq!(card.action().device_id(), device.instance_id);
        assert_eq!(
            card.is_selected(),
            model.selected_device_id() == Some(device.instance_id)
        );
    }
    assert_eq!(cards.iter().filter(|card| card.is_selected()).count(), 1);
}

#[test]
fn build_presentation_selected_state_tracks_device_focus() {
    let mut model = AppModel::prototype();
    let gain_id = model.devices()[1].instance_id;

    open_device_in_shape_from_ui(&mut model, gain_id, &mut String::new());

    let selected: Vec<_> = model
        .build_presentation()
        .cards()
        .filter(|card| card.is_selected())
        .map(|card| card.device().instance_id)
        .collect();
    assert_eq!(selected, [gain_id]);
}

#[test]
fn shape_presentation_exposes_only_selected_descriptor_backed_controls() {
    let mut model = AppModel::prototype();
    let saturator_id = model.devices()[2].instance_id;
    model.open_device_in_shape(saturator_id).unwrap();
    model
        .set_device_parameter("saturator", "drive", 4.25)
        .unwrap();

    let presentation = model.shape_presentation();
    let device = presentation.selected_device().unwrap();
    assert_eq!(device.instance_id, saturator_id);
    assert_eq!(device.key, "saturator");
    assert_eq!(device.parameters.len(), SATURATOR_PARAMETERS.len());
    assert_eq!(device.parameters[0].descriptor, SATURATOR_PARAMETERS[0]);
    assert_eq!(device.parameters[0].value, 4.25);
    assert_eq!(device.parameters[1].descriptor, SATURATOR_PARAMETERS[1]);
    assert_eq!(presentation.empty_state_message(), None);
    assert!(!model
        .devices()
        .iter()
        .filter(|candidate| candidate.instance_id != saturator_id)
        .any(|candidate| std::ptr::eq(candidate, device)));
}

#[test]
fn shape_presentation_has_truthful_empty_selection_copy() {
    let presentation = ShapePresentation::from_selected_device(None);

    assert_eq!(presentation.selected_device(), None);
    assert_eq!(
        presentation.empty_state_message(),
        Some(SHAPE_EMPTY_MESSAGE)
    );
    assert_eq!(
        SHAPE_EMPTY_MESSAGE,
        "No device selected. Open a device from Build to shape it."
    );
}

#[test]
fn ui_open_error_updates_status_without_mutating_valid_model_state() {
    let mut model = AppModel::prototype();
    model.select_lens(Lens::Mix);
    let before_lens = model.lens();
    let before_track = model.selected_track_id();
    let before_device = model.selected_device_id();
    let before_values = model.devices().to_vec();
    let unknown = ObjectId::from_raw(u64::MAX).unwrap();
    let mut status = String::new();

    open_device_in_shape_from_ui(&mut model, unknown, &mut status);

    assert_eq!(
        status,
        "Could not open device in Shape: unknown device. Return to Build and try again."
    );
    assert_eq!(model.lens(), before_lens);
    assert_eq!(model.selected_track_id(), before_track);
    assert_eq!(model.selected_device_id(), before_device);
    assert_eq!(model.devices(), before_values);
}

#[test]
fn ui_parameter_errors_update_status_without_mutating_valid_model_state() {
    for (device_key, parameter_key, expected_error) in [
        ("missing", "gain", "unknown device"),
        ("gain", "missing", "unknown parameter"),
    ] {
        let mut model = AppModel::prototype();
        let gain_id = model.devices()[1].instance_id;
        model.open_device_in_shape(gain_id).unwrap();
        let before_lens = model.lens();
        let before_track = model.selected_track_id();
        let before_device = model.selected_device_id();
        let before_values = model.devices().to_vec();
        let mut status = String::new();

        set_device_parameter_from_ui(&mut model, device_key, parameter_key, 0.25, &mut status);

        assert_eq!(
            status,
            format!(
                "Could not update {device_key}.{parameter_key}: {expected_error}. Reopen the device from Build and try again."
            )
        );
        assert_eq!(model.lens(), before_lens);
        assert_eq!(model.selected_track_id(), before_track);
        assert_eq!(model.selected_device_id(), before_device);
        assert_eq!(model.devices(), before_values);
    }
}

#[test]
fn open_device_in_shape_atomically_focuses_existing_device_and_preserves_track() {
    let mut model = AppModel::prototype();
    model.select_lens(Lens::Build);
    let track = model.selected_track_id();
    let saturator_id = model
        .devices()
        .iter()
        .find(|device| device.key == "saturator")
        .unwrap()
        .instance_id;

    assert_eq!(model.open_device_in_shape(saturator_id), Ok(()));
    assert_eq!(model.lens(), Lens::Shape);
    assert_eq!(model.selected_track_id(), track);
    assert_eq!(model.selected_device_id(), Some(saturator_id));
    let selected = model.selected_device().unwrap();
    assert_eq!(selected.key, "saturator");
    assert_eq!(selected.parameters.len(), SATURATOR_PARAMETERS.len());
    for (control, descriptor) in selected.parameters.iter().zip(SATURATOR_PARAMETERS) {
        assert_eq!(control.descriptor, descriptor);
        assert_ne!(control.instance_id.raw(), 0);
    }
}

#[test]
fn unknown_device_open_fails_without_partial_state_change() {
    let mut model = AppModel::prototype();
    model.select_lens(Lens::Mix);
    let lens = model.lens();
    let track = model.selected_track_id();
    let selection = model.selected_device_id();
    let devices = model.devices().to_vec();
    let unknown = ObjectId::from_raw(u64::MAX).unwrap();
    assert!(!model
        .devices()
        .iter()
        .any(|device| device.instance_id == unknown));

    assert_eq!(model.open_device_in_shape(unknown), Err("unknown device"));
    assert_eq!(model.lens(), lens);
    assert_eq!(model.selected_track_id(), track);
    assert_eq!(model.selected_device_id(), selection);
    assert_eq!(model.devices(), devices);
}

#[test]
fn ordinary_lens_changes_preserve_device_selection() {
    let mut model = AppModel::prototype();
    let gain_id = model
        .devices()
        .iter()
        .find(|device| device.key == "gain")
        .unwrap()
        .instance_id;
    model.open_device_in_shape(gain_id).unwrap();

    for lens in [Lens::Arrange, Lens::Build, Lens::Shape, Lens::Mix] {
        model.select_lens(lens);
        assert_eq!(model.selected_device_id(), Some(gain_id), "{lens}");
    }
}

#[test]
fn parameter_edits_preserve_selected_device_and_descriptor_identity() {
    let mut model = AppModel::prototype();
    let saturator_id = model
        .devices()
        .iter()
        .find(|device| device.key == "saturator")
        .unwrap()
        .instance_id;
    model.open_device_in_shape(saturator_id).unwrap();
    let parameter_ids: Vec<_> = model
        .selected_device()
        .unwrap()
        .parameters
        .iter()
        .map(|parameter| parameter.instance_id)
        .collect();

    model
        .set_device_parameter("saturator", "drive", 100.0)
        .unwrap();

    let selected = model.selected_device().unwrap();
    assert_eq!(model.selected_device_id(), Some(saturator_id));
    assert_eq!(selected.parameters[0].descriptor, SATURATOR_PARAMETERS[0]);
    assert_eq!(
        selected.parameters[0].value,
        SATURATOR_PARAMETERS[0].maximum()
    );
    assert_eq!(
        selected
            .parameters
            .iter()
            .map(|parameter| parameter.instance_id)
            .collect::<Vec<_>>(),
        parameter_ids
    );
}

#[test]
fn offline_snapshot_is_selection_independent_and_attributes_focused_edits() {
    let mut model = AppModel::prototype();
    let before = model.device_parameter_snapshot().unwrap();
    let saturator = model
        .devices()
        .iter()
        .find(|device| device.key == "saturator")
        .unwrap();
    let saturator_id = saturator.instance_id;
    let drive_id = saturator.parameters[0].instance_id;

    model.open_device_in_shape(saturator_id).unwrap();
    assert_eq!(model.device_parameter_snapshot().unwrap(), before);

    model
        .set_device_parameter("saturator", "drive", 4.25)
        .unwrap();
    let after = model.device_parameter_snapshot().unwrap();
    assert_eq!(after.len(), before.len());
    let drive = after
        .iter()
        .find(|entry| {
            entry.device_key() == "saturator" && entry.parameter_key().as_str() == "drive"
        })
        .unwrap();
    assert_eq!(drive.device_instance_id(), saturator_id);
    assert_eq!(drive.parameter_instance_id(), drive_id);
    assert_eq!(drive.value(), 4.25);
    for prior in before.iter().filter(|entry| {
        !(entry.device_key() == "saturator" && entry.parameter_key().as_str() == "drive")
    }) {
        assert!(after.iter().any(|entry| entry == prior));
    }
}

#[test]
fn prototype_device_controls_derive_from_backend_descriptors() {
    let model = AppModel::prototype();
    let devices = model.devices();
    assert_eq!(devices.len(), 5);
    assert_eq!(devices[0].parameters[0].descriptor, PULSE_PARAMETERS[0]);
    assert_eq!(devices[1].parameters[0].descriptor, GAIN_PARAMETERS[0]);
    assert_eq!(devices[2].parameters[0].descriptor, SATURATOR_PARAMETERS[0]);
    assert_eq!(devices[2].parameters[1].descriptor, SATURATOR_PARAMETERS[1]);
    // R4-6 appends; the three above must keep both their slots and their descriptors
    assert_eq!(devices[3].parameters[0].descriptor, FILAMENT_PARAMETERS[0]);
    assert_eq!(devices[4].parameters[0].descriptor, GLOAM_PARAMETERS[0]);
}

#[test]
fn prototype_devices_and_parameters_have_unique_nonzero_instance_ids() {
    let model = AppModel::prototype();
    let mut ids = HashSet::new();

    for track in model.tracks() {
        assert_ne!(track.id().raw(), 0);
        assert!(ids.insert(track.id()));
    }
    for device in model.devices() {
        assert_ne!(device.instance_id.raw(), 0);
        assert!(ids.insert(device.instance_id));
        for parameter in &device.parameters {
            assert_ne!(parameter.instance_id.raw(), 0);
            assert!(ids.insert(parameter.instance_id));
        }
    }
}

#[test]
fn snapshot_preserves_control_instance_ids_across_value_edits() {
    let mut model = AppModel::prototype();
    let before = model.device_parameter_snapshot().unwrap();

    model
        .set_device_parameter("saturator", "drive", 7.25)
        .unwrap();
    let after = model.device_parameter_snapshot().unwrap();

    for prior in before {
        let current = after
            .iter()
            .find(|entry| {
                entry.device_key() == prior.device_key()
                    && entry.parameter_key() == prior.parameter_key()
            })
            .unwrap();
        assert_eq!(current.device_instance_id(), prior.device_instance_id());
        assert_eq!(
            current.parameter_instance_id(),
            prior.parameter_instance_id()
        );
    }
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

#[test]
fn setter_and_snapshot_preserve_signed_zero_and_subnormal_bits() {
    let mut model = AppModel::prototype();
    for value in [-0.0_f32, f32::from_bits(1)] {
        model.set_device_parameter("gain", "gain", value).unwrap();
        let control = model
            .devices()
            .iter()
            .find(|device| device.key == "gain")
            .unwrap()
            .parameters
            .iter()
            .find(|parameter| parameter.descriptor.key == GAIN_PARAMETERS[0].key)
            .unwrap();
        let snapshot = model.device_parameter_snapshot().unwrap();
        let published = snapshot
            .iter()
            .find(|parameter| parameter.device_key() == "gain")
            .unwrap();

        assert_eq!(control.value.to_bits(), value.to_bits());
        assert_eq!(published.value().to_bits(), value.to_bits());
    }

    model
        .set_device_parameter("gain", "gain", f32::from_bits(0x8000_0001))
        .unwrap();
    let published = model
        .device_parameter_snapshot()
        .unwrap()
        .into_iter()
        .find(|parameter| parameter.device_key() == "gain")
        .unwrap();
    assert_eq!(published.value().to_bits(), 0.0_f32.to_bits());

    model
        .set_device_parameter("gain", "gain", f32::NAN)
        .unwrap();
    let published = model
        .device_parameter_snapshot()
        .unwrap()
        .into_iter()
        .find(|parameter| parameter.device_key() == "gain")
        .unwrap();
    assert_eq!(
        published.value().to_bits(),
        GAIN_PARAMETERS[0].default().to_bits()
    );
}

// R4-2 test 15 — edit_device_parameter is the single clamping site and reports the identities
// the live lane addresses by, so a value cannot be clamped in one place and published from another
#[test]
fn editing_a_parameter_reports_the_clamped_value_and_its_stable_identities() {
    let mut model = AppModel::prototype();
    let gain = model
        .devices()
        .iter()
        .find(|device| device.key == "gain")
        .expect("the prototype ships a gain device");
    let device_id = gain.instance_id;
    let parameter_id = gain.parameters[0].instance_id;
    let maximum = gain.parameters[0].descriptor.maximum();

    let edit = model
        .edit_device_parameter("gain", "gain", 999.0)
        .expect("a known device and parameter must accept an edit");

    assert_eq!(edit.device_instance_id, device_id);
    assert_eq!(edit.parameter_instance_id, parameter_id);
    assert_eq!(
        edit.value, maximum,
        "the reported value must be the clamped one the model stored"
    );
    assert_eq!(
        model
            .devices()
            .iter()
            .find(|device| device.key == "gain")
            .unwrap()
            .parameters[0]
            .value,
        maximum
    );

    assert!(model.edit_device_parameter("gain", "nope", 0.5).is_err());
    assert!(model.edit_device_parameter("nope", "gain", 0.5).is_err());
}

// R4-6 test 1 — both new devices reach the Build surface with their own descriptors intact
#[test]
fn prototype_exposes_the_two_new_devices() {
    let model = AppModel::prototype();

    for (key, name, descriptors) in [
        ("filament", "Filament", FILAMENT_PARAMETERS.as_slice()),
        ("gloam", "Gloam", GLOAM_PARAMETERS.as_slice()),
    ] {
        let device = model
            .devices()
            .iter()
            .find(|device| device.key == key)
            .unwrap_or_else(|| panic!("{key} must appear in the prototype device list"));
        assert_eq!(device.name, name);
        assert_eq!(device.parameters.len(), descriptors.len());
        for (control, descriptor) in device.parameters.iter().zip(descriptors) {
            assert_eq!(control.descriptor, *descriptor);
            // A device whose controls did not start at their own defaults would be a surface
            // showing values the device does not hold
            assert_eq!(control.value, descriptor.default());
            assert_ne!(control.instance_id.raw(), 0);
        }
    }
}

// R4-6 test 2 — the offline fixture's arity is a contract, not an accident.
// `DeviceValues::from_snapshot` refuses any snapshot whose length is not four, so this fails the
// moment someone changes `device_parameter_snapshot` to iterate `self.devices`
#[test]
fn snapshot_arity_is_unchanged_by_new_devices() {
    let model = AppModel::prototype();
    let snapshot = model.device_parameter_snapshot().unwrap();

    assert_eq!(snapshot.len(), 4);
    assert!(
        model.devices().len() > 4,
        "the guard is vacuous unless the model carries more devices than the fixture"
    );
    for entry in &snapshot {
        assert_ne!(entry.device_key(), "filament");
        assert_ne!(entry.device_key(), "gloam");
    }
}

// R4-6 test 3 — the existing drill-in works against a device that did not exist when it was
// written. The lookup is asserted first: a wrong id leaves lens and selection untouched, and the
// two assertions below would then report a focus failure instead of the lookup failure
#[test]
fn opening_a_new_device_focuses_shape() {
    let mut model = AppModel::prototype();
    model.select_lens(Lens::Build);
    let filament_id = model
        .devices()
        .iter()
        .find(|device| device.key == "filament")
        .unwrap()
        .instance_id;

    assert_eq!(model.open_device_in_shape(filament_id), Ok(()));
    assert_eq!(model.lens(), Lens::Shape);
    assert_eq!(model.selected_device_id(), Some(filament_id));
}

// R4-5 E-1 — a new clip takes selection without moving the user's track context
#[test]
fn creating_a_clip_selects_it_and_leaves_track_selection_intact() {
    let mut model = AppModel::prototype();
    let track = model.selected_track_id().unwrap();
    let lens = model.lens();

    let placement = model
        .create_clip(track, "phrase", BeatTicks::from_beats(4), BeatTicks(0))
        .unwrap();

    assert_eq!(model.selected_clip(), Some(placement));
    assert_eq!(model.selected_track_id(), Some(track));
    assert_eq!(model.lens(), lens);
    assert_eq!(model.clip_track(placement), Some(track));
}

// R4-5 E-2 — selecting a clip changes no lens, unlike open_device_in_shape which does deliberately
#[test]
fn selecting_a_clip_does_not_change_the_lens() {
    let mut model = AppModel::prototype();
    let track = model.selected_track_id().unwrap();
    let first = model
        .create_clip(track, "one", BeatTicks::from_beats(4), BeatTicks(0))
        .unwrap();
    let second = model
        .create_clip(
            track,
            "two",
            BeatTicks::from_beats(4),
            BeatTicks::from_beats(4),
        )
        .unwrap();

    model.select_lens(Lens::Arrange);
    assert_eq!(model.select_clip(first), Ok(()));
    assert_eq!(model.lens(), Lens::Arrange);
    assert_eq!(model.selected_clip(), Some(first));

    model.select_lens(Lens::Build);
    assert_eq!(model.select_clip(second), Ok(()));
    assert_eq!(
        model.lens(),
        Lens::Build,
        "a clip is edited where it lives, not in a separate surface"
    );
}

// R4-5 E-3 — a refused clip edit reports the specific error and rolls back completely
#[test]
fn an_invalid_clip_edit_reports_and_rolls_back() {
    let mut model = AppModel::prototype();
    let track = model.selected_track_id().unwrap();
    model
        .create_clip(track, "held", BeatTicks::from_beats(4), BeatTicks(0))
        .unwrap();
    let before_clips = model.track_list().placement_count();
    let before_material = model.track_list().clips().len();
    let before_selection = model.selected_clip();

    // Overlaps the placement above
    let refused = model.create_clip(
        track,
        "clashing",
        BeatTicks::from_beats(4),
        BeatTicks::from_beats(2),
    );
    assert!(matches!(
        refused,
        Err(ClipError::OverlappingPlacement { .. })
    ));
    assert_eq!(model.track_list().placement_count(), before_clips);
    assert_eq!(
        model.track_list().clips().len(),
        before_material,
        "a refused placement must not leave orphaned clip material behind"
    );
    assert_eq!(model.selected_clip(), before_selection);

    // A blank name is refused before anything is minted
    assert_eq!(
        model.create_clip(
            track,
            "  ",
            BeatTicks::from_beats(4),
            BeatTicks::from_beats(8)
        ),
        Err(ClipError::BlankName)
    );
    assert_eq!(model.track_list().clips().len(), before_material);

    // And an unknown placement cannot be selected
    let stranger = model
        .create_clip(
            track,
            "temp",
            BeatTicks::from_beats(1),
            BeatTicks::from_beats(16),
        )
        .unwrap();
    model.select_clip(stranger).unwrap();
    assert!(model.select_clip(track).is_err());
    assert_eq!(model.selected_clip(), Some(stranger));
}

// R4-5 E-4 — the empty lane states the fact rather than advertising
#[test]
fn the_empty_clip_lane_reports_no_clips() {
    let model = AppModel::prototype();
    let track = model.selected_track_id().unwrap();

    assert_eq!(model.track_list().placement_count(), 0);
    assert_eq!(CLIP_LANE_EMPTY_MESSAGE, "No clips on this track.");
    assert_eq!(
        CLIP_INSPECTOR_EMPTY_MESSAGE,
        "No clip selected. Select a clip in Arrange to edit its notes."
    );
    assert_eq!(model.selected_clip(), None);
    assert!(model.track_list().get(track).unwrap().clips().is_empty());
}

// R4-5 E-5 — every clip row exposes a label carrying its whole state
#[test]
fn every_clip_row_exposes_a_non_empty_accessible_label() {
    let mut model = AppModel::prototype();
    let track = model.selected_track_id().unwrap();
    let placement = model
        .create_clip(
            track,
            "phrase",
            BeatTicks::from_beats(4),
            BeatTicks::from_beats(8),
        )
        .unwrap();

    let label = model.clip_label(placement).unwrap();
    for fragment in ["phrase", "8.000", "4.000", "0 notes", "active"] {
        assert!(
            label.contains(fragment),
            "the label must carry {fragment}; it was {label:?}"
        );
    }

    // Deactivating must be legible in the label, not only in a color
    model.set_clip_active(placement, false).unwrap();
    assert!(model.clip_label(placement).unwrap().contains("inactive"));
    assert_eq!(model.clip_label(track), None);
}
