// =============================================================================
// File: plugins/geist-fx/src/saturator/daw_node.rs
// Layer: effects plugin
// Purpose: Saturator wrapped as a graph AudioNode
// Status: Implemented; stateless waveshaping over the whole output buffer.
// Notes: The saturator has no inter-sample state, so one instance shapes every
//        channel; it runs over the flat channel-major buffer in one pass.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use crate::io::copy_input_to_output;
use crate::saturator::engine::{SaturationCurve, Saturator};

// Graph node applying a waveshaping saturator
pub struct SaturatorNode {
    saturator: Saturator,
}

impl SaturatorNode {
    // Build a saturator node with the given curve at unity drive
    pub fn new(curve: SaturationCurve) -> Self {
        Self {
            saturator: Saturator::new(curve),
        }
    }

    // Mutable access to the underlying saturator for parameter changes
    pub fn saturator_mut(&mut self) -> &mut Saturator {
        &mut self.saturator
    }
}

impl Default for SaturatorNode {
    fn default() -> Self {
        Self::new(SaturationCurve::Tanh)
    }
}

impl AudioNode for SaturatorNode {
    // Stateless: nothing to allocate or clear
    fn prepare(&mut self, _config: &AudioConfig) {}

    // Mirror input to output, then shape the whole buffer in one pass
    fn process(&mut self, ctx: &mut ProcessContext) {
        copy_input_to_output(ctx);
        let (_input, output) = ctx.io();
        self.saturator.process(output);
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

    fn run(node: &mut SaturatorNode, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * 2];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn hard_clip_bounds_hot_input() {
        let mut node = SaturatorNode::new(SaturationCurve::HardClip);
        node.saturator_mut().set_drive(1.0);
        node.prepare(&config());
        // Input well above unity on both channels
        let input = vec![2.0f32; FRAMES * 2];
        let out = run(&mut node, &input);
        assert!(out.iter().all(|&s| (s - 1.0).abs() < 1e-6));
    }

    #[test]
    fn low_level_is_near_transparent() {
        let mut node = SaturatorNode::default();
        node.prepare(&config());
        let input = vec![0.02f32; FRAMES * 2];
        let out = run(&mut node, &input);
        assert!(out.iter().all(|&s| (s - 0.02).abs() < 1e-3));
    }

    #[test]
    fn output_is_odd_symmetric() {
        let mut node = SaturatorNode::new(SaturationCurve::Tanh);
        node.saturator_mut().set_drive(3.0);
        node.prepare(&config());
        let pos = run(&mut node, &vec![0.5f32; FRAMES * 2]);
        let neg = run(&mut node, &vec![-0.5f32; FRAMES * 2]);
        for (p, n) in pos.iter().zip(&neg) {
            assert!((p + n).abs() < 1e-6);
        }
    }
}
