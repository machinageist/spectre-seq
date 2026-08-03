// =============================================================================
// File: crates/spectre-dsp/src/fx/reverb.rs
// Layer: DSP primitives
// Purpose: FFT convolution reverb (rustfft)
// Status: Implemented; overlap-add FFT Convolver + stereo Reverb wrapper.
// Notes: Accumulator overlap-add handles impulse responses longer than the block.
//        FFTs/buffers are planned once in new(); process_block never allocates.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::math::lerp;
use crate::rng::Rng;

// Block-based FFT convolution of an input stream with a fixed impulse response
// Uses overlap-add: each input block's full convolution is summed into an
// accumulator, which is drained one block at a time
pub struct Convolver {
    forward: Arc<dyn Fft<f32>>,
    inverse: Arc<dyn Fft<f32>>,
    // Precomputed FFT of the zero-padded impulse response, length fft_size
    ir_spectrum: Vec<Complex<f32>>,
    block_size: usize,
    fft_size: usize,
    // Per-block frequency-domain workspace and FFT scratch
    work: Vec<Complex<f32>>,
    scratch: Vec<Complex<f32>>,
    // Time-domain accumulator holding pending (overlapping) output
    accumulator: Vec<f32>,
    scale: f32,
}

impl Convolver {
    // Build a convolver for an impulse response processed in fixed `block_size` blocks
    pub fn new(impulse_response: &[f32], block_size: usize) -> Self {
        let block_size = block_size.max(1);
        let ir_len = impulse_response.len().max(1);
        // FFT must hold the full linear convolution of one block with the IR
        let fft_size = (block_size + ir_len - 1).next_power_of_two();

        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(fft_size);
        let inverse = planner.plan_fft_inverse(fft_size);

        // Transform the zero-padded impulse response once
        let mut ir_spectrum = vec![Complex::new(0.0, 0.0); fft_size];
        for (slot, &sample) in ir_spectrum.iter_mut().zip(impulse_response) {
            slot.re = sample;
        }
        let scratch_len = forward
            .get_inplace_scratch_len()
            .max(inverse.get_inplace_scratch_len());
        let mut scratch = vec![Complex::new(0.0, 0.0); scratch_len];
        forward.process_with_scratch(&mut ir_spectrum, &mut scratch);

        Self {
            forward,
            inverse,
            ir_spectrum,
            block_size,
            fft_size,
            work: vec![Complex::new(0.0, 0.0); fft_size],
            scratch,
            accumulator: vec![0.0; fft_size],
            scale: 1.0 / fft_size as f32,
        }
    }

    // Block size this convolver expects per call
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    // Clear pending output so the next block starts dry
    pub fn reset(&mut self) {
        self.accumulator.iter_mut().for_each(|s| *s = 0.0);
    }

    // Convolve one block; `input` and `output` must both be block_size long
    pub fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert_eq!(input.len(), self.block_size);
        debug_assert_eq!(output.len(), self.block_size);

        // Load the zero-padded block into the frequency workspace
        for (i, slot) in self.work.iter_mut().enumerate() {
            slot.re = if i < self.block_size { input[i] } else { 0.0 };
            slot.im = 0.0;
        }
        self.forward
            .process_with_scratch(&mut self.work, &mut self.scratch);

        // Multiply spectra (convolution in time)
        for (w, h) in self.work.iter_mut().zip(&self.ir_spectrum) {
            *w *= *h;
        }
        self.inverse
            .process_with_scratch(&mut self.work, &mut self.scratch);

        // Overlap-add the (normalized) result into the accumulator
        for (acc, w) in self.accumulator.iter_mut().zip(&self.work) {
            *acc += w.re * self.scale;
        }

        // Drain one block of finished output, then shift the accumulator down
        output.copy_from_slice(&self.accumulator[..self.block_size]);
        self.accumulator.copy_within(self.block_size.., 0);
        let tail = self.fft_size - self.block_size;
        self.accumulator[tail..].iter_mut().for_each(|s| *s = 0.0);
    }
}

// Time at which the noise tail reaches roughly -60 dB, in time constants
const RT60_LN: f32 = 6.907_755; // ln(1000)

// Decorrelated seeds give the two channels independent reverb tails
const LEFT_SEED: u64 = 0x51ED_2A17_9C4B_88D3;
const RIGHT_SEED: u64 = 0xC0FF_EE12_3456_789A;

// Stereo convolution reverb driven by a synthetic decaying-noise impulse response
// Left and right convolve against independent noise tails for stereo width
pub struct Reverb {
    left: Convolver,
    right: Convolver,
    wet_l: Vec<f32>,
    wet_r: Vec<f32>,
    mix: f32,
}

impl Reverb {
    // Build a stereo reverb whose tail decays ~60 dB over `decay_seconds`
    pub fn new(sample_rate_hz: f32, block_size: usize, decay_seconds: f32) -> Self {
        let ir_left = Self::decay_ir(sample_rate_hz, decay_seconds, LEFT_SEED);
        let ir_right = Self::decay_ir(sample_rate_hz, decay_seconds, RIGHT_SEED);
        let block = block_size.max(1);
        Self {
            left: Convolver::new(&ir_left, block),
            right: Convolver::new(&ir_right, block),
            wet_l: vec![0.0; block],
            wet_r: vec![0.0; block],
            mix: 0.3,
        }
    }

    // Build a unit-energy decaying-noise impulse response
    fn decay_ir(sample_rate_hz: f32, decay_seconds: f32, seed: u64) -> Vec<f32> {
        let decay = decay_seconds.max(0.001);
        let len = ((decay * sample_rate_hz) as usize).max(1);
        let tau = decay * sample_rate_hz / RT60_LN;
        let mut rng = Rng::new(seed);
        let mut ir: Vec<f32> = (0..len)
            .map(|n| rng.next_bipolar() * (-(n as f32) / tau).exp())
            .collect();
        // Normalize to unit energy so the wet level is predictable
        let energy: f32 = ir.iter().map(|s| s * s).sum();
        if energy > 0.0 {
            let scale = 1.0 / energy.sqrt();
            ir.iter_mut().for_each(|s| *s *= scale);
        }
        ir
    }

    // Dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Block size both channels expect per call
    pub fn block_size(&self) -> usize {
        self.left.block_size()
    }

    // Clear the reverb tails
    pub fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }

    // Process one stereo block in place; both slices must be block_size long
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.left.process_block(left, &mut self.wet_l);
        self.right.process_block(right, &mut self.wet_r);
        for (((l, r), wl), wr) in left
            .iter_mut()
            .zip(right.iter_mut())
            .zip(&self.wet_l)
            .zip(&self.wet_r)
        {
            *l = lerp(*l, *wl, self.mix);
            *r = lerp(*r, *wr, self.mix);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Direct time-domain convolution for ground truth
    fn naive_convolution(input: &[f32], ir: &[f32], len: usize) -> Vec<f32> {
        let mut out = vec![0.0f32; len];
        for (n, slot) in out.iter_mut().enumerate() {
            let mut acc = 0.0;
            for (k, &h) in ir.iter().enumerate() {
                if n >= k {
                    acc += input[n - k] * h;
                }
            }
            *slot = acc;
        }
        out
    }

    // Run an input through the convolver in fixed blocks, collecting output
    fn run_blocks(conv: &mut Convolver, input: &[f32]) -> Vec<f32> {
        let b = conv.block_size();
        let mut out = vec![0.0f32; input.len()];
        for (xin, xout) in input.chunks_exact(b).zip(out.chunks_exact_mut(b)) {
            conv.process_block(xin, xout);
        }
        out
    }

    #[test]
    fn identity_ir_passes_input() {
        let mut conv = Convolver::new(&[1.0], 8);
        let input: Vec<f32> = (0..64).map(|n| (n as f32 * 0.1).sin()).collect();
        let out = run_blocks(&mut conv, &input);
        for (o, i) in out.iter().zip(&input) {
            assert!((o - i).abs() < 1e-4, "identity mismatch: {o} vs {i}");
        }
    }

    #[test]
    fn impulse_input_returns_the_ir() {
        let ir = [0.5, 0.25, -0.1, 0.05];
        let mut conv = Convolver::new(&ir, 8);
        let mut input = vec![0.0f32; 32];
        input[0] = 1.0;
        let out = run_blocks(&mut conv, &input);
        for (i, &h) in ir.iter().enumerate() {
            assert!((out[i] - h).abs() < 1e-5, "ir[{i}] = {} vs {h}", out[i]);
        }
        assert!(out[ir.len()..].iter().all(|&s| s.abs() < 1e-5));
    }

    #[test]
    fn matches_naive_convolution_short_ir() {
        let ir = [0.5, 0.25, -0.1, 0.05];
        let mut conv = Convolver::new(&ir, 8);
        let input: Vec<f32> = (0..64).map(|n| (n as f32 * 0.37).sin() * 0.8).collect();
        let out = run_blocks(&mut conv, &input);
        let reference = naive_convolution(&input, &ir, input.len());
        for (o, r) in out.iter().zip(&reference) {
            assert!((o - r).abs() < 1e-4, "fft={o} naive={r}");
        }
    }

    #[test]
    fn matches_naive_when_ir_longer_than_block() {
        // The accumulator must carry the tail across several blocks
        let ir: Vec<f32> = (0..20).map(|k| 0.9f32.powi(k) * 0.3).collect();
        let mut conv = Convolver::new(&ir, 4);
        let input: Vec<f32> = (0..128).map(|n| (n as f32 * 0.11).cos()).collect();
        let out = run_blocks(&mut conv, &input);
        let reference = naive_convolution(&input, &ir, input.len());
        for (n, (o, r)) in out.iter().zip(&reference).enumerate() {
            assert!(
                (o - r).abs() < 1e-4,
                "block-spanning mismatch at {n}: {o} vs {r}"
            );
        }
    }

    #[test]
    fn reset_clears_the_tail() {
        let ir = [0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
        let mut conv = Convolver::new(&ir, 4);
        let mut input = [0.0f32; 8];
        input[0] = 1.0;
        let mut scratch = vec![0.0f32; 4];
        conv.process_block(&input[..4], &mut scratch);
        conv.reset();
        // After reset, a silent block must produce silence (no lingering tail)
        let silent = [0.0f32; 4];
        let mut out = [0.0f32; 4];
        conv.process_block(&silent, &mut out);
        assert!(
            out.iter().all(|&s| s.abs() < 1e-6),
            "tail survived reset: {out:?}"
        );
    }
}

#[cfg(test)]
mod reverb_tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;
    const BLOCK: usize = 64;

    // Feed a left+right impulse and collect per-block output energy
    fn block_energies(rev: &mut Reverb, blocks: usize) -> Vec<(f64, f64)> {
        let mut out = Vec::new();
        for b in 0..blocks {
            let mut l = [0.0f32; BLOCK];
            let mut r = [0.0f32; BLOCK];
            if b == 0 {
                l[0] = 1.0;
                r[0] = 1.0;
            }
            rev.process(&mut l, &mut r);
            let el: f64 = l.iter().map(|s| (*s as f64) * (*s as f64)).sum();
            let er: f64 = r.iter().map(|s| (*s as f64) * (*s as f64)).sum();
            out.push((el, er));
        }
        out
    }

    #[test]
    fn dry_mix_passes_input() {
        let mut rev = Reverb::new(SAMPLE_RATE, BLOCK, 0.05);
        rev.set_mix(0.0);
        let mut l = [0.3f32; BLOCK];
        let mut r = [-0.4f32; BLOCK];
        rev.process(&mut l, &mut r);
        assert!(l.iter().all(|&s| (s - 0.3).abs() < 1e-6));
        assert!(r.iter().all(|&s| (s + 0.4).abs() < 1e-6));
    }

    #[test]
    fn wet_tail_decays_over_time() {
        let mut rev = Reverb::new(SAMPLE_RATE, BLOCK, 0.05); // ~2400-sample tail
        rev.set_mix(1.0);
        let energies = block_energies(&mut rev, 35);
        let early = energies[0].0;
        let late = energies[30].0;
        assert!(early > 0.0, "no wet energy");
        assert!(
            late < early,
            "tail did not decay: early={early} late={late}"
        );
    }

    #[test]
    fn channels_are_decorrelated() {
        let mut rev = Reverb::new(SAMPLE_RATE, BLOCK, 0.05);
        rev.set_mix(1.0);
        let mut l = [0.0f32; BLOCK];
        let mut r = [0.0f32; BLOCK];
        l[0] = 1.0;
        r[0] = 1.0;
        rev.process(&mut l, &mut r);
        // Same input both channels, but independent IRs give different tails
        assert!(l != r, "stereo channels were identical");
    }

    #[test]
    fn output_stays_finite() {
        let mut rev = Reverb::new(SAMPLE_RATE, BLOCK, 0.05);
        rev.set_mix(0.5);
        for _ in 0..200 {
            let mut l = [0.5f32; BLOCK];
            let mut r = [0.5f32; BLOCK];
            rev.process(&mut l, &mut r);
            assert!(l.iter().chain(&r).all(|s| s.is_finite()));
        }
    }
}
