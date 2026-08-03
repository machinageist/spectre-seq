// =============================================================================
// File: plugins/geist-modular/src/timing.rs
// Layer: modular utilities
// Purpose: Clock divider, gate delay, slew limiter nodes
// Status: Implemented; edge counting, per-channel delay lines, rate limiting.
// Notes: Gate delay allocates ring buffers in prepare() sized to a max delay;
//        process() never allocates. Slew rates derive from the stream rate.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use spectre_graph::node::AudioNode;

use crate::util::{is_high, process_mono_ch0, GATE_HIGH, GATE_LOW};

// Divides an incoming clock: passes every Nth rising-edge pulse through
// Pulse width is preserved; non-passing pulses output low for their duration
pub struct ClockDividerNode {
    division: u32,
    count: u64,
    prev_high: bool,
    pass: bool,
}

impl ClockDividerNode {
    // Build a divider passing one pulse per `division` input pulses (>= 1)
    pub fn new(division: u32) -> Self {
        Self {
            division: division.max(1),
            count: 0,
            prev_high: false,
            pass: false,
        }
    }

    // Set the division factor; clamped to at least 1
    pub fn set_division(&mut self, division: u32) {
        self.division = division.max(1);
    }
}

impl Default for ClockDividerNode {
    fn default() -> Self {
        Self::new(2)
    }
}

impl AudioNode for ClockDividerNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let division = self.division as u64;
        let mut count = self.count;
        let mut prev = self.prev_high;
        let mut pass = self.pass;
        process_mono_ch0(ctx, |x| {
            let now = is_high(x);
            if now && !prev {
                count += 1;
                pass = count.is_multiple_of(division);
            }
            prev = now;
            if now && pass {
                GATE_HIGH
            } else {
                GATE_LOW
            }
        });
        self.count = count;
        self.prev_high = prev;
        self.pass = pass;
    }

    fn reset(&mut self) {
        self.count = 0;
        self.prev_high = false;
        self.pass = false;
    }
}

// Fixed-length sample delay ring; read tap trails the write head by `delay`
struct DelayLine {
    buf: Vec<f32>,
    write: usize,
    delay: usize,
}

impl DelayLine {
    // Allocate a line able to delay up to capacity-1 samples
    fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0.0; capacity.max(1)],
            write: 0,
            delay: 0,
        }
    }

    // Set the delay in samples, clamped below the buffer length
    fn set_delay(&mut self, delay: usize) {
        self.delay = delay.min(self.buf.len() - 1);
    }

    // Write the input, then read the sample `delay` writes in the past
    #[inline]
    fn tick(&mut self, x: f32) -> f32 {
        let len = self.buf.len();
        self.buf[self.write] = x;
        let read = (self.write + len - self.delay) % len;
        let out = self.buf[read];
        self.write = (self.write + 1) % len;
        out
    }

    // Clear the line back to silence
    fn clear(&mut self) {
        self.buf.fill(0.0);
        self.write = 0;
    }
}

// Delays every channel by a fixed time; useful for offsetting gates and CV
pub struct GateDelayNode {
    max_delay_ms: f32,
    delay_ms: f32,
    sample_rate: f32,
    delay_samples: usize,
    lines: Vec<DelayLine>,
}

impl GateDelayNode {
    // Build a delay with a ceiling on how far it can be set
    pub fn new(max_delay_ms: f32) -> Self {
        Self {
            max_delay_ms: max_delay_ms.max(0.0),
            delay_ms: 0.0,
            sample_rate: 48_000.0,
            delay_samples: 0,
            lines: Vec::new(),
        }
    }

    // Set the active delay in milliseconds, clamped to the configured max
    pub fn set_delay_ms(&mut self, delay_ms: f32) {
        self.delay_ms = delay_ms.clamp(0.0, self.max_delay_ms);
        self.recompute_delay();
    }

    // Current delay resolved to whole samples at the active sample rate
    pub fn delay_samples(&self) -> usize {
        self.delay_samples
    }

    // Convert the delay time to samples and push it to every line
    fn recompute_delay(&mut self) {
        self.delay_samples = (self.delay_ms * self.sample_rate / 1000.0).round() as usize;
        for line in &mut self.lines {
            line.set_delay(self.delay_samples);
        }
    }
}

impl Default for GateDelayNode {
    fn default() -> Self {
        Self::new(1000.0)
    }
}

impl AudioNode for GateDelayNode {
    // Allocate one ring per output channel sized to the maximum delay
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate = config.sample_rate_hz as f32;
        let cap = (self.max_delay_ms * self.sample_rate / 1000.0).ceil() as usize + 1;
        let n = config.output_channels as usize;
        let mut lines = Vec::with_capacity(n);
        for _ in 0..n {
            lines.push(DelayLine::new(cap));
        }
        self.lines = lines;
        self.recompute_delay();
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        let (input, output) = ctx.io();
        for ch in 0..out_ch {
            let dst = &mut output[ch * frames..(ch + 1) * frames];
            if ch < self.lines.len() && ch < in_ch {
                let src = &input[ch * frames..(ch + 1) * frames];
                let line = &mut self.lines[ch];
                for (o, &i) in dst.iter_mut().zip(src) {
                    *o = line.tick(i);
                }
            } else {
                dst.fill(0.0);
            }
        }
    }

    fn reset(&mut self) {
        for line in &mut self.lines {
            line.clear();
        }
    }
}

// Linear slew limiter: caps the per-sample rate of change of each channel
// Independent rise and fall times shape attack and release of a CV
pub struct SlewLimiterNode {
    rise_ms: f32,
    fall_ms: f32,
    sample_rate: f32,
    current: Vec<f32>,
}

impl SlewLimiterNode {
    // Build a slew with rise and fall times in milliseconds
    pub fn new(rise_ms: f32, fall_ms: f32) -> Self {
        Self {
            rise_ms: rise_ms.max(0.0),
            fall_ms: fall_ms.max(0.0),
            sample_rate: 48_000.0,
            current: Vec::new(),
        }
    }

    // Set the rising slew time in milliseconds
    pub fn set_rise_ms(&mut self, rise_ms: f32) {
        self.rise_ms = rise_ms.max(0.0);
    }

    // Set the falling slew time in milliseconds
    pub fn set_fall_ms(&mut self, fall_ms: f32) {
        self.fall_ms = fall_ms.max(0.0);
    }

    // Per-sample step for a 0..1 transition over the given milliseconds
    // Zero time means an instantaneous transition
    fn step_for(&self, ms: f32) -> f32 {
        if ms <= 0.0 {
            f32::INFINITY
        } else {
            1.0 / (ms * 0.001 * self.sample_rate)
        }
    }
}

impl Default for SlewLimiterNode {
    fn default() -> Self {
        Self::new(10.0, 10.0)
    }
}

impl AudioNode for SlewLimiterNode {
    // One running value per output channel
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate = config.sample_rate_hz as f32;
        self.current = vec![0.0; config.output_channels as usize];
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let rise = self.step_for(self.rise_ms);
        let fall = self.step_for(self.fall_ms);
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        let (input, output) = ctx.io();
        for ch in 0..out_ch {
            let dst = &mut output[ch * frames..(ch + 1) * frames];
            if ch < self.current.len() && ch < in_ch {
                let src = &input[ch * frames..(ch + 1) * frames];
                let mut cur = self.current[ch];
                for (o, &target) in dst.iter_mut().zip(src) {
                    if target > cur {
                        cur = (cur + rise).min(target);
                    } else {
                        cur = (cur - fall).max(target);
                    }
                    *o = cur;
                }
                self.current[ch] = cur;
            } else {
                dst.fill(0.0);
            }
        }
    }

    fn reset(&mut self) {
        for c in &mut self.current {
            *c = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::transport::TransportSnapshot;

    const SR: u32 = 48_000;
    const FRAMES: usize = 8;

    fn config(out_ch: u16) -> AudioConfig {
        AudioConfig::new(SR, FRAMES as u32, 1, out_ch).unwrap()
    }

    fn run(node: &mut impl AudioNode, input: &[f32], out_ch: usize) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * out_ch];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn clock_divider_passes_every_nth_pulse() {
        let mut node = ClockDividerNode::new(2);
        // One-sample pulses on every odd frame: edges at 1, 3, 5, 7
        let input = vec![0.0f32, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        let out = run(&mut node, &input, 1);
        // /2 passes the 2nd and 4th edges (frames 3 and 7)
        assert_eq!(out, vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn clock_divider_by_one_is_passthrough() {
        let mut node = ClockDividerNode::new(1);
        let input = vec![0.0f32, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        let out = run(&mut node, &input, 1);
        assert_eq!(out, input);
    }

    #[test]
    fn gate_delay_shifts_impulse_by_exact_samples() {
        let mut node = GateDelayNode::new(50.0);
        node.set_delay_ms(0.5); // 0.5 ms * 48 kHz = 24 samples
        node.prepare(&config(1));
        let n = node.delay_samples();
        assert_eq!(n, 24);

        // Place an impulse at frame 0 of the first block, then locate it
        let mut found = None;
        for block in 0..6 {
            let mut input = vec![0.0f32; FRAMES];
            if block == 0 {
                input[0] = 1.0;
            }
            let out = run(&mut node, &input, 1);
            for (i, &s) in out.iter().enumerate() {
                if s > 0.5 {
                    found = Some(block * FRAMES + i);
                }
            }
        }
        assert_eq!(found, Some(n), "impulse did not land at the delay tap");
    }

    #[test]
    fn slew_limits_a_rising_step() {
        let mut node = SlewLimiterNode::new(10.0, 10.0);
        node.prepare(&config(1));
        // Step from 0 to 1 should ramp, not jump
        let input = vec![1.0f32; FRAMES];
        let out = run(&mut node, &input, 1);
        assert!(
            out[0] < 1.0 && out[0] > 0.0,
            "first sample should be partial"
        );
        assert!(
            out.windows(2).all(|w| w[1] >= w[0]),
            "ramp must be monotonic"
        );
        assert!(
            *out.last().unwrap() < 1.0,
            "10ms slew should not reach unity in 8 samples"
        );
    }

    #[test]
    fn slew_zero_rise_is_instant() {
        let mut node = SlewLimiterNode::new(0.0, 0.0);
        node.prepare(&config(1));
        let input = vec![1.0f32; FRAMES];
        let out = run(&mut node, &input, 1);
        assert!(out.iter().all(|&s| (s - 1.0).abs() < 1e-6));
    }
}
