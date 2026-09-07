// Author: Jeff
// Date: 2026-09-07
// Description: Evidence that a device's shaping controls reach live audio, not just its level
// Notes: "Sound design in first-party devices" is the vision's second core-loop item, and it was
//   structurally absent. Every device contributed exactly ONE parameter target, so Filament's
//   lean, rise and fall — the three controls that shape its tone — reached no live node.
//
//   Worse than absent: the single slot was addressed as descriptor index 0, which is `level` on
//   Pulse and `lean` on Filament, and the route table labelled it with Pulse's level key. On a
//   Filament track that wired the LEAN slider to the level and left the LEVEL slider connected to
//   nothing. R4 QA row 3 would have caught it on a Filament track; no operator has run it.

use spectre_app::AppModel;
use spectre_core::ObjectId;
use spectre_project::{TrackInstrument, TrackList};

fn filament_track() -> (AppModel, ObjectId) {
    let mut model = AppModel::prototype();
    let id = model.add_track("Lead").expect("the track is added");
    model
        .set_track_instrument(id, TrackInstrument::Filament)
        .expect("the instrument is changed");
    (model, id)
}

// Every descriptor of every device on the track must have a distinct target
#[test]
fn every_filament_control_has_its_own_target() {
    let (model, id) = filament_track();
    let list: &TrackList = model.track_list();
    let index = list.index_of(id).expect("the track is in the list");
    let descriptors = spectre_project::instrument_parameters(TrackInstrument::Filament);
    assert_eq!(descriptors.len(), 4, "Filament's descriptor set changed");

    let mut seen = Vec::new();
    for position in 0..descriptors.len() {
        let target = list
            .instrument_parameter_index(index, position)
            .unwrap_or_else(|| panic!("no target for descriptor {position}"));
        assert!(
            !seen.contains(&target),
            "descriptor {position} shares a target with an earlier one"
        );
        seen.push(target);
    }
}

// The defect this slice exists for: lean and level are different controls and must not share a
// target. They did — both resolved to the instrument's single slot
#[test]
fn lean_and_level_do_not_share_a_target() {
    let (model, id) = filament_track();
    let list = model.track_list();
    let index = list.index_of(id).expect("the track is in the list");

    let descriptors = spectre_project::instrument_parameters(TrackInstrument::Filament);
    let lean = descriptors
        .iter()
        .position(|d| d.key.as_str() == "lean")
        .expect("Filament has a lean control");
    let level = descriptors
        .iter()
        .position(|d| d.key.as_str() == "level")
        .expect("Filament has a level control");
    assert_ne!(
        lean, level,
        "the descriptor set no longer distinguishes them"
    );

    assert_ne!(
        list.instrument_parameter_index(index, lean),
        list.instrument_parameter_index(index, level),
        "lean and level resolve to the same target, so one control drives the other"
    );
}

// A position past the descriptor set has no target rather than aliasing onto the next device
#[test]
fn a_position_past_the_descriptor_set_has_no_target() {
    let (model, id) = filament_track();
    let list = model.track_list();
    let index = list.index_of(id).expect("the track is in the list");
    let count = spectre_project::instrument_parameters(TrackInstrument::Filament).len();
    assert_eq!(list.instrument_parameter_index(index, count), None);
}

// Pulse has one control and Filament four, so a Filament track occupies more of the target list.
// A fixed stride per device is what made that impossible to express
#[test]
fn a_track_occupies_as_many_targets_as_its_devices_have_controls() {
    let mut model = AppModel::prototype();
    let pulse = model.add_track("Pulse").expect("added");
    let filament = model.add_track("Filament").expect("added");
    model
        .set_track_instrument(filament, TrackInstrument::Filament)
        .expect("changed");

    let list = model.track_list();
    let pulse_index = list.index_of(pulse).expect("present");
    let filament_index = list.index_of(filament).expect("present");

    // The gain target sits after the instrument's controls, so the gap between a track's first
    // target and its gain is exactly its instrument's descriptor count
    let pulse_span =
        list.gain_target_index(pulse_index) - list.instrument_target_index(pulse_index);
    let filament_span =
        list.gain_target_index(filament_index) - list.instrument_target_index(filament_index);
    assert_eq!(pulse_span, 1, "Pulse has one control");
    assert_eq!(filament_span, 4, "Filament has four");
}

// ---- stored values must reach the engine ----
//
// Routing a control is only half of it. `instrument_for` builds every device from its DESCRIPTOR
// DEFAULTS and passes only the track's level, so a saved `lean` never reached the constructed
// device. Shape would show the musician's value while the engine played the default: the UI and
// the audio disagreeing about the same control.

#[test]
fn stored_device_values_are_published_when_the_engine_opens() {
    use spectre_app::engine::{build_track_engine_parts, publish_stored_parameters};
    use spectre_audio::null::{NullBackend, NULL_DEVICE_KEY, NULL_SAMPLE_RATE};
    use spectre_audio::{AudioStream, DeviceId, RenderBlock, StreamConfig};

    let (mut model, id) = filament_track();
    model.select_track(id);

    // A lean far from its descriptor default, so a device built from defaults cannot match it
    let default_lean = spectre_dsp::FILAMENT_PARAMETERS[0].default();
    let edited = if default_lean > 0.5 { 0.1 } else { 0.9 };
    model
        .edit_device_parameter("filament", "lean", edited)
        .expect("lean is an editable control");

    let config = StreamConfig::stereo(NULL_SAMPLE_RATE, 256).expect("a valid config");
    let parts = build_track_engine_parts(
        model.track_list(),
        model.tempo_map(),
        spectre_app::engine::APP_GRAPH_SEED,
        model.selected_track_id(),
        config,
    )
    .expect("the parts build");

    let mut bridge = parts.bridge;
    let backend = NullBackend::new();
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            config,
            Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),
        )
        .expect("the null stream opens");
    stream.start().expect("the stream starts");

    let mut engine = spectre_app::engine::LiveEngine::from_open_stream(
        Box::new(stream),
        parts.sender,
        parts.telemetry,
        spectre_audio::NULL_BACKEND_NAME,
        "Null Output".to_string(),
        config,
    );
    engine.set_targets(parts.targets);
    engine.set_primary_index(parts.primary_index);

    let published = publish_stored_parameters(&model, &engine).expect("the lane accepts them");
    assert!(
        published >= 4,
        "only {published} values were published; Filament alone has four controls"
    );

    // Drain the lane so the render thread applies them
    for _ in 0..4 {
        engine.stream_mut().pump().expect("blocks render");
    }
    assert!(
        engine.health().parameters_applied >= published as u64,
        "the render thread applied {} of {published} published values",
        engine.health().parameters_applied
    );
}
