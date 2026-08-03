// =============================================================================
// File: plugins/geist-fx/src/reverb/daw_node.rs
// Layer: effects plugin
// Purpose: FFT convolution Reverb wrapped as a graph AudioNode
// Status: Implemented; block-based stereo reverb over channel-major buffers.
// Notes: The reverb is FFT block-based, so it is built in prepare() sized to the
//        stream block. process() only runs when the block matches that size.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use crate::io::copy_input_to_output;
use crate::reverb::engine::Reverb;

// Defaults used before prepare supplies the real stream configuration
const DEFAULT_SAMPLE_RATE: f32 = 48_000.0;
const DEFAULT_BLOCK: usize = 512;
const DEFAULT_DECAY_SECONDS: f32 = 1.5;
const DEFAULT_MIX: f32 = 0.3;

// Graph node applying a stereo convolution reverb
pub struct ReverbNode {
    reverb: Reverb,
    decay_seconds: f32,
    mix: f32,
    // Right-channel scratch for a mono stream
    scratch: Vec<f32>,
}

impl ReverbNode {
    // Build a reverb node with a default room; prepare rebuilds at the stream rate
    pub fn new() -> Self {
        let mut reverb = Reverb::new(DEFAULT_SAMPLE_RATE, DEFAULT_BLOCK, DEFAULT_DECAY_SECONDS);
        reverb.set_mix(DEFAULT_MIX);
        Self {
            reverb,
            decay_seconds: DEFAULT_DECAY_SECONDS,
            mix: DEFAULT_MIX,
            scratch: Vec::new(),
        }
    }

    // Set the reverb decay time in seconds; takes effect on the next prepare
    pub fn set_decay_seconds(&mut self, seconds: f32) {
        self.decay_seconds = seconds.max(0.001);
    }

    // Set the dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
        self.reverb.set_mix(self.mix);
    }
}

impl Default for ReverbNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for ReverbNode {
    // Rebuild the reverb at the stream's sample rate and block size
    fn prepare(&mut self, config: &AudioConfig) {
        let block = config.frames_per_block();
        let mut reverb = Reverb::new(config.sample_rate_hz as f32, block, self.decay_seconds);
        reverb.set_mix(self.mix);
        self.reverb = reverb;
        self.scratch = vec![0.0; block];
    }

    // Mirror input to output, then apply the reverb in place when the block fits
    fn process(&mut self, ctx: &mut ProcessContext) {
        copy_input_to_output(ctx);
        let frames = ctx.frames();
        let channels = ctx.output_channels();
        // The FFT engine only accepts its configured block size
        if channels == 0 || frames != self.reverb.block_size() {
            return;
        }

        let (_input, output) = ctx.io();
        if channels >= 2 {
            let (left, right) = output.split_at_mut(frames);
            self.reverb
                .process(&mut left[..frames], &mut right[..frames]);
        } else {
            self.scratch[..frames].fill(0.0);
            self.reverb
                .process(&mut output[..frames], &mut self.scratch[..frames]);
        }
    }

    // Clear the reverb tails
    fn reset(&mut self) {
        self.reverb.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::transport::TransportSnapshot;

    const SR: u32 = 48_000;
    const FRAMES: usize = 256;

    fn config() -> AudioConfig {
        AudioConfig::new(SR, FRAMES as u32, 2, 2).unwrap()
    }

    fn run(node: &mut ReverbNode, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * 2];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn dry_passthrough_with_zero_mix() {
        let mut node = ReverbNode::new();
        node.set_mix(0.0);
        node.prepare(&config());
        let mut input = vec![0.0f32; FRAMES * 2];
        for (i, s) in input.iter_mut().enumerate() {
            *s = if i % 2 == 0 { 0.3 } else { -0.2 };
        }
        let out = run(&mut node, &input);
        assert_eq!(out, input);
    }

    #[test]
    fn wet_reverb_spreads_an_impulse() {
        let mut node = ReverbNode::new();
        node.set_decay_seconds(0.1);
        node.set_mix(1.0);
        node.prepare(&config());

        // Impulse on both channels at sample 0
        let mut input = vec![0.0f32; FRAMES * 2];
        input[0] = 1.0;
        input[FRAMES] = 1.0;
        let out = run(&mut node, &input);
        // Fully wet: energy is smeared across the block, not a lone spike
        let nonzero = out.iter().filter(|&&s| s.abs() > 1e-6).count();
        assert!(nonzero > 10, "reverb did not spread the impulse: {nonzero}");
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn silent_input_stays_silent() {
        let mut node = ReverbNode::new();
        node.prepare(&config());
        let input = vec![0.0f32; FRAMES * 2];
        let out = run(&mut node, &input);
        assert!(out.iter().all(|&s| s == 0.0));
    }
}
