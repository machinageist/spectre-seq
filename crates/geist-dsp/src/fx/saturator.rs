// =============================================================================
// File: crates/geist-dsp/src/fx/saturator.rs
// Layer: DSP primitives
// Purpose: waveshaper with multiple curves
// Status: Implemented; tanh / cubic / hard-clip shapers with drive and mix.
// Notes: Stateless and odd-symmetric, so no DC and no aliasing concerns beyond
//        the curve itself. Low-level gain is unity; drive pushes into the knee.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::math::{db_to_linear, fast_tanh, lerp};

// Cubic soft clipper saturates to this magnitude (output of x - x^3/3 at |x| = 1)
const CUBIC_CLAMP: f32 = 2.0 / 3.0;

// Shaping transfer curve
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SaturationCurve {
    // Smooth tanh knee, saturating toward +/-1
    #[default]
    Tanh,
    // Cubic soft clip with unity low-level gain
    Cubic,
    // Brick-wall clip at +/-1
    HardClip,
}

// Stateless waveshaping saturator with drive, curve, makeup, and dry/wet mix
#[derive(Clone, Copy, Debug)]
pub struct Saturator {
    curve: SaturationCurve,
    drive: f32,
    output_gain: f32,
    mix: f32,
}

impl Saturator {
    // Build a saturator with the given curve at unity drive
    pub fn new(curve: SaturationCurve) -> Self {
        Self {
            curve,
            drive: 1.0,
            output_gain: 1.0,
            mix: 1.0,
        }
    }

    // Select the shaping curve
    pub fn set_curve(&mut self, curve: SaturationCurve) {
        self.curve = curve;
    }

    // Linear input gain into the shaper; higher drive saturates harder
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(0.0);
    }

    // Input drive expressed in decibels
    pub fn set_drive_db(&mut self, drive_db: f32) {
        self.drive = db_to_linear(drive_db);
    }

    // Makeup gain applied after shaping
    pub fn set_output_gain(&mut self, gain: f32) {
        self.output_gain = gain;
    }

    // Dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Apply the selected curve to a driven sample
    #[inline]
    fn shape(&self, x: f32) -> f32 {
        match self.curve {
            SaturationCurve::Tanh => fast_tanh(x),
            SaturationCurve::Cubic => {
                if x <= -1.0 {
                    -CUBIC_CLAMP
                } else if x >= 1.0 {
                    CUBIC_CLAMP
                } else {
                    x - x * x * x / 3.0
                }
            }
            SaturationCurve::HardClip => x.clamp(-1.0, 1.0),
        }
    }

    // Process one sample
    #[inline]
    pub fn process_sample(&self, x: f32) -> f32 {
        let wet = self.shape(x * self.drive) * self.output_gain;
        lerp(x, wet, self.mix)
    }

    // Process a buffer in place
    pub fn process(&self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curves() -> [SaturationCurve; 3] {
        [
            SaturationCurve::Tanh,
            SaturationCurve::Cubic,
            SaturationCurve::HardClip,
        ]
    }

    #[test]
    fn low_level_is_near_transparent() {
        for c in curves() {
            let s = Saturator::new(c);
            let y = s.process_sample(0.05);
            assert!((y - 0.05).abs() < 1e-3, "{c:?} low-level = {y}");
        }
    }

    #[test]
    fn curves_are_odd_symmetric() {
        for c in curves() {
            let mut s = Saturator::new(c);
            s.set_drive(3.0);
            for &x in &[0.1, 0.5, 1.0, 2.0] {
                assert!(
                    (s.process_sample(x) + s.process_sample(-x)).abs() < 1e-6,
                    "{c:?} not odd at {x}"
                );
            }
        }
    }

    #[test]
    fn high_drive_compresses_and_bounds() {
        for c in curves() {
            let mut s = Saturator::new(c);
            s.set_drive(10.0);
            let y = s.process_sample(1.0);
            // Output is far below the driven level and within the curve limit
            assert!(y.abs() <= 1.0001, "{c:?} exceeded limit: {y}");
            assert!(y < 10.0);
        }
    }

    #[test]
    fn shaper_is_monotonic() {
        for c in curves() {
            let mut s = Saturator::new(c);
            s.set_drive(2.0);
            let mut prev = s.process_sample(-3.0);
            let mut x = -3.0;
            while x <= 3.0 {
                let y = s.process_sample(x);
                assert!(y >= prev - 1e-6, "{c:?} non-monotonic at {x}");
                prev = y;
                x += 0.05;
            }
        }
    }

    #[test]
    fn dry_mix_passes_input() {
        let mut s = Saturator::new(SaturationCurve::HardClip);
        s.set_drive(5.0);
        s.set_mix(0.0);
        assert_eq!(s.process_sample(0.9), 0.9);
    }

    #[test]
    fn drive_db_matches_linear() {
        let mut a = Saturator::new(SaturationCurve::Tanh);
        a.set_drive_db(6.0206); // ~2x
        let mut b = Saturator::new(SaturationCurve::Tanh);
        b.set_drive(2.0);
        assert!((a.process_sample(0.3) - b.process_sample(0.3)).abs() < 1e-3);
    }

    #[test]
    fn output_gain_scales_wet() {
        let mut s = Saturator::new(SaturationCurve::HardClip);
        s.set_output_gain(0.5);
        // Hard clip of a small signal is the signal; gain halves it
        assert!((s.process_sample(0.4) - 0.2).abs() < 1e-6);
    }
}
