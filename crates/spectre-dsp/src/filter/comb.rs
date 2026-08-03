// =============================================================================
// File: crates/spectre-dsp/src/filter/comb.rs
// Layer: DSP primitives
// Purpose: comb filter for flanging/chorus
// Status: Implemented; Zolzer universal comb with fractional (linear) delay.
// Notes: blend + feedforward + feedback covers flanger, chorus, and tuned delay
//        from one structure. Buffer is sized once; process() never allocates.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::math::lerp;

// Shortest delay that still leaves room for fractional interpolation
const MIN_DELAY_SAMPLES: f32 = 1.0;

// Feedback magnitude bound that keeps the recursive path stable
const MAX_FEEDBACK: f32 = 0.999;

// Universal comb: y = FF * d + BL * (x - FB * d), where d is the fractional delay tap
// Buffer length is a power of two so the circular index wraps with a mask
#[derive(Clone, Debug)]
pub struct Comb {
    buffer: Vec<f32>,
    mask: usize,
    write_index: usize,
    delay: f32,
    feedback: f32,
    feedforward: f32,
    blend: f32,
}

impl Comb {
    // Allocate a comb able to delay up to `max_delay_samples`; defaults to passthrough
    pub fn new(max_delay_samples: usize) -> Self {
        let len = (max_delay_samples + 2).next_power_of_two();
        Self {
            buffer: vec![0.0; len],
            mask: len - 1,
            write_index: 0,
            delay: MIN_DELAY_SAMPLES,
            feedback: 0.0,
            feedforward: 0.0,
            blend: 1.0,
        }
    }

    // Set the (fractional) delay in samples, clamped to the buffer capacity
    pub fn set_delay_samples(&mut self, delay: f32) {
        let max = (self.buffer.len() - 2) as f32;
        self.delay = delay.clamp(MIN_DELAY_SAMPLES, max);
    }

    // Set the recursive feedback gain, clamped to stay stable
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-MAX_FEEDBACK, MAX_FEEDBACK);
    }

    // Set the feedforward (delayed-tap) gain
    pub fn set_feedforward(&mut self, feedforward: f32) {
        self.feedforward = feedforward;
    }

    // Set the dry/blend gain of the direct path
    pub fn set_blend(&mut self, blend: f32) {
        self.blend = blend;
    }

    // Clear the delay line deterministically
    pub fn reset(&mut self) {
        self.buffer.iter_mut().for_each(|s| *s = 0.0);
        self.write_index = 0;
    }

    // Process one sample
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let d_int = self.delay as usize;
        let frac = self.delay - d_int as f32;
        // Two taps straddling the fractional delay; larger delay reads further back
        let i0 = self.write_index.wrapping_sub(d_int) & self.mask;
        let i1 = self.write_index.wrapping_sub(d_int + 1) & self.mask;
        let delayed = lerp(self.buffer[i0], self.buffer[i1], frac);

        let written = x - self.feedback * delayed;
        self.buffer[self.write_index] = written;
        self.write_index = (self.write_index + 1) & self.mask;

        self.feedforward * delayed + self.blend * written
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
    fn magnitude_at(comb: &mut Comb, test_hz: f32) -> f32 {
        let total = 48_000usize;
        let settle = 24_000usize;
        let mut sum_sq = 0.0f64;
        let mut count = 0u32;
        for n in 0..total {
            let phase = core::f32::consts::TAU * test_hz * n as f32 / SAMPLE_RATE;
            let out = comb.process_sample(phase.sin());
            if n >= settle {
                sum_sq += (out as f64) * (out as f64);
                count += 1;
            }
        }
        let out_rms = (sum_sq / count as f64).sqrt() as f32;
        out_rms / core::f32::consts::FRAC_1_SQRT_2
    }

    // Impulse response of the first `len` samples
    fn impulse_response(comb: &mut Comb, len: usize) -> Vec<f32> {
        (0..len)
            .map(|n| comb.process_sample(if n == 0 { 1.0 } else { 0.0 }))
            .collect()
    }

    #[test]
    fn default_passes_signal_through() {
        let mut comb = Comb::new(256);
        for x in [-0.5, 0.0, 0.2, 1.0, -1.0] {
            assert_eq!(comb.process_sample(x), x);
        }
    }

    #[test]
    fn feedforward_taps_the_delay() {
        let mut comb = Comb::new(256);
        comb.set_delay_samples(32.0);
        comb.set_feedforward(1.0);
        comb.set_blend(1.0);
        let ir = impulse_response(&mut comb, 64);
        // Direct hit at 0, delayed tap at 32, silence between
        assert!((ir[0] - 1.0).abs() < 1e-6);
        assert!((ir[32] - 1.0).abs() < 1e-6);
        assert!(ir[1..32].iter().all(|&s| s.abs() < 1e-6));
    }

    #[test]
    fn feedback_produces_decaying_echoes() {
        let mut comb = Comb::new(256);
        comb.set_delay_samples(16.0);
        comb.set_feedback(0.5);
        comb.set_blend(1.0);
        let ir = impulse_response(&mut comb, 64);
        // written = x - FB*delayed; echoes alternate sign and decay by 0.5
        assert!((ir[0] - 1.0).abs() < 1e-6);
        assert!((ir[16] - -0.5).abs() < 1e-6);
        assert!((ir[32] - 0.25).abs() < 1e-6);
        assert!((ir[48] - -0.125).abs() < 1e-6);
    }

    #[test]
    fn fractional_delay_splits_across_two_samples() {
        let mut comb = Comb::new(256);
        comb.set_delay_samples(32.5);
        comb.set_feedforward(1.0);
        comb.set_blend(1.0);
        let ir = impulse_response(&mut comb, 64);
        // The delayed impulse lands between samples 32 and 33
        assert!(ir[32] > 0.0 && ir[33] > 0.0);
        assert!((ir[32] + ir[33] - 1.0).abs() < 1e-5, "energy not conserved");
        assert!((ir[32] - 0.5).abs() < 0.05);
    }

    #[test]
    fn feedforward_comb_notches_and_peaks() {
        // y = x + x[n-D]: peaks at fs/D, first notch at fs/(2D)
        let delay = 48.0; // fs/D = 1000 Hz, notch at 500 Hz
        let mut peak = Comb::new(256);
        peak.set_delay_samples(delay);
        peak.set_feedforward(1.0);
        peak.set_blend(1.0);
        let peak_mag = magnitude_at(&mut peak, 1_000.0);

        let mut notch = Comb::new(256);
        notch.set_delay_samples(delay);
        notch.set_feedforward(1.0);
        notch.set_blend(1.0);
        let notch_mag = magnitude_at(&mut notch, 500.0);

        assert!((peak_mag - 2.0).abs() < 0.1, "comb peak = {peak_mag}");
        assert!(notch_mag < 0.1, "comb notch = {notch_mag}");
    }

    #[test]
    fn high_feedback_stays_bounded() {
        let mut comb = Comb::new(256);
        comb.set_delay_samples(64.0);
        comb.set_feedback(5.0); // clamped to MAX_FEEDBACK
        comb.set_blend(1.0);
        let mut peak = 0.0f32;
        for n in 0..96_000 {
            let x = if n == 0 { 1.0 } else { 0.0 };
            let y = comb.process_sample(x);
            assert!(y.is_finite());
            peak = peak.max(y.abs());
        }
        assert!(peak < 100.0, "comb feedback unbounded: {peak}");
    }

    #[test]
    fn reset_clears_the_delay_line() {
        let mut comb = Comb::new(256);
        comb.set_delay_samples(20.0);
        comb.set_feedforward(0.8);
        for _ in 0..100 {
            comb.process_sample(0.7);
        }
        comb.reset();
        let mut fresh = Comb::new(256);
        fresh.set_delay_samples(20.0);
        fresh.set_feedforward(0.8);
        assert_eq!(comb.process_sample(0.3), fresh.process_sample(0.3));
    }
}
