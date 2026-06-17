// =============================================================================
// File: plugins/geist-modular/src/math.rs
// Layer: modular utilities
// Purpose: Add, Multiply, Abs, Clip, Rescale nodes
// Status: Implemented; reduction mixers and per-channel maps.
// Notes: Reduction nodes (Add, Multiply) fold all patched input channels into
//        output 0 around a scalar seed. Map nodes shape each channel in place.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use crate::util::{map_per_channel, reduce_into_ch0};

// Sum of every input channel plus a constant bias, into output 0
// With no inputs this emits the bias as DC; it doubles as a CV mixer
pub struct AddNode {
    bias: f32,
}

impl AddNode {
    // Build an adder with the given DC bias
    pub fn new(bias: f32) -> Self {
        Self { bias }
    }

    // Set the constant added to the channel sum
    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias;
    }
}

impl Default for AddNode {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl AudioNode for AddNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        reduce_into_ch0(ctx, self.bias, |a, b| a + b);
    }
}

// Product of every input channel scaled by a gain, into output 0
// Two inputs make a VCA or ring modulator; the seed acts as the gain control
pub struct MultiplyNode {
    gain: f32,
}

impl MultiplyNode {
    // Build a multiplier with the given output gain
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }

    // Set the scalar gain applied to the channel product
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl Default for MultiplyNode {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl AudioNode for MultiplyNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        reduce_into_ch0(ctx, self.gain, |a, b| a * b);
    }
}

// Full-wave rectifier: output is the magnitude of each input sample
#[derive(Default)]
pub struct AbsNode;

impl AudioNode for AbsNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        map_per_channel(ctx, f32::abs);
    }
}

// Hard range limiter clamping each sample into [lo, hi]
pub struct ClipNode {
    lo: f32,
    hi: f32,
}

impl ClipNode {
    // Build a clip with explicit bounds; bounds are ordered defensively
    pub fn new(lo: f32, hi: f32) -> Self {
        Self {
            lo: lo.min(hi),
            hi: lo.max(hi),
        }
    }

    // Set the clip bounds, ordered so lo <= hi
    pub fn set_bounds(&mut self, lo: f32, hi: f32) {
        self.lo = lo.min(hi);
        self.hi = lo.max(hi);
    }
}

impl Default for ClipNode {
    fn default() -> Self {
        Self::new(-1.0, 1.0)
    }
}

impl AudioNode for ClipNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let (lo, hi) = (self.lo, self.hi);
        map_per_channel(ctx, |x| x.clamp(lo, hi));
    }
}

// Linear remap from an input range onto an output range
// A degenerate input range collapses every sample to out_min
pub struct RescaleNode {
    in_min: f32,
    in_max: f32,
    out_min: f32,
    out_max: f32,
}

impl RescaleNode {
    // Build a rescale mapping [in_min, in_max] onto [out_min, out_max]
    pub fn new(in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> Self {
        Self {
            in_min,
            in_max,
            out_min,
            out_max,
        }
    }

    // Set the source range
    pub fn set_input_range(&mut self, in_min: f32, in_max: f32) {
        self.in_min = in_min;
        self.in_max = in_max;
    }

    // Set the target range
    pub fn set_output_range(&mut self, out_min: f32, out_max: f32) {
        self.out_min = out_min;
        self.out_max = out_max;
    }
}

impl Default for RescaleNode {
    // Map the bipolar unit range onto the unipolar unit range
    fn default() -> Self {
        Self::new(-1.0, 1.0, 0.0, 1.0)
    }
}

impl AudioNode for RescaleNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let span = self.in_max - self.in_min;
        let (in_min, out_min, out_max) = (self.in_min, self.out_min, self.out_max);
        if span.abs() <= f32::EPSILON {
            map_per_channel(ctx, |_| out_min);
            return;
        }
        let scale = (out_max - out_min) / span;
        map_per_channel(ctx, |x| out_min + (x - in_min) * scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::transport::TransportSnapshot;

    const SR: u32 = 48_000;
    const FRAMES: usize = 8;

    // Run one block; input is channel-major by FRAMES, output has out_ch channels
    fn run(node: &mut impl AudioNode, input: &[f32], out_ch: usize) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * out_ch];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn add_sums_two_channels_plus_bias() {
        let mut node = AddNode::new(0.5);
        let mut input = vec![0.0f32; FRAMES * 2];
        input[..FRAMES].fill(1.0);
        input[FRAMES..].fill(2.0);
        let out = run(&mut node, &input, 1);
        assert!(out.iter().all(|&s| (s - 3.5).abs() < 1e-6));
    }

    #[test]
    fn add_with_no_inputs_emits_bias_dc() {
        let mut node = AddNode::new(-0.25);
        let out = run(&mut node, &[], 1);
        assert!(out.iter().all(|&s| (s + 0.25).abs() < 1e-6));
    }

    #[test]
    fn multiply_is_a_two_input_vca() {
        let mut node = MultiplyNode::new(1.0);
        let mut input = vec![0.0f32; FRAMES * 2];
        input[..FRAMES].fill(0.8);
        input[FRAMES..].fill(0.5);
        let out = run(&mut node, &input, 1);
        assert!(out.iter().all(|&s| (s - 0.4).abs() < 1e-6));
    }

    #[test]
    fn abs_rectifies_each_channel() {
        let mut node = AbsNode;
        let input = vec![-0.3f32; FRAMES * 2];
        let out = run(&mut node, &input, 2);
        assert!(out.iter().all(|&s| (s - 0.3).abs() < 1e-6));
    }

    #[test]
    fn clip_bounds_are_inclusive_and_ordered() {
        let mut node = ClipNode::new(1.0, -1.0); // swapped on purpose
        let input = vec![5.0f32, -5.0, 0.2, 9.0, -9.0, 0.0, 1.0, -1.0];
        let out = run(&mut node, &input, 1);
        assert_eq!(out, vec![1.0, -1.0, 0.2, 1.0, -1.0, 0.0, 1.0, -1.0]);
    }

    #[test]
    fn rescale_maps_bipolar_to_unipolar() {
        let mut node = RescaleNode::default();
        let input = vec![-1.0f32, 0.0, 1.0, -0.5, 0.5, -1.0, 1.0, 0.0];
        let out = run(&mut node, &input, 1);
        let want = vec![0.0f32, 0.5, 1.0, 0.25, 0.75, 0.0, 1.0, 0.5];
        for (o, w) in out.iter().zip(&want) {
            assert!((o - w).abs() < 1e-6, "{o} != {w}");
        }
    }

    #[test]
    fn rescale_degenerate_input_range_collapses() {
        let mut node = RescaleNode::new(0.5, 0.5, 2.0, 9.0);
        let input = vec![0.5f32; FRAMES];
        let out = run(&mut node, &input, 1);
        assert!(out.iter().all(|&s| (s - 2.0).abs() < 1e-6));
    }
}
