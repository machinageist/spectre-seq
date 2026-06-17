// =============================================================================
// File: crates/geist-dsp/src/benches/osc_bench.rs
// Layer: DSP primitives
// Purpose: Criterion benchmarks for oscillator hot paths
// Status: Implemented; sine, PolyBLEP saw, wavetable per-block generation.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use geist_dsp::prelude::*;

const SAMPLE_RATE: f32 = 48_000.0;
const BLOCK: usize = 512;

// Benchmark filling one audio block from each oscillator
fn oscillators(c: &mut Criterion) {
    let mut group = c.benchmark_group("oscillators");

    group.bench_function("sine_512", |b| {
        let mut osc = SineOsc::new();
        osc.set_frequency(440.0, SAMPLE_RATE);
        let mut buf = vec![0.0f32; BLOCK];
        b.iter(|| osc.process(black_box(&mut buf)));
    });

    group.bench_function("polyblep_saw_512", |b| {
        let mut osc = PolyBlepOsc::new(Waveform::Saw);
        osc.set_frequency(440.0, SAMPLE_RATE);
        let mut buf = vec![0.0f32; BLOCK];
        b.iter(|| osc.process(black_box(&mut buf)));
    });

    group.bench_function("wavetable_512", |b| {
        let table = Wavetable::sine(2048);
        let mut osc = WavetableOsc::new();
        osc.set_frequency(440.0, SAMPLE_RATE);
        let mut buf = vec![0.0f32; BLOCK];
        b.iter(|| osc.process(&table, black_box(&mut buf)));
    });

    group.finish();
}

criterion_group!(benches, oscillators);
criterion_main!(benches);
