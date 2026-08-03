// =============================================================================
// File: crates/spectre-dsp/src/filter/svf.rs
// Layer: DSP primitives
// Purpose: state-variable filter (LP/HP/BP/notch)
// Status: Implemented; Cytomic/Zavalishin TPT zero-delay-feedback SVF.
// Notes: Trapezoidal-integrator topology stays stable under fast modulation
//        and yields all bands from shared state.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::PI;

// Cutoff is clamped below Nyquist so the prewarp tangent stays finite
const MIN_CUTOFF_HZ: f32 = 1.0;
const MAX_CUTOFF_RATIO: f32 = 0.49;

// Lowest resonance; guards the 1/Q damping term
const MIN_Q: f32 = 0.025;

// Output band tapped from the shared two-integrator state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SvfMode {
    #[default]
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
}

// Zero-delay-feedback state-variable filter
// Coefficients update on parameter change; the per-sample core is branch-light
#[derive(Clone, Copy, Debug, Default)]
pub struct Svf {
    mode: SvfMode,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ic1eq: f32,
    ic2eq: f32,
}

impl Svf {
    // Build a filter in the given mode; call set_params before processing
    pub fn new(mode: SvfMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    // Select the output band without disturbing state
    pub fn set_mode(&mut self, mode: SvfMode) {
        self.mode = mode;
    }

    // Recompute coefficients from cutoff, resonance, and sample rate
    pub fn set_params(&mut self, cutoff_hz: f32, q: f32, sample_rate_hz: f32) {
        let max_cutoff = MAX_CUTOFF_RATIO * sample_rate_hz;
        let cutoff = cutoff_hz.clamp(MIN_CUTOFF_HZ, max_cutoff);
        let q = q.max(MIN_Q);

        let g = (PI * cutoff / sample_rate_hz).tan();
        self.k = 1.0 / q;
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }

    // Clear integrator state deterministically
    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    // Process one sample, returning the configured band
    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let v3 = input - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a2 * v3;
        let v2 = self.ic2eq + self.a2 * self.ic1eq + self.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        match self.mode {
            SvfMode::Lowpass => v2,
            SvfMode::Highpass => input - self.k * v1 - v2,
            SvfMode::Bandpass => v1,
            SvfMode::Notch => input - self.k * v1,
        }
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
    const BUTTERWORTH_Q: f32 = std::f32::consts::FRAC_1_SQRT_2;

    // Steady-state amplitude response at a sine frequency
    fn magnitude_at(mode: SvfMode, cutoff: f32, q: f32, test_hz: f32) -> f32 {
        let mut svf = Svf::new(mode);
        svf.set_params(cutoff, q, SAMPLE_RATE);
        let total = 48_000usize; // one second to settle and measure
        let settle = 24_000usize;
        let mut sum_sq = 0.0f64;
        let mut count = 0u32;
        for n in 0..total {
            let phase = core::f32::consts::TAU * test_hz * n as f32 / SAMPLE_RATE;
            let out = svf.process_sample(phase.sin());
            if n >= settle {
                sum_sq += (out as f64) * (out as f64);
                count += 1;
            }
        }
        // RMS of output; input sine has RMS 1/sqrt(2), so |H| = out_rms / input_rms
        let out_rms = (sum_sq / count as f64).sqrt() as f32;
        out_rms / core::f32::consts::FRAC_1_SQRT_2
    }

    #[test]
    fn lowpass_passes_dc_and_blocks_high() {
        let mut svf = Svf::new(SvfMode::Lowpass);
        svf.set_params(1_000.0, BUTTERWORTH_Q, SAMPLE_RATE);
        // Settle on a DC input; lowpass gain at DC is unity
        let mut dc = 0.0;
        for _ in 0..10_000 {
            dc = svf.process_sample(1.0);
        }
        assert!((dc - 1.0).abs() < 1e-3, "LP DC gain = {dc}");
        // Far above cutoff is strongly attenuated
        let high = magnitude_at(SvfMode::Lowpass, 1_000.0, BUTTERWORTH_Q, 12_000.0);
        assert!(high < 0.05, "LP at 12 kHz = {high}");
    }

    #[test]
    fn highpass_blocks_dc_and_passes_high() {
        let mut svf = Svf::new(SvfMode::Highpass);
        svf.set_params(1_000.0, BUTTERWORTH_Q, SAMPLE_RATE);
        let mut dc = 0.0;
        for _ in 0..10_000 {
            dc = svf.process_sample(1.0);
        }
        assert!(dc.abs() < 1e-3, "HP DC gain = {dc}");
        let high = magnitude_at(SvfMode::Highpass, 1_000.0, BUTTERWORTH_Q, 12_000.0);
        assert!((high - 1.0).abs() < 0.05, "HP at 12 kHz = {high}");
    }

    #[test]
    fn lowpass_is_minus_3db_at_cutoff() {
        // Butterworth Q gives exactly 1/sqrt(2) magnitude at the cutoff frequency
        let mag = magnitude_at(SvfMode::Lowpass, 2_000.0, BUTTERWORTH_Q, 2_000.0);
        assert!(
            (mag - core::f32::consts::FRAC_1_SQRT_2).abs() < 0.03,
            "LP at cutoff = {mag}"
        );
    }

    #[test]
    fn bandpass_peaks_near_cutoff() {
        let at_cutoff = magnitude_at(SvfMode::Bandpass, 2_000.0, 2.0, 2_000.0);
        let below = magnitude_at(SvfMode::Bandpass, 2_000.0, 2.0, 200.0);
        let above = magnitude_at(SvfMode::Bandpass, 2_000.0, 2.0, 12_000.0);
        assert!(at_cutoff > below * 3.0, "BP not peaked vs below");
        assert!(at_cutoff > above * 3.0, "BP not peaked vs above");
    }

    #[test]
    fn notch_rejects_cutoff_and_passes_edges() {
        let at_cutoff = magnitude_at(SvfMode::Notch, 2_000.0, BUTTERWORTH_Q, 2_000.0);
        let below = magnitude_at(SvfMode::Notch, 2_000.0, BUTTERWORTH_Q, 100.0);
        assert!(at_cutoff < 0.2, "notch should reject cutoff: {at_cutoff}");
        assert!(below > 0.8, "notch should pass DC side: {below}");
    }

    #[test]
    fn stays_stable_at_high_resonance() {
        // High Q with a steep impulse must not blow up
        let mut svf = Svf::new(SvfMode::Lowpass);
        svf.set_params(5_000.0, 50.0, SAMPLE_RATE);
        let mut peak = 0.0f32;
        for n in 0..96_000 {
            let input = if n == 0 { 1.0 } else { 0.0 };
            let out = svf.process_sample(input);
            assert!(out.is_finite());
            peak = peak.max(out.abs());
        }
        assert!(peak < 100.0, "resonant impulse peak unbounded: {peak}");
    }

    #[test]
    fn reset_clears_state() {
        let mut svf = Svf::new(SvfMode::Lowpass);
        svf.set_params(800.0, 4.0, SAMPLE_RATE);
        for _ in 0..1000 {
            svf.process_sample(0.9);
        }
        svf.reset();
        // First post-reset output equals the response from a clean state
        let after_reset = svf.process_sample(0.5);
        let mut fresh = Svf::new(SvfMode::Lowpass);
        fresh.set_params(800.0, 4.0, SAMPLE_RATE);
        assert_eq!(after_reset, fresh.process_sample(0.5));
    }
}
