// =============================================================================
// File: crates/spectre-dsp/src/osc/noise.rs
// Layer: DSP primitives
// Purpose: white, pink, crackle
// Status: Implemented; xorshift64* white, Kellet pink, sparse-impulse crackle.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::rng::Rng;

// Nonzero default seed so a Default generator is immediately usable
const DEFAULT_SEED: u64 = 0x853C_49E6_748F_EA9B;

// Brings the refined Kellet pink filter sum to roughly unity RMS
const PINK_OUTPUT_GAIN: f32 = 0.11;

// Uniform white noise in [-1, 1)
#[derive(Clone, Copy, Debug)]
pub struct WhiteNoise {
    rng: Rng,
    seed: u64,
}

impl WhiteNoise {
    // Build a generator from a seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
            seed,
        }
    }

    // Restore the deterministic initial state
    pub fn reset(&mut self) {
        self.rng = Rng::new(self.seed);
    }

    // Generate the next sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        self.rng.next_bipolar()
    }

    // Fill a buffer with successive samples
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }
}

impl Default for WhiteNoise {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

// Pink noise (-3 dB/octave) via Paul Kellet's refined filter bank
// Seven one-pole sections summed; output scaled toward unity RMS
#[derive(Clone, Copy, Debug)]
pub struct PinkNoise {
    rng: Rng,
    seed: u64,
    b: [f32; 7],
}

impl PinkNoise {
    // Build a generator from a seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
            seed,
            b: [0.0; 7],
        }
    }

    // Restore the deterministic initial state and clear the filter memory
    pub fn reset(&mut self) {
        self.rng = Rng::new(self.seed);
        self.b = [0.0; 7];
    }

    // Generate the next sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let white = self.rng.next_bipolar();
        // Refined Kellet coefficients; b[6] feeds the next sample
        self.b[0] = 0.99886 * self.b[0] + white * 0.0555179;
        self.b[1] = 0.99332 * self.b[1] + white * 0.0750759;
        self.b[2] = 0.969 * self.b[2] + white * 0.153852;
        self.b[3] = 0.8665 * self.b[3] + white * 0.3104856;
        self.b[4] = 0.55 * self.b[4] + white * 0.5329522;
        self.b[5] = -0.7616 * self.b[5] - white * 0.016898;
        let pink = self.b[0]
            + self.b[1]
            + self.b[2]
            + self.b[3]
            + self.b[4]
            + self.b[5]
            + self.b[6]
            + white * 0.5362;
        self.b[6] = white * 0.115926;
        pink * PINK_OUTPUT_GAIN
    }

    // Fill a buffer with successive samples
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }
}

impl Default for PinkNoise {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

// Sparse random impulses, like vinyl crackle; density is the per-sample fire chance
#[derive(Clone, Copy, Debug)]
pub struct Crackle {
    rng: Rng,
    seed: u64,
    density: f32,
}

impl Crackle {
    // Build a generator; density is clamped to [0, 1]
    pub fn new(seed: u64, density: f32) -> Self {
        Self {
            rng: Rng::new(seed),
            seed,
            density: density.clamp(0.0, 1.0),
        }
    }

    // Set the per-sample probability of an impulse
    pub fn set_density(&mut self, density: f32) {
        self.density = density.clamp(0.0, 1.0);
    }

    // Restore the deterministic initial state
    pub fn reset(&mut self) {
        self.rng = Rng::new(self.seed);
    }

    // Generate the next sample: an impulse with probability `density`, else silence
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        if self.rng.next_unit() < self.density {
            self.rng.next_bipolar()
        } else {
            0.0
        }
    }

    // Fill a buffer with successive samples
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Normalized lag-1 autocorrelation; near 0 for white, positive for pink
    fn lag1_autocorr(buf: &[f32]) -> f32 {
        let mean: f32 = buf.iter().sum::<f32>() / buf.len() as f32;
        let mut num = 0.0;
        let mut den = 0.0;
        for w in buf.windows(2) {
            num += (w[0] - mean) * (w[1] - mean);
        }
        for &s in buf {
            den += (s - mean) * (s - mean);
        }
        num / den
    }

    fn rms(buf: &[f32]) -> f32 {
        (buf.iter().map(|s| s * s).sum::<f32>() / buf.len() as f32).sqrt()
    }

    #[test]
    fn white_is_bounded_and_reproducible() {
        let mut a = WhiteNoise::new(42);
        let mut b = WhiteNoise::new(42);
        let mut ba = [0.0f32; 256];
        let mut bb = [0.0f32; 256];
        a.process(&mut ba);
        b.process(&mut bb);
        assert_eq!(ba, bb); // same seed, same stream
        assert!(ba.iter().all(|&s| (-1.0..1.0).contains(&s)));
    }

    #[test]
    fn white_reset_restores_stream() {
        let mut n = WhiteNoise::new(7);
        let first: Vec<f32> = (0..64).map(|_| n.next_sample()).collect();
        n.reset();
        let again: Vec<f32> = (0..64).map(|_| n.next_sample()).collect();
        assert_eq!(first, again);
    }

    #[test]
    fn white_has_near_zero_mean_and_expected_rms() {
        let mut n = WhiteNoise::new(123);
        let mut buf = vec![0.0f32; 100_000];
        n.process(&mut buf);
        let mean: f32 = buf.iter().sum::<f32>() / buf.len() as f32;
        assert!(mean.abs() < 0.02, "white mean = {mean}");
        // Uniform [-1, 1) has theoretical RMS sqrt(1/3) ~= 0.577
        assert!((rms(&buf) - 0.577).abs() < 0.03);
    }

    #[test]
    fn pink_is_more_correlated_than_white() {
        let mut white = WhiteNoise::new(99);
        let mut pink = PinkNoise::new(99);
        let mut wb = vec![0.0f32; 50_000];
        let mut pb = vec![0.0f32; 50_000];
        white.process(&mut wb);
        pink.process(&mut pb);
        let wc = lag1_autocorr(&wb);
        let pc = lag1_autocorr(&pb);
        assert!(wc.abs() < 0.05, "white autocorr should be ~0: {wc}");
        assert!(pc > 0.3, "pink should be strongly correlated: {pc}");
    }

    #[test]
    fn pink_is_finite_with_sane_level() {
        let mut pink = PinkNoise::new(5);
        let mut buf = vec![0.0f32; 50_000];
        pink.process(&mut buf);
        assert!(buf.iter().all(|s| s.is_finite()));
        let r = rms(&buf);
        assert!((0.05..1.0).contains(&r), "pink RMS out of range: {r}");
    }

    #[test]
    fn pink_reset_clears_filter_and_stream() {
        let mut pink = PinkNoise::new(11);
        let first: Vec<f32> = (0..128).map(|_| pink.next_sample()).collect();
        pink.reset();
        let again: Vec<f32> = (0..128).map(|_| pink.next_sample()).collect();
        assert_eq!(first, again);
    }

    #[test]
    fn crackle_density_controls_impulse_rate() {
        let n = 20_000;

        let mut silent = Crackle::new(1, 0.0);
        let mut sbuf = vec![0.0f32; n];
        silent.process(&mut sbuf);
        assert!(sbuf.iter().all(|&s| s == 0.0));

        let mut sparse = Crackle::new(2, 0.1);
        let mut buf = vec![0.0f32; n];
        sparse.process(&mut buf);
        let fired = buf.iter().filter(|&&s| s != 0.0).count();
        // Expect ~10% of samples to fire
        assert!((1_700..=2_300).contains(&fired), "fired = {fired}");
        assert!(buf.iter().all(|&s| (-1.0..1.0).contains(&s)));
    }
}
