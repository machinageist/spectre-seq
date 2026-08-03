// =============================================================================
// File: crates/spectre-dsp/src/osc/polyblep.rs
// Layer: DSP primitives
// Purpose: bandlimited saw, square, tri
// Status: Implemented; PolyBLEP saw/square + PolyBLAMP triangle.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::math::{poly_blamp, poly_blep};
use crate::osc::Phasor;

// Slope magnitude of a unit triangle, in value per unit phase
const TRIANGLE_SLOPE: f32 = 4.0;

// Selectable bandlimited waveform
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Waveform {
    #[default]
    Saw,
    Square,
    Triangle,
}

// Bandlimited analog-style oscillator using PolyBLEP edge correction
// One phasor drives a switchable waveform; corrections localize to each discontinuity
#[derive(Clone, Copy, Debug, Default)]
pub struct PolyBlepOsc {
    phasor: Phasor,
    waveform: Waveform,
}

impl PolyBlepOsc {
    // Build an oscillator producing the given waveform
    pub fn new(waveform: Waveform) -> Self {
        Self {
            phasor: Phasor::new(),
            waveform,
        }
    }

    // Tune the oscillator to a frequency at a sample rate
    pub fn set_frequency(&mut self, frequency_hz: f32, sample_rate_hz: f32) {
        self.phasor.set_frequency(frequency_hz, sample_rate_hz);
    }

    // Switch the produced waveform, keeping phase continuous
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    // Reset to zero phase
    pub fn reset(&mut self) {
        self.phasor.reset();
    }

    // Generate the next bandlimited sample
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let dt = self.phasor.increment();
        let phase = self.phasor.tick();
        match self.waveform {
            Waveform::Saw => Self::saw(phase, dt),
            Waveform::Square => Self::square(phase, dt),
            Waveform::Triangle => Self::triangle(phase, dt),
        }
    }

    // Fill a buffer with successive samples
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }

    // Bandlimited rising saw in roughly [-1, 1]
    // Naive ramp jumps down by 2 at wrap; PolyBLEP subtracts the residual
    #[inline]
    fn saw(phase: f32, dt: f32) -> f32 {
        let naive = 2.0 * phase - 1.0;
        naive - poly_blep(phase, dt)
    }

    // Bandlimited square (50% duty) in roughly [-1, 1]
    // Up-step at phase 0 is added; down-step at phase 0.5 is subtracted
    #[inline]
    fn square(phase: f32, dt: f32) -> f32 {
        let naive = if phase < 0.5 { 1.0 } else { -1.0 };
        let mut half = phase + 0.5;
        if half >= 1.0 {
            half -= 1.0;
        }
        naive + poly_blep(phase, dt) - poly_blep(half, dt)
    }

    // Bandlimited triangle in roughly [-1, 1]
    // Naive corners at phase 0 (+slope) and 0.5 (-slope) get PolyBLAMP correction
    #[inline]
    fn triangle(phase: f32, dt: f32) -> f32 {
        let naive = if phase < 0.5 {
            TRIANGLE_SLOPE * phase - 1.0
        } else {
            3.0 - TRIANGLE_SLOPE * phase
        };
        let mut half = phase + 0.5;
        if half >= 1.0 {
            half -= 1.0;
        }
        // (slope_change / 2) = slope; +slope at the wrap corner, -slope at the 0.5 corner
        naive + TRIANGLE_SLOPE * dt * (poly_blamp(phase, dt) - poly_blamp(half, dt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Phase increment for a tone well below Nyquist
    const DT: f32 = 0.01;

    #[test]
    fn saw_equals_naive_away_from_the_edge() {
        // Outside dt of the wrap, PolyBLEP contributes nothing
        assert_eq!(PolyBlepOsc::saw(0.25, DT), -0.5);
        assert_eq!(PolyBlepOsc::saw(0.5, DT), 0.0);
        assert_eq!(PolyBlepOsc::saw(0.75, DT), 0.5);
        // At the discontinuity the output lands on the midpoint of the jump
        assert!((PolyBlepOsc::saw(0.0, DT) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn square_equals_naive_away_from_edges() {
        assert_eq!(PolyBlepOsc::square(0.25, DT), 1.0);
        assert_eq!(PolyBlepOsc::square(0.75, DT), -1.0);
        // Both discontinuities resolve to the midpoint of their step
        assert!((PolyBlepOsc::square(0.0, DT) - 0.0).abs() < 1e-6);
        assert!((PolyBlepOsc::square(0.5, DT) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn correction_is_active_only_near_edges() {
        // Within dt of an edge the bandlimited value departs from the naive one
        let near = 0.002; // < DT
        let naive_saw = 2.0 * near - 1.0;
        assert!((PolyBlepOsc::saw(near, DT) - naive_saw).abs() > 1e-3);
    }

    #[test]
    fn output_is_finite_and_bounded_at_high_frequency() {
        // A steep tone keeps PolyBLEP busy; output stays finite and bounded
        let mut osc = PolyBlepOsc::new(Waveform::Saw);
        osc.set_frequency(8_000.0, 48_000.0);
        let mut buf = [0.0f32; 2048];
        osc.process(&mut buf);
        assert!(buf.iter().all(|s| s.is_finite() && s.abs() <= 2.0));
    }

    #[test]
    fn saw_has_near_zero_dc_over_whole_cycles() {
        // 100 Hz over one second is an integer number of cycles
        let mut osc = PolyBlepOsc::new(Waveform::Saw);
        osc.set_frequency(100.0, 4_800.0);
        let mut buf = vec![0.0f32; 4_800];
        osc.process(&mut buf);
        let mean: f32 = buf.iter().sum::<f32>() / buf.len() as f32;
        assert!(mean.abs() < 1e-2, "saw DC offset too large: {mean}");
    }

    #[test]
    fn square_duty_is_balanced() {
        let mut osc = PolyBlepOsc::new(Waveform::Square);
        osc.set_frequency(100.0, 4_800.0);
        let mut buf = vec![0.0f32; 4_800];
        osc.process(&mut buf);
        let positive = buf.iter().filter(|&&s| s > 0.0).count();
        let negative = buf.iter().filter(|&&s| s < 0.0).count();
        assert!((positive as i32 - negative as i32).abs() < 50);
    }

    #[test]
    fn triangle_equals_naive_away_from_corners() {
        assert_eq!(PolyBlepOsc::triangle(0.125, DT), -0.5);
        assert_eq!(PolyBlepOsc::triangle(0.25, DT), 0.0);
        assert_eq!(PolyBlepOsc::triangle(0.75, DT), 0.0);
    }

    #[test]
    fn triangle_corners_are_rounded_symmetrically() {
        // Band-limiting rounds the peak/trough by slope*dt/3 toward zero
        let rounding = TRIANGLE_SLOPE * DT / 3.0;
        let peak = PolyBlepOsc::triangle(0.5, DT);
        let trough = PolyBlepOsc::triangle(0.0, DT);
        assert!((peak - (1.0 - rounding)).abs() < 1e-5, "peak = {peak}");
        assert!(
            (trough - (-1.0 + rounding)).abs() < 1e-5,
            "trough = {trough}"
        );
        // Symmetric about zero
        assert!((peak + trough).abs() < 1e-6);
    }

    #[test]
    fn triangle_is_bounded_with_near_zero_dc() {
        let mut osc = PolyBlepOsc::new(Waveform::Triangle);
        osc.set_frequency(100.0, 4_800.0);
        let mut buf = vec![0.0f32; 4_800];
        osc.process(&mut buf);
        assert!(buf.iter().all(|s| s.is_finite() && s.abs() <= 1.05));
        let mean: f32 = buf.iter().sum::<f32>() / buf.len() as f32;
        assert!(mean.abs() < 1e-2, "triangle DC offset too large: {mean}");
    }

    #[test]
    fn process_is_continuous_across_blocks() {
        let mut whole = PolyBlepOsc::new(Waveform::Saw);
        whole.set_frequency(220.0, 48_000.0);
        let mut full = [0.0f32; 64];
        whole.process(&mut full);

        let mut split = PolyBlepOsc::new(Waveform::Saw);
        split.set_frequency(220.0, 48_000.0);
        let mut a = [0.0f32; 32];
        let mut b = [0.0f32; 32];
        split.process(&mut a);
        split.process(&mut b);

        for i in 0..32 {
            assert_eq!(full[i], a[i]);
            assert_eq!(full[32 + i], b[i]);
        }
    }
}
