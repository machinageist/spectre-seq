// =============================================================================
// File: crates/geist-graph/src/nodes/monitor.rs
// Layer: audio graph
// Purpose: metering node; publishes latest block peak/RMS to the UI
// Status: Implemented; atomic latest-value cell. Sample-stream rings land with channels.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use geist_core::context::ProcessContext;

use crate::node::AudioNode;

// Lock-free latest-value meter shared between the audio thread and UI
// Audio thread stores each block's peak/RMS; UI polls without blocking
// A level meter only needs the newest value, so a ring buffer is unnecessary here
#[derive(Debug, Default)]
pub struct MeterCell {
    peak: AtomicU32,
    rms: AtomicU32,
}

impl MeterCell {
    // Build a meter cell reading zero until the first block runs
    pub fn new() -> Self {
        Self::default()
    }

    // Publish the latest block measurements from the audio thread
    fn store(&self, peak: f32, rms: f32) {
        self.peak.store(peak.to_bits(), Ordering::Relaxed);
        self.rms.store(rms.to_bits(), Ordering::Relaxed);
    }

    // Read the most recent block peak and RMS
    pub fn read(&self) -> (f32, f32) {
        (
            f32::from_bits(self.peak.load(Ordering::Relaxed)),
            f32::from_bits(self.rms.load(Ordering::Relaxed)),
        )
    }
}

// Taps level metering while passing audio through unchanged
// Peak is the max absolute sample; RMS is taken over all input channels this block
pub struct MonitorNode {
    meter: Arc<MeterCell>,
}

impl MonitorNode {
    // Build a monitor publishing into an existing meter cell
    pub fn new(meter: Arc<MeterCell>) -> Self {
        Self { meter }
    }

    // Build a monitor with a fresh meter cell, returning a UI-side handle
    pub fn with_cell() -> (Self, Arc<MeterCell>) {
        let meter = Arc::new(MeterCell::new());
        (Self::new(meter.clone()), meter)
    }
}

impl AudioNode for MonitorNode {
    // Measure the block, publish it, then mirror inputs onto outputs
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let (input, output) = ctx.io();

        // Peak and RMS across every input sample; f64 accumulation guards precision
        let mut peak = 0.0f32;
        let mut sum_squares = 0.0f64;
        for &sample in input {
            peak = peak.max(sample.abs());
            sum_squares += (sample as f64) * (sample as f64);
        }
        let rms = if input.is_empty() {
            0.0
        } else {
            (sum_squares / input.len() as f64).sqrt() as f32
        };
        self.meter.store(peak, rms);

        // Passthrough so the monitor can sit inline in a signal chain
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
    use geist_core::transport::TransportSnapshot;

    #[test]
    fn meter_cell_round_trips() {
        let cell = MeterCell::new();
        assert_eq!(cell.read(), (0.0, 0.0));
        cell.store(0.8, 0.3);
        assert_eq!(cell.read(), (0.8, 0.3));
    }

    #[test]
    fn measures_peak_and_rms_and_passes_audio_through() {
        let input = [0.5f32, -1.0, 0.25, 0.0];
        let mut output = [0.0f32; 4];
        let (mut node, meter) = MonitorNode::with_cell();
        let mut ctx = ProcessContext::new(
            4,
            48_000,
            &input,
            &mut output,
            &[],
            &[],
            TransportSnapshot::stopped(48_000),
        );
        node.process(&mut ctx);

        let (peak, rms) = meter.read();
        assert!((peak - 1.0).abs() < 1e-6);
        // sqrt((0.25 + 1.0 + 0.0625 + 0.0) / 4) = 0.572822
        assert!((rms - 0.572_822).abs() < 1e-4);
        assert_eq!(ctx.output(0).unwrap(), &[0.5, -1.0, 0.25, 0.0]);
    }
}
