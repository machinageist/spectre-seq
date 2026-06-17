// =============================================================================
// File: crates/geist-dsp/src/filter/biquad.rs
// Layer: DSP primitives
// Purpose: direct form II biquad (EQ building block)
// Status: Implemented; RBJ cookbook coefficients, transposed direct form II.
// Notes: TDF-II keeps float state small and well-conditioned; one section here,
//        cascade externally for steeper slopes.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::TAU;

// Center/cutoff is clamped below Nyquist; Q is floored to stay finite
const MIN_FREQ_HZ: f32 = 1.0;
const MAX_FREQ_RATIO: f32 = 0.49;
const MIN_Q: f32 = 0.025;

// Second-order IIR section with normalized coefficients (a0 == 1)
// Configure with one of the set_* methods, then process samples
#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    s1: f32,
    s2: f32,
}

impl Default for Biquad {
    // Identity passthrough until configured
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            s1: 0.0,
            s2: 0.0,
        }
    }
}

impl Biquad {
    // Build an identity (passthrough) section
    pub fn new() -> Self {
        Self::default()
    }

    // Clear delay state deterministically; coefficients are untouched
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    // Process one sample through the transposed direct form II structure
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.s1;
        self.s1 = self.b1 * x - self.a1 * y + self.s2;
        self.s2 = self.b2 * x - self.a2 * y;
        y
    }

    // Filter a buffer in place
    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }

    // Lowpass at cutoff with resonance q
    pub fn set_lowpass(&mut self, freq_hz: f32, q: f32, sample_rate_hz: f32) {
        let (cos_w0, alpha) = self.shape(freq_hz, q, sample_rate_hz);
        let b1 = 1.0 - cos_w0;
        self.normalize(
            b1 * 0.5,
            b1,
            b1 * 0.5,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        );
    }

    // Highpass at cutoff with resonance q
    pub fn set_highpass(&mut self, freq_hz: f32, q: f32, sample_rate_hz: f32) {
        let (cos_w0, alpha) = self.shape(freq_hz, q, sample_rate_hz);
        let b1 = -(1.0 + cos_w0);
        self.normalize(
            (1.0 + cos_w0) * 0.5,
            b1,
            (1.0 + cos_w0) * 0.5,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        );
    }

    // Bandpass with constant 0 dB peak gain at center
    pub fn set_bandpass(&mut self, freq_hz: f32, q: f32, sample_rate_hz: f32) {
        let (cos_w0, alpha) = self.shape(freq_hz, q, sample_rate_hz);
        self.normalize(alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha);
    }

    // Notch rejecting the center frequency
    pub fn set_notch(&mut self, freq_hz: f32, q: f32, sample_rate_hz: f32) {
        let (cos_w0, alpha) = self.shape(freq_hz, q, sample_rate_hz);
        self.normalize(
            1.0,
            -2.0 * cos_w0,
            1.0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        );
    }

    // Peaking EQ: boost or cut gain_db around the center
    pub fn set_peaking(&mut self, freq_hz: f32, q: f32, gain_db: f32, sample_rate_hz: f32) {
        let (cos_w0, alpha) = self.shape(freq_hz, q, sample_rate_hz);
        let a = Self::shelf_gain(gain_db);
        self.normalize(
            1.0 + alpha * a,
            -2.0 * cos_w0,
            1.0 - alpha * a,
            1.0 + alpha / a,
            -2.0 * cos_w0,
            1.0 - alpha / a,
        );
    }

    // Low shelf: boost or cut gain_db below the corner
    pub fn set_low_shelf(&mut self, freq_hz: f32, q: f32, gain_db: f32, sample_rate_hz: f32) {
        let (cos_w0, alpha) = self.shape(freq_hz, q, sample_rate_hz);
        let a = Self::shelf_gain(gain_db);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        self.normalize(
            a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha),
            2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
            a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha),
            (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha,
            -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
            (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha,
        );
    }

    // High shelf: boost or cut gain_db above the corner
    pub fn set_high_shelf(&mut self, freq_hz: f32, q: f32, gain_db: f32, sample_rate_hz: f32) {
        let (cos_w0, alpha) = self.shape(freq_hz, q, sample_rate_hz);
        let a = Self::shelf_gain(gain_db);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        self.normalize(
            a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
            a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha),
            (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha,
            2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
            (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha,
        );
    }

    // Shared intermediate terms: cos(w0) and alpha = sin(w0) / (2Q)
    fn shape(&self, freq_hz: f32, q: f32, sample_rate_hz: f32) -> (f32, f32) {
        let max_freq = MAX_FREQ_RATIO * sample_rate_hz;
        let freq = freq_hz.clamp(MIN_FREQ_HZ, max_freq);
        let q = q.max(MIN_Q);
        let w0 = TAU * freq / sample_rate_hz;
        let alpha = w0.sin() / (2.0 * q);
        (w0.cos(), alpha)
    }

    // Cookbook A term; gain_db/40 because A is the square root of linear gain
    fn shelf_gain(gain_db: f32) -> f32 {
        10.0_f32.powf(gain_db / 40.0)
    }

    // Store coefficients normalized so a0 == 1
    fn normalize(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        let inv = 1.0 / a0;
        self.b0 = b0 * inv;
        self.b1 = b1 * inv;
        self.b2 = b2 * inv;
        self.a1 = a1 * inv;
        self.a2 = a2 * inv;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;
    const BUTTERWORTH_Q: f32 = std::f32::consts::FRAC_1_SQRT_2;

    // Steady-state amplitude response of a configured filter at test_hz
    fn magnitude_at(filter: &mut Biquad, test_hz: f32) -> f32 {
        let total = 48_000usize;
        let settle = 24_000usize;
        let mut sum_sq = 0.0f64;
        let mut count = 0u32;
        for n in 0..total {
            let phase = TAU * test_hz * n as f32 / SAMPLE_RATE;
            let out = filter.process_sample(phase.sin());
            if n >= settle {
                sum_sq += (out as f64) * (out as f64);
                count += 1;
            }
        }
        let out_rms = (sum_sq / count as f64).sqrt() as f32;
        out_rms / core::f32::consts::FRAC_1_SQRT_2
    }

    // Steady-state DC gain
    fn dc_gain(filter: &mut Biquad) -> f32 {
        let mut y = 0.0;
        for _ in 0..20_000 {
            y = filter.process_sample(1.0);
        }
        y
    }

    #[test]
    fn default_is_passthrough() {
        let mut bq = Biquad::new();
        for x in [-0.7, 0.0, 0.3, 1.0, -1.0] {
            assert_eq!(bq.process_sample(x), x);
        }
    }

    #[test]
    fn lowpass_response_is_correct() {
        let mut bq = Biquad::new();
        bq.set_lowpass(2_000.0, BUTTERWORTH_Q, SAMPLE_RATE);
        assert!((magnitude_at(&mut bq, 2_000.0) - core::f32::consts::FRAC_1_SQRT_2).abs() < 0.03);

        let mut dc = Biquad::new();
        dc.set_lowpass(2_000.0, BUTTERWORTH_Q, SAMPLE_RATE);
        assert!((dc_gain(&mut dc) - 1.0).abs() < 1e-3);

        let mut hi = Biquad::new();
        hi.set_lowpass(2_000.0, BUTTERWORTH_Q, SAMPLE_RATE);
        assert!(magnitude_at(&mut hi, 16_000.0) < 0.05);
    }

    #[test]
    fn highpass_blocks_dc() {
        let mut bq = Biquad::new();
        bq.set_highpass(2_000.0, BUTTERWORTH_Q, SAMPLE_RATE);
        assert!(dc_gain(&mut bq).abs() < 1e-3);
        let mut hi = Biquad::new();
        hi.set_highpass(2_000.0, BUTTERWORTH_Q, SAMPLE_RATE);
        assert!((magnitude_at(&mut hi, 16_000.0) - 1.0).abs() < 0.05);
    }

    #[test]
    fn notch_rejects_center() {
        let mut bq = Biquad::new();
        bq.set_notch(2_000.0, 5.0, SAMPLE_RATE);
        assert!(magnitude_at(&mut bq, 2_000.0) < 0.15);
        let mut edge = Biquad::new();
        edge.set_notch(2_000.0, 5.0, SAMPLE_RATE);
        assert!(magnitude_at(&mut edge, 200.0) > 0.85);
    }

    #[test]
    fn bandpass_peaks_at_center() {
        let mut center = Biquad::new();
        center.set_bandpass(2_000.0, 2.0, SAMPLE_RATE);
        let at_center = magnitude_at(&mut center, 2_000.0);
        let mut low = Biquad::new();
        low.set_bandpass(2_000.0, 2.0, SAMPLE_RATE);
        let below = magnitude_at(&mut low, 200.0);
        assert!(
            (at_center - 1.0).abs() < 0.1,
            "BP center gain = {at_center}"
        );
        assert!(at_center > below * 3.0);
    }

    #[test]
    fn peaking_boosts_and_cuts_at_center() {
        let mut boost = Biquad::new();
        boost.set_peaking(2_000.0, 3.0, 6.0, SAMPLE_RATE);
        let g = magnitude_at(&mut boost, 2_000.0);
        // +6 dB is a linear factor of ~1.995
        assert!((g - 1.995).abs() < 0.15, "peak boost = {g}");

        let mut cut = Biquad::new();
        cut.set_peaking(2_000.0, 3.0, -6.0, SAMPLE_RATE);
        let gc = magnitude_at(&mut cut, 2_000.0);
        assert!((gc - 0.501).abs() < 0.1, "peak cut = {gc}");
    }

    #[test]
    fn low_shelf_lifts_dc() {
        let mut bq = Biquad::new();
        bq.set_low_shelf(2_000.0, BUTTERWORTH_Q, 6.0, SAMPLE_RATE);
        // +6 dB shelf => DC gain ~1.995, high frequencies near unity
        assert!((dc_gain(&mut bq) - 1.995).abs() < 0.1);
        let mut hi = Biquad::new();
        hi.set_low_shelf(2_000.0, BUTTERWORTH_Q, 6.0, SAMPLE_RATE);
        assert!((magnitude_at(&mut hi, 18_000.0) - 1.0).abs() < 0.1);
    }

    #[test]
    fn high_shelf_lifts_treble() {
        let mut bq = Biquad::new();
        bq.set_high_shelf(2_000.0, BUTTERWORTH_Q, 6.0, SAMPLE_RATE);
        assert!((dc_gain(&mut bq) - 1.0).abs() < 0.05);
        let mut hi = Biquad::new();
        hi.set_high_shelf(2_000.0, BUTTERWORTH_Q, 6.0, SAMPLE_RATE);
        assert!((magnitude_at(&mut hi, 18_000.0) - 1.995).abs() < 0.15);
    }

    #[test]
    fn impulse_is_stable() {
        let mut bq = Biquad::new();
        bq.set_lowpass(5_000.0, 8.0, SAMPLE_RATE);
        let mut peak = 0.0f32;
        for n in 0..48_000 {
            let x = if n == 0 { 1.0 } else { 0.0 };
            let y = bq.process_sample(x);
            assert!(y.is_finite());
            peak = peak.max(y.abs());
        }
        assert!(peak < 100.0);
    }

    #[test]
    fn reset_matches_fresh_instance() {
        let mut used = Biquad::new();
        used.set_lowpass(1_000.0, 2.0, SAMPLE_RATE);
        for _ in 0..500 {
            used.process_sample(0.8);
        }
        used.reset();
        let mut fresh = Biquad::new();
        fresh.set_lowpass(1_000.0, 2.0, SAMPLE_RATE);
        assert_eq!(used.process_sample(0.4), fresh.process_sample(0.4));
    }
}
