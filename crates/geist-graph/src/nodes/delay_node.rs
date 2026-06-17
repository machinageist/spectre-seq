// =============================================================================
// File: crates/geist-graph/src/nodes/delay_node.rs
// Layer: audio graph
// Purpose: auto-inserted one-block delay for feedback loops
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;

use crate::node::AudioNode;

// Delays each channel by exactly one processing block
// Topology compilation inserts this to break feedback cycles
// History is sized on prepare; process never allocates
pub struct DelayNode {
    channels: usize,
    block_frames: usize,
    // Flat channel-major history of the previous block
    history: Vec<f32>,
}

impl DelayNode {
    // Build a delay for the given channel count; call prepare before processing
    pub fn new(channels: usize) -> Self {
        Self {
            channels,
            block_frames: 0,
            history: Vec::new(),
        }
    }
}

impl AudioNode for DelayNode {
    // Size the one-block history for this configuration
    fn prepare(&mut self, config: &AudioConfig) {
        self.block_frames = config.frames_per_block();
        self.history = vec![0.0; self.channels * self.block_frames];
    }

    // Emit the previous block, then store the current block
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let count = frames.min(self.block_frames);
        let (input, output) = ctx.io();
        for (channel, out) in output.chunks_mut(frames).enumerate() {
            if channel >= self.channels {
                break;
            }
            let base = channel * self.block_frames;
            let history = &mut self.history[base..base + count];
            out[..count].copy_from_slice(history);
            out[count..].fill(0.0);
            let start = channel * frames;
            if let Some(current) = input.get(start..start + count) {
                history.copy_from_slice(current);
            }
        }
    }

    // Clear the delay line back to silence
    fn reset(&mut self) {
        self.history.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::transport::TransportSnapshot;

    // Run one mono block through a freshly built context
    fn run_block(node: &mut DelayNode, samples: &[f32]) -> Vec<f32> {
        let frames = samples.len();
        let mut output = vec![0.0f32; frames];
        let mut ctx = ProcessContext::new(
            frames,
            48_000,
            samples,
            &mut output,
            &[],
            &[],
            TransportSnapshot::stopped(48_000),
        );
        node.process(&mut ctx);
        output
    }

    #[test]
    fn first_block_is_silent_then_delays_by_one_block() {
        let mut node = DelayNode::new(1);
        node.prepare(&AudioConfig::new(48_000, 4, 0, 1).unwrap());
        assert_eq!(
            run_block(&mut node, &[1.0, 2.0, 3.0, 4.0]),
            vec![0.0, 0.0, 0.0, 0.0]
        );
        assert_eq!(
            run_block(&mut node, &[5.0, 6.0, 7.0, 8.0]),
            vec![1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(
            run_block(&mut node, &[0.0, 0.0, 0.0, 0.0]),
            vec![5.0, 6.0, 7.0, 8.0]
        );
    }

    #[test]
    fn reset_clears_the_delay_line() {
        let mut node = DelayNode::new(1);
        node.prepare(&AudioConfig::new(48_000, 4, 0, 1).unwrap());
        run_block(&mut node, &[1.0, 2.0, 3.0, 4.0]);
        node.reset();
        assert_eq!(
            run_block(&mut node, &[9.0, 9.0, 9.0, 9.0]),
            vec![0.0, 0.0, 0.0, 0.0]
        );
    }
}
