// Author: Jeff
// Date: 2026-08-24
// Description: Contract tests for the stereo summing bus
// Notes: In its own file rather than appended to devices.rs, so a concurrent slice editing that
//   file cannot collide with this one

use spectre_dsp::{AudioProcessor, DeviceClass, DeviceIo, ProcessContext, SumBus, MAX_SUM_BUSES};

const FRAMES: usize = 8;

// 10
#[test]
fn sum_bus_adds_its_buses_and_contains_non_finite_input() {
    let mut bus = SumBus::new(3).unwrap();
    let nan = vec![f32::NAN; FRAMES];
    let infinity = vec![f32::INFINITY; FRAMES];
    let finite = vec![0.25_f32; FRAMES];
    let inputs: [&[f32]; 6] = [&nan, &nan, &infinity, &infinity, &finite, &finite];

    let (mut left, mut right) = (vec![9.0_f32; FRAMES], vec![9.0_f32; FRAMES]);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    let context = ProcessContext::new(48_000.0, FRAMES, &[]).unwrap();
    bus.process(&context, &inputs, &mut outputs).unwrap();

    for channel in &outputs {
        for sample in channel.iter() {
            // The poisoned buses contribute exactly 0.0, so the finite bus survives untouched
            assert_eq!(*sample, 0.25);
            assert!(sample.is_finite());
        }
    }

    // Zero buses is legal and renders exact silence, not an error
    let mut empty = SumBus::new(0).unwrap();
    let (mut left, mut right) = (vec![9.0_f32; FRAMES], vec![9.0_f32; FRAMES]);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    empty.process(&context, &[], &mut outputs).unwrap();
    for channel in &outputs {
        for sample in channel.iter() {
            assert_eq!(sample.to_bits(), 0.0_f32.to_bits());
        }
    }

    assert!(SumBus::new(MAX_SUM_BUSES + 1).is_err());
    assert!(SumBus::new(MAX_SUM_BUSES).is_ok());
}

// 11
#[test]
fn sum_bus_layout_matches_the_declared_bus_count() {
    for buses in [0, 1, 2, MAX_SUM_BUSES] {
        let bus = SumBus::new(buses).unwrap();
        assert_eq!(
            bus.io(),
            DeviceIo {
                class: DeviceClass::Effect,
                audio_inputs: buses * 2,
                audio_outputs: 2,
                accepts_notes: false,
            }
        );
        assert_eq!(bus.buses(), buses);
    }
}

// The bus sums; a single bus is a pass-through and two buses add
#[test]
fn sum_bus_sums_its_buses_in_bus_order() {
    let mut bus = SumBus::new(2).unwrap();
    let first = vec![0.25_f32; FRAMES];
    let second = vec![0.5_f32; FRAMES];
    let inputs: [&[f32]; 4] = [&first, &first, &second, &second];

    let (mut left, mut right) = (vec![0.0_f32; FRAMES], vec![0.0_f32; FRAMES]);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    let context = ProcessContext::new(48_000.0, FRAMES, &[]).unwrap();
    bus.process(&context, &inputs, &mut outputs).unwrap();

    for channel in &outputs {
        for sample in channel.iter() {
            assert_eq!(*sample, 0.75);
        }
    }
}

// No limiter, no clip, no normalization: summing may exceed unity and R4-4 lets it
#[test]
fn the_sum_is_not_clipped_or_normalized() {
    let mut bus = SumBus::new(4).unwrap();
    let loud = vec![0.9_f32; FRAMES];
    let inputs: [&[f32]; 8] = [&loud; 8];

    let (mut left, mut right) = (vec![0.0_f32; FRAMES], vec![0.0_f32; FRAMES]);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    let context = ProcessContext::new(48_000.0, FRAMES, &[]).unwrap();
    bus.process(&context, &inputs, &mut outputs).unwrap();

    for channel in &outputs {
        for sample in channel.iter() {
            assert!(
                (*sample - 3.6).abs() < 1e-5,
                "four buses at 0.9 must sum to 3.6, not be clipped to unity"
            );
        }
    }
}

// Layout mismatches are recoverable errors, never panics
#[test]
fn a_layout_mismatch_is_refused_rather_than_panicking() {
    let mut bus = SumBus::new(2).unwrap();
    let short = vec![0.0_f32; FRAMES];
    // Two buses declare four input channels; supplying two is a refusal
    let inputs: [&[f32]; 2] = [&short, &short];
    let (mut left, mut right) = (vec![0.0_f32; FRAMES], vec![0.0_f32; FRAMES]);
    let mut outputs = [left.as_mut_slice(), right.as_mut_slice()];
    let context = ProcessContext::new(48_000.0, FRAMES, &[]).unwrap();
    assert!(bus.process(&context, &inputs, &mut outputs).is_err());
}
