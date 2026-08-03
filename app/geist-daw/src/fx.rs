// =============================================================================
// File: app/geist-daw/src/fx.rs
// Layer: application binary
// Purpose: Post-synth effects chain (delay -> reverb) over channel-major buffers
// Status: Implemented; toggleable delay and reverb with a ping-pong scratch.
// Notes: Each effect mirrors input to output then processes in place, so the
//        chain feeds one effect's output into the next through a single scratch
//        buffer sized once on the app thread. process() never allocates.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use spectre_core::transport::TransportSnapshot;
use spectre_fx::prelude::{DelayNode, ReverbNode};
use spectre_graph::node::AudioNode;

// Ordered effects applied after the synth, each independently bypassable
pub struct FxChain {
    delay: DelayNode,
    reverb: ReverbNode,
    delay_on: bool,
    reverb_on: bool,
    sample_rate_hz: u32,
    // Ping-pong buffer the size of one channel-major block
    scratch: Vec<f32>,
}

impl FxChain {
    // Build a bypassed chain sized for the stream block
    pub fn new(channels: usize, block_frames: usize, sample_rate_hz: u32) -> Self {
        Self {
            delay: DelayNode::new(),
            reverb: ReverbNode::new(),
            delay_on: false,
            reverb_on: false,
            sample_rate_hz,
            scratch: vec![0.0; channels * block_frames],
        }
    }

    // Rebuild both effects for the stream's rate and block size
    pub fn prepare(&mut self, config: &AudioConfig) {
        self.delay.prepare(config);
        self.reverb.prepare(config);
    }

    // Toggle the delay
    pub fn set_delay(&mut self, on: bool) {
        self.delay_on = on;
    }

    // Set the delay time in seconds on both channels
    pub fn set_delay_time(&mut self, seconds: f32) {
        let sr = self.sample_rate_hz as f32;
        self.delay
            .delay_mut()
            .set_delay_seconds(seconds, seconds, sr);
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

    // Apply the enabled effects in order, in place on channel-major `output`
    pub fn process(&mut self, output: &mut [f32], frames: usize) {
        let FxChain {
            delay,
            reverb,
            delay_on,
            reverb_on,
            sample_rate_hz,
            scratch,
        } = self;
        if *delay_on {
            apply(delay, output, scratch, frames, *sample_rate_hz);
        }
        if *reverb_on {
            apply(reverb, output, scratch, frames, *sample_rate_hz);
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
