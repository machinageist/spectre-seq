// =============================================================================
// File: app/geist-daw/src/fx.rs
// Layer: application binary
// Purpose: Post-synth effects chain over channel-major buffers
// Status: Implemented; distortion/phaser/flanger/chorus character effects then
//         delay and reverb, each independently bypassable.
// Notes: The character effects are per-sample geist-dsp units held one instance
//        per channel (to keep per-channel state); delay/reverb are stereo
//        AudioNodes run through a single scratch buffer. process() never
//        allocates; effects are sized once on the app thread.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;
use geist_core::transport::TransportSnapshot;
use geist_dsp::prelude::{Chorus, Distortion, DistortionMode, Flanger, Phaser};
use geist_fx::prelude::{DelayNode, ReverbNode};
use geist_graph::node::AudioNode;

use crate::control::FxKind;

// Ordered effects applied after the synth, each independently bypassable
pub struct FxChain {
    // Per-sample character effects, one instance per channel
    distortion: Vec<Distortion>,
    phaser: Vec<Phaser>,
    flanger: Vec<Flanger>,
    chorus: Vec<Chorus>,
    distortion_on: bool,
    phaser_on: bool,
    flanger_on: bool,
    chorus_on: bool,
    // Stereo time-based effects
    delay: DelayNode,
    reverb: ReverbNode,
    delay_on: bool,
    reverb_on: bool,
    channels: usize,
    sample_rate_hz: u32,
    // Ping-pong buffer the size of one channel-major block
    scratch: Vec<f32>,
}

impl FxChain {
    // Build a bypassed chain sized for the stream block
    pub fn new(channels: usize, block_frames: usize, sample_rate_hz: u32) -> Self {
        let sr = sample_rate_hz as f32;
        Self {
            distortion: vec![Distortion::new(DistortionMode::Soft); channels],
            phaser: vec![Phaser::new(sr); channels],
            flanger: vec![Flanger::new(sr); channels],
            chorus: vec![Chorus::new(sr); channels],
            distortion_on: false,
            phaser_on: false,
            flanger_on: false,
            chorus_on: false,
            delay: DelayNode::new(),
            reverb: ReverbNode::new(),
            delay_on: false,
            reverb_on: false,
            channels,
            sample_rate_hz,
            scratch: vec![0.0; channels * block_frames],
        }
    }

    // Rebuild the effects for the stream's rate and block size
    pub fn prepare(&mut self, config: &AudioConfig) {
        let sr = config.sample_rate_hz as f32;
        self.sample_rate_hz = config.sample_rate_hz;
        for d in &mut self.distortion {
            d.reset();
        }
        for p in &mut self.phaser {
            *p = Phaser::new(sr);
        }
        for f in &mut self.flanger {
            *f = Flanger::new(sr);
        }
        for c in &mut self.chorus {
            *c = Chorus::new(sr);
        }
        self.delay.prepare(config);
        self.reverb.prepare(config);
    }

    // Toggle one character effect
    pub fn set_fx_on(&mut self, fx: FxKind, on: bool) {
        match fx {
            FxKind::Distortion => self.distortion_on = on,
            FxKind::Phaser => self.phaser_on = on,
            FxKind::Flanger => self.flanger_on = on,
            FxKind::Chorus => self.chorus_on = on,
        }
    }

    // Set one parameter (by index) of a character effect, on every channel
    pub fn set_fx_param(&mut self, fx: FxKind, param: u8, value: f32) {
        match fx {
            FxKind::Distortion => {
                for d in &mut self.distortion {
                    match param {
                        0 => d.set_drive(value),
                        1 => d.set_tone(value),
                        _ => d.set_mix(value),
                    }
                }
            }
            FxKind::Phaser => {
                for p in &mut self.phaser {
                    match param {
                        0 => p.set_rate(value),
                        1 => p.set_depth(value),
                        2 => p.set_feedback(value),
                        _ => p.set_mix(value),
                    }
                }
            }
            FxKind::Flanger => {
                for f in &mut self.flanger {
                    match param {
                        0 => f.set_rate(value),
                        1 => f.set_depth_ms(value),
                        2 => f.set_feedback(value),
                        _ => f.set_mix(value),
                    }
                }
            }
            FxKind::Chorus => {
                for c in &mut self.chorus {
                    match param {
                        0 => c.set_rate(value),
                        1 => c.set_depth_ms(value),
                        _ => c.set_mix(value),
                    }
                }
            }
        }
    }

    // Toggle the delay
    pub fn set_delay(&mut self, on: bool) {
        self.delay_on = on;
    }

    // Set the delay time in seconds on both channels
    pub fn set_delay_time(&mut self, seconds: f32) {
        let sr = self.sample_rate_hz as f32;
        self.delay.delay_mut().set_delay_seconds(seconds, seconds, sr);
    }

    // Set the delay feedback amount
    pub fn set_delay_feedback(&mut self, feedback: f32) {
        self.delay.delay_mut().set_feedback(feedback);
    }

    // Set the delay dry/wet mix
    pub fn set_delay_mix(&mut self, mix: f32) {
        self.delay.delay_mut().set_mix(mix);
    }

    // Toggle the reverb
    pub fn set_reverb(&mut self, on: bool) {
        self.reverb_on = on;
    }

    // Set the reverb dry/wet mix
    pub fn set_reverb_mix(&mut self, mix: f32) {
        self.reverb.set_mix(mix);
    }

    // Apply the enabled effects in order, in place on channel-major `output`:
    // character effects (per channel) first, then the stereo time effects.
    pub fn process(&mut self, output: &mut [f32], frames: usize) {
        for ch in 0..self.channels {
            let lane = &mut output[ch * frames..ch * frames + frames];
            if self.distortion_on {
                self.distortion[ch].process(lane);
            }
            if self.phaser_on {
                self.phaser[ch].process(lane);
            }
            if self.flanger_on {
                self.flanger[ch].process(lane);
            }
            if self.chorus_on {
                self.chorus[ch].process(lane);
            }
        }
        if self.delay_on {
            apply(&mut self.delay, output, &mut self.scratch, frames, self.sample_rate_hz);
        }
        if self.reverb_on {
            apply(&mut self.reverb, output, &mut self.scratch, frames, self.sample_rate_hz);
        }
    }
}

// Run one effect: current output becomes its input, scratch its output, then copy back
fn apply(
    node: &mut dyn AudioNode,
    output: &mut [f32],
    scratch: &mut [f32],
    frames: usize,
    sample_rate_hz: u32,
) {
    {
        let transport = TransportSnapshot::stopped(sample_rate_hz);
        let mut ctx =
            ProcessContext::new(frames, sample_rate_hz, output, scratch, &[], &[], transport);
        node.process(&mut ctx);
    }
    output.copy_from_slice(scratch);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48_000;
    const FRAMES: usize = 256;
    const CHANNELS: usize = 2;

    fn config() -> AudioConfig {
        AudioConfig::new(SR, FRAMES as u32, CHANNELS as u16, CHANNELS as u16).unwrap()
    }

    // A channel-major stereo impulse on both channels
    fn impulse() -> Vec<f32> {
        let mut buf = vec![0.0f32; CHANNELS * FRAMES];
        buf[0] = 1.0;
        buf[FRAMES] = 1.0;
        buf
    }

    #[test]
    fn bypassed_chain_leaves_audio_untouched() {
        let mut fx = FxChain::new(CHANNELS, FRAMES, SR);
        fx.prepare(&config());
        let original = impulse();
        let mut buf = original.clone();
        fx.process(&mut buf, FRAMES);
        assert_eq!(buf, original, "bypassed chain must be transparent");
    }

    #[test]
    fn enabled_reverb_spreads_an_impulse() {
        let mut fx = FxChain::new(CHANNELS, FRAMES, SR);
        fx.prepare(&config());
        fx.set_reverb(true);
        fx.set_reverb_mix(1.0);
        let mut buf = impulse();
        fx.process(&mut buf, FRAMES);
        let nonzero = buf.iter().filter(|&&s| s.abs() > 1e-6).count();
        assert!(nonzero > 10, "reverb did not spread the impulse: {nonzero}");
        assert!(buf.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn enabled_delay_adds_a_later_tap() {
        let mut fx = FxChain::new(CHANNELS, FRAMES, SR);
        fx.prepare(&config());
        fx.set_delay(true);
        // Feed an impulse, then run enough silent blocks to pass the 250 ms tap
        // (12 000 samples / 256-frame blocks ~= 47 blocks)
        let mut buf = impulse();
        fx.process(&mut buf, FRAMES);
        let mut energy_later = 0.0f32;
        for _ in 0..64 {
            let mut silent = vec![0.0f32; CHANNELS * FRAMES];
            fx.process(&mut silent, FRAMES);
            energy_later += silent.iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(energy_later > 0.0, "delay produced no later tap");
    }
}
