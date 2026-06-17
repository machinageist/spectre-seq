// =============================================================================
// File: crates/geist-dsp/src/benches/reverb_bench.rs
// Layer: DSP primitives
// Purpose: Criterion benchmark for the FFT convolution hot path
// Status: Implemented; Convolver block processing with a realistic IR length.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use geist_dsp::prelude::*;

const BLOCK: usize = 512;
const IR_LEN: usize = 4_800; // ~0.1 s at 48 kHz

// Benchmark one block of FFT overlap-add convolution
fn convolution(c: &mut Criterion) {
    // Simple decaying impulse response stands in for a real one
    let ir: Vec<f32> = (0..IR_LEN)
        .map(|n| (1.0 - n as f32 / IR_LEN as f32) * if n % 7 == 0 { 1.0 } else { -0.3 })
        .collect();
    let mut conv = Convolver::new(&ir, BLOCK);
    let input: Vec<f32> = (0..BLOCK).map(|n| (n as f32 * 0.05).sin()).collect();
    let mut output = vec![0.0f32; BLOCK];

    c.bench_function("convolver_block512_ir4800", |b| {
        b.iter(|| conv.process_block(black_box(&input), black_box(&mut output)));
    });
}

criterion_group!(benches, convolution);
criterion_main!(benches);
