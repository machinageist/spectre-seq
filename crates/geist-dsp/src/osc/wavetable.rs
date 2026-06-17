// =============================================================================
// File: crates/geist-dsp/src/osc/wavetable.rs
// Layer: DSP primitives
// Purpose: wavetable engine with morph + linear interp
// Status: Implemented; single-cycle linear-interp read + two-table morph.
// Notes: Naive read aliases at high notes; band-limited mip selection is a
//        future enhancement once table generation lands in the synth layer.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::TAU;

use crate::math::lerp;
use crate::osc::Phasor;

// Single-cycle waveform read with linear interpolation
// Length is a power of two so phase wrap is a cheap mask, not a modulo
#[derive(Clone, Debug)]
pub struct Wavetable {
    samples: Vec<f32>,
    mask: usize,
}

impl Wavetable {
    // Wrap owned single-cycle data; length must be a power of two >= 2
    pub fn new(samples: Vec<f32>) -> Self {
        assert!(samples.len() >= 2, "wavetable needs at least two samples");
        assert!(
            samples.len().is_power_of_two(),
            "wavetable length must be a power of two"
        );
        let mask = samples.len() - 1;
        Self { samples, mask }
    }

    // Build a single-cycle sine of the given power-of-two length
    pub fn sine(len: usize) -> Self {
        let mut samples = vec![0.0; len];
        for (i, sample) in samples.iter_mut().enumerate() {
            *sample = (TAU * i as f32 / len as f32).sin();
        }
        Self::new(samples)
    }

    // Number of samples in the cycle
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    // Always false for a constructed table; present for API completeness
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    // Read with linear interpolation at normalized phase in [0, 1)
    // The final sample interpolates back to the first for a seamless cycle
    #[inline]
    pub fn sample(&self, phase: f32) -> f32 {
        let scaled = phase * self.samples.len() as f32;
        let floor = scaled.floor();
        let i0 = floor as usize & self.mask;
        let i1 = (i0 + 1) & self.mask;
        lerp(self.samples[i0], self.samples[i1], scaled - floor)
    }
}

// Wavetable oscillator: a phase accumulator reading borrowed tables
// Tables live with the caller (a synth voice); the oscillator allocates nothing
#[derive(Clone, Copy, Debug, Default)]
pub struct WavetableOsc {
    phasor: Phasor,
}

impl WavetableOsc {
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

    // Next sample read from a single table
    #[inline]
    pub fn next_sample(&mut self, table: &Wavetable) -> f32 {
        table.sample(self.phasor.tick())
    }

    // Next sample morphing two equal-length tables by `morph` in [0, 1]
    #[inline]
    pub fn next_morphed(&mut self, a: &Wavetable, b: &Wavetable, morph: f32) -> f32 {
        debug_assert_eq!(a.len(), b.len(), "morph tables must match length");
        let phase = self.phasor.tick();
        lerp(a.sample(phase), b.sample(phase), morph)
    }

    // Fill a buffer from a single table
    pub fn process(&mut self, table: &Wavetable, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample(table);
        }
    }

    // Fill a buffer morphing two equal-length tables
    pub fn process_morphed(
        &mut self,
        a: &Wavetable,
        b: &Wavetable,
        morph: f32,
        output: &mut [f32],
    ) {
        for sample in output.iter_mut() {
            *sample = self.next_morphed(a, b, morph);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_interpolation_hits_known_points() {
        // Ramp table 0,1,2,3 of length 4 (power of two)
        let table = Wavetable::new(vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(table.sample(0.0), 0.0);
        // Index 0.5 interpolates between samples 0 and 1
        assert_eq!(table.sample(0.125), 0.5);
        // Index 3.5 wraps: interpolates between sample 3 and sample 0
        assert_eq!(table.sample(0.875), 1.5);
    }

    #[test]
    fn sine_table_matches_quarter_phases() {
        let table = Wavetable::sine(2048);
        assert!(table.sample(0.0).abs() < 1e-3);
        assert!((table.sample(0.25) - 1.0).abs() < 1e-3);
        assert!(table.sample(0.5).abs() < 1e-3);
        assert!((table.sample(0.75) + 1.0).abs() < 1e-3);
    }

    #[test]
    #[should_panic(expected = "power of two")]
    fn non_power_of_two_length_is_rejected() {
        let _ = Wavetable::new(vec![0.0, 1.0, 2.0]);
    }

    #[test]
    #[should_panic(expected = "at least two")]
    fn single_sample_table_is_rejected() {
        let _ = Wavetable::new(vec![0.0]);
    }

    #[test]
    fn osc_reads_sine_at_correct_frequency() {
        let table = Wavetable::sine(2048);
        let mut osc = WavetableOsc::new();
        osc.set_frequency(100.0, 4_800.0);
        let mut buf = vec![0.0f32; 4_800];
        osc.process(&table, &mut buf);

        assert!(buf.iter().all(|&s| (-1.001..=1.001).contains(&s)));
        let mut rising = 0;
        for w in buf.windows(2) {
            if w[0] < 0.0 && w[1] >= 0.0 {
                rising += 1;
            }
        }
        assert!((99..=101).contains(&rising), "rising crossings = {rising}");
    }

    #[test]
    fn morph_blends_between_tables() {
        let a = Wavetable::new(vec![0.0; 4]);
        let b = Wavetable::new(vec![1.0; 4]);

        let mut at_a = WavetableOsc::new();
        let mut at_b = WavetableOsc::new();
        let mut at_mid = WavetableOsc::new();
        // Constant tables make the morph value the only variable
        assert_eq!(at_a.next_morphed(&a, &b, 0.0), 0.0);
        assert_eq!(at_b.next_morphed(&a, &b, 1.0), 1.0);
        assert_eq!(at_mid.next_morphed(&a, &b, 0.5), 0.5);
    }

    #[test]
    fn process_is_continuous_across_blocks() {
        let table = Wavetable::sine(1024);

        let mut whole = WavetableOsc::new();
        whole.set_frequency(220.0, 48_000.0);
        let mut full = [0.0f32; 64];
        whole.process(&table, &mut full);

        let mut split = WavetableOsc::new();
        split.set_frequency(220.0, 48_000.0);
        let mut a = [0.0f32; 32];
        let mut b = [0.0f32; 32];
        split.process(&table, &mut a);
        split.process(&table, &mut b);

        for i in 0..32 {
            assert_eq!(full[i], a[i]);
            assert_eq!(full[32 + i], b[i]);
        }
    }
}
