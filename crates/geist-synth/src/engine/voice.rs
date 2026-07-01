// =============================================================================
// File: crates/geist-synth/src/engine/voice.rs
// Layer: internal synth device
// Purpose: per-voice state: oscs, filters, envs
// Status: Implemented; OscStack -> FilterStack with amp + filter ADSR.
// Notes: Filter cutoff is modulated at control rate (per block) so the SVF
//        coefficient update stays off the per-sample path; amp env is per sample.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_dsp::prelude::{Adsr, SvfMode};

use crate::engine::filter_stack::FilterStack;
use crate::engine::osc_stack::OscStack;

// Default filter envelope sweep depth, in octaves of cutoff
const DEFAULT_FILTER_ENV_OCTAVES: f32 = 4.0;

// One polyphonic voice: oscillators, filters, and two envelopes
#[derive(Clone, Debug)]
pub struct Voice {
    osc: OscStack,
    filter: FilterStack,
    amp_env: Adsr,
    filter_env: Adsr,
    note: u8,
    velocity: f32,
    filter_env_octaves: f32,
    last_filter_env: f32,
}

impl Voice {
    // Build an idle voice with musical default envelopes and filter
    pub fn new(sample_rate_hz: f32) -> Self {
        let mut amp_env = Adsr::new(sample_rate_hz);
        amp_env.set_attack(0.005);
        amp_env.set_decay(0.1);
        amp_env.set_sustain(0.8);
        amp_env.set_release(0.3);

        let mut filter_env = Adsr::new(sample_rate_hz);
        filter_env.set_attack(0.01);
        filter_env.set_decay(0.2);
        filter_env.set_sustain(0.3);
        filter_env.set_release(0.3);

        let mut filter = FilterStack::new(sample_rate_hz);
        filter.set_filter_a(1_500.0, 0.9, SvfMode::Lowpass);
        filter.set_filter_b(20_000.0, 0.707, SvfMode::Lowpass); // effectively open

        Self {
            osc: OscStack::new(sample_rate_hz),
            filter,
            amp_env,
            filter_env,
            note: 0,
            velocity: 0.0,
            filter_env_octaves: DEFAULT_FILTER_ENV_OCTAVES,
            last_filter_env: 0.0,
        }
    }

    // Mutable access to the oscillator stack for patch configuration
    pub fn osc_mut(&mut self) -> &mut OscStack {
        &mut self.osc
    }

    // Mutable access to the filter stack for patch configuration
    pub fn filter_mut(&mut self) -> &mut FilterStack {
        &mut self.filter
    }

    // Set the amplitude envelope (attack/decay/release seconds, sustain 0..1)
    pub fn set_amp_env(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.amp_env.set_attack(attack);
        self.amp_env.set_decay(decay);
        self.amp_env.set_sustain(sustain);
        self.amp_env.set_release(release);
    }

    // Set the filter envelope (attack/decay/release seconds, sustain 0..1)
    pub fn set_filter_env(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.filter_env.set_attack(attack);
        self.filter_env.set_decay(decay);
        self.filter_env.set_sustain(sustain);
        self.filter_env.set_release(release);
    }

    // Start a note: retune, retrigger envelopes from a clean state
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.note = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.osc.set_note(note as f32);
        self.osc.reset();
        self.filter.reset();
        self.last_filter_env = 0.0;
        self.amp_env.reset();
        self.filter_env.reset();
        self.amp_env.gate_on();
        self.filter_env.gate_on();
    }

    // Release the note into the envelope tails
    pub fn note_off(&mut self) {
        self.amp_env.gate_off();
        self.filter_env.gate_off();
    }

    // Force the voice silent and idle
    pub fn reset(&mut self) {
        self.amp_env.reset();
        self.filter_env.reset();
        self.osc.reset();
        self.filter.reset();
        self.velocity = 0.0;
        self.last_filter_env = 0.0;
    }

    // True while the amp envelope is still producing sound
    pub fn is_active(&self) -> bool {
        self.amp_env.is_active()
    }

    // MIDI note this voice is playing
    pub fn note(&self) -> u8 {
        self.note
    }

    // Current loudness proxy for voice stealing (amp envelope x velocity)
    pub fn amp_level(&self) -> f32 {
        self.amp_env.value() * self.velocity
    }

    // Render one block, summing the voice into existing output
    pub fn render_additive(&mut self, output: &mut [f32]) {
        if !self.is_active() {
            return;
        }
        // Control-rate cutoff: open the filter by the last block's env value
        let factor = 2.0_f32.powf(self.last_filter_env * self.filter_env_octaves);
        self.filter.set_cutoff_mod(factor);

        for sample in output.iter_mut() {
            let amp = self.amp_env.process_sample();
            self.last_filter_env = self.filter_env.process_sample();
            let osc = self.osc.render();
            let filtered = self.filter.process_sample(osc);
            *sample += filtered * amp * self.velocity;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;
    const BLOCK: usize = 256;

    fn block_peak(voice: &mut Voice) -> f32 {
        let mut buf = vec![0.0f32; BLOCK];
        voice.render_additive(&mut buf);
        buf.iter().fold(0.0f32, |m, &v| m.max(v.abs()))
    }

    #[test]
    fn idle_voice_is_silent() {
        let mut voice = Voice::new(SAMPLE_RATE);
        assert!(!voice.is_active());
        let mut buf = vec![1.0f32; BLOCK]; // pre-filled; additive render must add nothing
        voice.render_additive(&mut buf);
        assert!(buf.iter().all(|&s| s == 1.0));
    }

    #[test]
    fn note_on_produces_sound() {
        let mut voice = Voice::new(SAMPLE_RATE);
        voice.note_on(69, 1.0);
        assert!(voice.is_active());
        // Skip the attack, then expect signal
        for _ in 0..4 {
            block_peak(&mut voice);
        }
        assert!(block_peak(&mut voice) > 0.01, "voice produced no sound");
    }

    #[test]
    fn note_off_eventually_frees_the_voice() {
        let mut voice = Voice::new(SAMPLE_RATE);
        voice.note_on(60, 1.0);
        for _ in 0..8 {
            block_peak(&mut voice);
        }
        voice.note_off();
        // Run well past the release time
        let mut buf = vec![0.0f32; BLOCK];
        for _ in 0..400 {
            buf.iter_mut().for_each(|s| *s = 0.0);
            voice.render_additive(&mut buf);
        }
        assert!(!voice.is_active(), "voice never freed after release");
    }

    #[test]
    fn velocity_scales_loudness() {
        let mut loud = Voice::new(SAMPLE_RATE);
        loud.note_on(72, 1.0);
        let mut soft = Voice::new(SAMPLE_RATE);
        soft.note_on(72, 0.5);
        // Settle past attack into sustain
        for _ in 0..40 {
            block_peak(&mut loud);
            block_peak(&mut soft);
        }
        let loud_peak = block_peak(&mut loud);
        let soft_peak = block_peak(&mut soft);
        assert!(
            loud_peak > soft_peak,
            "velocity did not scale: {loud_peak} vs {soft_peak}"
        );
    }

    #[test]
    fn output_is_additive() {
        // Two voices into the same buffer should sum, not overwrite
        let mut a = Voice::new(SAMPLE_RATE);
        a.note_on(60, 1.0);
        let mut b = Voice::new(SAMPLE_RATE);
        b.note_on(67, 1.0);
        for _ in 0..8 {
            let mut tmp = vec![0.0f32; BLOCK];
            a.render_additive(&mut tmp);
            b.render_additive(&mut tmp);
        }
        let mut buf = vec![0.0f32; BLOCK];
        a.render_additive(&mut buf);
        let after_a: f32 = buf.iter().map(|s| s.abs()).sum();
        b.render_additive(&mut buf);
        let after_both: f32 = buf.iter().map(|s| s.abs()).sum();
        assert!(after_both > after_a, "second voice did not add energy");
    }
}
