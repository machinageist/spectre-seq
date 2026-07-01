// =============================================================================
// File: app/geist-daw/src/fx.rs
// Layer: application binary
// Purpose: Post-synth effects chain over channel-major buffers
// Status: Implemented; distortion/phaser/flanger/chorus character effects then
//         delay and reverb, each independently bypassable.
// Notes: The character effects are per-sample geist-dsp units held one instance
//        per channel (to keep per-channel state); delay/reverb are stereo
//        AudioNodes run through a single scratch buffer. process() never
//        allocates; effects are sized once on the app thread.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;
use geist_core::transport::TransportSnapshot;
use geist_dsp::prelude::{
    Chorus, Distortion, DistortionMode, Flanger, ParametricEq, Phaser, SaturationCurve, Saturator,
};
use geist_fx::prelude::{DelayNode, ReverbNode};
use geist_graph::node::AudioNode;

use crate::control::{FxKind, FxSlot, FX_CHAIN_MAX};

// Prebuilt instances per duplicate-capable character effect kind.
const FX_INSTANCE_POOL: usize = 4;

// Ordered effects applied after the synth, each independently bypassable
pub struct FxChain {
    // Per-sample character effects, one instance per channel
    distortion: [Vec<Distortion>; FX_INSTANCE_POOL],
    phaser: [Vec<Phaser>; FX_INSTANCE_POOL],
    flanger: [Vec<Flanger>; FX_INSTANCE_POOL],
    chorus: [Vec<Chorus>; FX_INSTANCE_POOL],
    eq: [Vec<ParametricEq>; FX_INSTANCE_POOL],
    saturator: [Vec<Saturator>; FX_INSTANCE_POOL],
    distortion_on: [bool; FX_INSTANCE_POOL],
    phaser_on: [bool; FX_INSTANCE_POOL],
    flanger_on: [bool; FX_INSTANCE_POOL],
    chorus_on: [bool; FX_INSTANCE_POOL],
    eq_on: [bool; FX_INSTANCE_POOL],
    saturator_on: [bool; FX_INSTANCE_POOL],
    // EQ peaking-band params per instance, recomputed together since set_peaking
    // needs frequency, gain, and Q at once
    eq_freq: [f32; FX_INSTANCE_POOL],
    eq_gain_db: [f32; FX_INSTANCE_POOL],
    eq_q: [f32; FX_INSTANCE_POOL],
    // Bounded order of active character-FX slots, followed by delay/reverb tail
    character_chain: [FxSlot; FX_CHAIN_MAX],
    character_chain_len: usize,
    // Stereo time-based effects
    delay: DelayNode,
    reverb: ReverbNode,
    delay_on: bool,
    reverb_on: bool,
    channels: usize,
    sample_rate_hz: u32,
    // Ping-pong buffer the size of one channel-major block
    scratch: Vec<f32>,
}

impl FxChain {
    // Build a bypassed chain sized for the stream block
    pub fn new(channels: usize, block_frames: usize, sample_rate_hz: u32) -> Self {
        let sr = sample_rate_hz as f32;
        let mut character_chain = [FxSlot::empty(); FX_CHAIN_MAX];
        character_chain[0] = FxSlot {
            kind: FxKind::Distortion,
            instance: 0,
        };
        character_chain[1] = FxSlot {
            kind: FxKind::Phaser,
            instance: 0,
        };
        character_chain[2] = FxSlot {
            kind: FxKind::Flanger,
            instance: 0,
        };
        character_chain[3] = FxSlot {
            kind: FxKind::Chorus,
            instance: 0,
        };
        let mut chain = Self {
            distortion: std::array::from_fn(|_| {
                vec![Distortion::new(DistortionMode::Soft); channels]
            }),
            phaser: std::array::from_fn(|_| vec![Phaser::new(sr); channels]),
            flanger: std::array::from_fn(|_| vec![Flanger::new(sr); channels]),
            chorus: std::array::from_fn(|_| vec![Chorus::new(sr); channels]),
            eq: std::array::from_fn(|_| (0..channels).map(|_| ParametricEq::new(1)).collect()),
            saturator: std::array::from_fn(|_| {
                vec![Saturator::new(SaturationCurve::Tanh); channels]
            }),
            distortion_on: [false; FX_INSTANCE_POOL],
            phaser_on: [false; FX_INSTANCE_POOL],
            flanger_on: [false; FX_INSTANCE_POOL],
            chorus_on: [false; FX_INSTANCE_POOL],
            eq_on: [false; FX_INSTANCE_POOL],
            saturator_on: [false; FX_INSTANCE_POOL],
            eq_freq: [1_000.0; FX_INSTANCE_POOL],
            eq_gain_db: [0.0; FX_INSTANCE_POOL],
            eq_q: [1.0; FX_INSTANCE_POOL],
            character_chain,
            character_chain_len: 4,
            delay: DelayNode::new(),
            reverb: ReverbNode::new(),
            delay_on: false,
            reverb_on: false,
            channels,
            sample_rate_hz,
            scratch: vec![0.0; channels * block_frames],
        };
        for i in 0..FX_INSTANCE_POOL {
            chain.apply_eq_band(i);
        }
        chain
    }

    // Rebuild the effects for the stream's rate and block size
    pub fn prepare(&mut self, config: &AudioConfig) {
        let sr = config.sample_rate_hz as f32;
        self.sample_rate_hz = config.sample_rate_hz;
        for instance in &mut self.distortion {
            for d in instance {
                d.reset();
            }
        }
        for instance in &mut self.phaser {
            for p in instance {
                *p = Phaser::new(sr);
            }
        }
        for instance in &mut self.flanger {
            for f in instance {
                *f = Flanger::new(sr);
            }
        }
        for instance in &mut self.chorus {
            for c in instance {
                *c = Chorus::new(sr);
            }
        }
        for instance in &mut self.eq {
            for e in instance {
                e.reset();
            }
        }
        for i in 0..FX_INSTANCE_POOL {
            self.apply_eq_band(i);
        }
        self.delay.prepare(config);
        self.reverb.prepare(config);
    }

    // Replace the ordered character-FX chain with bounded app-thread input
    pub fn set_character_chain(&mut self, slots: &[FxSlot]) {
        self.character_chain = [FxSlot::empty(); FX_CHAIN_MAX];
        self.character_chain_len = slots.len().min(FX_CHAIN_MAX);
        for (index, slot) in slots.iter().take(FX_CHAIN_MAX).enumerate() {
            self.character_chain[index] = *slot;
        }
    }

    // Toggle one character effect instance
    pub fn set_fx_on(&mut self, fx: FxKind, instance: u8, on: bool) {
        if let Some(i) = instance_index(instance) {
            match fx {
                FxKind::Distortion => self.distortion_on[i] = on,
                FxKind::Phaser => self.phaser_on[i] = on,
                FxKind::Flanger => self.flanger_on[i] = on,
                FxKind::Chorus => self.chorus_on[i] = on,
                FxKind::Eq => self.eq_on[i] = on,
                FxKind::Saturator => self.saturator_on[i] = on,
            }
        }
    }

    // Set one parameter (by index) of one character effect instance, on every channel
    pub fn set_fx_param(&mut self, fx: FxKind, instance: u8, param: u8, value: f32) {
        let Some(i) = instance_index(instance) else {
            return;
        };
        match fx {
            FxKind::Distortion => {
                for d in &mut self.distortion[i] {
                    match param {
                        0 => d.set_drive(value),
                        1 => d.set_tone(value),
                        _ => d.set_mix(value),
                    }
                }
            }
            FxKind::Phaser => {
                for p in &mut self.phaser[i] {
                    match param {
                        0 => p.set_rate(value),
                        1 => p.set_depth(value),
                        2 => p.set_feedback(value),
                        _ => p.set_mix(value),
                    }
                }
            }
            FxKind::Flanger => {
                for f in &mut self.flanger[i] {
                    match param {
                        0 => f.set_rate(value),
                        1 => f.set_depth_ms(value),
                        2 => f.set_feedback(value),
                        _ => f.set_mix(value),
                    }
                }
            }
            FxKind::Chorus => {
                for c in &mut self.chorus[i] {
                    match param {
                        0 => c.set_rate(value),
                        1 => c.set_depth_ms(value),
                        _ => c.set_mix(value),
                    }
                }
            }
            FxKind::Eq => {
                match param {
                    0 => self.eq_freq[i] = value,
                    1 => self.eq_gain_db[i] = value,
                    _ => self.eq_q[i] = value,
                }
                self.apply_eq_band(i);
            }
            FxKind::Saturator => {
                for s in &mut self.saturator[i] {
                    match param {
                        0 => s.set_drive(value),
                        1 => s.set_output_gain(value),
                        _ => s.set_mix(value),
                    }
                }
            }
        }
    }

    // Recompute the single peaking band of one EQ instance on every channel
    fn apply_eq_band(&mut self, i: usize) {
        let sr = self.sample_rate_hz as f32;
        let (freq, q, gain_db) = (self.eq_freq[i], self.eq_q[i], self.eq_gain_db[i]);
        for eq in &mut self.eq[i] {
            if let Some(band) = eq.band_mut(0) {
                band.set_peaking(freq, q, gain_db, sr);
            }
        }
    }

    // Toggle the delay
    pub fn set_delay(&mut self, on: bool) {
        self.delay_on = on;
    }

    // Set the delay time in seconds on both channels
    pub fn set_delay_time(&mut self, seconds: f32) {
        let sr = self.sample_rate_hz as f32;
        self.delay
            .delay_mut()
            .set_delay_seconds(seconds, seconds, sr);
    }

    // Set the delay feedback amount
    pub fn set_delay_feedback(&mut self, feedback: f32) {
        self.delay.delay_mut().set_feedback(feedback);
    }

    // Set the delay dry/wet mix
    pub fn set_delay_mix(&mut self, mix: f32) {
        self.delay.delay_mut().set_mix(mix);
    }

    // Toggle the reverb
    pub fn set_reverb(&mut self, on: bool) {
        self.reverb_on = on;
    }

    // Set the reverb dry/wet mix
    pub fn set_reverb_mix(&mut self, mix: f32) {
        self.reverb.set_mix(mix);
    }

    // Apply the enabled effects in order, in place on channel-major `output`:
    // character effects (per channel) first, then the stereo time effects.
    pub fn process(&mut self, output: &mut [f32], frames: usize) {
        for ch in 0..self.channels {
            let lane = &mut output[ch * frames..ch * frames + frames];
            for slot in self.character_chain.iter().take(self.character_chain_len) {
                let Some(i) = instance_index(slot.instance) else {
                    continue;
                };
                match slot.kind {
                    FxKind::Distortion if self.distortion_on[i] => {
                        self.distortion[i][ch].process(lane)
                    }
                    FxKind::Phaser if self.phaser_on[i] => self.phaser[i][ch].process(lane),
                    FxKind::Flanger if self.flanger_on[i] => self.flanger[i][ch].process(lane),
                    FxKind::Chorus if self.chorus_on[i] => self.chorus[i][ch].process(lane),
                    FxKind::Eq if self.eq_on[i] => self.eq[i][ch].process(lane),
                    FxKind::Saturator if self.saturator_on[i] => {
                        self.saturator[i][ch].process(lane)
                    }
                    _ => {}
                }
            }
        }
        if self.delay_on {
            apply(
                &mut self.delay,
                output,
                &mut self.scratch,
                frames,
                self.sample_rate_hz,
            );
        }
        if self.reverb_on {
            apply(
                &mut self.reverb,
                output,
                &mut self.scratch,
                frames,
                self.sample_rate_hz,
            );
        }
    }
}

// Convert an app-thread instance id to an in-pool index
fn instance_index(instance: u8) -> Option<usize> {
    let index = instance as usize;
    (index < FX_INSTANCE_POOL).then_some(index)
}

// Run one effect: current output becomes its input, scratch its output, then copy back
fn apply(
    node: &mut dyn AudioNode,
    output: &mut [f32],
    scratch: &mut [f32],
    frames: usize,
    sample_rate_hz: u32,
) {
    {
        let transport = TransportSnapshot::stopped(sample_rate_hz);
        let mut ctx =
            ProcessContext::new(frames, sample_rate_hz, output, scratch, &[], &[], transport);
        node.process(&mut ctx);
    }
    output.copy_from_slice(scratch);
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48_000;
    const FRAMES: usize = 256;
    const CHANNELS: usize = 2;

    fn config() -> AudioConfig {
        AudioConfig::new(SR, FRAMES as u32, CHANNELS as u16, CHANNELS as u16).unwrap()
    }

    // A channel-major stereo impulse on both channels
    fn impulse() -> Vec<f32> {
        let mut buf = vec![0.0f32; CHANNELS * FRAMES];
        buf[0] = 1.0;
        buf[FRAMES] = 1.0;
        buf
    }

    #[test]
    fn bypassed_chain_leaves_audio_untouched() {
        let mut fx = FxChain::new(CHANNELS, FRAMES, SR);
        fx.prepare(&config());
        let original = impulse();
        let mut buf = original.clone();
        fx.process(&mut buf, FRAMES);
        assert_eq!(buf, original, "bypassed chain must be transparent");
    }

    #[test]
    fn enabled_reverb_spreads_an_impulse() {
        let mut fx = FxChain::new(CHANNELS, FRAMES, SR);
        fx.prepare(&config());
        fx.set_reverb(true);
        fx.set_reverb_mix(1.0);
        let mut buf = impulse();
        fx.process(&mut buf, FRAMES);
        let nonzero = buf.iter().filter(|&&s| s.abs() > 1e-6).count();
        assert!(nonzero > 10, "reverb did not spread the impulse: {nonzero}");
        assert!(buf.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn enabled_delay_adds_a_later_tap() {
        let mut fx = FxChain::new(CHANNELS, FRAMES, SR);
        fx.prepare(&config());
        fx.set_delay(true);
        // Feed an impulse, then run enough silent blocks to pass the 250 ms tap
        // (12 000 samples / 256-frame blocks ~= 47 blocks)
        let mut buf = impulse();
        fx.process(&mut buf, FRAMES);
        let mut energy_later = 0.0f32;
        for _ in 0..64 {
            let mut silent = vec![0.0f32; CHANNELS * FRAMES];
            fx.process(&mut silent, FRAMES);
            energy_later += silent.iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(energy_later > 0.0, "delay produced no later tap");
    }

    #[test]
    fn duplicate_character_fx_instances_are_addressed_independently() {
        let input = vec![0.2f32; CHANNELS * FRAMES];
        let mut one = FxChain::new(CHANNELS, FRAMES, SR);
        one.prepare(&config());
        one.set_character_chain(&[FxSlot {
            kind: FxKind::Distortion,
            instance: 0,
        }]);
        one.set_fx_on(FxKind::Distortion, 0, true);
        one.set_fx_param(FxKind::Distortion, 0, 0, 18.0);
        one.set_fx_param(FxKind::Distortion, 0, 2, 1.0);

        let mut two = FxChain::new(CHANNELS, FRAMES, SR);
        two.prepare(&config());
        two.set_character_chain(&[
            FxSlot {
                kind: FxKind::Distortion,
                instance: 0,
            },
            FxSlot {
                kind: FxKind::Distortion,
                instance: 1,
            },
        ]);
        two.set_fx_on(FxKind::Distortion, 0, true);
        two.set_fx_param(FxKind::Distortion, 0, 0, 18.0);
        two.set_fx_param(FxKind::Distortion, 0, 2, 1.0);
        two.set_fx_on(FxKind::Distortion, 1, true);
        two.set_fx_param(FxKind::Distortion, 1, 0, 18.0);
        two.set_fx_param(FxKind::Distortion, 1, 2, 1.0);

        let mut one_buf = input.clone();
        let mut two_buf = input;
        one.process(&mut one_buf, FRAMES);
        two.process(&mut two_buf, FRAMES);

        assert_ne!(
            one_buf, two_buf,
            "two distortion instances must process independently"
        );
    }

    #[test]
    fn eq_and_saturator_process_when_enabled() {
        let config = config();

        // Saturator with strong drive reshapes a steady signal and stays finite
        let steady = vec![0.3f32; CHANNELS * FRAMES];
        let mut sat = FxChain::new(CHANNELS, FRAMES, SR);
        sat.prepare(&config);
        sat.set_character_chain(&[FxSlot {
            kind: FxKind::Saturator,
            instance: 0,
        }]);
        sat.set_fx_on(FxKind::Saturator, 0, true);
        sat.set_fx_param(FxKind::Saturator, 0, 0, 12.0); // drive
        sat.set_fx_param(FxKind::Saturator, 0, 2, 1.0); // fully wet
        let mut sat_buf = steady.clone();
        sat.process(&mut sat_buf, FRAMES);
        assert_ne!(sat_buf, steady, "enabled saturator must reshape the signal");
        assert!(sat_buf.iter().all(|s| s.is_finite()));

        // EQ at the default flat band (0 dB) passes an impulse transparently
        let original = impulse();
        let mut flat_eq = FxChain::new(CHANNELS, FRAMES, SR);
        flat_eq.prepare(&config);
        flat_eq.set_character_chain(&[FxSlot {
            kind: FxKind::Eq,
            instance: 0,
        }]);
        flat_eq.set_fx_on(FxKind::Eq, 0, true);
        let mut flat = original.clone();
        flat_eq.process(&mut flat, FRAMES);
        assert!(
            flat.iter()
                .zip(&original)
                .all(|(a, b)| (a - b).abs() < 1e-5),
            "0 dB EQ band must be transparent"
        );

        // A boosted EQ band rings the impulse, changing it while staying finite
        let mut boost_eq = FxChain::new(CHANNELS, FRAMES, SR);
        boost_eq.prepare(&config);
        boost_eq.set_character_chain(&[FxSlot {
            kind: FxKind::Eq,
            instance: 0,
        }]);
        boost_eq.set_fx_on(FxKind::Eq, 0, true);
        boost_eq.set_fx_param(FxKind::Eq, 0, 1, 12.0); // +12 dB at the band
        let mut boosted = original.clone();
        boost_eq.process(&mut boosted, FRAMES);
        assert_ne!(boosted, original, "boosted EQ band must change the impulse");
        assert!(boosted.iter().all(|s| s.is_finite()));
    }
}
