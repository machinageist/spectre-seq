// =============================================================================
// File: crates/geist-modular/src/util.rs
// Layer: modular utilities
// Purpose: Gate convention and channel-major buffer walkers
// Status: Implemented; gate levels, per-channel map, channel-0 reducers.
// Notes: ProcessContext buffers are channel-major by frames(); io() hands back
//        disjoint input/output so nodes read and write in one borrow.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::context::ProcessContext;

use crate::standards::POLY_MAX;

// Gate/trigger convention shared by every logic and timing node
// A signal reads high at or above the threshold; logic emits clean 0/1 levels
pub const GATE_THRESHOLD: f32 = 0.5;
pub const GATE_HIGH: f32 = 1.0;
pub const GATE_LOW: f32 = 0.0;

// Test a sampled CV value against the gate threshold
#[inline]
pub fn is_high(x: f32) -> bool {
    x >= GATE_THRESHOLD
}

// Map a boolean gate state to its output level
#[inline]
pub fn gate_level(high: bool) -> f32 {
    if high {
        GATE_HIGH
    } else {
        GATE_LOW
    }
}

// Read a secondary poly input for one active engine using VCV-style rules
// M=0 returns 0 V; M=1 broadcasts; M>1 maps by index and zero-fills overflow
#[inline]
pub fn get_poly_voltage(
    input: &[f32],
    frames: usize,
    channels: usize,
    engine: usize,
    frame: usize,
) -> f32 {
    if channels == 0 || frames == 0 || frame >= frames {
        return 0.0;
    }
    let channels = channels.min(POLY_MAX);
    let ch = if channels == 1 {
        0
    } else if engine < channels {
        engine
    } else {
        return 0.0;
    };
    input.get(ch * frames + frame).copied().unwrap_or(0.0)
}

// Sum all channels of a poly audio input for a mono audio-only module
#[inline]
pub fn mono_audio_sum(input: &[f32], frames: usize, channels: usize, frame: usize) -> f32 {
    if frames == 0 || frame >= frames {
        return 0.0;
    }
    let mut sum = 0.0;
    for ch in 0..channels.min(POLY_MAX) {
        sum += input.get(ch * frames + frame).copied().unwrap_or(0.0);
    }
    sum
}

// Read channel 0 of a poly CV/hybrid input for a mono module fallback
#[inline]
pub fn mono_cv_first(input: &[f32], frames: usize, channels: usize, frame: usize) -> f32 {
    if channels == 0 || frames == 0 || frame >= frames {
        0.0
    } else {
        input.get(frame).copied().unwrap_or(0.0)
    }
}

// Map input channel i to output channel i through a unary function
// Output channels beyond the input count are cleared to silence
#[inline]
pub fn map_per_channel(ctx: &mut ProcessContext, f: impl Fn(f32) -> f32) {
    let frames = ctx.frames();
    let in_ch = ctx.input_channels();
    let out_ch = ctx.output_channels();
    let (input, output) = ctx.io();
    for ch in 0..out_ch {
        let dst = &mut output[ch * frames..(ch + 1) * frames];
        if ch < in_ch {
            let src = &input[ch * frames..(ch + 1) * frames];
            for (o, &i) in dst.iter_mut().zip(src) {
                *o = f(i);
            }
        } else {
            dst.fill(0.0);
        }
    }
}

// Fold every input channel into output channel 0 sample by sample
// `init` seeds the accumulator (a node's scalar bias or gain); extra output
// channels are cleared. A node with no inputs emits the seed as DC
#[inline]
pub fn reduce_into_ch0(ctx: &mut ProcessContext, init: f32, fold: impl Fn(f32, f32) -> f32) {
    let frames = ctx.frames();
    let in_ch = ctx.input_channels();
    let out_ch = ctx.output_channels();
    if out_ch == 0 {
        return;
    }
    let (input, output) = ctx.io();
    for (f, slot) in output[..frames].iter_mut().enumerate() {
        let mut acc = init;
        for ch in 0..in_ch {
            acc = fold(acc, input[ch * frames + f]);
        }
        *slot = acc;
    }
    for ch in 1..out_ch {
        output[ch * frames..(ch + 1) * frames].fill(0.0);
    }
}

// Fold the gate state of every input channel into output channel 0
// `init` seeds the boolean accumulator; the result writes clean gate levels
#[inline]
pub fn reduce_gate_into_ch0(
    ctx: &mut ProcessContext,
    init: bool,
    fold: impl Fn(bool, bool) -> bool,
) {
    let frames = ctx.frames();
    let in_ch = ctx.input_channels();
    let out_ch = ctx.output_channels();
    if out_ch == 0 {
        return;
    }
    let (input, output) = ctx.io();
    for (f, slot) in output[..frames].iter_mut().enumerate() {
        let mut acc = init;
        for ch in 0..in_ch {
            acc = fold(acc, is_high(input[ch * frames + f]));
        }
        *slot = gate_level(acc);
    }
    for ch in 1..out_ch {
        output[ch * frames..(ch + 1) * frames].fill(0.0);
    }
}

// Drive output channel 0 from input channel 0 through a stateful per-sample step
// Missing input reads as silence; extra output channels are cleared
#[inline]
pub fn process_mono_ch0(ctx: &mut ProcessContext, mut step: impl FnMut(f32) -> f32) {
    let frames = ctx.frames();
    let in_ch = ctx.input_channels();
    let out_ch = ctx.output_channels();
    if out_ch == 0 {
        return;
    }
    let (input, output) = ctx.io();
    for (f, slot) in output[..frames].iter_mut().enumerate() {
        let x = if in_ch >= 1 { input[f] } else { 0.0 };
        *slot = step(x);
    }
    for ch in 1..out_ch {
        output[ch * frames..(ch + 1) * frames].fill(0.0);
    }
}

// Drive output channel 0 from a signal (input 0) and a control (input 1)
// Both missing inputs read as silence; extra output channels are cleared
#[inline]
pub fn process_pair_ch0(ctx: &mut ProcessContext, mut step: impl FnMut(f32, f32) -> f32) {
    let frames = ctx.frames();
    let in_ch = ctx.input_channels();
    let out_ch = ctx.output_channels();
    if out_ch == 0 {
        return;
    }
    let (input, output) = ctx.io();
    for (f, slot) in output[..frames].iter_mut().enumerate() {
        let signal = if in_ch >= 1 { input[f] } else { 0.0 };
        let control = if in_ch >= 2 { input[frames + f] } else { 0.0 };
        *slot = step(signal, control);
    }
    for ch in 1..out_ch {
        output[ch * frames..(ch + 1) * frames].fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAMES: usize = 4;

    fn sample(input: &[f32], channels: usize, engine: usize, frame: usize) -> f32 {
        get_poly_voltage(input, FRAMES, channels, engine, frame)
    }

    #[test]
    fn poly_voltage_broadcasts_one_channel_to_all_engines() {
        let input = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(sample(&input, 1, 0, 2), 3.0);
        assert_eq!(sample(&input, 1, 7, 2), 3.0);
    }

    #[test]
    fn poly_voltage_maps_index_when_source_has_enough_channels() {
        let input = [
            1.0, 2.0, 3.0, 4.0, 10.0, 20.0, 30.0, 40.0, 100.0, 200.0, 300.0, 400.0,
        ];
        assert_eq!(sample(&input, 3, 0, 1), 2.0);
        assert_eq!(sample(&input, 3, 1, 1), 20.0);
        assert_eq!(sample(&input, 3, 2, 1), 200.0);
    }

    #[test]
    fn poly_voltage_zero_fills_engines_beyond_short_poly_source() {
        let input = [1.0, 2.0, 3.0, 4.0, 10.0, 20.0, 30.0, 40.0];
        assert_eq!(sample(&input, 2, 0, 0), 1.0);
        assert_eq!(sample(&input, 2, 1, 0), 10.0);
        assert_eq!(sample(&input, 2, 2, 0), 0.0);
    }

    #[test]
    fn poly_voltage_zero_fills_unpatched_source() {
        assert_eq!(sample(&[], 0, 0, 0), 0.0);
        assert_eq!(sample(&[], 0, 3, 2), 0.0);
    }

    #[test]
    fn poly_voltage_caps_at_public_channel_limit() {
        let input = [1.0f32; FRAMES * (POLY_MAX + 1)];
        assert_eq!(sample(&input, POLY_MAX + 1, POLY_MAX - 1, 0), 1.0);
        assert_eq!(sample(&input, POLY_MAX + 1, POLY_MAX, 0), 0.0);
    }

    #[test]
    fn mono_audio_fallback_sums_all_channels() {
        let input = [
            1.0, 2.0, 3.0, 4.0, 10.0, 20.0, 30.0, 40.0, 100.0, 200.0, 300.0, 400.0,
        ];
        assert_eq!(mono_audio_sum(&input, FRAMES, 3, 2), 333.0);
    }

    #[test]
    fn mono_cv_fallback_reads_first_channel() {
        let input = [1.0, 2.0, 3.0, 4.0, 10.0, 20.0, 30.0, 40.0];
        assert_eq!(mono_cv_first(&input, FRAMES, 2, 3), 4.0);
        assert_eq!(mono_cv_first(&[], FRAMES, 0, 3), 0.0);
    }
}
