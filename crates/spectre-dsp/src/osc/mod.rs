// =============================================================================
// File: crates/spectre-dsp/src/osc/mod.rs
// Layer: DSP primitives
// Purpose: Oscillator core (Phasor) and oscillator implementations
// Status: Implemented; Phasor + sine. PolyBLEP, noise, wavetable land next.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod noise;
pub mod polyblep;
pub mod sine;
pub mod wavetable;

// Normalized phase accumulator shared by every oscillator and LFO
// Phase lives in [0, 1); the increment is frequency / sample_rate
// Forward-only: assumes increment in [0, 1), which holds below Nyquist
#[derive(Clone, Copy, Debug, Default)]
pub struct Phasor {
    phase: f32,
    increment: f32,
}

impl Phasor {
    // Build a phasor at zero phase with no increment
    pub fn new() -> Self {
        Self::default()
    }

    // Set the per-sample increment from frequency and sample rate
    pub fn set_frequency(&mut self, frequency_hz: f32, sample_rate_hz: f32) {
        self.increment = frequency_hz / sample_rate_hz;
    }

    // Current per-sample phase increment
    #[inline]
    pub fn increment(&self) -> f32 {
        self.increment
    }

    // Current phase in [0, 1)
    #[inline]
    pub fn phase(&self) -> f32 {
        self.phase
    }

    // Set absolute phase, wrapping into [0, 1)
    pub fn set_phase(&mut self, phase: f32) {
        self.phase = phase - phase.floor();
    }

    // Reset to zero phase, leaving the increment intact
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    // Return the current phase, then advance by one sample
    #[inline]
    pub fn tick(&mut self) -> f32 {
        let current = self.phase;
        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phasor_advances_and_wraps() {
        let mut p = Phasor::new();
        p.set_frequency(1.0, 4.0); // increment 0.25
        assert_eq!(p.tick(), 0.0);
        assert_eq!(p.tick(), 0.25);
        assert_eq!(p.tick(), 0.5);
        assert_eq!(p.tick(), 0.75);
        assert_eq!(p.tick(), 0.0); // wrapped
    }

    #[test]
    fn phasor_phase_stays_in_unit_range() {
        let mut p = Phasor::new();
        p.set_frequency(7000.0, 48_000.0);
        for _ in 0..10_000 {
            let phase = p.tick();
            assert!((0.0..1.0).contains(&phase));
        }
    }

    #[test]
    fn phasor_set_phase_wraps() {
        let mut p = Phasor::new();
        p.set_phase(1.25);
        assert!((p.phase() - 0.25).abs() < 1e-6);
        p.set_phase(-0.25);
        assert!((p.phase() - 0.75).abs() < 1e-6);
    }
}
