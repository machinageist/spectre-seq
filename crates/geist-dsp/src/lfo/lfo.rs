// =============================================================================
// File: crates/geist-dsp/src/lfo/lfo.rs
// Layer: DSP primitives
// Purpose: free + tempo-synced LFO
// Status: Implemented; phase-accumulator LFO, all waveforms, bipolar output.
// Notes: Sub-audio rates need no bandlimiting, so shapes are naive. Free running
//        uses set_frequency; sync resets phase via retrigger. Output is [-1, 1].
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use core::f32::consts::TAU;

use crate::osc::Phasor;
use crate::rng::Rng;

// Default seed so sample-and-hold is reproducible without explicit seeding
const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

// Modulation shape produced by the LFO
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LfoWaveform {
    #[default]
    Sine,
    Triangle,
    SawUp,
    SawDown,
    Square,
    SampleHold,
}

// Low-frequency oscillator for modulation; bipolar output in [-1, 1]
#[derive(Clone, Copy, Debug)]
pub struct Lfo {
    phasor: Phasor,
    waveform: LfoWaveform,
    rng: Rng,
    sample_hold: f32,
    last_phase: f32,
}

impl Lfo {
    // Build an LFO of the given shape; call set_frequency before processing
    pub fn new(waveform: LfoWaveform) -> Self {
        let mut rng = Rng::new(DEFAULT_SEED);
        let sample_hold = rng.next_bipolar();
        Self {
            phasor: Phasor::new(),
            waveform,
            rng,
            sample_hold,
            // Force the first tick to refresh sample-and-hold
            last_phase: 1.0,
        }
    }

    // Set the free-running rate in Hz at a sample rate
    pub fn set_frequency(&mut self, frequency_hz: f32, sample_rate_hz: f32) {
        self.phasor.set_frequency(frequency_hz, sample_rate_hz);
    }

    // Switch the produced shape, keeping phase continuous
    pub fn set_waveform(&mut self, waveform: LfoWaveform) {
        self.waveform = waveform;
    }

    // Reset phase to the cycle start (used for tempo-sync retriggering)
    pub fn retrigger(&mut self) {
        self.phasor.reset();
        self.last_phase = 1.0;
    }

    // Reset phase and sample-and-hold state deterministically
    pub fn reset(&mut self) {
        self.phasor.reset();
        self.rng = Rng::new(DEFAULT_SEED);
        self.sample_hold = self.rng.next_bipolar();
        self.last_phase = 1.0;
    }

    // Advance one sample and return the modulation value in [-1, 1]
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let phase = self.phasor.tick();
        // Refresh sample-and-hold once per cycle, detected by phase wrap
        if self.waveform == LfoWaveform::SampleHold && phase < self.last_phase {
            self.sample_hold = self.rng.next_bipolar();
        }
        self.last_phase = phase;

        match self.waveform {
            LfoWaveform::Sine => (TAU * phase).sin(),
            LfoWaveform::Triangle => {
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                }
            }
            LfoWaveform::SawUp => 2.0 * phase - 1.0,
            LfoWaveform::SawDown => 1.0 - 2.0 * phase,
            LfoWaveform::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoWaveform::SampleHold => self.sample_hold,
        }
    }

    // Fill a buffer with successive modulation values
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    fn all_waveforms() -> [LfoWaveform; 6] {
        [
            LfoWaveform::Sine,
            LfoWaveform::Triangle,
            LfoWaveform::SawUp,
            LfoWaveform::SawDown,
            LfoWaveform::Square,
            LfoWaveform::SampleHold,
        ]
    }

    #[test]
    fn every_waveform_stays_bipolar() {
        for wf in all_waveforms() {
            let mut lfo = Lfo::new(wf);
            lfo.set_frequency(3.0, SAMPLE_RATE);
            for _ in 0..100_000 {
                let v = lfo.next_sample();
                assert!((-1.001..=1.001).contains(&v), "{wf:?} out of range: {v}");
            }
        }
    }

    #[test]
    fn sine_frequency_is_correct() {
        let mut lfo = Lfo::new(LfoWaveform::Sine);
        lfo.set_frequency(5.0, SAMPLE_RATE);
        let mut buf = vec![0.0f32; SAMPLE_RATE as usize];
        lfo.process(&mut buf);
        let mut rising = 0;
        for w in buf.windows(2) {
            if w[0] < 0.0 && w[1] >= 0.0 {
                rising += 1;
            }
        }
        assert!((4..=6).contains(&rising), "rising crossings = {rising}");
    }

    #[test]
    fn saw_up_and_down_are_mirror_images() {
        let mut up = Lfo::new(LfoWaveform::SawUp);
        let mut down = Lfo::new(LfoWaveform::SawDown);
        up.set_frequency(2.0, SAMPLE_RATE);
        down.set_frequency(2.0, SAMPLE_RATE);
        for _ in 0..5_000 {
            assert!((up.next_sample() + down.next_sample()).abs() < 1e-6);
        }
    }

    #[test]
    fn square_is_balanced() {
        let mut lfo = Lfo::new(LfoWaveform::Square);
        lfo.set_frequency(10.0, SAMPLE_RATE);
        let mut buf = vec![0.0f32; SAMPLE_RATE as usize];
        lfo.process(&mut buf);
        let positive = buf.iter().filter(|&&s| s > 0.0).count();
        let negative = buf.iter().filter(|&&s| s < 0.0).count();
        assert!((positive as i32 - negative as i32).abs() < 50);
        assert!(buf.iter().all(|&s| s == 1.0 || s == -1.0));
    }

    #[test]
    fn sample_hold_holds_within_a_cycle() {
        // One cycle spans 100 samples at this rate
        let mut lfo = Lfo::new(LfoWaveform::SampleHold);
        lfo.set_frequency(SAMPLE_RATE / 100.0, SAMPLE_RATE);
        let first = lfo.next_sample();
        // The value is constant for the rest of the cycle
        for _ in 0..98 {
            assert_eq!(lfo.next_sample(), first);
        }
        // A new cycle may bring a new value (overwhelmingly likely with this RNG)
        let mut changed = false;
        let mut prev = first;
        for _ in 0..400 {
            let v = lfo.next_sample();
            if v != prev {
                changed = true;
            }
            prev = v;
        }
        assert!(changed, "sample-and-hold never updated across cycles");
    }

    #[test]
    fn sample_hold_is_deterministic_with_seed() {
        let mut a = Lfo::new(LfoWaveform::SampleHold);
        let mut b = Lfo::new(LfoWaveform::SampleHold);
        a.set_frequency(7.0, SAMPLE_RATE);
        b.set_frequency(7.0, SAMPLE_RATE);
        for _ in 0..10_000 {
            assert_eq!(a.next_sample(), b.next_sample());
        }
    }

    #[test]
    fn retrigger_restarts_the_cycle() {
        let mut lfo = Lfo::new(LfoWaveform::Sine);
        lfo.set_frequency(3.0, SAMPLE_RATE);
        let first = lfo.next_sample();
        assert_eq!(first, 0.0); // sine starts at phase 0
        for _ in 0..500 {
            lfo.next_sample();
        }
        lfo.retrigger();
        assert_eq!(lfo.next_sample(), first);
    }
}
