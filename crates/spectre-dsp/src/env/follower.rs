// =============================================================================
// File: crates/spectre-dsp/src/env/follower.rs
// Layer: DSP primitives
// Purpose: envelope follower (peak + RMS)
// Status: Implemented; rectified-peak and mean-square detectors with A/R ballistics.
// Notes: One-pole smoothing picks the attack coefficient when rising, release when
//        falling. RMS smooths the squared signal and returns its square root.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Detector front-end ahead of the smoother
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FollowerMode {
    // Track the rectified peak amplitude
    #[default]
    Peak,
    // Track the running root-mean-square level
    Rms,
}

// Amplitude envelope detector with separate attack and release times
#[derive(Clone, Copy, Debug)]
pub struct EnvelopeFollower {
    sample_rate: f32,
    mode: FollowerMode,
    attack_coef: f32,
    release_coef: f32,
    // Peak mode holds amplitude here; RMS mode holds the mean square
    state: f32,
}

impl EnvelopeFollower {
    // Build a follower at a sample rate in the given mode with fast defaults
    pub fn new(sample_rate_hz: f32, mode: FollowerMode) -> Self {
        let mut f = Self {
            sample_rate: sample_rate_hz,
            mode,
            attack_coef: 0.0,
            release_coef: 0.0,
            state: 0.0,
        };
        f.set_attack(0.005);
        f.set_release(0.05);
        f
    }

    // One-pole coefficient for a smoothing time in seconds
    fn coef(time_seconds: f32, sample_rate_hz: f32) -> f32 {
        (-1.0 / (time_seconds * sample_rate_hz)).exp()
    }

    // Attack time in seconds (how fast the envelope rises)
    pub fn set_attack(&mut self, seconds: f32) {
        self.attack_coef = Self::coef(seconds, self.sample_rate);
    }

    // Release time in seconds (how fast the envelope falls)
    pub fn set_release(&mut self, seconds: f32) {
        self.release_coef = Self::coef(seconds, self.sample_rate);
    }

    // Switch detector mode; resets state since the unit changes
    pub fn set_mode(&mut self, mode: FollowerMode) {
        self.mode = mode;
        self.state = 0.0;
    }

    // Clear the detector state
    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    // Current envelope value as a linear amplitude
    pub fn current(&self) -> f32 {
        match self.mode {
            FollowerMode::Peak => self.state,
            FollowerMode::Rms => self.state.max(0.0).sqrt(),
        }
    }

    // Process one input sample and return the linear amplitude envelope
    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let detector = match self.mode {
            FollowerMode::Peak => input.abs(),
            FollowerMode::Rms => input * input,
        };
        // Rising toward the detector uses attack; falling away uses release
        let coef = if detector > self.state {
            self.attack_coef
        } else {
            self.release_coef
        };
        self.state = detector + coef * (self.state - detector);

        match self.mode {
            FollowerMode::Peak => self.state,
            FollowerMode::Rms => self.state.max(0.0).sqrt(),
        }
    }

    // Replace each sample with the envelope at that point
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

    // Final envelope after feeding a steady sine for one second
    fn settle_sine(mode: FollowerMode, amplitude: f32, freq: f32) -> f32 {
        let mut f = EnvelopeFollower::new(SAMPLE_RATE, mode);
        f.set_attack(0.01);
        f.set_release(0.01);
        let mut env = 0.0;
        for n in 0..SAMPLE_RATE as usize {
            let phase = core::f32::consts::TAU * freq * n as f32 / SAMPLE_RATE;
            env = f.process_sample(amplitude * phase.sin());
        }
        // Average the final cycle to remove ripple
        let mut acc = 0.0f64;
        let cycle = (SAMPLE_RATE / freq) as usize;
        for n in 0..cycle {
            let phase = core::f32::consts::TAU * freq * n as f32 / SAMPLE_RATE;
            acc += f.process_sample(amplitude * phase.sin()) as f64;
        }
        let _ = env;
        (acc / cycle as f64) as f32
    }

    #[test]
    fn peak_tracks_dc_level() {
        let mut f = EnvelopeFollower::new(SAMPLE_RATE, FollowerMode::Peak);
        let mut env = 0.0;
        for _ in 0..10_000 {
            env = f.process_sample(0.7);
        }
        assert!((env - 0.7).abs() < 1e-3, "peak DC = {env}");
    }

    #[test]
    fn rms_of_dc_equals_level() {
        let mut f = EnvelopeFollower::new(SAMPLE_RATE, FollowerMode::Rms);
        let mut env = 0.0;
        for _ in 0..10_000 {
            env = f.process_sample(0.5);
        }
        assert!((env - 0.5).abs() < 1e-3, "rms DC = {env}");
    }

    #[test]
    fn rms_of_sine_is_amplitude_over_sqrt_two() {
        // The defining difference from a peak detector
        let env = settle_sine(FollowerMode::Rms, 1.0, 1_000.0);
        assert!(
            (env - core::f32::consts::FRAC_1_SQRT_2).abs() < 0.02,
            "rms sine = {env}"
        );
    }

    #[test]
    fn peak_of_sine_approaches_amplitude() {
        // Fast attack catches each crest; slow release holds it near the peak
        let mut f = EnvelopeFollower::new(SAMPLE_RATE, FollowerMode::Peak);
        f.set_attack(0.0002);
        f.set_release(0.05);
        let freq = 1_000.0;
        let mut env = 0.0;
        for n in 0..SAMPLE_RATE as usize {
            let phase = core::f32::consts::TAU * freq * n as f32 / SAMPLE_RATE;
            env = f.process_sample(phase.sin());
        }
        assert!(env > 0.9, "peak sine = {env}");
        assert!(env <= 1.001);
    }

    #[test]
    fn attack_reaches_63_percent_near_time_constant() {
        let mut f = EnvelopeFollower::new(SAMPLE_RATE, FollowerMode::Peak);
        f.set_attack(0.01); // 480-sample time constant
        let mut n = 0;
        while f.process_sample(1.0) < 0.632 {
            n += 1;
            if n > 48_000 {
                break;
            }
        }
        let expected = 0.01 * SAMPLE_RATE;
        assert!(
            ((n as f32) - expected).abs() < expected * 0.1,
            "attack n = {n}"
        );
    }

    #[test]
    fn release_is_slower_than_attack() {
        let mut f = EnvelopeFollower::new(SAMPLE_RATE, FollowerMode::Peak);
        f.set_attack(0.001);
        f.set_release(0.1);
        // Charge up fast
        for _ in 0..2_000 {
            f.process_sample(1.0);
        }
        // Then release into silence; should still be well above zero shortly after
        let mut env = 1.0;
        for _ in 0..1_000 {
            env = f.process_sample(0.0);
        }
        assert!(env > 0.7, "release decayed too fast: {env}");
    }

    #[test]
    fn envelope_is_never_negative() {
        let mut f = EnvelopeFollower::new(SAMPLE_RATE, FollowerMode::Rms);
        for n in 0..5_000 {
            let x = if n % 2 == 0 { -0.9 } else { 0.6 };
            assert!(f.process_sample(x) >= 0.0);
        }
    }
}
