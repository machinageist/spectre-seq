// =============================================================================
// File: crates/geist-dsp/src/osc/sine.rs
// Layer: DSP primitives
// Purpose: phase-accumulator sine
// Status: Implemented; accurate reference tone via std sin.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::TAU;

use crate::osc::Phasor;

// Phase-accumulator sine oscillator
// Uses std sin for an exact reference tone; the wavetable engine is the fast path
#[derive(Clone, Copy, Debug, Default)]
pub struct SineOsc {
    phasor: Phasor,
}

impl SineOsc {
    // Build a silent oscillator at zero phase
    pub fn new() -> Self {
        Self::default()
    }

    // Tune the oscillator to a frequency at a sample rate
    pub fn set_frequency(&mut self, frequency_hz: f32, sample_rate_hz: f32) {
        self.phasor.set_frequency(frequency_hz, sample_rate_hz);
    }

    // Reset to zero phase
    pub fn reset(&mut self) {
        self.phasor.reset();
    }

    // Generate the next sample in [-1, 1]
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        (TAU * self.phasor.tick()).sin()
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

    #[test]
    fn first_quarter_period_matches_known_values() {
        let mut osc = SineOsc::new();
        osc.set_frequency(1.0, 8.0); // 8 samples per cycle
        let expected = [
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            1.0,
            std::f32::consts::FRAC_1_SQRT_2,
        ];
        for &want in &expected {
            assert!((osc.next_sample() - want).abs() < 1e-6);
        }
    }

    #[test]
    fn output_stays_bounded() {
        let mut osc = SineOsc::new();
        osc.set_frequency(997.0, 44_100.0);
        let mut buf = [0.0f32; 1024];
        osc.process(&mut buf);
        assert!(buf.iter().all(|&s| (-1.0..=1.0).contains(&s)));
    }

    #[test]
    fn reset_returns_to_zero_phase() {
        let mut osc = SineOsc::new();
        osc.set_frequency(440.0, 48_000.0);
        let first = osc.next_sample();
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        assert!((osc.next_sample() - first).abs() < 1e-6);
        assert_eq!(first, 0.0);
    }

    #[test]
    fn process_is_continuous_across_blocks() {
        // One long run must equal two consecutive blocks from the same state
        let mut whole = SineOsc::new();
        whole.set_frequency(523.0, 48_000.0);
        let mut full = [0.0f32; 64];
        whole.process(&mut full);

        let mut split = SineOsc::new();
        split.set_frequency(523.0, 48_000.0);
        let mut a = [0.0f32; 32];
        let mut b = [0.0f32; 32];
        split.process(&mut a);
        split.process(&mut b);

        for i in 0..32 {
            assert!((full[i] - a[i]).abs() < 1e-6);
            assert!((full[32 + i] - b[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn frequency_is_correct_by_zero_crossings() {
        // 100 Hz over exactly one second should show ~100 rising zero crossings
        let sample_rate = 4_800.0;
        let mut osc = SineOsc::new();
        osc.set_frequency(100.0, sample_rate);
        let mut buf = vec![0.0f32; sample_rate as usize];
        osc.process(&mut buf);

        let mut rising = 0;
        for w in buf.windows(2) {
            if w[0] < 0.0 && w[1] >= 0.0 {
                rising += 1;
            }
        }
        assert!((99..=101).contains(&rising), "rising crossings = {rising}");
    }
}
