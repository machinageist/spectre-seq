// =============================================================================
// File: crates/geist-dsp/src/fx/chorus.rs
// Layer: DSP primitives
// Purpose: stereo chorus/flanger
// Status: Implemented; three-voice modulated-delay chorus over one delay line.
// Notes: One shared buffer is read at several LFO-modulated taps and summed.
//        Slightly detuned LFO rates decorrelate the voices. Buffer sized once.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::fx::DelayLine;
use crate::lfo::{Lfo, LfoWaveform};
use crate::math::lerp;

// Number of modulated voices summed into the wet signal
const VOICE_COUNT: usize = 3;

// Per-voice rate detune so the voices drift out of phase
const RATE_MULTIPLIERS: [f32; VOICE_COUNT] = [1.0, 1.27, 0.84];

// Buffer headroom; comfortably covers base delay plus modulation depth
const MAX_DELAY_MS: f32 = 60.0;

const MS_PER_SECOND: f32 = 1000.0;

// Multi-voice chorus: one delay line read at several modulated positions
#[derive(Clone, Debug)]
pub struct Chorus {
    line: DelayLine,
    lfos: [Lfo; VOICE_COUNT],
    sample_rate: f32,
    base_delay: f32,
    depth: f32,
    mix: f32,
}

impl Chorus {
    // Build a chorus at a sample rate with musical defaults
    pub fn new(sample_rate_hz: f32) -> Self {
        let max_samples = (MAX_DELAY_MS / MS_PER_SECOND * sample_rate_hz) as usize;
        let mut chorus = Self {
            line: DelayLine::new(max_samples),
            lfos: [Lfo::new(LfoWaveform::Sine); VOICE_COUNT],
            sample_rate: sample_rate_hz,
            base_delay: 0.0,
            depth: 0.0,
            mix: 0.5,
        };
        chorus.set_rate(0.8);
        chorus.set_base_delay_ms(12.0);
        chorus.set_depth_ms(4.0);
        chorus
    }

    // Set the base LFO rate in Hz; voices spread around it
    pub fn set_rate(&mut self, rate_hz: f32) {
        for (lfo, mult) in self.lfos.iter_mut().zip(RATE_MULTIPLIERS) {
            lfo.set_frequency(rate_hz * mult, self.sample_rate);
        }
    }

    // Set the center delay in milliseconds
    pub fn set_base_delay_ms(&mut self, ms: f32) {
        self.base_delay = ms / MS_PER_SECOND * self.sample_rate;
    }

    // Set the modulation depth in milliseconds, clamped to keep delay positive
    pub fn set_depth_ms(&mut self, ms: f32) {
        let samples = ms / MS_PER_SECOND * self.sample_rate;
        self.depth = samples.clamp(0.0, self.base_delay - 1.0);
    }

    // Dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Clear the delay line and reset modulation phase
    pub fn reset(&mut self) {
        self.line.clear();
        for lfo in &mut self.lfos {
            lfo.retrigger();
        }
    }

    // Process one sample
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let mut wet = 0.0;
        for lfo in &mut self.lfos {
            let modulated = self.base_delay + self.depth * lfo.next_sample();
            self.line.set_delay(modulated);
            wet += self.line.read();
        }
        wet /= VOICE_COUNT as f32;
        self.line.write(x);
        lerp(x, wet, self.mix)
    }

    // Process a buffer in place
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

    #[test]
    fn dry_mix_passes_input() {
        let mut c = Chorus::new(SAMPLE_RATE);
        c.set_mix(0.0);
        for x in [-0.5, 0.0, 0.3, 0.9] {
            assert_eq!(c.process_sample(x), x);
        }
    }

    #[test]
    fn wet_output_is_delayed_then_present() {
        let mut c = Chorus::new(SAMPLE_RATE);
        c.set_base_delay_ms(12.0); // ~576 samples
        c.set_depth_ms(4.0); // ~192 samples
        c.set_mix(1.0);

        let mut out = vec![0.0f32; 1200];
        for (n, s) in out.iter_mut().enumerate() {
            *s = c.process_sample(if n == 0 { 1.0 } else { 0.0 });
        }
        // Earliest tap is base - depth (~384); nothing before that
        let early: f32 = out[..300].iter().map(|s| s.abs()).sum();
        let around: f32 = out[400..760].iter().map(|s| s.abs()).sum();
        assert!(early < 1e-4, "energy leaked before the delay: {early}");
        assert!(around > 0.1, "no wet energy around the delay: {around}");
    }

    #[test]
    fn constant_input_settles_near_constant() {
        let mut c = Chorus::new(SAMPLE_RATE);
        c.set_mix(0.5);
        let mut y = 0.0;
        for _ in 0..10_000 {
            y = c.process_sample(0.5);
        }
        // Dry 0.5 plus wet (delayed 0.5) both sit at 0.5
        assert!((y - 0.5).abs() < 0.05, "settled at {y}");
    }

    #[test]
    fn output_stays_bounded() {
        let mut c = Chorus::new(SAMPLE_RATE);
        c.set_mix(0.5);
        for n in 0..20_000 {
            let x = (n as f32 * 0.01).sin();
            let y = c.process_sample(x);
            assert!(y.is_finite() && y.abs() <= 1.5);
        }
    }

    #[test]
    fn depth_is_clamped_below_base_delay() {
        let mut c = Chorus::new(SAMPLE_RATE);
        c.set_base_delay_ms(5.0);
        c.set_depth_ms(100.0); // absurd; must clamp
                               // Still produces finite, bounded output (delay never goes non-positive)
        for n in 0..2_000 {
            let y = c.process_sample(if n == 0 { 1.0 } else { 0.0 });
            assert!(y.is_finite());
        }
    }
}
