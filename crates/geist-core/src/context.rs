// Author: Jeff
// Date: 2026-05-27
// Description: Borrowed realtime processing context passed to AudioNode::process.
// Notes: The context must not allocate, lock, own graph state, or cross thread boundaries.

use crate::events::{NoteEvent, ParameterChange};
use crate::transport::TransportSnapshot;
use core::marker::PhantomData;

// Per-block processing context handed to every node on the audio thread
// Buffers are channel-major and flat: channel c occupies [c*frames, (c+1)*frames)
// Flat layout lets the executor hand out disjoint input/output regions with no
// allocation and no unsafe; the raw-pointer marker keeps the type off other threads
pub struct ProcessContext<'a> {
    frames: usize,
    sample_rate_hz: u32,
    input: &'a [f32],
    output: &'a mut [f32],
    notes: &'a [NoteEvent],
    params: &'a [ParameterChange],
    transport: TransportSnapshot,
    _not_send: PhantomData<*const ()>,
}

impl<'a> ProcessContext<'a> {
    // Assemble a context for one processing block
    // Channel counts are derived from buffer length divided by frames
    pub fn new(
        frames: usize,
        sample_rate_hz: u32,
        input: &'a [f32],
        output: &'a mut [f32],
        notes: &'a [NoteEvent],
        params: &'a [ParameterChange],
        transport: TransportSnapshot,
    ) -> Self {
        debug_assert!(frames > 0, "block must have at least one frame");
        debug_assert!(
            input.len().is_multiple_of(frames),
            "input is not channel-major by frames"
        );
        debug_assert!(
            output.len().is_multiple_of(frames),
            "output is not channel-major by frames"
        );
        Self {
            frames,
            sample_rate_hz,
            input,
            output,
            notes,
            params,
            transport,
            _not_send: PhantomData,
        }
    }

    // Frames to process this block
    #[inline]
    pub fn frames(&self) -> usize {
        self.frames
    }

    // Active sample rate in hertz
    #[inline]
    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    // Block-stable transport readout
    #[inline]
    pub fn transport(&self) -> &TransportSnapshot {
        &self.transport
    }

    // Note events scheduled within this block, in arrival order
    #[inline]
    pub fn notes(&self) -> &[NoteEvent] {
        self.notes
    }

    // Parameter changes scheduled within this block, in arrival order
    #[inline]
    pub fn param_changes(&self) -> &[ParameterChange] {
        self.params
    }

    // Count of available input channels
    #[inline]
    pub fn input_channels(&self) -> usize {
        self.input.len() / self.frames
    }

    // Count of available output channels
    #[inline]
    pub fn output_channels(&self) -> usize {
        self.output.len() / self.frames
    }

    // Borrow one input channel for this block
    #[inline]
    pub fn input(&self, channel: usize) -> Option<&[f32]> {
        let start = channel.checked_mul(self.frames)?;
        self.input.get(start..start + self.frames)
    }

    // Borrow one output channel mutably for this block
    #[inline]
    pub fn output(&mut self, channel: usize) -> Option<&mut [f32]> {
        let frames = self.frames;
        let start = channel.checked_mul(frames)?;
        self.output.get_mut(start..start + frames)
    }

    // Borrow the flat input and output for nodes that read and write together
    // Both are channel-major by frames(); split with chunks/chunks_mut
    #[inline]
    pub fn io(&mut self) -> (&[f32], &mut [f32]) {
        (self.input, self.output)
    }

    // Write silence to every output channel
    pub fn clear_outputs(&mut self) {
        self.output.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_block_shape_and_transport() {
        let input = [0.0f32; 4];
        let mut output = [0.0f32; 4];
        let transport = TransportSnapshot::stopped(48_000);
        let ctx = ProcessContext::new(4, 48_000, &input, &mut output, &[], &[], transport);
        assert_eq!(ctx.frames(), 4);
        assert_eq!(ctx.sample_rate_hz(), 48_000);
        assert_eq!(ctx.input_channels(), 1);
        assert_eq!(ctx.output_channels(), 1);
        assert_eq!(ctx.transport().sample_rate_hz, 48_000);
    }

    #[test]
    fn channel_access_is_bounds_checked() {
        // Two input channels laid out channel-major: [ch0 x4, ch1 x4]
        let input = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut output = [0.0f32; 4];
        let transport = TransportSnapshot::stopped(48_000);
        let ctx = ProcessContext::new(4, 48_000, &input, &mut output, &[], &[], transport);
        assert_eq!(ctx.input_channels(), 2);
        assert_eq!(ctx.input(0), Some(&[1.0, 2.0, 3.0, 4.0][..]));
        assert_eq!(ctx.input(1), Some(&[5.0, 6.0, 7.0, 8.0][..]));
        assert!(ctx.input(2).is_none());
    }

    #[test]
    fn outputs_can_be_written_and_cleared() {
        let mut output = [9.0f32; 8];
        let transport = TransportSnapshot::stopped(48_000);
        let mut ctx = ProcessContext::new(4, 48_000, &[], &mut output, &[], &[], transport);
        assert_eq!(ctx.output_channels(), 2);
        ctx.output(1).unwrap().fill(0.5);
        assert_eq!(ctx.output(1).unwrap(), &[0.5, 0.5, 0.5, 0.5]);
        ctx.clear_outputs();
        assert_eq!(ctx.output(0).unwrap(), &[0.0, 0.0, 0.0, 0.0]);
    }
}
