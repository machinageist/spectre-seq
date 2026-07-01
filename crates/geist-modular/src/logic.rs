// =============================================================================
// File: crates/geist-modular/src/logic.rs
// Layer: modular utilities
// Purpose: Comparator, AND, OR, NOT, flip-flop nodes
// Status: Implemented; Schmitt comparator, gate reducers, edge-toggle.
// Notes: Gates use the shared threshold/level convention. The comparator and
//        flip-flop carry edge state, so they run mono on channel 0.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use crate::util::{
    gate_level, is_high, map_per_channel, process_mono_ch0, reduce_gate_into_ch0, GATE_HIGH,
    GATE_LOW,
};

// Schmitt-trigger comparator: output goes high above threshold + hysteresis
// and low below threshold - hysteresis, latching between the two rails
pub struct ComparatorNode {
    threshold: f32,
    hysteresis: f32,
    high: bool,
}

impl ComparatorNode {
    // Build a comparator at a threshold with optional hysteresis (>= 0)
    pub fn new(threshold: f32, hysteresis: f32) -> Self {
        Self {
            threshold,
            hysteresis: hysteresis.max(0.0),
            high: false,
        }
    }

    // Set the comparison threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    // Set the symmetric hysteresis band around the threshold
    pub fn set_hysteresis(&mut self, hysteresis: f32) {
        self.hysteresis = hysteresis.max(0.0);
    }
}

impl Default for ComparatorNode {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl AudioNode for ComparatorNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let upper = self.threshold + self.hysteresis;
        let lower = self.threshold - self.hysteresis;
        let mut high = self.high;
        process_mono_ch0(ctx, |x| {
            if high {
                if x <= lower {
                    high = false;
                }
            } else if x >= upper {
                high = true;
            }
            gate_level(high)
        });
        self.high = high;
    }

    fn reset(&mut self) {
        self.high = false;
    }
}

// Logical AND of every input channel's gate state into output 0
// The empty product is true, matching the identity of conjunction
#[derive(Default)]
pub struct AndNode;

impl AudioNode for AndNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        reduce_gate_into_ch0(ctx, true, |a, b| a && b);
    }
}

// Logical OR of every input channel's gate state into output 0
#[derive(Default)]
pub struct OrNode;

impl AudioNode for OrNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        reduce_gate_into_ch0(ctx, false, |a, b| a || b);
    }
}

// Per-channel gate inversion: high becomes low and low becomes high
#[derive(Default)]
pub struct NotNode;

impl AudioNode for NotNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        map_per_channel(ctx, |x| if is_high(x) { GATE_LOW } else { GATE_HIGH });
    }
}

// T flip-flop: each rising edge on input 0 toggles the output gate
// Halves an incoming clock; output level reflects the latched state
#[derive(Default)]
pub struct FlipFlopNode {
    state: bool,
    prev_high: bool,
}

impl FlipFlopNode {
    // Build a flip-flop starting in the low state
    pub fn new() -> Self {
        Self::default()
    }
}

impl AudioNode for FlipFlopNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let mut state = self.state;
        let mut prev = self.prev_high;
        process_mono_ch0(ctx, |x| {
            let now = is_high(x);
            if now && !prev {
                state = !state;
            }
            prev = now;
            gate_level(state)
        });
        self.state = state;
        self.prev_high = prev;
    }

    fn reset(&mut self) {
        self.state = false;
        self.prev_high = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::transport::TransportSnapshot;

    const SR: u32 = 48_000;
    const FRAMES: usize = 8;

    fn run(node: &mut impl AudioNode, input: &[f32], out_ch: usize) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * out_ch];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn comparator_hysteresis_latches() {
        let mut node = ComparatorNode::new(0.0, 0.2);
        // Sequence crosses 0 but stays inside the hysteresis band after rising
        let input = vec![0.3f32, 0.1, -0.1, -0.3, 0.1, 0.25, -0.25, 0.0];
        let out = run(&mut node, &input, 1);
        // 0.3 -> high; 0.1,-0.1 inside band stay high; -0.3 -> low; 0.1 inside stays low;
        // 0.25 -> high; -0.25 -> low; 0.0 inside stays low
        assert_eq!(out, vec![1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn and_requires_all_inputs_high() {
        let mut node = AndNode;
        let mut input = vec![0.0f32; FRAMES * 2];
        input[..FRAMES].fill(1.0);
        // ch1 high only in the first half of the block
        input[FRAMES..FRAMES + 4].fill(1.0);
        let out = run(&mut node, &input, 1);
        assert_eq!(out, vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn or_passes_any_high_input() {
        let mut node = OrNode;
        let mut input = vec![0.0f32; FRAMES * 2];
        input[2] = 1.0; // ch0 high at one frame
        input[FRAMES + 5] = 1.0; // ch1 high at another
        let out = run(&mut node, &input, 1);
        assert_eq!(out, vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn not_inverts_gates() {
        let mut node = NotNode;
        let input = vec![1.0f32, 0.0, 1.0, 0.0, 0.7, 0.2, 1.0, 0.0];
        let out = run(&mut node, &input, 1);
        assert_eq!(out, vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn flip_flop_toggles_on_rising_edges() {
        let mut node = FlipFlopNode::new();
        // Two clean pulses: each rising edge flips the output
        let input = vec![0.0f32, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
        let out = run(&mut node, &input, 1);
        // edge at idx1 -> high (stays for 1,2); falls at 3; edge at idx5 -> low
        assert_eq!(out, vec![0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn flip_flop_state_carries_across_blocks() {
        let mut node = FlipFlopNode::new();
        // One rising edge per block; state must persist between calls
        let block = vec![0.0f32, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let first = run(&mut node, &block, 1);
        assert_eq!(first[7], 1.0);
        // No new rising edge here (starts high), so no toggle
        let high = vec![1.0f32; FRAMES];
        let second = run(&mut node, &high, 1);
        assert!(second.iter().all(|&s| s == 1.0));
    }
}
