// Author: Jeff
// Date: 2026-07-12
// Description: Contract tests for native Spectre sources, instruments, and effects
// Notes: Tests pin process behavior before device implementation

use spectre_dsp::{
    AudioProcessor, DeviceClass, DeviceParameterKey, Gain, NoteEvent, NoteEventKind,
    ParameterError, ProcessContext, PulseInstrument, Saturator, ToneSource, Waveform,
    GAIN_PARAMETERS, NORMALIZED_ROUND_TRIP_MAX_ULPS, PULSE_PARAMETERS, SATURATOR_PARAMETERS,
    TONE_PARAMETERS,
};

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

    for io in [gain.io(), saturator.io()] {
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

    let expected = GAIN_PARAMETERS[0].maximum();
    assert!(
        outputs[0].iter().all(|sample| *sample == expected),
        "an out-of-range value must clamp to the descriptor maximum, not pass through"
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
