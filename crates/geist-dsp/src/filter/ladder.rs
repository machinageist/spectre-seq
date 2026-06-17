// =============================================================================
// File: crates/geist-dsp/src/filter/ladder.rs
// Layer: DSP primitives
// Purpose: Moog ladder approximation
// Status: Implemented; Zavalishin TPT zero-delay-feedback 4-pole ladder.
// Notes: Closed-form feedback solve (no iteration); linear core stays stable to
//        self-oscillation. Resonance loses bass like the real ladder, by design.
//        Optional drive saturation is a future enhancement.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::PI;

// Cutoff is clamped below Nyquist so the prewarp tangent stays finite
const MIN_CUTOFF_HZ: f32 = 1.0;
const MAX_CUTOFF_RATIO: f32 = 0.49;

// Resonance in [0, 1] maps to feedback k in [0, 4]; k = 4 self-oscillates
const RESONANCE_TO_K: f32 = 4.0;

// Four-pole (24 dB/octave) transistor-ladder lowpass with resonance
// Topology-preserving one-poles plus a zero-delay feedback path
#[derive(Clone, Copy, Debug, Default)]
pub struct Ladder {
    // Per-stage instantaneous gain G = g / (1 + g), and its powers
    big_g: f32,
    g2: f32,
    g3: f32,
    g4: f32,
    // Feedback amount
    k: f32,
    // One integrator state per stage
    s: [f32; 4],
}

impl Ladder {
    // Build an unconfigured ladder; call set_params before processing
    pub fn new() -> Self {
        Self::default()
    }

    // Recompute coefficients from cutoff, resonance in [0, 1], and sample rate
    pub fn set_params(&mut self, cutoff_hz: f32, resonance: f32, sample_rate_hz: f32) {
        let max_cutoff = MAX_CUTOFF_RATIO * sample_rate_hz;
        let cutoff = cutoff_hz.clamp(MIN_CUTOFF_HZ, max_cutoff);
        let resonance = resonance.clamp(0.0, 1.0);

        let g = (PI * cutoff / sample_rate_hz).tan();
        self.big_g = g / (1.0 + g);
        self.g2 = self.big_g * self.big_g;
        self.g3 = self.g2 * self.big_g;
        self.g4 = self.g3 * self.big_g;
        self.k = resonance * RESONANCE_TO_K;
    }

    // Clear integrator state deterministically
    pub fn reset(&mut self) {
        self.s = [0.0; 4];
    }

    // Process one sample through the ladder lowpass
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let g = self.big_g;
        let one_minus_g = 1.0 - g;

        // Input-independent contribution of each stage
        let s0 = one_minus_g * self.s[0];
        let s1 = one_minus_g * self.s[1];
        let s2 = one_minus_g * self.s[2];
        let s3 = one_minus_g * self.s[3];

        // Solve the zero-delay feedback in closed form
        let sigma = self.g3 * s0 + self.g2 * s1 + g * s2 + s3;
        let y4 = (self.g4 * x + sigma) / (1.0 + self.k * self.g4);
        let u1 = x - self.k * y4;

        // Run the cascade forward, updating each integrator
        let y1 = g * u1 + s0;
        self.s[0] = 2.0 * y1 - self.s[0];
        let y2 = g * y1 + s1;
        self.s[1] = 2.0 * y2 - self.s[1];
        let y3 = g * y2 + s2;
        self.s[2] = 2.0 * y3 - self.s[2];
        let y = g * y3 + s3;
        self.s[3] = 2.0 * y - self.s[3];
        y
    }

    // Filter a buffer in place
    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    // Steady-state amplitude response at a sine frequency
    fn magnitude_at(cutoff: f32, resonance: f32, test_hz: f32) -> f32 {
        let mut ladder = Ladder::new();
        ladder.set_params(cutoff, resonance, SAMPLE_RATE);
        let total = 48_000usize;
        let settle = 24_000usize;
        let mut sum_sq = 0.0f64;
        let mut count = 0u32;
        for n in 0..total {
            let phase = core::f32::consts::TAU * test_hz * n as f32 / SAMPLE_RATE;
            let out = ladder.process_sample(phase.sin());
            if n >= settle {
                sum_sq += (out as f64) * (out as f64);
                count += 1;
            }
        }
        let out_rms = (sum_sq / count as f64).sqrt() as f32;
        out_rms / core::f32::consts::FRAC_1_SQRT_2
    }

    #[test]
    fn passes_dc_without_resonance() {
        let mut ladder = Ladder::new();
        ladder.set_params(1_000.0, 0.0, SAMPLE_RATE);
        let mut dc = 0.0;
        for _ in 0..20_000 {
            dc = ladder.process_sample(1.0);
        }
        assert!((dc - 1.0).abs() < 1e-3, "ladder DC gain = {dc}");
    }

    #[test]
    fn rolls_off_high_frequencies_steeply() {
        // Four poles give a steep stopband; 16x cutoff is deeply attenuated
        let mag = magnitude_at(1_000.0, 0.0, 16_000.0);
        assert!(mag < 0.01, "ladder high-freq magnitude = {mag}");
    }

    #[test]
    fn resonance_lifts_the_cutoff_region() {
        let flat = magnitude_at(2_000.0, 0.0, 2_000.0);
        let resonant = magnitude_at(2_000.0, 0.9, 2_000.0);
        assert!(
            resonant > flat * 2.0,
            "resonance should peak: flat={flat} resonant={resonant}"
        );
    }

    // Energy in the ring tail after an impulse, skipping the initial transient
    fn ring_energy(resonance: f32) -> f64 {
        let mut ladder = Ladder::new();
        ladder.set_params(1_000.0, resonance, SAMPLE_RATE);
        let mut energy = 0.0f64;
        for n in 0..48_000 {
            let x = if n == 0 { 1.0 } else { 0.0 };
            let y = ladder.process_sample(x);
            assert!(y.is_finite());
            if n >= 2_000 {
                energy += (y as f64) * (y as f64);
            }
        }
        energy
    }

    #[test]
    fn higher_resonance_rings_longer() {
        // A linear ladder rings with finite decay below self-oscillation;
        // higher resonance keeps far more energy in the tail
        let high = ring_energy(0.95);
        let low = ring_energy(0.2);
        assert!(high > low * 5.0, "resonance ringing: high={high} low={low}");
    }

    #[test]
    fn stays_bounded_at_max_resonance() {
        let mut ladder = Ladder::new();
        ladder.set_params(2_000.0, 1.0, SAMPLE_RATE);
        let mut peak = 0.0f32;
        for n in 0..96_000 {
            let x = if n % 4_800 == 0 { 1.0 } else { 0.0 };
            let y = ladder.process_sample(x);
            assert!(y.is_finite());
            peak = peak.max(y.abs());
        }
        assert!(peak < 100.0, "max-resonance ladder unbounded: {peak}");
    }

    #[test]
    fn reset_clears_state() {
        let mut used = Ladder::new();
        used.set_params(800.0, 0.7, SAMPLE_RATE);
        for _ in 0..1000 {
            used.process_sample(0.8);
        }
        used.reset();
        let mut fresh = Ladder::new();
        fresh.set_params(800.0, 0.7, SAMPLE_RATE);
        assert_eq!(used.process_sample(0.4), fresh.process_sample(0.4));
    }
}
