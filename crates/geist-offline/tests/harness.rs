// Author: Jeff
// Date: 2026-07-12
// Description: Contract tests for deterministic offline harness reports and plan rendering
// Notes: R2 gates — the fixture renders through the compiled plan and matches a hand-wired chain

use geist_dsp::{AudioProcessor, Gain, ProcessContext, PulseInstrument, Saturator, Waveform};
use geist_offline::{
    default_project, fixture_events, inspect_project, render_silence, render_vertical_slice,
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
