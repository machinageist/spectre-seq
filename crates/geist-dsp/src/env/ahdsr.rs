// =============================================================================
// File: crates/geist-dsp/src/env/ahdsr.rs
// Layer: DSP primitives
// Purpose: AHDSR extended envelope
// Status: Implemented; ADSR plus a peak Hold stage between attack and decay.
// Notes: Shares the analog segment-curve math in env::mod. Output is in [0, 1].
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::env::{stage_coef, ANALOG_CURVE_EXP, ATTACK_CURVE_EXP, MIN_STAGE_SAMPLES};

// Position in the envelope's life cycle; Hold sits at the peak before decay
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AhdsrStage {
    #[default]
    Idle,
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
}

// Analog-modeled AHDSR envelope; a hold time keeps the peak before decaying
#[derive(Clone, Copy, Debug)]
pub struct Ahdsr {
    sample_rate: f32,
    sustain_level: f32,
    decay_samples: f32,
    hold_samples: u32,

    attack_coef: f32,
    attack_base: f32,
    decay_coef: f32,
    decay_base: f32,
    release_coef: f32,
    release_base: f32,

    stage: AhdsrStage,
    value: f32,
    hold_counter: u32,
}

impl Ahdsr {
    // Build an envelope at a sample rate with short default times and no hold
    pub fn new(sample_rate_hz: f32) -> Self {
        let mut env = Self {
            sample_rate: sample_rate_hz,
            sustain_level: 0.7,
            decay_samples: MIN_STAGE_SAMPLES,
            hold_samples: 0,
            attack_coef: 0.0,
            attack_base: 1.0,
            decay_coef: 0.0,
            decay_base: 0.0,
            release_coef: 0.0,
            release_base: 0.0,
            stage: AhdsrStage::Idle,
            value: 0.0,
            hold_counter: 0,
        };
        env.set_attack(0.005);
        env.set_decay(0.1);
        env.set_release(0.2);
        env
    }

    // Attack time in seconds; aims past 1.0 so it lands on 1.0 in the set time
    pub fn set_attack(&mut self, seconds: f32) {
        let tco = ATTACK_CURVE_EXP.exp();
        self.attack_coef = stage_coef(seconds * self.sample_rate, tco);
        self.attack_base = (1.0 + tco) * (1.0 - self.attack_coef);
    }

    // Hold time in seconds; the peak is sustained this long before decay
    pub fn set_hold(&mut self, seconds: f32) {
        self.hold_samples = (seconds * self.sample_rate).max(0.0) as u32;
    }

    // Decay time in seconds; falls from 1.0 toward the sustain level
    pub fn set_decay(&mut self, seconds: f32) {
        self.decay_samples = seconds * self.sample_rate;
        self.update_decay();
    }

    // Sustain level in [0, 1]; also reshapes the decay target
    pub fn set_sustain(&mut self, level: f32) {
        self.sustain_level = level.clamp(0.0, 1.0);
        self.update_decay();
    }

    // Release time in seconds; falls from the current value toward 0
    pub fn set_release(&mut self, seconds: f32) {
        let tco = ANALOG_CURVE_EXP.exp();
        self.release_coef = stage_coef(seconds * self.sample_rate, tco);
        self.release_base = -tco * (1.0 - self.release_coef);
    }

    // Recompute decay coefficient and target from time and sustain
    fn update_decay(&mut self) {
        let tco = ANALOG_CURVE_EXP.exp();
        self.decay_coef = stage_coef(self.decay_samples, tco);
        self.decay_base = (self.sustain_level - tco) * (1.0 - self.decay_coef);
    }

    // Begin the attack stage, retriggering from the current value
    pub fn gate_on(&mut self) {
        self.stage = AhdsrStage::Attack;
    }

    // Begin the release stage from wherever the envelope is
    pub fn gate_off(&mut self) {
        if self.stage != AhdsrStage::Idle {
            self.stage = AhdsrStage::Release;
        }
    }

    // Force the envelope back to rest
    pub fn reset(&mut self) {
        self.stage = AhdsrStage::Idle;
        self.value = 0.0;
        self.hold_counter = 0;
    }

    // True while the envelope is producing a non-idle signal
    pub fn is_active(&self) -> bool {
        self.stage != AhdsrStage::Idle
    }

    // Current stage, for tests and instrumentation
    pub fn stage(&self) -> AhdsrStage {
        self.stage
    }

    // Advance one sample and return the envelope value in [0, 1]
    #[inline]
    pub fn process_sample(&mut self) -> f32 {
        match self.stage {
            AhdsrStage::Idle => self.value = 0.0,
            AhdsrStage::Attack => {
                self.value = self.attack_base + self.value * self.attack_coef;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = AhdsrStage::Hold;
                    self.hold_counter = self.hold_samples;
                }
            }
            AhdsrStage::Hold => {
                self.value = 1.0;
                if self.hold_counter == 0 {
                    self.stage = AhdsrStage::Decay;
                } else {
                    self.hold_counter -= 1;
                }
            }
            AhdsrStage::Decay => {
                self.value = self.decay_base + self.value * self.decay_coef;
                if self.value <= self.sustain_level {
                    self.value = self.sustain_level;
                    self.stage = AhdsrStage::Sustain;
                }
            }
            AhdsrStage::Sustain => self.value = self.sustain_level,
            AhdsrStage::Release => {
                self.value = self.release_base + self.value * self.release_coef;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = AhdsrStage::Idle;
                }
            }
        }
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    #[test]
    fn hold_keeps_peak_for_the_hold_time() {
        let mut e = Ahdsr::new(SAMPLE_RATE);
        e.set_attack(0.001);
        e.set_hold(0.05); // 2400 samples
        e.set_decay(0.2);
        e.set_sustain(0.5);
        e.gate_on();

        // Advance until the peak is first reached
        let mut n = 0;
        while e.process_sample() < 0.999 {
            n += 1;
            assert!(n < 10_000, "never reached peak");
        }
        // Count how long it stays pinned at the peak
        let mut held = 0;
        while e.process_sample() >= 0.999 {
            held += 1;
            assert!(held < 10_000, "hold never ended");
        }
        let expected = 0.05 * SAMPLE_RATE;
        assert!(
            ((held as f32) - expected).abs() < expected * 0.1,
            "held {held} samples, expected ~{expected}"
        );
    }

    #[test]
    fn zero_hold_behaves_like_adsr() {
        let mut e = Ahdsr::new(SAMPLE_RATE);
        e.set_attack(0.001);
        e.set_hold(0.0);
        e.set_decay(0.05);
        e.set_sustain(0.5);
        e.gate_on();
        // Decays toward sustain without a long plateau
        for _ in 0..6_000 {
            e.process_sample();
        }
        let v = e.process_sample();
        assert!((v - 0.5).abs() < 1e-3, "sustain = {v}");
        assert_eq!(e.stage(), AhdsrStage::Sustain);
    }

    #[test]
    fn full_cycle_runs_through_every_stage() {
        let mut e = Ahdsr::new(SAMPLE_RATE);
        e.set_attack(0.001);
        e.set_hold(0.005);
        e.set_decay(0.01);
        e.set_sustain(0.4);
        e.set_release(0.01);

        e.gate_on();
        let mut seen_hold = false;
        for _ in 0..4_000 {
            e.process_sample();
            if e.stage() == AhdsrStage::Hold {
                seen_hold = true;
            }
        }
        assert!(seen_hold, "hold stage was skipped");
        assert_eq!(e.stage(), AhdsrStage::Sustain);

        e.gate_off();
        let mut v = 1.0;
        for _ in 0..4_000 {
            v = e.process_sample();
        }
        assert!(v.abs() < 1e-3, "release tail = {v}");
        assert!(!e.is_active());
    }

    #[test]
    fn output_stays_bounded() {
        let mut e = Ahdsr::new(SAMPLE_RATE);
        e.set_attack(0.01);
        e.set_hold(0.02);
        e.set_decay(0.05);
        e.set_sustain(0.6);
        e.gate_on();
        for _ in 0..20_000 {
            let v = e.process_sample();
            assert!((0.0..=1.001).contains(&v));
        }
    }
}
