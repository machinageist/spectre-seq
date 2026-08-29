// Author: Jeff
// Date: 2026-07-12
// Description: Contract tests for native Spectre sources, instruments, and effects
// Notes: Tests pin process behavior before device implementation

use spectre_dsp::{
    AudioProcessor, DeviceClass, DeviceParameterKey, Filament, Gain, Gloam, NoteEvent,
    NoteEventKind, ParameterError, ProcessContext, PulseInstrument, Saturator, ToneSource,
    Waveform, FILAMENT_PARAMETERS, GAIN_PARAMETERS, GLOAM_PARAMETERS,
    NORMALIZED_ROUND_TRIP_MAX_ULPS, PULSE_PARAMETERS, SATURATOR_PARAMETERS, TONE_PARAMETERS,
};
use std::f64::consts::{PI, TAU};

fn output(frames: usize) -> (Vec<f32>, Vec<f32>) {
    (vec![0.0; frames], vec![0.0; frames])
}

#[test]
fn parameter_metadata_uses_typed_device_keys_and_core_semantics() {
    let drive = SATURATOR_PARAMETERS[0];
    let key: DeviceParameterKey = drive.key;

    assert_eq!(key.as_str(), "drive");
    assert_eq!(drive.clamp(100.0), drive.maximum());
    assert_eq!(drive.clamp(f32::NAN), drive.default());
    assert_eq!(drive.to_normalized(12.5), 0.5);
    assert_eq!(drive.from_normalized(0.5), 12.5);
    assert!(drive.validate(24.0).is_ok());
    assert!(drive.validate(24.1).is_err());
}

#[test]
fn processor_construction_uses_descriptor_validation() {
    assert!(Gain::new(GAIN_PARAMETERS[0].maximum()).is_ok());
    assert!(Gain::new(GAIN_PARAMETERS[0].maximum() + 0.1).is_err());
    assert!(Saturator::new(f32::NAN, SATURATOR_PARAMETERS[1].default()).is_err());
}

fn native_parameters() -> impl Iterator<Item = spectre_dsp::DspParameter> {
    PULSE_PARAMETERS
        .into_iter()
        .chain(GAIN_PARAMETERS)
        .chain(SATURATOR_PARAMETERS)
        .chain(TONE_PARAMETERS)
        .chain(FILAMENT_PARAMETERS)
        .chain(GLOAM_PARAMETERS)
}

fn next_up(value: f32) -> f32 {
    if value == 0.0 {
        f32::from_bits(1)
    } else if value > 0.0 {
        f32::from_bits(value.to_bits() + 1)
    } else {
        f32::from_bits(value.to_bits() - 1)
    }
}

fn next_down(value: f32) -> f32 {
    if value == 0.0 {
        f32::from_bits(0x8000_0001)
    } else if value > 0.0 {
        f32::from_bits(value.to_bits() - 1)
    } else {
        f32::from_bits(value.to_bits() + 1)
    }
}

fn ulp_diff(left: f32, right: f32) -> u32 {
    left.to_bits().abs_diff(right.to_bits())
}

#[test]
fn every_native_parameter_meets_the_declared_normalized_round_trip_policy() {
    let mut state = 0x1234_5678_u32;
    for parameter in native_parameters() {
        let fixtures = [0.0, next_up(0.0), 0.25, 0.5, 0.75, next_down(1.0), 1.0];
        let mut previous_plain = f32::NEG_INFINITY;
        for normalized in fixtures.into_iter().chain((0..10_000).map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (f64::from(state) / f64::from(u32::MAX)) as f32
        })) {
            let plain = parameter.from_normalized(normalized);
            let round_trip = parameter.to_normalized(plain);
            let adjacent_plain = parameter.from_normalized(next_up(normalized).min(1.0));
            assert!(plain.is_finite());
            assert!(round_trip.is_finite());
            assert!(adjacent_plain >= plain);
            assert!(plain >= previous_plain || !fixtures.contains(&normalized));
            if fixtures.contains(&normalized) {
                previous_plain = plain;
            }
            assert!(
                ulp_diff(normalized, round_trip) <= NORMALIZED_ROUND_TRIP_MAX_ULPS,
                "{}.{}: {normalized:e} -> {plain:e} -> {round_trip:e}",
                parameter.name,
                parameter.key.as_str()
            );
        }

        assert_eq!(
            parameter.from_normalized(0.0).to_bits(),
            parameter.minimum().to_bits()
        );
        assert_eq!(
            parameter.from_normalized(1.0).to_bits(),
            parameter.maximum().to_bits()
        );
        assert_eq!(
            parameter.to_normalized(parameter.minimum()).to_bits(),
            0.0_f32.to_bits()
        );
        assert_eq!(
            parameter.to_normalized(parameter.maximum()).to_bits(),
            1.0_f32.to_bits()
        );
    }
}

#[test]
fn normalized_to_plain_uses_nearest_f32_quantization_or_an_exact_endpoint() {
    let mut state = 0xa341_316c_u32;
    for parameter in native_parameters() {
        for normalized in
            [0.0, next_up(0.0), next_down(1.0), 1.0]
                .into_iter()
                .chain((0..10_000).map(|_| {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    (f64::from(state) / f64::from(u32::MAX)) as f32
                }))
        {
            let exact = f64::from(parameter.minimum())
                + f64::from(normalized)
                    * (f64::from(parameter.maximum()) - f64::from(parameter.minimum()));
            let plain = parameter.from_normalized(normalized);
            assert_eq!(plain.to_bits(), (exact as f32).to_bits());
        }

        let next_from_zero = parameter.from_normalized(next_up(0.0));
        if next_from_zero == parameter.minimum() {
            assert_eq!(
                parameter.to_normalized(next_from_zero).to_bits(),
                0.0_f32.to_bits()
            );
        }
        let next_from_one = parameter.from_normalized(next_down(1.0));
        if next_from_one == parameter.maximum() {
            assert_eq!(
                parameter.to_normalized(next_from_one).to_bits(),
                1.0_f32.to_bits()
            );
        }
    }
}

#[test]
fn descriptor_clamping_has_an_exact_signed_zero_and_subnormal_policy() {
    let positive_zero_minimum = GAIN_PARAMETERS[0];
    assert_eq!(
        positive_zero_minimum.clamp(-0.0).to_bits(),
        (-0.0_f32).to_bits()
    );
    assert_eq!(
        positive_zero_minimum.clamp(f32::from_bits(1)).to_bits(),
        f32::from_bits(1).to_bits()
    );
    assert_eq!(
        positive_zero_minimum
            .clamp(f32::from_bits(0x8000_0001))
            .to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(
        positive_zero_minimum.clamp(f32::NAN).to_bits(),
        positive_zero_minimum.default().to_bits()
    );

    for parameter in native_parameters() {
        assert_eq!(
            parameter.clamp(next_down(parameter.minimum())).to_bits(),
            parameter.minimum().to_bits()
        );
        assert_eq!(
            parameter.clamp(next_up(parameter.maximum())).to_bits(),
            parameter.maximum().to_bits()
        );
    }
}

#[test]
fn device_layouts_match_the_v1_contract() {
    let source = ToneSource::new(440.0, 0.25).unwrap();
    let instrument = PulseInstrument::new(Waveform::Saw, 0.2).unwrap();
    let gain = Gain::new(1.0).unwrap();
    let saturator = Saturator::new(1.0, 1.0).unwrap();

    assert_eq!(source.io().class, DeviceClass::Source);
    assert_eq!(source.io().audio_inputs, 0);
    assert_eq!(source.io().audio_outputs, 2);
    assert!(!source.io().accepts_notes);

    assert_eq!(instrument.io().class, DeviceClass::Instrument);
    assert_eq!(instrument.io().audio_inputs, 0);
    assert_eq!(instrument.io().audio_outputs, 2);
    assert!(instrument.io().accepts_notes);

    let filament = default_filament();
    let gloam = default_gloam();

    assert_eq!(filament.io().class, DeviceClass::Instrument);
    assert_eq!(filament.io().audio_inputs, 0);
    assert_eq!(filament.io().audio_outputs, 2);
    assert!(filament.io().accepts_notes);

    for io in [gain.io(), saturator.io(), gloam.io()] {
        assert_eq!(io.class, DeviceClass::Effect);
        assert_eq!(io.audio_inputs, 2);
        assert_eq!(io.audio_outputs, 2);
        assert!(!io.accepts_notes);
    }
}

#[test]
fn tone_source_is_deterministic_and_stereo() {
    let context = ProcessContext::new(48_000.0, 64, &[]).unwrap();
    let mut first = ToneSource::new(440.0, 0.25).unwrap();
    let mut second = ToneSource::new(440.0, 0.25).unwrap();
    let (mut left_a, mut right_a) = output(64);
    let (mut left_b, mut right_b) = output(64);

    first
        .process(&context, &[], &mut [&mut left_a, &mut right_a])
        .unwrap();
    second
        .process(&context, &[], &mut [&mut left_b, &mut right_b])
        .unwrap();

    assert_eq!(left_a, left_b);
    assert_eq!(right_a, right_b);
    assert_eq!(left_a, right_a);
    assert!(left_a.iter().any(|sample| *sample != 0.0));
}

#[test]
fn gain_scales_stereo_input() {
    let context = ProcessContext::new(48_000.0, 4, &[]).unwrap();
    let mut gain = Gain::new(0.5).unwrap();
    let left = [1.0, -1.0, 0.5, -0.5];
    let right = [0.25, -0.25, 0.0, 1.0];
    let (mut out_left, mut out_right) = output(4);

    gain.process(
        &context,
        &[&left, &right],
        &mut [&mut out_left, &mut out_right],
    )
    .unwrap();

    assert_eq!(out_left, vec![0.5, -0.5, 0.25, -0.25]);
    assert_eq!(out_right, vec![0.125, -0.125, 0.0, 0.5]);
}

#[test]
fn saturator_contains_non_finite_input_and_bounds_output() {
    let context = ProcessContext::new(48_000.0, 5, &[]).unwrap();
    let mut saturator = Saturator::new(12.0, 1.0).unwrap();
    let left = [0.0, 0.5, -0.5, f32::NAN, f32::INFINITY];
    let right = left;
    let (mut out_left, mut out_right) = output(5);

    saturator
        .process(
            &context,
            &[&left, &right],
            &mut [&mut out_left, &mut out_right],
        )
        .unwrap();

    for sample in out_left.iter().chain(out_right.iter()) {
        assert!(sample.is_finite());
        assert!((-1.0..=1.0).contains(sample));
    }
}

#[test]
fn instrument_obeys_note_event_offsets() {
    let events = [
        NoteEvent {
            frame_offset: 4,
            sequence: 0,
            kind: NoteEventKind::On {
                id: 1,
                channel: 0,
                note: 69,
                velocity: 1.0,
            },
        },
        NoteEvent {
            frame_offset: 12,
            sequence: 1,
            kind: NoteEventKind::Off {
                id: 1,
                channel: 0,
                note: 69,
                velocity: 0.0,
            },
        },
    ];
    let context = ProcessContext::new(48_000.0, 16, &events).unwrap();
    let mut instrument = PulseInstrument::new(Waveform::Sine, 0.5).unwrap();
    let (mut left, mut right) = output(16);

    instrument
        .process(&context, &[], &mut [&mut left, &mut right])
        .unwrap();

    assert!(left[..=4].iter().all(|sample| *sample == 0.0));
    assert!(left[5..12].iter().any(|sample| *sample != 0.0));
    assert!(left[12..].iter().all(|sample| *sample == 0.0));
    assert_eq!(left, right);
}

#[test]
fn context_rejects_out_of_order_and_out_of_block_events() {
    let out_of_order = [
        NoteEvent {
            frame_offset: 4,
            sequence: 2,
            kind: NoteEventKind::Off {
                id: 1,
                channel: 0,
                note: 60,
                velocity: 0.0,
            },
        },
        NoteEvent {
            frame_offset: 4,
            sequence: 1,
            kind: NoteEventKind::Off {
                id: 2,
                channel: 0,
                note: 60,
                velocity: 0.0,
            },
        },
    ];
    assert!(ProcessContext::new(48_000.0, 16, &out_of_order).is_err());

    let outside = [NoteEvent {
        frame_offset: 16,
        sequence: 0,
        kind: NoteEventKind::Off {
            id: 1,
            channel: 0,
            note: 60,
            velocity: 0.0,
        },
    }];
    assert!(ProcessContext::new(48_000.0, 16, &outside).is_err());
}

#[test]
fn same_frame_note_off_must_precede_note_on() {
    let events = [
        NoteEvent {
            frame_offset: 4,
            sequence: 0,
            kind: NoteEventKind::On {
                id: 2,
                channel: 0,
                note: 60,
                velocity: 1.0,
            },
        },
        NoteEvent {
            frame_offset: 4,
            sequence: 1,
            kind: NoteEventKind::Off {
                id: 1,
                channel: 0,
                note: 60,
                velocity: 0.0,
            },
        },
    ];
    assert!(ProcessContext::new(48_000.0, 16, &events).is_err());
}

// R4-2 test 1 — every shipping device answers its own descriptor keys and refuses others
#[test]
fn every_device_accepts_exactly_its_own_parameter_keys() {
    let stranger = DeviceParameterKey::new("not_a_parameter").unwrap();

    let mut gain = Gain::new(1.0).unwrap();
    assert_eq!(gain.set_parameter(GAIN_PARAMETERS[0].key, 0.5), Ok(()));
    assert_eq!(
        gain.set_parameter(stranger, 0.5),
        Err(ParameterError::UnknownKey(stranger))
    );

    let mut saturator = Saturator::new(1.0, 1.0).unwrap();
    assert_eq!(
        saturator.set_parameter(SATURATOR_PARAMETERS[0].key, 4.0),
        Ok(())
    );
    assert_eq!(
        saturator.set_parameter(SATURATOR_PARAMETERS[1].key, 0.5),
        Ok(())
    );
    assert_eq!(
        saturator.set_parameter(stranger, 0.5),
        Err(ParameterError::UnknownKey(stranger))
    );

    let mut pulse = PulseInstrument::new(Waveform::Saw, 0.3).unwrap();
    assert_eq!(pulse.set_parameter(PULSE_PARAMETERS[0].key, 0.6), Ok(()));
    assert_eq!(
        pulse.set_parameter(stranger, 0.6),
        Err(ParameterError::UnknownKey(stranger))
    );

    let mut tone = ToneSource::new(440.0, 0.2).unwrap();
    assert_eq!(tone.set_parameter(TONE_PARAMETERS[0].key, 880.0), Ok(()));
    assert_eq!(tone.set_parameter(TONE_PARAMETERS[1].key, 0.4), Ok(()));
    assert_eq!(
        tone.set_parameter(stranger, 0.4),
        Err(ParameterError::UnknownKey(stranger))
    );
}

// R4-2 test 2 — the setter clamps against the same descriptor the constructor uses, so a value
// out of range is contained rather than refused or propagated
#[test]
fn a_setter_clamps_against_its_own_descriptor() {
    let mut gain = Gain::new(1.0).unwrap();
    gain.set_parameter(GAIN_PARAMETERS[0].key, 999.0).unwrap();

    let (mut left, mut right) = output(4);
    let context = ProcessContext::new(48_000.0, 4, &[]).unwrap();
    let ones = vec![1.0_f32; 4];
    let inputs: [&[f32]; 2] = [&ones, &ones];
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    gain.process(&context, &inputs, &mut outputs).unwrap();

    // Gain ramps a change across one block, so the clamp is what the block ARRIVES at rather
    // than what every sample already is. The three assertions together still fail if the setter
    // passed 999.0 through, refused it, or clamped to the wrong bound
    let expected = GAIN_PARAMETERS[0].maximum();
    assert!(
        outputs[0].iter().all(|sample| *sample <= expected),
        "no sample may exceed the descriptor maximum; an unclamped 999.0 would"
    );
    assert_eq!(
        outputs[0][outputs[0].len() - 1],
        expected,
        "the ramp must arrive exactly at the clamped maximum"
    );
    assert!(
        outputs[0][0] > 1.0,
        "a refused setter would leave the block at the constructed 1.0"
    );
    let _ = (&mut left, &mut right);
}

// D-R3 — the accepted device contract says "Gain already smooths"; it did not until 2026-08-28.
// R4-2 left it unimplemented for two stated reasons and this pins that both are answered: the
// ramp spans exactly one block, so it needs no numeric bound of its own, and a render with no
// parameter change is bit-identical to the unsmoothed device
#[test]
fn a_gain_change_ramps_across_one_block_instead_of_stepping() {
    let mut gain = Gain::new(1.0).unwrap();
    let context = ProcessContext::new(48_000.0, 8, &[]).unwrap();
    let ones = vec![1.0_f32; 8];
    let inputs: [&[f32]; 2] = [&ones, &ones];

    // No change: every sample is the constructed value, exactly as before smoothing existed
    let (mut left, mut right) = output(8);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    gain.process(&context, &inputs, &mut outputs).unwrap();
    assert!(
        outputs[0].iter().all(|sample| *sample == 1.0),
        "an unchanged gain must not ramp"
    );

    // A change: the block ramps and lands on the target at its final sample
    gain.set_parameter(GAIN_PARAMETERS[0].key, 0.5).unwrap();
    let (mut left, mut right) = output(8);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    gain.process(&context, &inputs, &mut outputs).unwrap();
    assert!(
        outputs[0][0] > 0.5 && outputs[0][0] < 1.0,
        "the first sample must be between the old value and the new one, not either"
    );
    assert_eq!(
        outputs[0][7], 0.5,
        "the ramp must land exactly on the target at the block's last sample"
    );
    assert!(
        outputs[0].windows(2).all(|w| w[1] < w[0]),
        "the ramp must be monotonic"
    );

    // The block after arrival is flat at the target: the ramp settles rather than drifting
    let (mut left, mut right) = output(8);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    gain.process(&context, &inputs, &mut outputs).unwrap();
    assert!(
        outputs[0].iter().all(|sample| *sample == 0.5),
        "a settled gain must be flat"
    );
    let _ = (&mut left, &mut right);
}

// R4-2 test 3 — a frequency change is phase-continuous; resetting the phase would click
#[test]
fn changing_tone_frequency_does_not_reset_phase() {
    let mut tone = ToneSource::new(440.0, 0.2).unwrap();
    let context = ProcessContext::new(48_000.0, 8, &[]).unwrap();

    let (mut left, mut right) = output(8);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    tone.process(&context, &[], &mut outputs).unwrap();
    let last_before = outputs[0][7];

    tone.set_parameter(TONE_PARAMETERS[0].key, 441.0).unwrap();
    let (mut next_left, mut next_right) = output(8);
    let mut next = [next_left.as_mut_slice(), next_right.as_mut_slice()];
    tone.process(&context, &[], &mut next).unwrap();

    // One sample at 440 Hz / 48 kHz advances the phase by ~0.0092 of a cycle, so a continuous
    // waveform cannot jump by more than a small fraction of its amplitude across the boundary
    let step = (next[0][0] - last_before).abs();
    assert!(
        step < 0.05,
        "a frequency change must not restart the phase; step was {step}"
    );
    let _ = (&mut next_left, &mut next_right);
}

// Descriptor slots for the two R4-6 devices; mirrors of the private slots in each module
const FILAMENT_LEAN: usize = 0;
const FILAMENT_RISE_MS: usize = 1;
const FILAMENT_FALL_MS: usize = 2;
const FILAMENT_LEVEL: usize = 3;
const GLOAM_DAMP_HZ: usize = 0;
const GLOAM_DEPTH: usize = 1;
const GLOAM_TRACK_MS: usize = 2;

// One rate every assertion below is written against; not a bound, a fixture
const FIXTURE_SAMPLE_RATE: f64 = 48_000.0;
// The render quantum R4-1 requests, reused so block-boundary behavior is exercised as shipped
const FIXTURE_QUANTUM_FRAMES: usize = 256;
// Milliseconds-to-seconds conversion, matching the devices
const SECONDS_PER_MILLISECOND: f64 = 0.001;
// DEV-013: how long the silence search waits for Gloam's state to flush to exact zero
const SILENCE_SETTLE_MAX_QUANTA: usize = 64;

// Build one device at its declared defaults
fn default_filament() -> Filament {
    Filament::new(
        FILAMENT_PARAMETERS[FILAMENT_LEAN].default(),
        FILAMENT_PARAMETERS[FILAMENT_RISE_MS].default(),
        FILAMENT_PARAMETERS[FILAMENT_FALL_MS].default(),
        FILAMENT_PARAMETERS[FILAMENT_LEVEL].default(),
    )
    .unwrap()
}

// Build one effect at its declared defaults
fn default_gloam() -> Gloam {
    Gloam::new(
        GLOAM_PARAMETERS[GLOAM_DAMP_HZ].default(),
        GLOAM_PARAMETERS[GLOAM_DEPTH].default(),
        GLOAM_PARAMETERS[GLOAM_TRACK_MS].default(),
    )
    .unwrap()
}

// Build one note-on event
fn note_on(frame_offset: usize, sequence: u64, id: u32, note: u8, velocity: f32) -> NoteEvent {
    NoteEvent {
        frame_offset,
        sequence,
        kind: NoteEventKind::On {
            id,
            channel: 0,
            note,
            velocity,
        },
    }
}

// Build one note-off event
fn note_off(frame_offset: usize, sequence: u64, id: u32, note: u8) -> NoteEvent {
    NoteEvent {
        frame_offset,
        sequence,
        kind: NoteEventKind::Off {
            id,
            channel: 0,
            note,
            velocity: 0.0,
        },
    }
}

// Render one instrument block and return both channels
fn render_instrument(
    device: &mut Filament,
    frames: usize,
    events: &[NoteEvent],
) -> (Vec<f32>, Vec<f32>) {
    let context = ProcessContext::new(FIXTURE_SAMPLE_RATE, frames, events).unwrap();
    let (mut left, mut right) = output(frames);
    device
        .process(&context, &[], &mut [&mut left, &mut right])
        .unwrap();
    (left, right)
}

// Render one effect block over identical stereo input and return both channels
fn render_effect(device: &mut Gloam, input: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let context = ProcessContext::new(FIXTURE_SAMPLE_RATE, input.len(), &[]).unwrap();
    let (mut left, mut right) = output(input.len());
    device
        .process(&context, &[input, input], &mut [&mut left, &mut right])
        .unwrap();
    (left, right)
}

// Recompute the contour increment independently of the device under test
fn reference_contour_step(time_ms: f32, sample_rate: f64) -> f32 {
    (1.0 / (f64::from(time_ms) * SECONDS_PER_MILLISECOND * sample_rate).max(1.0)) as f32
}

// Recompute the equal-tempered note frequency independently of the device under test
fn reference_note_hz(note: u8) -> f64 {
    440.0 * 2.0_f64.powf((f64::from(note) - 69.0) / 12.0)
}

#[test]
fn filament_is_an_exact_sine_at_the_default_lean() {
    // lean = 0.5 makes the phase map the identity in exact binary arithmetic, so the device
    // must agree with a plain sine bit for bit, not within a tolerance
    let rise_ms = FILAMENT_PARAMETERS[FILAMENT_RISE_MS].minimum();
    let mut device = Filament::new(
        FILAMENT_PARAMETERS[FILAMENT_LEAN].default(),
        rise_ms,
        FILAMENT_PARAMETERS[FILAMENT_FALL_MS].default(),
        FILAMENT_PARAMETERS[FILAMENT_LEVEL].maximum(),
    )
    .unwrap();
    let frames = 4_096;
    let events = [note_on(0, 0, 1, 69, 1.0)];
    let (left, right) = render_instrument(&mut device, frames, &events);

    let step = reference_contour_step(rise_ms, FIXTURE_SAMPLE_RATE);
    let increment = reference_note_hz(69) / FIXTURE_SAMPLE_RATE;
    let mut phase = 0.0_f64;
    let mut contour = 0.0_f32;
    for (frame, sample) in left.iter().enumerate() {
        contour = (contour + step).min(1.0);
        let expected = (phase * TAU).sin() as f32 * contour;
        assert_eq!(sample.to_bits(), expected.to_bits(), "frame {frame}");
        phase = (phase + increment).fract();
    }
    assert_eq!(left, right);
    assert!(left.iter().any(|sample| *sample != 0.0));
}

#[test]
fn filament_stays_finite_at_both_lean_endpoints() {
    // Both endpoints put one branch of the phase map on a zero denominator unless the strict
    // comparison and the else are written exactly as the contract states
    let level = FILAMENT_PARAMETERS[FILAMENT_LEVEL].maximum();
    for lean in [
        FILAMENT_PARAMETERS[FILAMENT_LEAN].minimum(),
        FILAMENT_PARAMETERS[FILAMENT_LEAN].maximum(),
    ] {
        let mut device = Filament::new(
            lean,
            FILAMENT_PARAMETERS[FILAMENT_RISE_MS].minimum(),
            FILAMENT_PARAMETERS[FILAMENT_FALL_MS].default(),
            level,
        )
        .unwrap();
        let (left, right) = render_instrument(&mut device, 4_096, &[note_on(0, 0, 1, 69, 1.0)]);

        for sample in left.iter().chain(right.iter()) {
            assert!(sample.is_finite(), "lean {lean}");
            assert!(sample.abs() <= level, "lean {lean}");
        }
        assert!(left.iter().any(|sample| *sample != 0.0), "lean {lean}");
    }
}

#[test]
fn filament_attacks_from_a_zero_crossing() {
    // The only nonzero value the phase map can produce at phase 0 is sin(PI), which is f64's
    // representation error for PI and nothing else; that exact value is the bound
    let zero_crossing_bound = PI.sin().abs() as f32;
    for lean in [
        FILAMENT_PARAMETERS[FILAMENT_LEAN].minimum(),
        FILAMENT_PARAMETERS[FILAMENT_LEAN].default(),
        FILAMENT_PARAMETERS[FILAMENT_LEAN].maximum(),
    ] {
        let mut device = Filament::new(
            lean,
            FILAMENT_PARAMETERS[FILAMENT_RISE_MS].minimum(),
            FILAMENT_PARAMETERS[FILAMENT_FALL_MS].default(),
            FILAMENT_PARAMETERS[FILAMENT_LEVEL].maximum(),
        )
        .unwrap();
        let (left, _) = render_instrument(&mut device, 64, &[note_on(0, 0, 1, 69, 1.0)]);

        assert!(left[0].abs() <= zero_crossing_bound, "lean {lean}");
    }
}

#[test]
fn filament_release_reaches_exact_silence() {
    // The shortest fall the descriptor allows must still land on exact positive zero, not on an
    // exponential residue that would generate denormals forever
    let fall_ms = FILAMENT_PARAMETERS[FILAMENT_FALL_MS].minimum();
    let mut device = Filament::new(
        FILAMENT_PARAMETERS[FILAMENT_LEAN].default(),
        FILAMENT_PARAMETERS[FILAMENT_RISE_MS].minimum(),
        fall_ms,
        FILAMENT_PARAMETERS[FILAMENT_LEVEL].maximum(),
    )
    .unwrap();
    let release_frame = 64;
    let ramp_frames =
        (f64::from(fall_ms) * SECONDS_PER_MILLISECOND * FIXTURE_SAMPLE_RATE).ceil() as usize + 2;
    let events = [note_on(0, 0, 1, 69, 1.0), note_off(release_frame, 1, 1, 69)];
    let (left, right) = render_instrument(&mut device, FIXTURE_QUANTUM_FRAMES, &events);

    assert!(left[..release_frame].iter().any(|sample| *sample != 0.0));
    for (frame, sample) in left.iter().enumerate().skip(release_frame + ramp_frames) {
        assert_eq!(sample.to_bits(), 0.0_f32.to_bits(), "frame {frame}");
        assert!(sample.is_sign_positive(), "frame {frame}");
    }
    assert_eq!(left, right);
}

#[test]
fn filament_release_is_a_ramp_rather_than_a_step() {
    // A release that zeroed the voice instead of the contour would show as one sample of full
    // amplitude followed by silence; the ramp is what the shortest-fall rationale buys
    let mut device = Filament::new(
        FILAMENT_PARAMETERS[FILAMENT_LEAN].default(),
        FILAMENT_PARAMETERS[FILAMENT_RISE_MS].minimum(),
        FILAMENT_PARAMETERS[FILAMENT_FALL_MS].default(),
        FILAMENT_PARAMETERS[FILAMENT_LEVEL].maximum(),
    )
    .unwrap();
    let release_frame = 64;
    let events = [note_on(0, 0, 1, 69, 1.0), note_off(release_frame, 1, 1, 69)];
    let (left, _) = render_instrument(&mut device, FIXTURE_QUANTUM_FRAMES, &events);

    let tail_peak = left[release_frame..]
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    assert!(tail_peak > 0.0);
    assert!(left[release_frame..].iter().any(|sample| *sample != 0.0));
}

#[test]
fn gloam_at_zero_depth_is_a_plain_one_pole() {
    // At depth 0 the opening is exactly +0, so the per-sample coefficient is the stored f32 with
    // no arithmetic standing between them; the reference is the same operations in the same order
    let damp_hz = GLOAM_PARAMETERS[GLOAM_DAMP_HZ].default();
    let mut device = Gloam::new(
        damp_hz,
        GLOAM_PARAMETERS[GLOAM_DEPTH].minimum(),
        GLOAM_PARAMETERS[GLOAM_TRACK_MS].default(),
    )
    .unwrap();
    let input = vec![1.0_f32; FIXTURE_QUANTUM_FRAMES];
    let (left, right) = render_effect(&mut device, &input);

    let coefficient = (1.0 - (-TAU * f64::from(damp_hz) / FIXTURE_SAMPLE_RATE).exp()) as f32;
    let mut expected = 0.0_f32;
    for frame in 0..FIXTURE_QUANTUM_FRAMES {
        expected += coefficient * (input[frame] - expected);
        assert_eq!(left[frame].to_bits(), expected.to_bits(), "frame {frame}");
    }
    assert_eq!(left, right);
}

#[test]
fn gloam_contains_non_finite_input_without_latching() {
    // The poison block is entirely non-finite: not one sample in it is finite. That is what makes
    // the state assertion below provable, because every contained sample drives the recursion
    // from zero by zero and leaves both state scalars exactly where a fresh device starts
    let poison: Vec<f32> = (0..FIXTURE_QUANTUM_FRAMES)
        .map(|frame| match frame % 3 {
            0 => f32::NAN,
            1 => f32::INFINITY,
            _ => f32::NEG_INFINITY,
        })
        .collect();
    assert!(poison.iter().all(|sample| !sample.is_finite()));
    let clean: Vec<f32> = (0..FIXTURE_QUANTUM_FRAMES)
        .map(|frame| ((frame as f32) * 0.01).sin())
        .collect();

    let mut poisoned = default_gloam();
    let mut fresh = default_gloam();
    let (poison_left, poison_right) = render_effect(&mut poisoned, &poison);
    for sample in poison_left.iter().chain(poison_right.iter()) {
        assert_eq!(sample.to_bits(), 0.0_f32.to_bits());
    }

    let (recovered_left, _) = render_effect(&mut poisoned, &clean);
    let (reference_left, _) = render_effect(&mut fresh, &clean);
    for sample in recovered_left.iter().chain(reference_left.iter()) {
        assert!(sample.is_finite());
    }
    assert_eq!(recovered_left, reference_left);
}

#[test]
fn gloam_state_reaches_exact_zero_after_silence() {
    // DEV-013 bounds the search at 64 quanta; the default coefficient needs about four
    let mut device = default_gloam();
    let driven = vec![1.0_f32; FIXTURE_QUANTUM_FRAMES];
    let silence = vec![0.0_f32; FIXTURE_QUANTUM_FRAMES];
    render_effect(&mut device, &driven);

    let mut settled = None;
    for quantum in 0..SILENCE_SETTLE_MAX_QUANTA {
        let (left, right) = render_effect(&mut device, &silence);
        if left
            .iter()
            .chain(right.iter())
            .all(|sample| sample.to_bits() == 0.0_f32.to_bits())
        {
            settled = Some(quantum);
            break;
        }
    }
    assert!(settled.is_some(), "state never reached exact zero");
}

#[test]
fn gloam_never_exceeds_its_input_peak() {
    // A one-pole whose coefficient stays inside [0, 1] is a convex combination of past inputs,
    // so it cannot ring, cannot self-oscillate, and cannot raise the peak
    let square: Vec<f32> = (0..FIXTURE_QUANTUM_FRAMES)
        .map(|frame| if frame % 32 < 16 { 1.0 } else { -1.0 })
        .collect();
    let sine: Vec<f32> = (0..FIXTURE_QUANTUM_FRAMES)
        .map(|frame| ((frame as f64) * TAU / 64.0).sin() as f32)
        .collect();
    let impulse: Vec<f32> = (0..FIXTURE_QUANTUM_FRAMES)
        .map(|frame| if frame == 0 { 1.0 } else { 0.0 })
        .collect();

    for input in [&square, &sine, &impulse] {
        let input_peak = input.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));
        for depth in [
            GLOAM_PARAMETERS[GLOAM_DEPTH].minimum(),
            GLOAM_PARAMETERS[GLOAM_DEPTH].default(),
            GLOAM_PARAMETERS[GLOAM_DEPTH].maximum(),
        ] {
            for damp_hz in [
                GLOAM_PARAMETERS[GLOAM_DAMP_HZ].minimum(),
                GLOAM_PARAMETERS[GLOAM_DAMP_HZ].maximum(),
            ] {
                let mut device =
                    Gloam::new(damp_hz, depth, GLOAM_PARAMETERS[GLOAM_TRACK_MS].default()).unwrap();
                let (left, right) = render_effect(&mut device, input);
                let peak = left
                    .iter()
                    .chain(right.iter())
                    .fold(0.0_f32, |peak, s| peak.max(s.abs()));
                assert!(
                    peak <= input_peak,
                    "depth {depth} damp {damp_hz}: {peak} > {input_peak}"
                );
                assert!(left.iter().all(|sample| sample.is_finite()));
            }
        }
    }
}

// Enumerate the values one descriptor must refuse: just outside each bound, and both non-finites
fn out_of_range(parameter: spectre_dsp::DspParameter) -> [f32; 5] {
    [
        next_down(parameter.minimum()),
        next_up(parameter.maximum()),
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ]
}

#[test]
fn new_devices_reject_out_of_range_construction() {
    let filament_defaults = [
        FILAMENT_PARAMETERS[FILAMENT_LEAN].default(),
        FILAMENT_PARAMETERS[FILAMENT_RISE_MS].default(),
        FILAMENT_PARAMETERS[FILAMENT_FALL_MS].default(),
        FILAMENT_PARAMETERS[FILAMENT_LEVEL].default(),
    ];
    assert!(Filament::new(
        filament_defaults[0],
        filament_defaults[1],
        filament_defaults[2],
        filament_defaults[3]
    )
    .is_ok());
    for slot in 0..FILAMENT_PARAMETERS.len() {
        for bad in out_of_range(FILAMENT_PARAMETERS[slot]) {
            let mut values = filament_defaults;
            values[slot] = bad;
            assert!(
                Filament::new(values[0], values[1], values[2], values[3]).is_err(),
                "{} accepted {bad}",
                FILAMENT_PARAMETERS[slot].key.as_str()
            );
        }
    }

    let gloam_defaults = [
        GLOAM_PARAMETERS[GLOAM_DAMP_HZ].default(),
        GLOAM_PARAMETERS[GLOAM_DEPTH].default(),
        GLOAM_PARAMETERS[GLOAM_TRACK_MS].default(),
    ];
    assert!(Gloam::new(gloam_defaults[0], gloam_defaults[1], gloam_defaults[2]).is_ok());
    for slot in 0..GLOAM_PARAMETERS.len() {
        for bad in out_of_range(GLOAM_PARAMETERS[slot]) {
            let mut values = gloam_defaults;
            values[slot] = bad;
            assert!(
                Gloam::new(values[0], values[1], values[2]).is_err(),
                "{} accepted {bad}",
                GLOAM_PARAMETERS[slot].key.as_str()
            );
        }
    }
}

#[test]
fn new_device_setters_clamp_and_refuse_unknown_keys() {
    let lean = FILAMENT_PARAMETERS[FILAMENT_LEAN];
    let mut filament = default_filament();
    assert_eq!(filament.set_parameter(lean.key, lean.minimum()), Ok(()));
    assert_eq!(filament.parameter_value(lean.key), Some(lean.minimum()));
    assert_eq!(
        filament.set_parameter(lean.key, next_up(lean.maximum())),
        Ok(())
    );
    assert_eq!(filament.parameter_value(lean.key), Some(lean.maximum()));
    assert_eq!(filament.set_parameter(lean.key, f32::NAN), Ok(()));
    assert_eq!(filament.parameter_value(lean.key), Some(lean.default()));

    let foreign = GLOAM_PARAMETERS[GLOAM_DAMP_HZ].key;
    assert_eq!(
        filament.set_parameter(foreign, 1_000.0),
        Err(ParameterError::UnknownKey(foreign))
    );
    assert_eq!(filament.parameter_value(foreign), None);
    for parameter in FILAMENT_PARAMETERS {
        assert!(filament.parameter_value(parameter.key).is_some());
    }

    let depth = GLOAM_PARAMETERS[GLOAM_DEPTH];
    let mut gloam = default_gloam();
    assert_eq!(gloam.set_parameter(depth.key, depth.maximum()), Ok(()));
    assert_eq!(gloam.parameter_value(depth.key), Some(depth.maximum()));
    assert_eq!(
        gloam.set_parameter(depth.key, next_down(depth.minimum())),
        Ok(())
    );
    assert_eq!(gloam.parameter_value(depth.key), Some(depth.minimum()));
    assert_eq!(gloam.set_parameter(depth.key, f32::INFINITY), Ok(()));
    assert_eq!(gloam.parameter_value(depth.key), Some(depth.default()));

    let foreign = FILAMENT_PARAMETERS[FILAMENT_LEAN].key;
    assert_eq!(
        gloam.set_parameter(foreign, 0.25),
        Err(ParameterError::UnknownKey(foreign))
    );
    assert_eq!(gloam.parameter_value(foreign), None);
    for parameter in GLOAM_PARAMETERS {
        assert!(gloam.parameter_value(parameter.key).is_some());
    }
}

#[test]
fn new_device_setters_reach_the_rendered_signal() {
    // A setter that clamped correctly but never reached the DSP would be a fake surface
    let level = FILAMENT_PARAMETERS[FILAMENT_LEVEL];
    let mut device = default_filament();
    let events = [note_on(0, 0, 1, 69, 1.0)];
    let (loud, _) = render_instrument(&mut device, FIXTURE_QUANTUM_FRAMES, &events);
    let loud_peak = loud.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()));

    let mut device = default_filament();
    device.set_parameter(level.key, level.minimum()).unwrap();
    let (quiet, _) = render_instrument(&mut device, FIXTURE_QUANTUM_FRAMES, &events);

    assert!(loud_peak > 0.0);
    assert!(quiet.iter().all(|sample| *sample == 0.0));
}

#[test]
fn filament_contour_minimum_is_at_least_eight_sample_periods() {
    // Pins the descriptor, not the DSP: it fails if someone lowers the minimum and silently
    // breaks the no-step guarantee. MIN_ENGINE_SAMPLE_RATE_HZ mirrors spectre_audio's
    // MIN_SAMPLE_RATE, which spectre-dsp does not depend on and therefore cannot import
    const MIN_ENGINE_SAMPLE_RATE_HZ: f32 = 8_000.0;
    const MIN_CONTOUR_SAMPLE_PERIODS: f32 = 8.0;

    for slot in [FILAMENT_RISE_MS, FILAMENT_FALL_MS] {
        let minimum = FILAMENT_PARAMETERS[slot].minimum();
        assert!(minimum >= 1.0);
        assert!(
            minimum * 0.001 * MIN_ENGINE_SAMPLE_RATE_HZ >= MIN_CONTOUR_SAMPLE_PERIODS,
            "{}",
            FILAMENT_PARAMETERS[slot].key.as_str()
        );
    }
    let track_minimum = GLOAM_PARAMETERS[GLOAM_TRACK_MS].minimum();
    assert!(track_minimum * 0.001 * MIN_ENGINE_SAMPLE_RATE_HZ >= MIN_CONTOUR_SAMPLE_PERIODS);
}

// Render Filament into Gloam over one quantum and return the effect's left channel
fn render_voice_chain(
    instrument: &mut Filament,
    effect: &mut Gloam,
    frames: usize,
    events: &[NoteEvent],
) -> Vec<f32> {
    let (voice_left, voice_right) = render_instrument(instrument, frames, events);
    let context = ProcessContext::new(FIXTURE_SAMPLE_RATE, frames, &[]).unwrap();
    let (mut left, mut right) = output(frames);
    effect
        .process(
            &context,
            &[&voice_left, &voice_right],
            &mut [&mut left, &mut right],
        )
        .unwrap();
    left
}

#[test]
fn voice_chain_renders_deterministically_and_returns_to_exact_silence() {
    // The slice's success signal, as far as spectre-dsp can prove it on its own: identical
    // inputs render identically, the chain is audible, and it settles to bit-exact positive zero
    let events = [note_on(0, 0, 1, 69, 1.0), note_off(64, 1, 1, 69)];
    let mut first = (default_filament(), default_gloam());
    let mut second = (default_filament(), default_gloam());
    let left_a = render_voice_chain(&mut first.0, &mut first.1, FIXTURE_QUANTUM_FRAMES, &events);
    let left_b = render_voice_chain(
        &mut second.0,
        &mut second.1,
        FIXTURE_QUANTUM_FRAMES,
        &events,
    );

    assert_eq!(left_a, left_b);
    assert!(left_a.iter().all(|sample| sample.is_finite()));
    assert!(left_a.iter().any(|sample| *sample != 0.0));

    let mut settled = None;
    for quantum in 0..SILENCE_SETTLE_MAX_QUANTA {
        let tail = render_voice_chain(&mut first.0, &mut first.1, FIXTURE_QUANTUM_FRAMES, &[]);
        if tail
            .iter()
            .all(|sample| sample.to_bits() == 0.0_f32.to_bits())
        {
            settled = Some(quantum);
            break;
        }
    }
    assert!(settled.is_some(), "chain never reached exact silence");
}

#[test]
fn voice_chain_output_stays_inside_unity() {
    // Filament is bounded by level and Gloam cannot raise a peak, so the pair is bounded by 1
    let level = FILAMENT_PARAMETERS[FILAMENT_LEVEL].maximum();
    let mut instrument = Filament::new(
        FILAMENT_PARAMETERS[FILAMENT_LEAN].minimum(),
        FILAMENT_PARAMETERS[FILAMENT_RISE_MS].minimum(),
        FILAMENT_PARAMETERS[FILAMENT_FALL_MS].minimum(),
        level,
    )
    .unwrap();
    let mut effect = Gloam::new(
        GLOAM_PARAMETERS[GLOAM_DAMP_HZ].maximum(),
        GLOAM_PARAMETERS[GLOAM_DEPTH].maximum(),
        GLOAM_PARAMETERS[GLOAM_TRACK_MS].minimum(),
    )
    .unwrap();
    let left = render_voice_chain(
        &mut instrument,
        &mut effect,
        FIXTURE_QUANTUM_FRAMES,
        &[note_on(0, 0, 1, 127, 1.0)],
    );

    for sample in left {
        assert!(sample.is_finite());
        assert!(sample.abs() <= level);
    }
}
