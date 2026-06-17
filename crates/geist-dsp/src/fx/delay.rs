// =============================================================================
// File: crates/geist-dsp/src/fx/delay.rs
// Layer: DSP primitives
// Purpose: stereo delay with feedback + filtering
// Status: Implemented; dual fractional delay lines, damped feedback, ping-pong.
// Notes: One-pole lowpass in the feedback path darkens each repeat. Buffers are
//        sized once; process() never allocates.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::fx::DelayLine;
use crate::math::lerp;

// Feedback magnitude bound that keeps the recursive path stable
const MAX_FEEDBACK: f32 = 0.999;

// Stereo feedback delay with a damping lowpass in the feedback path
#[derive(Clone, Debug)]
pub struct StereoDelay {
    left: DelayLine,
    right: DelayLine,
    feedback: f32,
    damping: f32,
    damp_l: f32,
    damp_r: f32,
    mix: f32,
    ping_pong: bool,
}

impl StereoDelay {
    // Allocate a delay able to hold up to `max_delay_samples` per channel
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            left: DelayLine::new(max_delay_samples),
            right: DelayLine::new(max_delay_samples),
            feedback: 0.0,
            damping: 0.0,
            damp_l: 0.0,
            damp_r: 0.0,
            mix: 0.5,
            ping_pong: false,
        }
    }

    // Set per-channel delay directly in samples
    pub fn set_delay_samples(&mut self, left: f32, right: f32) {
        self.left.set_delay(left);
        self.right.set_delay(right);
    }

    // Set per-channel delay in seconds
    pub fn set_delay_seconds(&mut self, left: f32, right: f32, sample_rate_hz: f32) {
        self.set_delay_samples(left * sample_rate_hz, right * sample_rate_hz);
    }

    // Feedback gain, clamped to stay stable
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-MAX_FEEDBACK, MAX_FEEDBACK);
    }

    // Feedback-path damping in [0, 1]; 0 is bright, higher darkens repeats
    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
    }

    // Dry/wet mix in [0, 1]; 0 is dry, 1 is fully wet
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    // Cross the feedback between channels for a ping-pong bounce
    pub fn set_ping_pong(&mut self, enabled: bool) {
        self.ping_pong = enabled;
    }

    // Clear the delay lines and damping state
    pub fn reset(&mut self) {
        self.left.clear();
        self.right.clear();
        self.damp_l = 0.0;
        self.damp_r = 0.0;
    }

    // Process one stereo frame
    #[inline]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let d_l = self.left.read();
        let d_r = self.right.read();

        // One-pole lowpass on the feedback signal
        self.damp_l = lerp(d_l, self.damp_l, self.damping);
        self.damp_r = lerp(d_r, self.damp_r, self.damping);

        if self.ping_pong {
            self.left.write(in_l + self.feedback * self.damp_r);
            self.right.write(in_r + self.feedback * self.damp_l);
        } else {
            self.left.write(in_l + self.feedback * self.damp_l);
            self.right.write(in_r + self.feedback * self.damp_r);
        }

        (lerp(in_l, d_l, self.mix), lerp(in_r, d_r, self.mix))
    }

    // Process stereo buffers in place
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let (out_l, out_r) = self.process_sample(*l, *r);
            *l = out_l;
            *r = out_r;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const D: usize = 100;

    // Impulse response of the left channel for a left-only input
    fn left_impulse(delay: &mut StereoDelay, len: usize) -> Vec<f32> {
        (0..len)
            .map(|n| delay.process_sample(if n == 0 { 1.0 } else { 0.0 }, 0.0).0)
            .collect()
    }

    fn dry_delay() -> StereoDelay {
        let mut d = StereoDelay::new(1024);
        d.set_delay_samples(D as f32, D as f32);
        d
    }

    #[test]
    fn fully_wet_delays_the_signal() {
        let mut d = dry_delay();
        d.set_mix(1.0);
        d.set_feedback(0.0);
        let ir = left_impulse(&mut d, 256);
        assert!((ir[D] - 1.0).abs() < 1e-5, "delayed peak = {}", ir[D]);
        assert!(ir[..D].iter().all(|&s| s.abs() < 1e-6));
    }

    #[test]
    fn fully_dry_passes_input_through() {
        let mut d = dry_delay();
        d.set_mix(0.0);
        let (l, r) = d.process_sample(0.8, -0.3);
        assert_eq!(l, 0.8);
        assert_eq!(r, -0.3);
    }

    #[test]
    fn feedback_produces_decaying_repeats() {
        let mut d = dry_delay();
        d.set_mix(1.0);
        d.set_feedback(0.5);
        d.set_damping(0.0);
        let ir = left_impulse(&mut d, 4 * D + 1);
        assert!((ir[D] - 1.0).abs() < 1e-5);
        assert!((ir[2 * D] - 0.5).abs() < 1e-4);
        assert!((ir[3 * D] - 0.25).abs() < 1e-4);
    }

    #[test]
    fn damping_darkens_repeats() {
        // With damping, the lowpass spreads each echo, lowering its peak
        let mut bright = dry_delay();
        bright.set_mix(1.0);
        bright.set_feedback(0.6);
        bright.set_damping(0.0);
        let bright_ir = left_impulse(&mut bright, 3 * D);

        let mut dark = dry_delay();
        dark.set_mix(1.0);
        dark.set_feedback(0.6);
        dark.set_damping(0.7);
        let dark_ir = left_impulse(&mut dark, 3 * D);

        assert!(
            dark_ir[2 * D].abs() < bright_ir[2 * D].abs(),
            "damped echo peak should be lower: dark={} bright={}",
            dark_ir[2 * D],
            bright_ir[2 * D]
        );
    }

    #[test]
    fn ping_pong_bounces_across_channels() {
        let mut d = StereoDelay::new(1024);
        d.set_delay_samples(D as f32, D as f32);
        d.set_mix(1.0);
        d.set_feedback(0.5);
        d.set_ping_pong(true);

        let mut out_l = vec![0.0f32; 3 * D];
        let mut out_r = vec![0.0f32; 3 * D];
        for n in 0..3 * D {
            let (l, r) = d.process_sample(if n == 0 { 1.0 } else { 0.0 }, 0.0);
            out_l[n] = l;
            out_r[n] = r;
        }
        // First repeat on the left, the bounce shows up on the right an octave later
        assert!((out_l[D] - 1.0).abs() < 1e-5);
        assert!(out_r[D].abs() < 1e-6);
        assert!((out_r[2 * D] - 0.5).abs() < 1e-4);
        assert!(out_l[2 * D].abs() < 1e-6);
    }

    #[test]
    fn reset_clears_tails() {
        let mut d = dry_delay();
        d.set_mix(1.0);
        d.set_feedback(0.7);
        for _ in 0..500 {
            d.process_sample(0.5, 0.5);
        }
        d.reset();
        let (l, r) = d.process_sample(0.0, 0.0);
        assert_eq!(l, 0.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn high_feedback_stays_bounded() {
        let mut d = dry_delay();
        d.set_mix(1.0);
        d.set_feedback(5.0); // clamped
        let mut peak = 0.0f32;
        for n in 0..96_000 {
            let (l, _) = d.process_sample(if n == 0 { 1.0 } else { 0.0 }, 0.0);
            assert!(l.is_finite());
            peak = peak.max(l.abs());
        }
        assert!(peak < 100.0, "delay feedback unbounded: {peak}");
    }
}
