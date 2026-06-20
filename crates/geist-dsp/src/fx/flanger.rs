// =============================================================================
// File: crates/geist-dsp/src/fx/flanger.rs
// Layer: DSP primitives
// Purpose: LFO-modulated short delay with feedback (flanger)
// Status: Implemented; single modulated tap with feedback over one delay line.
// Notes: Distinct from chorus: shorter delay and a feedback path, giving the
//        characteristic resonant comb sweep. One buffer sized once at new().
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::fx::DelayLine;
use crate::lfo::{Lfo, LfoWaveform};
use crate::math::lerp;

// Buffer headroom in milliseconds; covers base delay plus modulation depth
const MAX_DELAY_MS: f32 = 20.0;
const MS_PER_SECOND: f32 = 1000.0;

// Modulated short-delay flanger with feedback
#[derive(Clone, Debug)]
pub struct Flanger {
    line: DelayLine,
    lfo: Lfo,
    sample_rate: f32,
    base_delay: f32,
    depth: f32,
    feedback: f32,
    mix: f32,
}

impl Flanger {
    // Build a flanger at a sample rate with musical defaults
    pub fn new(sample_rate_hz: f32) -> Self {
        let max_samples = (MAX_DELAY_MS / MS_PER_SECOND * sample_rate_hz) as usize;
        let mut flanger = Self {
            line: DelayLine::new(max_samples),
            lfo: Lfo::new(LfoWaveform::Sine),
            sample_rate: sample_rate_hz,
            base_delay: 0.0,
            depth: 0.0,
            feedback: 0.5,
            mix: 0.5,
        };
        flanger.set_rate(0.3);
        flanger.set_base_delay_ms(3.0);
        flanger.set_depth_ms(2.0);
        flanger
    }

    // Set the LFO sweep rate in hertz
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.lfo.set_frequency(rate_hz.max(0.0), self.sample_rate);
    }

    // Set the center delay in milliseconds
    pub fn set_base_delay_ms(&mut self, ms: f32) {
        self.base_delay = ms / MS_PER_SECOND * self.sample_rate;
    }

    // Set the modulation depth in milliseconds, clamped to keep the delay positive
    pub fn set_depth_ms(&mut self, ms: f32) {
        let samples = ms / MS_PER_SECOND * self.sample_rate;
        self.depth = samples.clamp(0.0, (self.base_delay - 1.0).max(0.0));
    }

    // Feedback amount in [-0.95, 0.95]; resonance of the comb
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-0.95, 0.95);
    }

    // Dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Clear the delay line and reset the sweep phase
    pub fn reset(&mut self) {
        self.line.clear();
        self.lfo.retrigger();
    }

    // Process one sample
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let modulated = self.base_delay + self.depth * self.lfo.next_sample();
        self.line.set_delay(modulated);
        let delayed = self.line.read();
        self.line.write(x + delayed * self.feedback);
        lerp(x, delayed, self.mix)
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

    const SR: f32 = 48_000.0;

    #[test]
    fn dry_mix_is_transparent() {
        let mut fl = Flanger::new(SR);
        fl.set_mix(0.0);
        for i in 0..256 {
            let x = (i as f32 * 0.1).sin();
            assert!((fl.process_sample(x) - x).abs() < 1e-6);
        }
    }

    #[test]
    fn wet_changes_the_signal_and_stays_finite() {
        let mut fl = Flanger::new(SR);
        fl.set_mix(1.0);
        let mut differ = false;
        for i in 0..4_096 {
            let x = (i as f32 * 0.05).sin();
            let y = fl.process_sample(x);
            assert!(y.is_finite());
            if (y - x).abs() > 1e-3 {
                differ = true;
            }
        }
        assert!(differ, "flanger had no audible effect");
    }

    #[test]
    fn high_feedback_stays_bounded() {
        let mut fl = Flanger::new(SR);
        fl.set_mix(1.0);
        fl.set_feedback(0.95);
        let mut peak = 0.0f32;
        for i in 0..48_000 {
            let x = (i as f32 * 0.02).sin();
            peak = peak.max(fl.process_sample(x).abs());
        }
        assert!(peak < 20.0, "flanger feedback blew up: {peak}");
    }
}
