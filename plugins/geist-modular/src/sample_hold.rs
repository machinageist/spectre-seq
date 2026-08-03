// =============================================================================
// File: plugins/geist-modular/src/sample_hold.rs
// Layer: modular utilities
// Purpose: Sample & Hold, Track & Hold nodes
// Status: Implemented; trigger-latched and gate-tracked holds.
// Notes: Both read signal on input 0 and a control on input 1. Sample & Hold
//        latches on the control's rising edge; Track & Hold follows while high.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use crate::util::{is_high, process_pair_ch0};

// Latches the signal on each rising edge of the trigger and holds it
// Feeding noise into the signal with a clock trigger yields stepped random CV
#[derive(Default)]
pub struct SampleAndHoldNode {
    held: f32,
    prev_high: bool,
}

impl SampleAndHoldNode {
    // Build a sample & hold starting from silence
    pub fn new() -> Self {
        Self::default()
    }
}

impl AudioNode for SampleAndHoldNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let mut held = self.held;
        let mut prev = self.prev_high;
        process_pair_ch0(ctx, |signal, trigger| {
            let now = is_high(trigger);
            if now && !prev {
                held = signal;
            }
            prev = now;
            held
        });
        self.held = held;
        self.prev_high = prev;
    }

    fn reset(&mut self) {
        self.held = 0.0;
        self.prev_high = false;
    }
}

// Tracks the signal while the gate is high and holds the last value when low
#[derive(Default)]
pub struct TrackAndHoldNode {
    held: f32,
}

impl TrackAndHoldNode {
    // Build a track & hold starting from silence
    pub fn new() -> Self {
        Self::default()
    }
}

impl AudioNode for TrackAndHoldNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let mut held = self.held;
        process_pair_ch0(ctx, |signal, gate| {
            if is_high(gate) {
                held = signal;
            }
            held
        });
        self.held = held;
    }

    fn reset(&mut self) {
        self.held = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::transport::TransportSnapshot;

    const SR: u32 = 48_000;
    const FRAMES: usize = 8;

    // Build a two-channel input: signal on ch0, control on ch1
    fn pair(signal: &[f32], control: &[f32]) -> Vec<f32> {
        let mut v = vec![0.0f32; FRAMES * 2];
        v[..FRAMES].copy_from_slice(signal);
        v[FRAMES..].copy_from_slice(control);
        v
    }

    fn run(node: &mut impl AudioNode, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn sample_hold_latches_on_trigger_edges() {
        let mut node = SampleAndHoldNode::new();
        let signal = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        // Rising edges at frames 2 and 5
        let trigger = [0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0];
        let out = run(&mut node, &pair(&signal, &trigger));
        // Holds 0 until first edge, then 0.3 (frame 2), then 0.6 (frame 5)
        assert_eq!(out, vec![0.0, 0.0, 0.3, 0.3, 0.3, 0.6, 0.6, 0.6]);
    }

    #[test]
    fn sample_hold_persists_across_blocks() {
        let mut node = SampleAndHoldNode::new();
        let signal = [0.9; FRAMES];
        let trigger = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let _ = run(&mut node, &pair(&signal, &trigger));
        // No new trigger; the held 0.9 must survive into the next block
        let silent_trigger = [0.0; FRAMES];
        let out = run(&mut node, &pair(&[0.0; FRAMES], &silent_trigger));
        assert!(out.iter().all(|&s| (s - 0.9).abs() < 1e-6));
    }

    #[test]
    fn track_hold_follows_while_gate_high() {
        let mut node = TrackAndHoldNode::new();
        let signal = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let gate = [1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let out = run(&mut node, &pair(&signal, &gate));
        // Tracks 0.1,0.2,0.3; holds 0.3 while low; tracks 0.6; holds 0.6
        assert_eq!(out, vec![0.1, 0.2, 0.3, 0.3, 0.3, 0.6, 0.6, 0.6]);
    }

    #[test]
    fn missing_control_holds_initial_value() {
        let mut node = SampleAndHoldNode::new();
        // Only the signal channel patched; no trigger ever fires
        let input = vec![0.5f32; FRAMES];
        let out = run(&mut node, &input);
        assert!(out.iter().all(|&s| s == 0.0));
    }
}
