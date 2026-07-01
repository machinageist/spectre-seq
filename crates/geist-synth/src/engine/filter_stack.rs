// =============================================================================
// File: crates/geist-synth/src/engine/filter_stack.rs
// Layer: internal synth device
// Purpose: 2× SVF in series/parallel with FM routing
// Status: Implemented; two SVF filters, series/parallel routing, cutoff mod.
// Notes: Cutoff modulation is multiplicative (octave-style) and applied per
//        block via set_cutoff_mod, recomputing coefficients off the audio path.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_dsp::prelude::{Svf, SvfMode};

// How the two filters combine
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterRouting {
    // A then B; rolloffs multiply
    #[default]
    Series,
    // A plus B, summed and halved
    Parallel,
}

// Two state-variable filters with shared cutoff modulation
#[derive(Clone, Copy, Debug)]
pub struct FilterStack {
    a: Svf,
    b: Svf,
    routing: FilterRouting,
    sample_rate: f32,
    cutoff_a: f32,
    cutoff_b: f32,
    res_a: f32,
    res_b: f32,
}

impl FilterStack {
    // Build a stack of two lowpass filters
    pub fn new(sample_rate_hz: f32) -> Self {
        let mut stack = Self {
            a: Svf::new(SvfMode::Lowpass),
            b: Svf::new(SvfMode::Lowpass),
            routing: FilterRouting::Series,
            sample_rate: sample_rate_hz,
            cutoff_a: 2_000.0,
            cutoff_b: 2_000.0,
            res_a: 0.707,
            res_b: 0.707,
        };
        stack.set_cutoff_mod(1.0);
        stack
    }

    // Choose series or parallel routing
    pub fn set_routing(&mut self, routing: FilterRouting) {
        self.routing = routing;
    }

    // Configure filter A base cutoff, resonance, and mode
    pub fn set_filter_a(&mut self, cutoff_hz: f32, resonance: f32, mode: SvfMode) {
        self.cutoff_a = cutoff_hz;
        self.res_a = resonance;
        self.a.set_mode(mode);
        self.a.set_params(cutoff_hz, resonance, self.sample_rate);
    }

    // Configure filter B base cutoff, resonance, and mode
    pub fn set_filter_b(&mut self, cutoff_hz: f32, resonance: f32, mode: SvfMode) {
        self.cutoff_b = cutoff_hz;
        self.res_b = resonance;
        self.b.set_mode(mode);
        self.b.set_params(cutoff_hz, resonance, self.sample_rate);
    }

    // Scale both base cutoffs by `factor` (e.g., from a filter envelope)
    pub fn set_cutoff_mod(&mut self, factor: f32) {
        let f = factor.max(0.0);
        self.a
            .set_params(self.cutoff_a * f, self.res_a, self.sample_rate);
        self.b
            .set_params(self.cutoff_b * f, self.res_b, self.sample_rate);
    }

    // Clear filter state
    pub fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
    }

    // Process one sample through the configured routing
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        match self.routing {
            FilterRouting::Series => self.b.process_sample(self.a.process_sample(x)),
            FilterRouting::Parallel => 0.5 * (self.a.process_sample(x) + self.b.process_sample(x)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    fn magnitude_at(stack: &mut FilterStack, test_hz: f32) -> f32 {
        let total = 48_000usize;
        let settle = 24_000usize;
        let mut sum_sq = 0.0f64;
        let mut count = 0u32;
        for n in 0..total {
            let phase = core::f32::consts::TAU * test_hz * n as f32 / SAMPLE_RATE;
            let out = stack.process_sample(phase.sin());
            if n >= settle {
                sum_sq += (out as f64) * (out as f64);
                count += 1;
            }
        }
        let out_rms = (sum_sq / count as f64).sqrt() as f32;
        out_rms / core::f32::consts::FRAC_1_SQRT_2
    }

    #[test]
    fn series_lowpass_rolls_off_steeper_than_one() {
        let cutoff = 1_000.0;
        // Two lowpass in series attenuate the stopband twice as hard (in dB)
        let mut single = FilterStack::new(SAMPLE_RATE);
        single.set_filter_a(cutoff, 0.707, SvfMode::Lowpass);
        single.set_filter_b(20_000.0, 0.707, SvfMode::Lowpass); // ~open
        single.set_cutoff_mod(1.0);

        let mut double = FilterStack::new(SAMPLE_RATE);
        double.set_filter_a(cutoff, 0.707, SvfMode::Lowpass);
        double.set_filter_b(cutoff, 0.707, SvfMode::Lowpass);
        double.set_cutoff_mod(1.0);

        let single_hi = magnitude_at(&mut single, 8_000.0);
        let double_hi = magnitude_at(&mut double, 8_000.0);
        assert!(
            double_hi < single_hi * 0.5,
            "series not steeper: {double_hi} vs {single_hi}"
        );
    }

    #[test]
    fn parallel_lp_plus_hp_passes_both_ends() {
        let mut stack = FilterStack::new(SAMPLE_RATE);
        stack.set_routing(FilterRouting::Parallel);
        stack.set_filter_a(500.0, 0.707, SvfMode::Lowpass);
        stack.set_filter_b(5_000.0, 0.707, SvfMode::Highpass);
        stack.set_cutoff_mod(1.0);
        let low = magnitude_at(&mut stack, 100.0);
        let high = magnitude_at(&mut stack, 12_000.0);
        // Lowpass carries the lows, highpass carries the highs
        assert!(low > 0.3, "lows lost: {low}");
        assert!(high > 0.3, "highs lost: {high}");
    }

    #[test]
    fn cutoff_mod_opens_the_filter() {
        let mut closed = FilterStack::new(SAMPLE_RATE);
        closed.set_filter_a(500.0, 0.707, SvfMode::Lowpass);
        closed.set_filter_b(20_000.0, 0.707, SvfMode::Lowpass);
        closed.set_cutoff_mod(1.0);

        let mut opened = FilterStack::new(SAMPLE_RATE);
        opened.set_filter_a(500.0, 0.707, SvfMode::Lowpass);
        opened.set_filter_b(20_000.0, 0.707, SvfMode::Lowpass);
        opened.set_cutoff_mod(8.0); // raise cutoff three octaves

        let probe = 3_000.0;
        let closed_mag = magnitude_at(&mut closed, probe);
        let opened_mag = magnitude_at(&mut opened, probe);
        assert!(
            opened_mag > closed_mag * 2.0,
            "mod did not open: {opened_mag} vs {closed_mag}"
        );
    }

    #[test]
    fn stays_bounded() {
        let mut stack = FilterStack::new(SAMPLE_RATE);
        stack.set_filter_a(1_200.0, 4.0, SvfMode::Lowpass);
        stack.set_filter_b(3_000.0, 4.0, SvfMode::Bandpass);
        for n in 0..48_000 {
            let x = if n == 0 { 1.0 } else { 0.0 };
            let y = stack.process_sample(x);
            assert!(y.is_finite() && y.abs() < 50.0);
        }
    }
}
