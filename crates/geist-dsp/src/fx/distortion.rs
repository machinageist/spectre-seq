// =============================================================================
// File: crates/geist-dsp/src/fx/distortion.rs
// Layer: DSP primitives
// Purpose: drive + tone waveshaping distortion
// Notes: Richer than the saturator: adds a post-shaper one-pole tone control and
//        a foldback mode alongside soft/hard clipping. Output level and dry/wet
//        keep it usable in a chain. Stateless beyond the one-pole tone memory.
// Status: Implemented; soft/hard/foldback curves with drive, tone, level, mix.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::math::{fast_tanh, lerp};

// Shaping character of the distortion
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DistortionMode {
    // Smooth tanh overdrive
    #[default]
    Soft,
    // Brick-wall clip at +/-1
    Hard,
    // Wavefolder: values past +/-1 reflect back inward
    Foldback,
}

// Drive + tone waveshaping distortion
#[derive(Clone, Copy, Debug)]
pub struct Distortion {
    mode: DistortionMode,
    drive: f32,
    tone: f32,
    level: f32,
    mix: f32,
    lp_state: f32,
}

impl Distortion {
    // Build a distortion with the given mode at unity drive
    pub fn new(mode: DistortionMode) -> Self {
        Self {
            mode,
            drive: 1.0,
            tone: 1.0,
            level: 1.0,
            mix: 1.0,
            lp_state: 0.0,
        }
    }

    // Select the shaping mode
    pub fn set_mode(&mut self, mode: DistortionMode) {
        self.mode = mode;
    }

    // Linear input gain into the shaper; higher drives harder
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(0.0);
    }

    // Tone in [0, 1]: 0 fully dark (heavy low-pass), 1 fully open (bright)
    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone.clamp(0.0, 1.0);
    }

    // Makeup output gain after shaping
    pub fn set_level(&mut self, level: f32) {
        self.level = level.max(0.0);
    }

    // Dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Reset the tone filter memory
    pub fn reset(&mut self) {
        self.lp_state = 0.0;
    }

    // Apply the selected curve to a driven sample
    #[inline]
    fn shape(&self, x: f32) -> f32 {
        match self.mode {
            DistortionMode::Soft => fast_tanh(x),
            DistortionMode::Hard => x.clamp(-1.0, 1.0),
            DistortionMode::Foldback => fold(x),
        }
    }

    // Process one sample
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let shaped = self.shape(x * self.drive);
        // One-pole low-pass; tone blends the bright shaped signal with its
        // smoothed version (tone=1 keeps it bright, tone=0 darkens fully)
        self.lp_state = lerp(self.lp_state, shaped, 0.25);
        let toned = lerp(self.lp_state, shaped, self.tone);
        lerp(x, toned * self.level, self.mix)
    }

    // Process a buffer in place
    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
}

// Fold a value back inside [-1, 1] by reflecting across the boundaries
#[inline]
fn fold(mut x: f32) -> f32 {
    while !(-1.0..=1.0).contains(&x) {
        if x > 1.0 {
            x = 2.0 - x;
        } else {
            x = -2.0 - x;
        }
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_mix_is_transparent() {
        let mut d = Distortion::new(DistortionMode::Soft);
        d.set_mix(0.0);
        for i in 0..256 {
            let x = (i as f32 * 0.1).sin() * 3.0;
            assert!((d.process_sample(x) - x).abs() < 1e-6);
        }
    }

    #[test]
    fn hard_clip_bounds_the_output() {
        let mut d = Distortion::new(DistortionMode::Hard);
        d.set_drive(10.0);
        d.set_tone(1.0);
        d.set_mix(1.0);
        for i in 0..1_000 {
            let x = (i as f32 * 0.3).sin() * 2.0;
            assert!(d.process_sample(x).abs() <= 1.0 + 1e-6);
        }
    }

    #[test]
    fn foldback_stays_in_range_and_is_finite() {
        let mut d = Distortion::new(DistortionMode::Foldback);
        d.set_drive(6.0);
        d.set_tone(1.0);
        d.set_mix(1.0);
        for i in 0..1_000 {
            let x = (i as f32 * 0.2).sin() * 4.0;
            let y = d.process_sample(x);
            assert!(y.is_finite());
            assert!(y.abs() <= 1.0 + 1e-6, "fold out of range: {y}");
        }
    }

    #[test]
    fn drive_increases_harmonic_content() {
        // A clean low-drive pass vs a hot one differ in shape
        let mut clean = Distortion::new(DistortionMode::Soft);
        clean.set_drive(1.0);
        let mut hot = Distortion::new(DistortionMode::Soft);
        hot.set_drive(20.0);
        let mut differ = false;
        for i in 0..512 {
            let x = (i as f32 * 0.05).sin();
            if (clean.process_sample(x) - hot.process_sample(x)).abs() > 1e-2 {
                differ = true;
            }
        }
        assert!(differ, "drive had no shaping effect");
    }
}
