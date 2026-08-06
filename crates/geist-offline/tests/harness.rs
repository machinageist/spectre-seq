// Author: Jeff
// Date: 2026-07-12
// Description: Contract tests for deterministic offline harness reports and plan rendering
// Notes: R2 gates — the fixture renders through the compiled plan and matches a hand-wired chain

use geist_app::AppModel;
use geist_core::{ParamSpec, ParamUnit};
use geist_dsp::{
    AudioProcessor, DeviceParameterSnapshot, DspParameter, Gain, ProcessContext, PulseInstrument,
    Saturator, Waveform, GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS,
};
use geist_offline::{
    default_project, fixture_events, inspect_project, render_app_snapshot, render_silence,
    render_vertical_slice, RenderReport,
};
use geist_project::to_bytes;

#[test]
fn default_project_report_is_deterministic() {
    let bytes = to_bytes(&default_project()).unwrap();
    let first = inspect_project(&bytes).unwrap();
    let second = inspect_project(&bytes).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.schema_version, 1);
    assert_eq!(first.project_name, "Untitled");
    assert_eq!(first.tempo_segment_count, 1);
    assert_eq!(first.transport_position_samples, 0);
}

#[test]
fn invalid_project_is_reported_without_panicking() {
    let error = inspect_project(b"not a project").unwrap_err();
    assert!(error.contains("malformed project"));
}

#[test]
fn native_device_chain_renders_deterministically() {
    let first = render_vertical_slice(48_000.0, 4_096).unwrap();
    let second = render_vertical_slice(48_000.0, 4_096).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.frames, 4_096);
    assert_eq!(first.channels, 2);
    assert_ne!(first.hash, 0);
    assert!(first.peak > 0.0 && first.peak <= 1.0);
}

// R2 silence gate: no events through the plan produce exact finite silence
#[test]
fn plan_renders_exact_silence_without_events() {
    let report = render_silence(48_000.0, 1_024).unwrap();
    assert_eq!(report.peak, 0.0);
    assert_eq!(report, render_silence(48_000.0, 1_024).unwrap());
}

// R2 hash gate: the compiled-plan render is bit-identical to a hand-wired device chain
#[test]
fn plan_render_matches_hand_wired_chain() {
    let sample_rate = 48_000.0;
    let frames = 2_048;
    let events = fixture_events(frames);
    let instrument_context = ProcessContext::new(sample_rate, frames, &events).unwrap();
    let effect_context = ProcessContext::new(sample_rate, frames, &[]).unwrap();

    let mut instrument = PulseInstrument::new(Waveform::Saw, 0.3).unwrap();
    let mut gain = Gain::new(0.7).unwrap();
    let mut saturator = Saturator::new(2.5, 0.35).unwrap();
    let mut synth = vec![vec![0.0_f32; frames]; 2];
    let mut gained = vec![vec![0.0_f32; frames]; 2];
    let mut wired = vec![vec![0.0_f32; frames]; 2];
    {
        let (left, right) = synth.split_at_mut(1);
        instrument
            .process(&instrument_context, &[], &mut [&mut left[0], &mut right[0]])
            .unwrap();
    }
    {
        let (left, right) = gained.split_at_mut(1);
        gain.process(
            &effect_context,
            &[&synth[0], &synth[1]],
            &mut [&mut left[0], &mut right[0]],
        )
        .unwrap();
    }
    {
        let (left, right) = wired.split_at_mut(1);
        saturator
            .process(
                &effect_context,
                &[&gained[0], &gained[1]],
                &mut [&mut left[0], &mut right[0]],
            )
            .unwrap();
    }

    let mut peak = 0.0_f32;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for sample in wired[0].iter().chain(wired[1].iter()) {
        peak = peak.max(sample.abs());
        for byte in sample.to_bits().to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    let plan_report = render_vertical_slice(sample_rate, frames).unwrap();
    assert_eq!(plan_report.hash, hash);
    assert_eq!(plan_report.peak, peak);
}

fn hand_wired_report(
    sample_rate: f64,
    frames: usize,
    level: f32,
    gain_value: f32,
    drive: f32,
    mix: f32,
) -> RenderReport {
    let events = fixture_events(frames);
    let instrument_context = ProcessContext::new(sample_rate, frames, &events).unwrap();
    let effect_context = ProcessContext::new(sample_rate, frames, &[]).unwrap();
    let mut instrument = PulseInstrument::new(Waveform::Saw, level).unwrap();
    let mut gain = Gain::new(gain_value).unwrap();
    let mut saturator = Saturator::new(drive, mix).unwrap();
    let mut synth = vec![vec![0.0_f32; frames]; 2];
    let mut gained = vec![vec![0.0_f32; frames]; 2];
    let mut wired = vec![vec![0.0_f32; frames]; 2];
    {
        let (left, right) = synth.split_at_mut(1);
        instrument
            .process(&instrument_context, &[], &mut [&mut left[0], &mut right[0]])
            .unwrap();
    }
    {
        let (left, right) = gained.split_at_mut(1);
        gain.process(
            &effect_context,
            &[&synth[0], &synth[1]],
            &mut [&mut left[0], &mut right[0]],
        )
        .unwrap();
    }
    {
        let (left, right) = wired.split_at_mut(1);
        saturator
            .process(
                &effect_context,
                &[&gained[0], &gained[1]],
                &mut [&mut left[0], &mut right[0]],
            )
            .unwrap();
    }

    let mut peak = 0.0_f32;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for sample in wired[0].iter().chain(wired[1].iter()) {
        peak = peak.max(sample.abs());
        for byte in sample.to_bits().to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    RenderReport {
        frames,
        channels: 2,
        peak,
        hash,
    }
}

#[test]
fn every_app_parameter_maps_exactly_to_the_compiled_plan() {
    let sample_rate = 48_000.0;
    let frames = 2_048;
    let defaults = [
        PULSE_PARAMETERS[0].default(),
        GAIN_PARAMETERS[0].default(),
        SATURATOR_PARAMETERS[0].default(),
        SATURATOR_PARAMETERS[1].default(),
    ];
    let cases = [
        ("pulse", "level", 0.61, 0),
        ("gain", "gain", 1.37, 1),
        ("saturator", "drive", 4.25, 2),
        ("saturator", "mix", 0.83, 3),
    ];

    for (device, parameter, value, expected_index) in cases {
        let mut model = AppModel::prototype();
        model
            .set_device_parameter(device, parameter, value)
            .unwrap();
        let mut expected_values = defaults;
        expected_values[expected_index] = value;
        let snapshot = model.device_parameter_snapshot().unwrap();
        let report = render_app_snapshot(sample_rate, frames, &snapshot).unwrap();
        let expected = hand_wired_report(
            sample_rate,
            frames,
            expected_values[0],
            expected_values[1],
            expected_values[2],
            expected_values[3],
        );

        assert_eq!(report, expected, "{device}.{parameter}");
    }
}

#[test]
fn identical_app_snapshots_render_identically() {
    let snapshot = AppModel::prototype().device_parameter_snapshot().unwrap();
    let first = render_app_snapshot(48_000.0, 2_048, &snapshot).unwrap();
    let second = render_app_snapshot(48_000.0, 2_048, &snapshot).unwrap();

    assert_eq!(first, second);
}

#[test]
fn app_defaults_match_backend_authoritative_default_render() {
    let frames = 2_048;
    let snapshot = AppModel::prototype().device_parameter_snapshot().unwrap();
    let report = render_app_snapshot(48_000.0, frames, &snapshot).unwrap();
    let expected = hand_wired_report(
        48_000.0,
        frames,
        PULSE_PARAMETERS[0].default(),
        GAIN_PARAMETERS[0].default(),
        SATURATOR_PARAMETERS[0].default(),
        SATURATOR_PARAMETERS[1].default(),
    );

    assert_eq!(report, expected);
}

fn complete_snapshot() -> Vec<DeviceParameterSnapshot> {
    vec![
        DeviceParameterSnapshot::new("pulse", PULSE_PARAMETERS[0], PULSE_PARAMETERS[0].default()),
        DeviceParameterSnapshot::new("gain", GAIN_PARAMETERS[0], GAIN_PARAMETERS[0].default()),
        DeviceParameterSnapshot::new(
            "saturator",
            SATURATOR_PARAMETERS[0],
            SATURATOR_PARAMETERS[0].default(),
        ),
        DeviceParameterSnapshot::new(
            "saturator",
            SATURATOR_PARAMETERS[1],
            SATURATOR_PARAMETERS[1].default(),
        ),
    ]
}

#[test]
fn snapshot_constructor_contains_non_finite_and_out_of_range_values() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let snapshot = DeviceParameterSnapshot::new("gain", GAIN_PARAMETERS[0], value);
        assert_eq!(snapshot.value(), GAIN_PARAMETERS[0].default());
    }

    let below = DeviceParameterSnapshot::new("gain", GAIN_PARAMETERS[0], -1.0);
    let above = DeviceParameterSnapshot::new("gain", GAIN_PARAMETERS[0], 100.0);
    assert_eq!(below.value(), GAIN_PARAMETERS[0].minimum());
    assert_eq!(above.value(), GAIN_PARAMETERS[0].maximum());
}

#[test]
fn mismatched_device_and_parameter_identity_is_rejected() {
    let mut snapshot = complete_snapshot();
    snapshot[0] = DeviceParameterSnapshot::new("pulse", GAIN_PARAMETERS[0], 0.0);
    let error = render_app_snapshot(48_000.0, 2_048, &snapshot).unwrap_err();

    assert!(error.contains("pulse"));
    assert!(error.contains("gain"));
}

#[test]
fn incomplete_snapshots_are_rejected() {
    let empty = render_app_snapshot(48_000.0, 2_048, &[]).unwrap_err();
    assert!(empty.contains("exactly four"));

    let partial = complete_snapshot().into_iter().take(3).collect::<Vec<_>>();
    let error = render_app_snapshot(48_000.0, 2_048, &partial).unwrap_err();
    assert!(error.contains("exactly four"));
}

#[test]
fn duplicate_parameter_identity_is_rejected() {
    let mut snapshot = complete_snapshot();
    snapshot[3] = snapshot[2];

    let error = render_app_snapshot(48_000.0, 2_048, &snapshot).unwrap_err();
    assert!(error.contains("duplicate"));
    assert!(error.contains("saturator.drive"));
}

#[test]
fn complete_snapshot_is_order_independent() {
    let canonical = complete_snapshot();
    let expected = render_app_snapshot(48_000.0, 2_048, &canonical).unwrap();
    let mut reordered = canonical;
    reordered.reverse();

    assert_eq!(
        render_app_snapshot(48_000.0, 2_048, &reordered).unwrap(),
        expected
    );
}

#[test]
fn offline_rejects_values_not_canonical_for_authoritative_descriptor() {
    let spoofed_spec = ParamSpec::new(ParamUnit::Linear, 0.0, 100.0, 50.0).unwrap();
    let spoofed_gain = DspParameter::new(GAIN_PARAMETERS[0].key, "Gain", spoofed_spec);
    let mut snapshot = complete_snapshot();
    snapshot[1] = DeviceParameterSnapshot::new("gain", spoofed_gain, 50.0);

    let error = render_app_snapshot(48_000.0, 2_048, &snapshot).unwrap_err();
    assert!(error.contains("non-canonical value"));
    assert!(error.contains("gain.gain"));
}

#[test]
fn model_snapshot_contains_nonfinite_edit_before_render() {
    let mut model = AppModel::prototype();
    model
        .set_device_parameter("gain", "gain", f32::NAN)
        .unwrap();

    let snapshot = model.device_parameter_snapshot().unwrap();
    let report = render_app_snapshot(48_000.0, 2_048, &snapshot).unwrap();
    assert_eq!(
        report,
        hand_wired_report(
            48_000.0,
            2_048,
            PULSE_PARAMETERS[0].default(),
            GAIN_PARAMETERS[0].default(),
            SATURATOR_PARAMETERS[0].default(),
            SATURATOR_PARAMETERS[1].default(),
        )
    );
}
