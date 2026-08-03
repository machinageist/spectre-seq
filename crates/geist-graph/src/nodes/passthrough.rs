// =============================================================================
// File: crates/geist-graph/src/nodes/passthrough.rs
// Layer: audio graph
// Purpose: identity node; useful for testing + routing
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::context::ProcessContext;

use crate::node::AudioNode;

// Copies each input channel to the matching output channel
// Extra output channels are silenced; stateless and allocation-free
#[derive(Default)]
pub struct PassthroughNode;

impl PassthroughNode {
    // Build a passthrough node
    pub fn new() -> Self {
        Self
    }
}

impl AudioNode for PassthroughNode {
    // Mirror inputs onto outputs for one block
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let (input, output) = ctx.io();
        for (channel, out) in output.chunks_mut(frames).enumerate() {
            let start = channel * frames;
            match input.get(start..start + frames) {
                Some(inp) => out.copy_from_slice(inp),
                None => out.fill(0.0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::transport::TransportSnapshot;

    #[test]
    fn copies_input_to_output() {
        let input = [1.0f32, 2.0, 3.0, 4.0];
        let mut output = [0.0f32; 4];
        let mut ctx = ProcessContext::new(
            4,
            48_000,
            &input,
            &mut output,
            &[],
            &[],
            TransportSnapshot::stopped(48_000),
        );
        PassthroughNode::new().process(&mut ctx);
        assert_eq!(ctx.output(0).unwrap(), &[1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn silences_unmatched_output_channels() {
        let input = [1.0f32, 1.0, 1.0, 1.0];
        let mut output = [9.0f32; 8];
        let mut ctx = ProcessContext::new(
            4,
            48_000,
            &input,
            &mut output,
            &[],
            &[],
            TransportSnapshot::stopped(48_000),
        );
        PassthroughNode::new().process(&mut ctx);
        assert_eq!(ctx.output(0).unwrap(), &[1.0, 1.0, 1.0, 1.0]);
        assert_eq!(ctx.output(1).unwrap(), &[0.0, 0.0, 0.0, 0.0]);
    }
}
