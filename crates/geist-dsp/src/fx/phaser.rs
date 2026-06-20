// =============================================================================
// File: crates/geist-dsp/src/fx/phaser.rs
// Layer: DSP primitives
// Purpose: LFO-swept allpass phaser
// Status: Implemented; cascade of first-order allpass stages swept by an LFO.
// Notes: Summing the allpass chain with the dry signal forms moving notches.
//        Stateless per sample beyond the per-stage allpass memory and feedback.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::PI;

use crate::lfo::{Lfo, LfoWaveform};
use crate::math::lerp;

// Number of cascaded allpass stages; more stages add more notches
const STAGES: usize = 4;

// Default sweep range in hertz
const DEFAULT_MIN_HZ: f32 = 200.0;
const DEFAULT_MAX_HZ: f32 = 2_000.0;

// One first-order allpass: H(z) = (a + z^-1) / (1 + a z^-1)
#[derive(Clone, Copy, Debug, Default)]
struct Allpass {
    x_prev: f32,
    y_prev: f32,
}

impl Allpass {
    // y[n] = a*x[n] + x[n-1] - a*y[n-1]
    #[inline]
    fn process(&mut self, x: f32, a: f32) -> f32 {
        let y = a * x + self.x_prev - a * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }
}

// LFO-swept allpass phaser with feedback and a dry/wet mix
#[derive(Clone, Debug)]
pub struct Phaser {
    stages: [Allpass; STAGES],
    lfo: Lfo,
    sample_rate: f32,
    min_hz: f32,
    max_hz: f32,
    depth: f32,
    feedback: f32,
    fb_state: f32,
    mix: f32,
}

impl Phaser {
    // Build a phaser at a sample rate with musical defaults
    pub fn new(sample_rate_hz: f32) -> Self {
        let mut phaser = Self {
            stages: [Allpass::default(); STAGES],
            lfo: Lfo::new(LfoWaveform::Sine),
            sample_rate: sample_rate_hz,
            min_hz: DEFAULT_MIN_HZ,
            max_hz: DEFAULT_MAX_HZ,
            depth: 1.0,
            feedback: 0.5,
            fb_state: 0.0,
            mix: 0.5,
        };
        phaser.set_rate(0.5);
        phaser
    }

    // Set the LFO sweep rate in hertz
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.lfo.set_frequency(rate_hz.max(0.0), self.sample_rate);
    }

    // Sweep depth in [0, 1]; scales how far the notches travel
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    // Resonance feedback in [0, 0.95]; sharpens the notches
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95);
    }

    // Dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Clear allpass memory and feedback, reset the sweep phase
    pub fn reset(&mut self) {
        self.stages = [Allpass::default(); STAGES];
        self.fb_state = 0.0;
        self.lfo.retrigger();
    }

    // Process one sample
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        // Map the LFO (-1..1) to a logarithmic sweep across the range
        let t = (self.lfo.next_sample() * 0.5 + 0.5) * self.depth;
        let fc = self.min_hz * (self.max_hz / self.min_hz).powf(t.clamp(0.0, 1.0));
        let tan = (PI * fc / self.sample_rate).tan();
        let a = (1.0 - tan) / (1.0 + tan);

        let mut s = x + self.fb_state * self.feedback;
        for stage in &mut self.stages {
            s = stage.process(s, a);
        }
        self.fb_state = s;

        // Dry + phase-shifted sum forms the notches; mix scales the effect
        lerp(x, 0.5 * (x + s), self.mix)
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
        let mut ph = Phaser::new(SR);
        ph.set_mix(0.0);
        for i in 0..256 {
            let x = (i as f32 * 0.1).sin();
            assert!((ph.process_sample(x) - x).abs() < 1e-6);
        }
    }

    #[test]
    fn wet_changes_the_signal_and_stays_finite() {
        let mut dry = Vec::new();
        let mut wet = Phaser::new(SR);
        wet.set_mix(1.0);
        let mut differ = false;
        for i in 0..2_048 {
            let x = (i as f32 * 0.05).sin();
            dry.push(x);
            let y = wet.process_sample(x);
            assert!(y.is_finite());
            if (y - x).abs() > 1e-3 {
                differ = true;
            }
        }
        assert!(differ, "phaser had no audible effect");
    }

    #[test]
    fn feedback_stays_bounded() {
        let mut ph = Phaser::new(SR);
        ph.set_mix(1.0);
        ph.set_feedback(0.95);
        let mut peak = 0.0f32;
        for i in 0..48_000 {
            let x = (i as f32 * 0.02).sin();
            let y = ph.process_sample(x);
            assert!(y.is_finite());
            peak = peak.max(y.abs());
        }
        // The allpass-feedback loop resonates but stays bounded (does not diverge)
        assert!(peak < 20.0, "phaser feedback blew up: {peak}");
    }
}
