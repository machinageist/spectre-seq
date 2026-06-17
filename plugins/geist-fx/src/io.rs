// =============================================================================
// File: plugins/geist-fx/src/io.rs
// Layer: effects plugin
// Purpose: Channel-major ProcessContext I/O helpers shared by effect nodes
// Status: Implemented; copy input to output, split output into channels.
// Notes: Effects process in place on the output buffer, so each node first
//        mirrors its input there. Buffers are channel-major by frames().
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::context::ProcessContext;

// Mirror the input buffer into the output, channel by channel
// Output channels beyond the input count are cleared to silence
pub fn copy_input_to_output(ctx: &mut ProcessContext) {
    let frames = ctx.frames();
    let in_ch = ctx.input_channels();
    let out_ch = ctx.output_channels();
    let (input, output) = ctx.io();
    for ch in 0..out_ch {
        let dst = &mut output[ch * frames..(ch + 1) * frames];
        if ch < in_ch {
            dst.copy_from_slice(&input[ch * frames..(ch + 1) * frames]);
        } else {
            dst.fill(0.0);
        }
    }
}
