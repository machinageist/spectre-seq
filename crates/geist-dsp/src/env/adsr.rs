// =============================================================================
// File: crates/geist-dsp/src/env/adsr.rs
// Layer: DSP primitives
// Purpose: ADSR with curve shapes
// Status: Implemented; analog-style exponential segments via target-capture offset.
// Notes: Each timed stage aims slightly past its target so it lands in the set
//        time with natural curvature. Output is in [0, 1].
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::env::{stage_coef, ANALOG_CURVE_EXP, ATTACK_CURVE_EXP, MIN_STAGE_SAMPLES};

// Position in the envelope's life cycle
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdsrStage {
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

// Analog-modeled ADSR envelope generator
// Coefficients update on parameter change; the per-sample core is a single madd
#[derive(Clone, Copy, Debug)]
pub struct Adsr {
    sample_rate: f32,
    sustain_level: f32,
    decay_samples: f32,

    attack_coef: f32,
    attack_base: f32,
    decay_coef: f32,
    decay_base: f32,
    release_coef: f32,
    release_base: f32,

    stage: AdsrStage,
    value: f32,
}

impl Adsr {
    // Build an envelope at a sample rate with short default times
    pub fn new(sample_rate_hz: f32) -> Self {
        let mut env = Self {
            sample_rate: sample_rate_hz,
            sustain_level: 0.7,
            decay_samples: MIN_STAGE_SAMPLES,
            attack_coef: 0.0,
            attack_base: 1.0,
            decay_coef: 0.0,
            decay_base: 0.0,
            release_coef: 0.0,
            release_base: 0.0,
            stage: AdsrStage::Idle,
            value: 0.0,
        };
        env.set_attack(0.005);
        env.set_decay(0.1);
        env.set_release(0.2);
        env
    }

    // Attack time in seconds; aims past 1.0 so it lands on 1.0 in the set time
    pub fn set_attack(&mut self, seconds: f32) {
        let tco = ATTACK_CURVE_EXP.exp();
        let samples = seconds * self.sample_rate;
        self.attack_coef = stage_coef(samples, tco);
        self.attack_base = (1.0 + tco) * (1.0 - self.attack_coef);
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
        let samples = seconds * self.sample_rate;
        self.release_coef = stage_coef(samples, tco);
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
        self.stage = AdsrStage::Attack;
    }

    // Begin the release stage from wherever the envelope is
    pub fn gate_off(&mut self) {
        if self.stage != AdsrStage::Idle {
            self.stage = AdsrStage::Release;
        }
    }

    // Force the envelope back to rest
    pub fn reset(&mut self) {
        self.stage = AdsrStage::Idle;
        self.value = 0.0;
    }

    // True while the envelope is producing a non-idle signal
    pub fn is_active(&self) -> bool {
        self.stage != AdsrStage::Idle
    }

    // Current stage, for tests and instrumentation
    pub fn stage(&self) -> AdsrStage {
        self.stage
    }

    // Current envelope value in [0, 1] without advancing
    pub fn value(&self) -> f32 {
        self.value
    }

    // Advance one sample and return the envelope value in [0, 1]
    #[inline]
    pub fn process_sample(&mut self) -> f32 {
        match self.stage {
            AdsrStage::Idle => self.value = 0.0,
            AdsrStage::Attack => {
                self.value = self.attack_base + self.value * self.attack_coef;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = AdsrStage::Decay;
                }
            }
            AdsrStage::Decay => {
                self.value = self.decay_base + self.value * self.decay_coef;
                if self.value <= self.sustain_level {
                    self.value = self.sustain_level;
                    self.stage = AdsrStage::Sustain;
                }
            }
            AdsrStage::Sustain => self.value = self.sustain_level,
            AdsrStage::Release => {
                self.value = self.release_base + self.value * self.release_coef;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = AdsrStage::Idle;
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

    fn env() -> Adsr {
        let mut e = Adsr::new(SAMPLE_RATE);
        e.set_attack(0.01);
        e.set_decay(0.05);
        e.set_sustain(0.5);
        e.set_release(0.02);
        e
    }

    #[test]
    fn idle_outputs_zero() {
        let mut e = env();
        for _ in 0..100 {
            assert_eq!(e.process_sample(), 0.0);
        }
        assert!(!e.is_active());
    }

    #[test]
    fn attack_rises_to_unity_then_decays_to_sustain() {
        let mut e = env();
        e.gate_on();
        // Run well past attack + decay
        let mut max = 0.0f32;
        for _ in 0..6_000 {
            let v = e.process_sample();
            max = max.max(v);
            assert!((0.0..=1.001).contains(&v));
        }
        assert!(max >= 0.999, "attack peak = {max}");
        // Settles at the sustain level
        let v = e.process_sample();
        assert!((v - 0.5).abs() < 1e-3, "sustain = {v}");
        assert_eq!(e.stage(), AdsrStage::Sustain);
    }

    #[test]
    fn release_falls_to_zero_and_idles() {
        let mut e = env();
        e.gate_on();
        for _ in 0..6_000 {
            e.process_sample();
        }
        e.gate_off();
        let mut v = 1.0;
        for _ in 0..6_000 {
            v = e.process_sample();
        }
        assert!(v.abs() < 1e-3, "release tail = {v}");
        assert!(!e.is_active());
    }

    #[test]
    fn attack_reaches_target_near_set_time() {
        let mut e = Adsr::new(SAMPLE_RATE);
        e.set_attack(0.02); // 960 samples
        e.set_decay(10.0); // long decay so we don't fall away
        e.set_sustain(1.0);
        e.gate_on();
        let mut n = 0;
        while e.process_sample() < 0.999 {
            n += 1;
            if n > 48_000 {
                break;
            }
        }
        let expected = 0.02 * SAMPLE_RATE;
        // The target-capture method lands close to the set time
        assert!(
            ((n as f32) - expected).abs() < expected * 0.1,
            "attack samples = {n}, expected ~{expected}"
        );
    }

    #[test]
    fn attack_is_monotonic_rising() {
        let mut e = env();
        e.gate_on();
        let mut prev = 0.0;
        for _ in 0..400 {
            let v = e.process_sample();
            if e.stage() != AdsrStage::Attack {
                break;
            }
            assert!(v >= prev, "attack not monotonic");
            prev = v;
        }
    }

    #[test]
    fn retrigger_restarts_attack() {
        let mut e = env();
        e.gate_on();
        for _ in 0..6_000 {
            e.process_sample();
        }
        e.gate_off();
        for _ in 0..100 {
            e.process_sample();
        }
        // Gate on again mid-release returns to attack
        e.gate_on();
        assert_eq!(e.stage(), AdsrStage::Attack);
        let v = e.process_sample();
        assert!(v > 0.0);
    }
}
