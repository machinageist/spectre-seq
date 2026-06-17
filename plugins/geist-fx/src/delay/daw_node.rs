// =============================================================================
// File: plugins/geist-fx/src/delay/daw_node.rs
// Layer: effects plugin
// Purpose: StereoDelay wrapped as a graph AudioNode
// Status: Implemented; stereo in-place delay over channel-major buffers.
// Notes: Mirrors input into the output, then delays channels 0/1 in place.
//        A mono stream is delayed against a throwaway right channel.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use crate::delay::engine::StereoDelay;
use crate::io::copy_input_to_output;

// Maximum delay the buffer can hold; ~4 s at 48 kHz
const MAX_DELAY_SAMPLES: usize = 192_000;

// Graph node applying a stereo feedback delay
pub struct DelayNode {
    delay: StereoDelay,
    // Right-channel scratch for delaying a mono stream
    scratch: Vec<f32>,
}

impl DelayNode {
    // Build a delay node with musical defaults
    pub fn new() -> Self {
        let mut delay = StereoDelay::new(MAX_DELAY_SAMPLES);
        delay.set_delay_samples(12_000.0, 12_000.0); // 250 ms at 48 kHz
        delay.set_feedback(0.3);
        delay.set_mix(0.3);
        Self {
            delay,
            scratch: Vec::new(),
        }
    }

    // Mutable access to the underlying delay for parameter changes
    pub fn delay_mut(&mut self) -> &mut StereoDelay {
        &mut self.delay
    }
}

impl Default for DelayNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for DelayNode {
    // Size the mono scratch buffer and clear the delay lines
    fn prepare(&mut self, config: &AudioConfig) {
        self.scratch = vec![0.0; config.frames_per_block()];
        self.delay.reset();
    }

    // Mirror input to output, then apply the delay in place
    fn process(&mut self, ctx: &mut ProcessContext) {
        copy_input_to_output(ctx);
        let frames = ctx.frames();
        let channels = ctx.output_channels();
        if channels == 0 {
            return;
        }

        let (_input, output) = ctx.io();
        if channels >= 2 {
            let (left, right) = output.split_at_mut(frames);
            self.delay
                .process(&mut left[..frames], &mut right[..frames]);
        } else {
            if self.scratch.len() < frames {
                self.scratch.resize(frames, 0.0);
            }
            self.scratch[..frames].fill(0.0);
            self.delay
                .process(&mut output[..frames], &mut self.scratch[..frames]);
        }
    }

    // Clear the delay tails
    fn reset(&mut self) {
        self.delay.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::transport::TransportSnapshot;

    const SR: u32 = 48_000;
    const FRAMES: usize = 256;

    fn config() -> AudioConfig {
        AudioConfig::new(SR, FRAMES as u32, 2, 2).unwrap()
    }

    // Run one block of stereo input through the node, returning the output
    fn run(node: &mut DelayNode, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * 2];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn dry_signal_passes_through_immediately() {
        let mut node = DelayNode::new();
        node.delay_mut().set_delay_samples(128.0, 128.0);
        node.delay_mut().set_feedback(0.0);
        node.delay_mut().set_mix(0.5);
        node.prepare(&config());

        // Impulse on the left channel only
        let mut input = vec![0.0f32; FRAMES * 2];
        input[0] = 1.0;
        let out = run(&mut node, &input);
        // Dry component appears at sample 0 (mix 0.5 -> 0.5)
        assert!((out[0] - 0.5).abs() < 1e-5, "dry passthrough = {}", out[0]);
    }

    #[test]
    fn wet_tap_appears_after_the_delay() {
        let mut node = DelayNode::new();
        node.delay_mut().set_delay_samples(100.0, 100.0);
        node.delay_mut().set_feedback(0.0);
        node.delay_mut().set_mix(1.0); // fully wet
        node.prepare(&config());

        let mut input = vec![0.0f32; FRAMES * 2];
        input[0] = 1.0; // left impulse at sample 0
        let out = run(&mut node, &input);
        // Fully wet: nothing at 0, the delayed tap lands at sample 100 on the left
        assert!(out[0].abs() < 1e-6, "unexpected dry signal: {}", out[0]);
        assert!((out[100] - 1.0).abs() < 1e-5, "delayed tap = {}", out[100]);
    }

    #[test]
    fn silent_input_stays_silent() {
        let mut node = DelayNode::new();
        node.prepare(&config());
        let input = vec![0.0f32; FRAMES * 2];
        let out = run(&mut node, &input);
        assert!(out.iter().all(|&s| s == 0.0));
    }
}
