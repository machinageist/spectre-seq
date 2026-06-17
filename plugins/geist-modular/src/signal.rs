// =============================================================================
// File: plugins/geist-modular/src/signal.rs
// Layer: modular utilities
// Purpose: Mux, Demux, Attenuverter, DC offset nodes
// Status: Implemented; selector routing and per-channel scaling.
// Notes: Mux/Demux route by a select index. Attenuverter and DC offset are
//        per-channel maps; attenuverter gain is bipolar within [-1, 1].
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use crate::util::map_per_channel;

// Selects one input channel onto output 0; other outputs are silenced
// An out-of-range selection yields silence rather than wrapping
pub struct MuxNode {
    select: usize,
}

impl MuxNode {
    // Build a mux routing the given input channel index
    pub fn new(select: usize) -> Self {
        Self { select }
    }

    // Choose which input channel reaches the output
    pub fn set_select(&mut self, select: usize) {
        self.select = select;
    }
}

impl Default for MuxNode {
    fn default() -> Self {
        Self::new(0)
    }
}

impl AudioNode for MuxNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let sel = self.select;
        let (input, output) = ctx.io();
        if sel < in_ch {
            output[..frames].copy_from_slice(&input[sel * frames..(sel + 1) * frames]);
        } else {
            output[..frames].fill(0.0);
        }
        for ch in 1..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }
}

// Routes input 0 onto the selected output channel; other outputs are silenced
pub struct DemuxNode {
    select: usize,
}

impl DemuxNode {
    // Build a demux routing to the given output channel index
    pub fn new(select: usize) -> Self {
        Self { select }
    }

    // Choose which output channel receives the input
    pub fn set_select(&mut self, select: usize) {
        self.select = select;
    }
}

impl Default for DemuxNode {
    fn default() -> Self {
        Self::new(0)
    }
}

impl AudioNode for DemuxNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        let sel = self.select;
        let (input, output) = ctx.io();
        output.fill(0.0);
        if sel < out_ch && in_ch >= 1 {
            output[sel * frames..(sel + 1) * frames].copy_from_slice(&input[..frames]);
        }
    }
}

// Attenuverter: scales each channel by a bipolar gain, then adds an offset
// Gain in [-1, 1] lets a single control attenuate or invert a CV
pub struct AttenuverterNode {
    gain: f32,
    offset: f32,
}

impl AttenuverterNode {
    // Build an attenuverter; gain is clamped to the bipolar unit range
    pub fn new(gain: f32, offset: f32) -> Self {
        Self {
            gain: gain.clamp(-1.0, 1.0),
            offset,
        }
    }

    // Set the bipolar gain, clamped to [-1, 1]
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(-1.0, 1.0);
    }

    // Set the DC offset added after scaling
    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }
}

impl Default for AttenuverterNode {
    fn default() -> Self {
        Self::new(1.0, 0.0)
    }
}

impl AudioNode for AttenuverterNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let (gain, offset) = (self.gain, self.offset);
        map_per_channel(ctx, |x| x * gain + offset);
    }
}

// Adds a constant offset to every channel
pub struct DcOffsetNode {
    offset: f32,
}

impl DcOffsetNode {
    // Build a DC offset of the given amount
    pub fn new(offset: f32) -> Self {
        Self { offset }
    }

    // Set the constant added to each sample
    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }
}

impl Default for DcOffsetNode {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl AudioNode for DcOffsetNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let offset = self.offset;
        map_per_channel(ctx, |x| x + offset);
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
    fn mux_selects_one_channel() {
        let mut node = MuxNode::new(1);
        let mut input = vec![0.0f32; FRAMES * 3];
        input[..FRAMES].fill(0.1);
        input[FRAMES..2 * FRAMES].fill(0.2);
        input[2 * FRAMES..].fill(0.3);
        let out = run(&mut node, &input, 1);
        assert!(out.iter().all(|&s| (s - 0.2).abs() < 1e-6));
    }

    #[test]
    fn mux_out_of_range_is_silent() {
        let mut node = MuxNode::new(5);
        let input = vec![0.7f32; FRAMES * 2];
        let out = run(&mut node, &input, 1);
        assert!(out.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn demux_routes_to_selected_output() {
        let mut node = DemuxNode::new(2);
        let input = vec![0.4f32; FRAMES];
        let out = run(&mut node, &input, 4);
        // Only channel 2 carries the signal; the rest stay silent
        assert!(out[..2 * FRAMES].iter().all(|&s| s == 0.0));
        assert!(out[2 * FRAMES..3 * FRAMES]
            .iter()
            .all(|&s| (s - 0.4).abs() < 1e-6));
        assert!(out[3 * FRAMES..].iter().all(|&s| s == 0.0));
    }

    #[test]
    fn attenuverter_inverts_and_offsets() {
        let mut node = AttenuverterNode::new(-0.5, 0.25);
        let input = vec![1.0f32, -1.0, 0.5, -0.5, 0.0, 2.0, -2.0, 1.0];
        let out = run(&mut node, &input, 1);
        let want = vec![-0.25f32, 0.75, 0.0, 0.5, 0.25, -0.75, 1.25, -0.25];
        for (o, w) in out.iter().zip(&want) {
            assert!((o - w).abs() < 1e-6, "{o} != {w}");
        }
    }

    #[test]
    fn attenuverter_gain_is_clamped_bipolar() {
        let mut node = AttenuverterNode::new(9.0, 0.0);
        let input = vec![1.0f32; FRAMES];
        let out = run(&mut node, &input, 1);
        // Gain clamps to +1.0, so unity passes through
        assert!(out.iter().all(|&s| (s - 1.0).abs() < 1e-6));
    }

    #[test]
    fn dc_offset_shifts_signal() {
        let mut node = DcOffsetNode::new(0.3);
        let input = vec![0.0f32, 0.1, -0.1, 0.2, -0.2, 0.0, 0.5, -0.5];
        let out = run(&mut node, &input, 1);
        for (o, i) in out.iter().zip(&input) {
            assert!((o - (i + 0.3)).abs() < 1e-6);
        }
    }
}
