// =============================================================================
// File: plugins/geist-synth/src/daw_node.rs
// Layer: synth plugin
// Purpose: implements AudioNode for DAW-internal use
// Status: Implemented; SynthNode drives the voice pool from block note events.
// Notes: Note events are applied sample-accurately by rendering the sub-blocks
//        between them. The mono mix is duplicated to every output channel.
//        Buffers are sized in prepare(); process() does not allocate.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use spectre_core::events::NoteEventKind;
use spectre_dsp::prelude::SvfMode;
use spectre_graph::node::AudioNode;

use crate::engine::voice_pool::VoicePool;

// Default voice count when none is specified
const DEFAULT_POLYPHONY: usize = 16;

// Graph node wrapping the synth voice pool for in-DAW playback
pub struct SynthNode {
    pool: VoicePool,
    scratch: Vec<f32>,
    sample_rate: f32,
    polyphony: usize,
}

impl SynthNode {
    // Build a synth node at a sample rate with a voice count
    pub fn new(sample_rate_hz: f32, polyphony: usize) -> Self {
        let polyphony = polyphony.max(1);
        Self {
            pool: VoicePool::new(sample_rate_hz, polyphony),
            scratch: Vec::new(),
            sample_rate: sample_rate_hz,
            polyphony,
        }
    }

    // Mutable access to the voice pool for patch and steal-mode configuration
    pub fn voice_pool_mut(&mut self) -> &mut VoicePool {
        &mut self.pool
    }

    // Set the base lowpass cutoff and resonance on every voice
    // The per-voice filter envelope still modulates relative to this base, so a
    // control surface can sweep the filter without disabling the envelope
    pub fn set_filter(&mut self, cutoff_hz: f32, resonance: f32) {
        for voice in self.pool.voices_mut() {
            voice
                .filter_mut()
                .set_filter_a(cutoff_hz, resonance, SvfMode::Lowpass);
        }
    }

    // Set oscillator A unison voice count and detune spread on every voice
    // Only retunes voice frequencies, so it is safe to call per block
    pub fn set_unison(&mut self, voices: usize, detune_cents: f32) {
        for voice in self.pool.voices_mut() {
            voice.osc_mut().set_osc_a_unison(voices, detune_cents);
        }
    }

    // Set the amplitude ADSR on every voice; safe to call per block
    pub fn set_amp_env(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        for voice in self.pool.voices_mut() {
            voice.set_amp_env(attack, decay, sustain, release);
        }
    }

    // Set the filter ADSR on every voice; safe to call per block
    pub fn set_filter_env(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        for voice in self.pool.voices_mut() {
            voice.set_filter_env(attack, decay, sustain, release);
        }
    }

    // Set the oscillator A/B blend (0 = sine, 1 = saw) on every voice
    pub fn set_osc_mix(&mut self, mix: f32) {
        for voice in self.pool.voices_mut() {
            voice.osc_mut().set_mix(mix);
        }
    }

    // Set oscillator B's pitch offset in semitones on every voice
    pub fn set_osc_b_semitones(&mut self, semitones: f32) {
        for voice in self.pool.voices_mut() {
            voice.osc_mut().set_osc_b_semitones(semitones);
        }
    }
}

impl Default for SynthNode {
    fn default() -> Self {
        Self::new(48_000.0, DEFAULT_POLYPHONY)
    }
}

impl AudioNode for SynthNode {
    // Size the mono scratch buffer and rebuild the pool at the stream rate
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate = config.sample_rate_hz as f32;
        self.pool = VoicePool::new(self.sample_rate, self.polyphony);
        self.scratch = vec![0.0; config.frames_per_block()];
    }

    // Render a block, applying note events at their sample offsets
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        // Safety net if process runs before prepare; prepare avoids this on audio
        if self.scratch.len() < frames {
            self.scratch.resize(frames, 0.0);
        }

        // Walk events in order, rendering the gaps between them
        let mut cursor = 0usize;
        for event in ctx.notes() {
            let offset = (event.sample_offset as usize).min(frames);
            if offset > cursor {
                self.pool.render_block(&mut self.scratch[cursor..offset]);
                cursor = offset;
            }
            match event.kind {
                NoteEventKind::On => self.pool.note_on(event.key, event.velocity),
                NoteEventKind::Off => self.pool.note_off(event.key),
            }
        }
        if cursor < frames {
            self.pool.render_block(&mut self.scratch[cursor..frames]);
        }

        // Fan the mono mix out to every output channel
        let channels = ctx.output_channels();
        for ch in 0..channels {
            if let Some(out) = ctx.output(ch) {
                out.copy_from_slice(&self.scratch[..frames]);
            }
        }
    }

    // Silence all voices
    fn reset(&mut self) {
        self.pool.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::events::NoteEvent;
    use spectre_core::transport::TransportSnapshot;

    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = 128;
    const CHANNELS: usize = 2;

    fn config() -> AudioConfig {
        AudioConfig::new(SAMPLE_RATE, FRAMES as u32, 0, CHANNELS as u16).unwrap()
    }

    // Render one block with the given note events, returning the stereo output
    fn render(node: &mut SynthNode, notes: &[NoteEvent]) -> Vec<f32> {
        let mut output = vec![0.0f32; FRAMES * CHANNELS];
        {
            let transport = TransportSnapshot::stopped(SAMPLE_RATE);
            let mut ctx =
                ProcessContext::new(FRAMES, SAMPLE_RATE, &[], &mut output, notes, &[], transport);
            node.process(&mut ctx);
        }
        output
    }

    fn block_energy(buf: &[f32]) -> f32 {
        buf.iter().map(|s| s.abs()).sum()
    }

    #[test]
    fn idle_node_is_silent() {
        let mut node = SynthNode::new(SAMPLE_RATE as f32, 8);
        node.prepare(&config());
        let out = render(&mut node, &[]);
        assert!(out.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn note_on_produces_audio_on_all_channels() {
        let mut node = SynthNode::new(SAMPLE_RATE as f32, 8);
        node.prepare(&config());

        // First block carries the note-on; later blocks hold the note
        let _ = render(&mut node, &[NoteEvent::on(0, 0, 69, 1.0)]);
        let mut out = vec![0.0f32; FRAMES * CHANNELS];
        for _ in 0..4 {
            out = render(&mut node, &[]);
        }
        assert!(block_energy(&out) > 0.01, "synth produced no audio");

        // Both channels carry the same mono mix
        let ch0 = &out[..FRAMES];
        let ch1 = &out[FRAMES..];
        assert_eq!(ch0, ch1);
    }

    #[test]
    fn note_off_eventually_returns_to_silence() {
        let mut node = SynthNode::new(SAMPLE_RATE as f32, 8);
        node.prepare(&config());
        let _ = render(&mut node, &[NoteEvent::on(0, 0, 60, 1.0)]);
        for _ in 0..4 {
            render(&mut node, &[]);
        }
        let _ = render(&mut node, &[NoteEvent::off(0, 0, 60)]);
        // Run past the release
        let mut out = vec![0.0f32; FRAMES * CHANNELS];
        for _ in 0..400 {
            out = render(&mut node, &[]);
        }
        assert!(block_energy(&out) < 1e-4, "synth never returned to silence");
    }

    #[test]
    fn reset_silences_active_voices() {
        let mut node = SynthNode::new(SAMPLE_RATE as f32, 8);
        node.prepare(&config());
        let _ = render(&mut node, &[NoteEvent::on(0, 0, 64, 1.0)]);
        for _ in 0..4 {
            render(&mut node, &[]);
        }
        node.reset();
        let out = render(&mut node, &[]);
        assert!(
            out.iter().all(|&s| s == 0.0),
            "reset did not silence the synth"
        );
    }

    #[test]
    fn sample_accurate_note_starts_mid_block() {
        let mut node = SynthNode::new(SAMPLE_RATE as f32, 8);
        node.prepare(&config());
        // Note begins at the block midpoint; the first half stays silent
        let out = render(&mut node, &[NoteEvent::on((FRAMES / 2) as u32, 0, 72, 1.0)]);
        let first_half: f32 = out[..FRAMES / 2].iter().map(|s| s.abs()).sum();
        assert_eq!(first_half, 0.0, "audio leaked before the note offset");
    }

    #[test]
    fn filter_macro_changes_brightness() {
        // A low base cutoff must pass less energy than a wide-open one
        let mut dark = SynthNode::new(SAMPLE_RATE as f32, 8);
        dark.prepare(&config());
        dark.set_filter(120.0, 0.7);
        let mut bright = SynthNode::new(SAMPLE_RATE as f32, 8);
        bright.prepare(&config());
        bright.set_filter(16_000.0, 0.7);

        let key = 84;
        let _ = render(&mut dark, &[NoteEvent::on(0, 0, key, 1.0)]);
        let _ = render(&mut bright, &[NoteEvent::on(0, 0, key, 1.0)]);
        // Skip past the filter envelope's attack/decay into sustain, where the
        // base cutoff dominates (the env opens the filter wide during attack)
        for _ in 0..120 {
            render(&mut dark, &[]);
            render(&mut bright, &[]);
        }
        let mut dark_energy = 0.0;
        let mut bright_energy = 0.0;
        for _ in 0..16 {
            dark_energy += block_energy(&render(&mut dark, &[]));
            bright_energy += block_energy(&render(&mut bright, &[]));
        }
        assert!(
            bright_energy > dark_energy * 1.5,
            "filter macro had no audible effect: {bright_energy} vs {dark_energy}"
        );
    }

    #[test]
    fn amp_env_release_shapes_the_tail() {
        // A short release dies out faster than a long one after note-off
        let mut quick = SynthNode::new(SAMPLE_RATE as f32, 4);
        quick.prepare(&config());
        quick.set_amp_env(0.001, 0.05, 1.0, 0.02);
        let mut slow = SynthNode::new(SAMPLE_RATE as f32, 4);
        slow.prepare(&config());
        slow.set_amp_env(0.001, 0.05, 1.0, 1.5);

        let _ = render(&mut quick, &[NoteEvent::on(0, 0, 69, 1.0)]);
        let _ = render(&mut slow, &[NoteEvent::on(0, 0, 69, 1.0)]);
        for _ in 0..8 {
            render(&mut quick, &[]);
            render(&mut slow, &[]);
        }
        let _ = render(&mut quick, &[NoteEvent::off(0, 0, 69)]);
        let _ = render(&mut slow, &[NoteEvent::off(0, 0, 69)]);

        let mut quick_tail = 0.0;
        let mut slow_tail = 0.0;
        for _ in 0..20 {
            quick_tail += block_energy(&render(&mut quick, &[]));
            slow_tail += block_energy(&render(&mut slow, &[]));
        }
        assert!(
            slow_tail > quick_tail * 2.0,
            "longer release did not sustain more energy: {slow_tail} vs {quick_tail}"
        );
    }

    #[test]
    fn osc_mix_changes_the_waveform() {
        // Full osc A (sine) and full osc B (saw) must render differently
        let mut sine = SynthNode::new(SAMPLE_RATE as f32, 4);
        sine.prepare(&config());
        sine.set_osc_mix(0.0);
        let mut saw = SynthNode::new(SAMPLE_RATE as f32, 4);
        saw.prepare(&config());
        saw.set_osc_mix(1.0);

        let _ = render(&mut sine, &[NoteEvent::on(0, 0, 69, 1.0)]);
        let _ = render(&mut saw, &[NoteEvent::on(0, 0, 69, 1.0)]);
        for _ in 0..4 {
            render(&mut sine, &[]);
            render(&mut saw, &[]);
        }
        assert_ne!(
            render(&mut sine, &[]),
            render(&mut saw, &[]),
            "osc mix had no effect"
        );
    }

    #[test]
    fn osc_b_semitones_shift_the_blend() {
        // With osc B audible, detuning it must change the output
        let mut base = SynthNode::new(SAMPLE_RATE as f32, 4);
        base.prepare(&config());
        base.set_osc_mix(0.5);
        let mut shifted = SynthNode::new(SAMPLE_RATE as f32, 4);
        shifted.prepare(&config());
        shifted.set_osc_mix(0.5);
        shifted.set_osc_b_semitones(-12.0);

        let _ = render(&mut base, &[NoteEvent::on(0, 0, 69, 1.0)]);
        let _ = render(&mut shifted, &[NoteEvent::on(0, 0, 69, 1.0)]);
        for _ in 0..4 {
            render(&mut base, &[]);
            render(&mut shifted, &[]);
        }
        assert_ne!(
            render(&mut base, &[]),
            render(&mut shifted, &[]),
            "osc B detune had no effect"
        );
    }

    #[test]
    fn unison_changes_the_oscillator_output() {
        let mut single = SynthNode::new(SAMPLE_RATE as f32, 8);
        single.prepare(&config());
        single.set_unison(1, 0.0);
        let mut wide = SynthNode::new(SAMPLE_RATE as f32, 8);
        wide.prepare(&config());
        wide.set_unison(5, 25.0);

        let _ = render(&mut single, &[NoteEvent::on(0, 0, 69, 1.0)]);
        let _ = render(&mut wide, &[NoteEvent::on(0, 0, 69, 1.0)]);
        for _ in 0..4 {
            render(&mut single, &[]);
            render(&mut wide, &[]);
        }
        let single_block = render(&mut single, &[]);
        let wide_block = render(&mut wide, &[]);
        assert!(
            single_block != wide_block,
            "unison/detune had no effect on the oscillator output"
        );
    }
}
