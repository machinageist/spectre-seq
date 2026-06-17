// =============================================================================
// File: crates/geist-graph/src/nodes/mixer.rs
// Layer: audio graph
// Purpose: n-input summing node
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::context::ProcessContext;

use crate::node::AudioNode;

// Sums input channels into outputs, folding by output channel count
// Input channel i contributes to output channel i mod output_count
// Stateless and allocation-free
#[derive(Default)]
pub struct MixerNode;

impl MixerNode {
    // Build a mixer node
    pub fn new() -> Self {
        Self
    }
}

impl AudioNode for MixerNode {
    // Accumulate every input into its folded output channel
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let (input, output) = ctx.io();
        let output_count = output.len() / frames;
        if output_count == 0 {
            return;
        }
        let input_count = input.len() / frames;
        for (channel, out) in output.chunks_mut(frames).enumerate() {
            out.fill(0.0);
            let mut source = channel;
            while source < input_count {
                let start = source * frames;
                for (acc, sample) in out.iter_mut().zip(&input[start..start + frames]) {
                    *acc += *sample;
                }
                source += output_count;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::transport::TransportSnapshot;

    #[test]
    fn sums_inputs_into_single_output() {
        // Two mono input channels: [a x4, b x4]
        let input = [1.0f32, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0];
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
        MixerNode::new().process(&mut ctx);
        assert_eq!(ctx.output(0).unwrap(), &[3.0, 3.0, 3.0, 3.0]);
    }

    #[test]
    fn folds_inputs_by_output_channel() {
        // Two stereo sources [L0, R0, L1, R1] fold to one stereo pair
        let input = [1.0f32, 1.0, 2.0, 2.0, 10.0, 10.0, 20.0, 20.0];
        let mut output = [0.0f32; 4];
        let mut ctx = ProcessContext::new(
            2,
            48_000,
            &input,
            &mut output,
            &[],
            &[],
            TransportSnapshot::stopped(48_000),
        );
        MixerNode::new().process(&mut ctx);
        assert_eq!(ctx.output(0).unwrap(), &[11.0, 11.0]);
        assert_eq!(ctx.output(1).unwrap(), &[22.0, 22.0]);
    }
}
