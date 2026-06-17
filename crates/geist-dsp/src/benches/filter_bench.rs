// =============================================================================
// File: crates/geist-dsp/src/benches/filter_bench.rs
// Layer: DSP primitives
// Purpose: Criterion benchmarks for filter hot paths
// Status: Implemented; SVF, biquad, ladder per-sample throughput.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use geist_dsp::prelude::*;

const SAMPLE_RATE: f32 = 48_000.0;
const BLOCK: usize = 512;

// Sum a block of samples through a per-sample closure; constant input avoids denormals
fn run_block(mut f: impl FnMut(f32) -> f32) -> f32 {
    let mut acc = 0.0;
    for _ in 0..BLOCK {
        acc += f(black_box(0.5));
    }
    acc
}

// Benchmark one audio block through each filter
fn filters(c: &mut Criterion) {
    let mut group = c.benchmark_group("filters");

    group.bench_function("svf_lowpass_512", |b| {
        let mut svf = Svf::new(SvfMode::Lowpass);
        svf.set_params(1_000.0, 0.707, SAMPLE_RATE);
        b.iter(|| run_block(|x| svf.process_sample(x)));
    });

    group.bench_function("biquad_lowpass_512", |b| {
        let mut bq = Biquad::new();
        bq.set_lowpass(1_000.0, 0.707, SAMPLE_RATE);
        b.iter(|| run_block(|x| bq.process_sample(x)));
    });

    group.bench_function("moog_ladder_512", |b| {
        let mut ladder = Ladder::new();
        ladder.set_params(1_000.0, 0.7, SAMPLE_RATE);
        b.iter(|| run_block(|x| ladder.process_sample(x)));
    });

    group.finish();
}

criterion_group!(benches, filters);
criterion_main!(benches);
