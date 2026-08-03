// =============================================================================
// File: plugins/geist-fx/src/chorus/daw_node.rs
// Layer: effects plugin
// Purpose: Chorus wrapped as a graph AudioNode
// Status: Implemented; one Chorus voice per output channel.
// Notes: Chorus carries per-channel delay state, so each channel owns an
//        instance. Voices are built in prepare() at the stream sample rate.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use spectre_graph::node::AudioNode;

use crate::chorus::engine::Chorus;
use crate::io::copy_input_to_output;

// Musical defaults applied to every voice
const DEFAULT_RATE_HZ: f32 = 0.8;
const DEFAULT_BASE_DELAY_MS: f32 = 12.0;
const DEFAULT_DEPTH_MS: f32 = 4.0;
const DEFAULT_MIX: f32 = 0.5;

// Graph node applying a per-channel chorus
pub struct ChorusNode {
    voices: Vec<Chorus>,
    sample_rate: f32,
    rate_hz: f32,
    base_delay_ms: f32,
    depth_ms: f32,
    mix: f32,
}

impl ChorusNode {
    // Build a chorus node with musical defaults
    pub fn new() -> Self {
        Self {
            voices: Vec::new(),
            sample_rate: 48_000.0,
            rate_hz: DEFAULT_RATE_HZ,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
            depth_ms: DEFAULT_DEPTH_MS,
            mix: DEFAULT_MIX,
        }
    }

    // Set the modulation rate in Hz; applies on the next prepare
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.rate_hz = rate_hz;
        for v in &mut self.voices {
            v.set_rate(rate_hz);
        }
    }

    // Set the dry/wet mix in [0, 1]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
        for v in &mut self.voices {
            v.set_mix(self.mix);
        }
    }

    // Build one configured chorus voice
    fn build_voice(&self) -> Chorus {
        let mut c = Chorus::new(self.sample_rate);
        c.set_rate(self.rate_hz);
        c.set_base_delay_ms(self.base_delay_ms);
        c.set_depth_ms(self.depth_ms);
        c.set_mix(self.mix);
        c
    }
}

impl Default for ChorusNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for ChorusNode {
    // Build one voice per output channel at the stream sample rate
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate = config.sample_rate_hz as f32;
        let n = config.output_channels as usize;
        let mut voices = Vec::with_capacity(n);
        for _ in 0..n {
            voices.push(self.build_voice());
        }
        self.voices = voices;
    }

    // Mirror input to output, then chorus each channel in place
    fn process(&mut self, ctx: &mut ProcessContext) {
        copy_input_to_output(ctx);
        let count = ctx.output_channels().min(self.voices.len());
        for ch in 0..count {
            if let Some(out) = ctx.output(ch) {
                self.voices[ch].process(out);
            }
        }
    }

    // Clear each voice's delay state
    fn reset(&mut self) {
        for v in &mut self.voices {
            v.reset();
        }
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

    fn run(node: &mut ChorusNode, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * 2];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn zero_mix_passes_through() {
        let mut node = ChorusNode::new();
        node.set_mix(0.0);
        node.prepare(&config());
        let mut input = vec![0.0f32; FRAMES * 2];
        for (i, s) in input.iter_mut().enumerate() {
            *s = ((i as f32) * 0.05).sin();
        }
        let out = run(&mut node, &input);
        assert_eq!(out, input);
    }

    #[test]
    fn wet_chorus_alters_and_bounds_signal() {
        let mut node = ChorusNode::new();
        node.set_mix(0.5);
        node.prepare(&config());
        let mut input = vec![0.0f32; FRAMES * 2];
        for (i, s) in input.iter_mut().enumerate() {
            *s = ((i as f32) * 0.05).sin();
        }
        let out = run(&mut node, &input);
        // Wet output differs from dry (the base delay shifts the signal)
        assert!(out != input);
        assert!(out.iter().all(|s| s.is_finite() && s.abs() <= 1.5));
    }

    #[test]
    fn silent_input_stays_silent() {
        let mut node = ChorusNode::new();
        node.prepare(&config());
        let out = run(&mut node, &vec![0.0f32; FRAMES * 2]);
        assert!(out.iter().all(|&s| s == 0.0));
    }
}
