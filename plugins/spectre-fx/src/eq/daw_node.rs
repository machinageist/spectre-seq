// =============================================================================
// File: plugins/spectre-fx/src/eq/daw_node.rs
// Layer: effects plugin
// Purpose: ParametricEq wrapped as a graph AudioNode
// Status: Implemented; one EQ band-cascade per output channel.
// Notes: Biquads carry per-channel state, so each channel owns an EQ. Band
//        settings are declarative and rebuilt at the stream sample rate.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use spectre_graph::node::AudioNode;

use crate::eq::engine::ParametricEq;
use crate::io::copy_input_to_output;

// Declarative configuration of one EQ band
#[derive(Clone, Copy, Debug)]
pub enum BandConfig {
    Bypass,
    Lowpass { hz: f32, q: f32 },
    Highpass { hz: f32, q: f32 },
    Peaking { hz: f32, q: f32, gain_db: f32 },
    LowShelf { hz: f32, q: f32, gain_db: f32 },
    HighShelf { hz: f32, q: f32, gain_db: f32 },
}

// Graph node applying a per-channel parametric EQ
pub struct EqNode {
    bands: Vec<BandConfig>,
    channels: Vec<ParametricEq>,
    sample_rate: f32,
}

impl EqNode {
    // Build an EQ node with `band_count` bypassed bands
    pub fn new(band_count: usize) -> Self {
        Self {
            bands: vec![BandConfig::Bypass; band_count.max(1)],
            channels: Vec::new(),
            sample_rate: 48_000.0,
        }
    }

    // Configure one band; takes effect immediately when already prepared
    pub fn set_band(&mut self, index: usize, config: BandConfig) {
        if index < self.bands.len() {
            self.bands[index] = config;
            if !self.channels.is_empty() {
                self.rebuild_channels();
            }
        }
    }

    // Build one EQ instance from the current band settings
    fn build_channel(&self) -> ParametricEq {
        let mut eq = ParametricEq::new(self.bands.len());
        for (i, band) in self.bands.iter().enumerate() {
            let Some(section) = eq.band_mut(i) else {
                continue;
            };
            match *band {
                BandConfig::Bypass => {}
                BandConfig::Lowpass { hz, q } => section.set_lowpass(hz, q, self.sample_rate),
                BandConfig::Highpass { hz, q } => section.set_highpass(hz, q, self.sample_rate),
                BandConfig::Peaking { hz, q, gain_db } => {
                    section.set_peaking(hz, q, gain_db, self.sample_rate)
                }
                BandConfig::LowShelf { hz, q, gain_db } => {
                    section.set_low_shelf(hz, q, gain_db, self.sample_rate)
                }
                BandConfig::HighShelf { hz, q, gain_db } => {
                    section.set_high_shelf(hz, q, gain_db, self.sample_rate)
                }
            }
        }
        eq
    }

    // Rebuild every channel's EQ from current settings, preserving count
    fn rebuild_channels(&mut self) {
        let n = self.channels.len();
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.build_channel());
        }
        self.channels = v;
    }
}

impl Default for EqNode {
    fn default() -> Self {
        Self::new(4)
    }
}

impl AudioNode for EqNode {
    // Build one EQ per output channel at the stream sample rate
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate = config.sample_rate_hz as f32;
        let n = config.output_channels as usize;
        let mut channels = Vec::with_capacity(n);
        for _ in 0..n {
            channels.push(self.build_channel());
        }
        self.channels = channels;
    }

    // Mirror input to output, then EQ each channel in place
    fn process(&mut self, ctx: &mut ProcessContext) {
        copy_input_to_output(ctx);
        let count = ctx.output_channels().min(self.channels.len());
        for ch in 0..count {
            if let Some(out) = ctx.output(ch) {
                self.channels[ch].process(out);
            }
        }
    }

    // Clear each channel's filter state
    fn reset(&mut self) {
        for eq in &mut self.channels {
            eq.reset();
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

    fn run(node: &mut EqNode, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * 2];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(FRAMES, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn bypassed_eq_passes_through() {
        let mut node = EqNode::new(4);
        node.prepare(&config());
        let mut input = vec![0.0f32; FRAMES * 2];
        for (i, s) in input.iter_mut().enumerate() {
            *s = ((i as f32) * 0.05).sin();
        }
        let out = run(&mut node, &input);
        assert_eq!(out, input);
    }

    #[test]
    fn highpass_band_blocks_dc() {
        let mut node = EqNode::new(2);
        node.set_band(
            0,
            BandConfig::Highpass {
                hz: 1_000.0,
                q: std::f32::consts::FRAC_1_SQRT_2,
            },
        );
        node.prepare(&config());

        // Feed sustained DC across several blocks; the highpass should drain it
        let dc_block = vec![1.0f32; FRAMES * 2];
        let mut out = vec![0.0f32; FRAMES * 2];
        for _ in 0..40 {
            out = run(&mut node, &dc_block);
        }
        assert!(out.iter().all(|&s| s.abs() < 1e-2), "DC leaked through EQ");
    }

    #[test]
    fn output_stays_finite() {
        let mut node = EqNode::new(3);
        node.set_band(
            0,
            BandConfig::Peaking {
                hz: 2_000.0,
                q: 4.0,
                gain_db: 9.0,
            },
        );
        node.prepare(&config());
        let mut input = vec![0.0f32; FRAMES * 2];
        for (i, s) in input.iter_mut().enumerate() {
            *s = ((i as f32) * 0.2).sin() * 0.5;
        }
        let out = run(&mut node, &input);
        assert!(out.iter().all(|s| s.is_finite()));
    }
}
