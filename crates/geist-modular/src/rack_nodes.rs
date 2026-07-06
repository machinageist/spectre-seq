// =============================================================================
// File: crates/geist-modular/src/rack_nodes.rs
// Layer: modular utilities
// Purpose: Rack adapters over geist-dsp: VCO, LFO, envelope, filter, VCA
// Status: Implemented; adapters only, no new DSP.
// Notes: Volt scaling per spec §2.2: audio and LFO outputs are ±5 V bipolar,
//        envelopes 0-10 V unipolar. Pitch/rate CV is 1 V/oct via standards
//        anchors; gates use Schmitt hysteresis levels. Filter cutoff CV is
//        read at control rate (frame 0 of each block), not audio rate.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;
use geist_dsp::prelude::{Adsr, Lfo, LfoWaveform, PolyBlepOsc, Svf, SvfMode, Waveform};
use geist_graph::node::AudioNode;

use crate::standards::{volts_to_hz, Schmitt, AUDIO_ZERO_V_HZ, GATE_V, LFO_ZERO_V_HZ};

// Nominal bipolar audio/LFO peak in volts (spec §2.2)
const AUDIO_V: f32 = 5.0;
// Keep converted frequencies inside a stable band for the primitives
const MIN_HZ: f32 = 0.01;

// Highest safe oscillator/cutoff frequency for the current stream rate
#[inline]
fn max_hz(sample_rate_hz: f32) -> f32 {
    sample_rate_hz * 0.45
}

// Bandlimited oscillator: input 0 = v/oct pitch CV, output 0 = ±5 V audio
pub struct VcoNode {
    osc: PolyBlepOsc,
    sample_rate_hz: f32,
    // Knob offset added to the pitch CV before conversion, in volts
    offset_volts: f32,
}

impl VcoNode {
    pub fn new() -> Self {
        Self {
            osc: PolyBlepOsc::new(Waveform::Saw),
            sample_rate_hz: 48_000.0,
            offset_volts: 0.0,
        }
    }

    // Set the waveform (saw/square/triangle)
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.osc.set_waveform(waveform);
    }

    // Set the pitch knob offset in volts (1 V per octave)
    pub fn set_offset_volts(&mut self, volts: f32) {
        self.offset_volts = volts;
    }
}

impl Default for VcoNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for VcoNode {
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate_hz = config.sample_rate_hz as f32;
        self.osc.reset();
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let sr = self.sample_rate_hz;
        let top = max_hz(sr);
        let (input, output) = ctx.io();
        for (f, slot) in output[..frames].iter_mut().enumerate() {
            let cv = if in_ch >= 1 { input[f] } else { 0.0 };
            let hz = volts_to_hz(cv + self.offset_volts, AUDIO_ZERO_V_HZ).clamp(MIN_HZ, top);
            self.osc.set_frequency(hz, sr);
            *slot = self.osc.next_sample() * AUDIO_V;
        }
        for ch in 1..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }

    fn reset(&mut self) {
        self.osc.reset();
    }
}

// Low-frequency oscillator: input 0 = rate CV (1 V/oct around 2 Hz),
// output 0 = ±5 V modulation
pub struct LfoRackNode {
    lfo: Lfo,
    sample_rate_hz: f32,
    offset_volts: f32,
}

impl LfoRackNode {
    pub fn new() -> Self {
        Self {
            lfo: Lfo::new(LfoWaveform::Sine),
            sample_rate_hz: 48_000.0,
            offset_volts: 0.0,
        }
    }

    // Set the LFO shape
    pub fn set_waveform(&mut self, waveform: LfoWaveform) {
        self.lfo.set_waveform(waveform);
    }

    // Set the rate knob offset in volts (1 V doubles the rate)
    pub fn set_offset_volts(&mut self, volts: f32) {
        self.offset_volts = volts;
    }
}

impl Default for LfoRackNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for LfoRackNode {
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate_hz = config.sample_rate_hz as f32;
        self.lfo.reset();
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let sr = self.sample_rate_hz;
        let (input, output) = ctx.io();
        for (f, slot) in output[..frames].iter_mut().enumerate() {
            let cv = if in_ch >= 1 { input[f] } else { 0.0 };
            let hz = volts_to_hz(cv + self.offset_volts, LFO_ZERO_V_HZ).clamp(MIN_HZ, 100.0);
            self.lfo.set_frequency(hz, sr);
            *slot = self.lfo.next_sample() * AUDIO_V;
        }
        for ch in 1..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }

    fn reset(&mut self) {
        self.lfo.reset();
    }
}

// ADSR envelope: input 0 = gate (Schmitt levels), output 0 = 0-10 V CV
pub struct EnvNode {
    adsr: Adsr,
    gate: Schmitt,
}

impl EnvNode {
    pub fn new() -> Self {
        Self {
            adsr: Adsr::new(48_000.0),
            gate: Schmitt::new(),
        }
    }

    // Set the four stages (attack/decay/release seconds, sustain 0..1)
    pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.adsr.set_attack(attack);
        self.adsr.set_decay(decay);
        self.adsr.set_sustain(sustain);
        self.adsr.set_release(release);
    }
}

impl Default for EnvNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for EnvNode {
    fn prepare(&mut self, config: &AudioConfig) {
        // Rebuilding keeps stage timing correct at the new rate (app thread)
        let mut adsr = Adsr::new(config.sample_rate_hz as f32);
        adsr.set_attack(0.005);
        adsr.set_decay(0.05);
        adsr.set_sustain(0.8);
        adsr.set_release(0.1);
        self.adsr = adsr;
        self.gate = Schmitt::new();
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let (input, output) = ctx.io();
        for (f, slot) in output[..frames].iter_mut().enumerate() {
            let level = if in_ch >= 1 { input[f] } else { 0.0 };
            let was_high = self.gate.is_high();
            if self.gate.step(level) {
                self.adsr.gate_on();
            } else if was_high && !self.gate.is_high() {
                self.adsr.gate_off();
            }
            *slot = self.adsr.process_sample() * GATE_V;
        }
        for ch in 1..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }

    fn reset(&mut self) {
        self.adsr.reset();
        self.gate = Schmitt::new();
    }
}

// State-variable filter: input 0 = audio, input 1 = cutoff v/oct CV.
// Cutoff CV is sampled at frame 0 (control rate)
pub struct VcfNode {
    svf: Svf,
    sample_rate_hz: f32,
    // Cutoff knob position in volts above the audio anchor
    offset_volts: f32,
    q: f32,
}

impl VcfNode {
    pub fn new() -> Self {
        Self {
            svf: Svf::new(SvfMode::Lowpass),
            sample_rate_hz: 48_000.0,
            // Two octaves above C4 ~ 1 kHz default cutoff
            offset_volts: 2.0,
            q: 0.707,
        }
    }

    // Set the filter response mode
    pub fn set_mode(&mut self, mode: SvfMode) {
        self.svf.set_mode(mode);
    }

    // Set the cutoff knob in volts and the resonance q
    pub fn set_cutoff(&mut self, offset_volts: f32, q: f32) {
        self.offset_volts = offset_volts;
        self.q = q.max(0.1);
    }
}

impl Default for VcfNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for VcfNode {
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate_hz = config.sample_rate_hz as f32;
        self.svf.reset();
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let sr = self.sample_rate_hz;
        let (input, output) = ctx.io();
        let cv = if in_ch >= 2 { input[frames] } else { 0.0 };
        let cutoff = volts_to_hz(cv + self.offset_volts, AUDIO_ZERO_V_HZ).clamp(10.0, max_hz(sr));
        self.svf.set_params(cutoff, self.q, sr);
        for (f, slot) in output[..frames].iter_mut().enumerate() {
            let audio = if in_ch >= 1 { input[f] } else { 0.0 };
            *slot = self.svf.process_sample(audio);
        }
        for ch in 1..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }

    fn reset(&mut self) {
        self.svf.reset();
    }
}

// Voltage-controlled amplifier: input 0 = audio, input 1 = 0-10 V level CV.
// An unpatched CV input passes audio at unity
pub struct VcaNode;

impl AudioNode for VcaNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let (input, output) = ctx.io();
        for (f, slot) in output[..frames].iter_mut().enumerate() {
            let audio = if in_ch >= 1 { input[f] } else { 0.0 };
            let gain = if in_ch >= 2 {
                (input[frames + f] / GATE_V).clamp(0.0, 1.0)
            } else {
                1.0
            };
            *slot = audio * gain;
        }
        for ch in 1..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standards::SCHMITT_HIGH_V;
    use geist_core::transport::TransportSnapshot;

    const SR: u32 = 48_000;

    fn run(node: &mut dyn AudioNode, input: &[f32], frames: usize) -> Vec<f32> {
        let mut output = vec![0.0f32; frames];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(frames, SR, input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        output
    }

    // Count positive-going zero crossings (one per cycle for saw/sine)
    fn rising_crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count()
    }

    #[test]
    fn vco_tracks_a4_at_nine_semitone_volts() {
        let mut vco = VcoNode::new();
        vco.prepare(&AudioConfig::new(SR, 256, 0, 1).unwrap());
        let frames = SR as usize; // one second
        let cv = vec![9.0 / 12.0; frames]; // A4 = nine semitones above C4
        let out = run(&mut vco, &cv, frames);
        let cycles = rising_crossings(&out);
        assert!(
            (438..=442).contains(&cycles),
            "expected ~440 cycles, got {cycles}"
        );
        assert!(out
            .iter()
            .all(|s| s.is_finite() && s.abs() <= AUDIO_V * 1.2));
    }

    #[test]
    fn lfo_runs_at_two_hertz_at_zero_volts() {
        let mut lfo = LfoRackNode::new();
        lfo.prepare(&AudioConfig::new(SR, 256, 0, 1).unwrap());
        let frames = SR as usize * 2; // two seconds
        let cv = vec![0.0; frames];
        let out = run(&mut lfo, &cv, frames);
        let cycles = rising_crossings(&out);
        assert!(
            (3..=5).contains(&cycles),
            "expected ~4 cycles, got {cycles}"
        );
    }

    #[test]
    fn envelope_fires_on_gate_edge_and_releases() {
        let mut env = EnvNode::new();
        env.prepare(&AudioConfig::new(SR, 256, 0, 1).unwrap());
        let frames = SR as usize / 2;
        // Gate high for the first half, low afterwards
        let mut gate = vec![GATE_V; frames / 2];
        gate.resize(frames, 0.0);
        let out = run(&mut env, &gate, frames);
        let peak_during_gate = out[..frames / 2].iter().cloned().fold(0.0f32, f32::max);
        assert!(peak_during_gate > 5.0, "envelope never rose on the gate");
        let tail = out[frames - 1];
        assert!(tail < 0.5, "envelope did not release after the gate fell");
        assert!(out.iter().all(|s| s.is_finite() && *s >= 0.0));
    }

    #[test]
    fn vca_scales_audio_by_level_cv_and_defaults_to_unity() {
        let frames = 64;
        // Channel 0 audio at 4 V, channel 1 CV at half level
        let mut input = vec![4.0f32; frames];
        input.resize(frames * 2, GATE_V * 0.5);
        let out = run(&mut VcaNode, &input, frames);
        assert!(out.iter().all(|s| (s - 2.0).abs() < 1e-5));
        // Without a CV channel the VCA passes at unity
        let mono = vec![4.0f32; frames];
        let out = run(&mut VcaNode, &mono, frames);
        assert!(out.iter().all(|s| (s - 4.0).abs() < 1e-5));
    }

    #[test]
    fn vcf_attenuates_content_above_a_low_cutoff() {
        let mut vcf = VcfNode::new();
        vcf.prepare(&AudioConfig::new(SR, 256, 0, 1).unwrap());
        // Cutoff two octaves below C4 (~65 Hz) against a 2 kHz-ish square CV
        vcf.set_cutoff(-2.0, 0.707);
        let frames = SR as usize / 4;
        let mut vco = VcoNode::new();
        vco.prepare(&AudioConfig::new(SR, 256, 0, 1).unwrap());
        vco.set_offset_volts(3.0); // ~2093 Hz
        let audio = run(&mut vco, &vec![0.0; frames], frames);
        let mut input = audio.clone();
        input.resize(frames * 2, 0.0); // cutoff CV channel = 0, knob only
        let out = run(&mut vcf, &input, frames);
        let in_rms = (audio.iter().map(|s| s * s).sum::<f32>() / frames as f32).sqrt();
        let out_rms = (out.iter().map(|s| s * s).sum::<f32>() / frames as f32).sqrt();
        assert!(
            out_rms < in_rms * 0.25,
            "filter passed {out_rms} of {in_rms} rms"
        );
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn gate_below_schmitt_high_never_fires_the_envelope() {
        let mut env = EnvNode::new();
        env.prepare(&AudioConfig::new(SR, 256, 0, 1).unwrap());
        let frames = 4_800;
        let gate = vec![SCHMITT_HIGH_V * 0.9; frames];
        let out = run(&mut env, &gate, frames);
        assert!(out.iter().all(|s| *s == 0.0), "sub-threshold gate fired");
    }
}
