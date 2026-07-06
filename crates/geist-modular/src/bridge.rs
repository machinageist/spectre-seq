// =============================================================================
// File: crates/geist-modular/src/bridge.rs
// Layer: modular utilities
// Purpose: Bridge nodes between the DAW and the rack: clock, MIDI-CV, rack out
// Status: Implemented; mono MIDI-CV, 24-PPQN clock, audio pass-through out.
// Notes: TransportClockNode emits one 1 ms trigger per 24-PPQN tick plus a
//        divided output. MidiCvNode is monophonic last-note-priority per spec
//        §6.4: V/OCT (out 0) holds the last key in volts, GATE (out 1) is 10 V
//        while any key is held and does not retrigger on legato, VEL (out 2)
//        is 0-10 V, RTRG (out 3) pulses on every new note-on including legato.
//        RackOutNode sums its input channels to a mono audio tap for the chain.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;
use geist_core::events::NoteEventKind;
use geist_graph::node::AudioNode;

use crate::standards::{Pulse, AUDIO_ZERO_V_HZ, GATE_V};

// MIDI clock resolution: pulses per quarter note (spec §6.4)
const PPQN: f64 = 24.0;
// MIDI note 60 (C4) sits at 0 V on the v/oct scale
const MIDI_C4: f32 = 60.0;

// Convert a MIDI key to 1 V/oct pitch CV (C4 = 0 V, one volt per octave)
#[inline]
fn key_to_volts(key: u8) -> f32 {
    (key as f32 - MIDI_C4) / 12.0
}

// Transport-driven clock: out 0 = 24-PPQN trigger, out 1 = divided trigger.
// Ticks are derived from the block-start beat so tempo changes stay accurate.
pub struct TransportClockNode {
    // Divisor for the second output (>= 1); e.g. 24 = one pulse per quarter
    division: u32,
    sample_rate_hz: f32,
    // Highest PPQN tick index already emitted, to detect new ticks
    last_tick: i64,
    clock_pulse: Pulse,
    divided_pulse: Pulse,
    // Whether the transport was rolling last block, to reset on stop
    was_playing: bool,
}

impl TransportClockNode {
    pub fn new() -> Self {
        Self {
            division: 24,
            sample_rate_hz: 48_000.0,
            last_tick: -1,
            clock_pulse: Pulse::new(),
            divided_pulse: Pulse::new(),
            was_playing: false,
        }
    }

    // Set the divided-output divisor in PPQN ticks (>= 1)
    pub fn set_division(&mut self, division: u32) {
        self.division = division.max(1);
    }
}

impl Default for TransportClockNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for TransportClockNode {
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate_hz = config.sample_rate_hz as f32;
        self.reset();
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let sr = self.sample_rate_hz as u32;
        let transport = *ctx.transport();
        let playing = matches!(
            transport.state,
            geist_core::transport::TransportState::Playing
                | geist_core::transport::TransportState::Recording
        );
        // Stopping resets the tick phase so playback restarts cleanly
        if self.was_playing && !playing {
            self.last_tick = -1;
        }
        self.was_playing = playing;

        let (_input, output) = ctx.io();
        for f in 0..frames {
            if playing {
                let sample = transport.sample_pos + f as u64;
                let beat = transport.samples_to_beats(sample);
                let tick = (beat * PPQN).floor() as i64;
                if tick > self.last_tick {
                    self.last_tick = tick;
                    self.clock_pulse.fire(sr);
                    if tick % self.division as i64 == 0 {
                        self.divided_pulse.fire(sr);
                    }
                }
            }
            output[f] = self.clock_pulse.step();
            if out_ch >= 2 {
                output[frames + f] = self.divided_pulse.step();
            }
        }
        for ch in 2..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }

    fn reset(&mut self) {
        self.last_tick = -1;
        self.was_playing = false;
        self.clock_pulse = Pulse::new();
        self.divided_pulse = Pulse::new();
    }
}

// Monophonic MIDI-to-CV: last-note priority with a held-key stack so releasing
// the top note falls back to the one below (spec §6.4 mono behavior).
pub struct MidiCvNode {
    sample_rate_hz: u32,
    // Held keys in press order; the last entry drives V/OCT and GATE
    held: Vec<u8>,
    // Most recent velocity, latched until the next note-on
    velocity: f32,
    // Pitch volts latched from the last key, held after release
    last_pitch_v: f32,
    retrigger: Pulse,
}

// Most keys the mono stack tracks before dropping the oldest (fixed, no realloc)
const MAX_HELD: usize = 16;

impl MidiCvNode {
    pub fn new() -> Self {
        Self {
            sample_rate_hz: 48_000,
            held: Vec::with_capacity(MAX_HELD),
            velocity: 0.0,
            last_pitch_v: 0.0,
            retrigger: Pulse::new(),
        }
    }
}

impl Default for MidiCvNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioNode for MidiCvNode {
    fn prepare(&mut self, config: &AudioConfig) {
        self.sample_rate_hz = config.sample_rate_hz;
        self.reset();
    }

    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let sr = self.sample_rate_hz;
        // Snapshot notes and drop the borrow before writing outputs
        let mut events: [(u32, bool, u8, f32); MAX_HELD] = [(0, false, 0, 0.0); MAX_HELD];
        let mut event_count = 0;
        for note in ctx.notes() {
            if event_count >= MAX_HELD {
                break;
            }
            match note.kind {
                NoteEventKind::On => {
                    events[event_count] = (note.sample_offset, true, note.key, note.velocity);
                    event_count += 1;
                }
                NoteEventKind::Off => {
                    events[event_count] = (note.sample_offset, false, note.key, 0.0);
                    event_count += 1;
                }
            }
        }

        let (_input, output) = ctx.io();
        let mut next_event = 0;
        for f in 0..frames {
            // Apply every event landing at this frame before emitting the sample
            while next_event < event_count && events[next_event].0 as usize <= f {
                let (_, is_on, key, vel) = events[next_event];
                if is_on {
                    self.held.retain(|&k| k != key);
                    if self.held.len() == MAX_HELD {
                        self.held.remove(0);
                    }
                    self.held.push(key);
                    self.velocity = vel;
                    // Retrigger fires on every new note-on, including legato
                    self.retrigger.fire(sr);
                } else {
                    self.held.retain(|&k| k != key);
                }
                next_event += 1;
            }

            // Held: last key drives pitch and holds the gate. Released: pitch
            // stays at the last key's volts (V/OCT does not snap to 0), gate low
            let (pitch_v, gate_v) = match self.held.last() {
                Some(&key) => {
                    self.last_pitch_v = key_to_volts(key);
                    (self.last_pitch_v, GATE_V)
                }
                None => (self.last_pitch_v, 0.0),
            };
            output[f] = pitch_v;
            if out_ch >= 2 {
                output[frames + f] = gate_v;
            }
            if out_ch >= 3 {
                output[2 * frames + f] = self.velocity * GATE_V;
            }
            if out_ch >= 4 {
                output[3 * frames + f] = self.retrigger.step();
            }
        }
        for ch in 4..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }

    fn reset(&mut self) {
        self.held.clear();
        self.velocity = 0.0;
        self.last_pitch_v = 0.0;
        self.retrigger = Pulse::new();
    }
}

// Rack output tap: sums input channels into mono audio channel 0 for the chain.
// A rack patch ends here; the DAW mixer reads channel 0.
pub struct RackOutNode;

impl AudioNode for RackOutNode {
    fn process(&mut self, ctx: &mut ProcessContext) {
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        if out_ch == 0 {
            return;
        }
        let (input, output) = ctx.io();
        for (f, slot) in output[..frames].iter_mut().enumerate() {
            let mut sum = 0.0;
            for ch in 0..in_ch {
                sum += input[ch * frames + f];
            }
            *slot = sum;
        }
        for ch in 1..out_ch {
            output[ch * frames..(ch + 1) * frames].fill(0.0);
        }
    }
}

// Convert MIDI-CV V/OCT volts back to Hz for downstream display
#[inline]
pub fn cv_to_hz(volts: f32) -> f32 {
    AUDIO_ZERO_V_HZ * volts.exp2()
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::events::NoteEvent;
    use geist_core::transport::{TransportSnapshot, TransportState};

    const SR: u32 = 48_000;

    fn config() -> AudioConfig {
        AudioConfig::new(SR, 512, 0, 4).unwrap()
    }

    // Run one block with the given notes and a rolling transport at `sample_pos`
    fn run_notes(
        node: &mut dyn AudioNode,
        notes: &[NoteEvent],
        frames: usize,
        out_ch: usize,
    ) -> Vec<f32> {
        let input = vec![0.0f32; frames * out_ch.max(1)];
        let mut output = vec![0.0f32; frames * out_ch];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(frames, SR, &input, &mut output, notes, &[], transport);
        node.process(&mut ctx);
        output
    }

    #[test]
    fn midi_cv_holds_pitch_and_gate_for_the_last_note() {
        let mut node = MidiCvNode::new();
        node.prepare(&config());
        let frames = 256;
        // C4 (key 60) pressed at frame 0 -> 0 V pitch, 10 V gate
        let out = run_notes(&mut node, &[NoteEvent::on(0, 0, 60, 0.8)], frames, 4);
        assert!((out[frames - 1]).abs() < 1e-6, "C4 should be 0 V");
        assert_eq!(out[frames + frames - 1], GATE_V, "gate high while held");
        assert!((out[2 * frames + frames - 1] - 0.8 * GATE_V).abs() < 1e-4);
    }

    #[test]
    fn midi_cv_tracks_an_octave_up() {
        let mut node = MidiCvNode::new();
        node.prepare(&config());
        let frames = 64;
        // Key 72 is one octave above C4 -> +1 V
        let out = run_notes(&mut node, &[NoteEvent::on(0, 0, 72, 1.0)], frames, 4);
        assert!((out[frames - 1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn midi_cv_gate_does_not_retrigger_on_legato_but_rtrg_does() {
        let mut node = MidiCvNode::new();
        node.prepare(&config());
        let frames = 256;
        // Press 60 at frame 0, then 64 legato at frame 100 without releasing 60
        let notes = [NoteEvent::on(0, 0, 60, 0.9), NoteEvent::on(100, 0, 64, 0.7)];
        let out = run_notes(&mut node, &notes, frames, 4);
        // GATE (out 1) stays continuously high across the legato transition
        assert!(out[frames..2 * frames].iter().all(|&g| g == GATE_V));
        // RTRG (out 3) pulses right after the second note-on
        let rtrg = &out[3 * frames..4 * frames];
        assert_eq!(rtrg[110], GATE_V, "retrigger fires on the legato note");
        // Pitch now follows the newer key (64 = +4/12 V)
        assert!((out[frames - 1] - 4.0 / 12.0).abs() < 1e-5);
    }

    #[test]
    fn midi_cv_release_falls_back_to_the_held_note_below() {
        let mut node = MidiCvNode::new();
        node.prepare(&config());
        let frames = 300;
        // Hold 60, add 67, then release 67 at frame 200 -> pitch returns to 60
        let notes = [
            NoteEvent::on(0, 0, 60, 1.0),
            NoteEvent::on(50, 0, 67, 1.0),
            NoteEvent::off(200, 0, 67),
        ];
        let out = run_notes(&mut node, &notes, frames, 4);
        assert!((out[frames - 1]).abs() < 1e-6, "pitch back to C4 (0 V)");
        assert_eq!(
            out[frames + frames - 1],
            GATE_V,
            "gate still high (60 held)"
        );
    }

    #[test]
    fn midi_cv_gate_falls_when_all_keys_released() {
        let mut node = MidiCvNode::new();
        node.prepare(&config());
        let frames = 200;
        let notes = [NoteEvent::on(0, 0, 60, 1.0), NoteEvent::off(100, 0, 60)];
        let out = run_notes(&mut node, &notes, frames, 4);
        assert_eq!(out[frames + frames - 1], 0.0, "gate low after release");
    }

    #[test]
    fn transport_clock_emits_ppqn_ticks_while_playing() {
        let mut node = TransportClockNode::new();
        node.prepare(&config());
        // One quarter note at 120 BPM = 0.5 s = 24000 samples; expect 24 ticks
        let frames = 24_000;
        let input = vec![0.0f32; frames * 2];
        let mut output = vec![0.0f32; frames * 2];
        let mut transport = TransportSnapshot::stopped(SR);
        transport.state = TransportState::Playing;
        transport.tempo_bpm = 120.0;
        transport.sample_pos = 0;
        let mut ctx = ProcessContext::new(frames, SR, &input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        // Count rising edges on the 24-PPQN output (channel 0)
        let ticks = output[..frames]
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count();
        // 24 ticks per quarter; tick at sample 0 has no rising edge, so 23-24
        assert!(
            (23..=24).contains(&ticks),
            "expected ~24 ticks, got {ticks}"
        );
    }

    #[test]
    fn transport_clock_is_silent_when_stopped() {
        let mut node = TransportClockNode::new();
        node.prepare(&config());
        let frames = 1000;
        let out = run_notes(&mut node, &[], frames, 2);
        assert!(out.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn rack_out_sums_input_channels_to_mono() {
        let mut node = RackOutNode;
        let frames = 8;
        // Two input channels at 0.3 and 0.2 -> 0.5 mono
        let mut input = vec![0.3f32; frames];
        input.resize(frames * 2, 0.2);
        let mut output = vec![0.0f32; frames * 2];
        let transport = TransportSnapshot::stopped(SR);
        let mut ctx = ProcessContext::new(frames, SR, &input, &mut output, &[], &[], transport);
        node.process(&mut ctx);
        assert!(output[..frames].iter().all(|&s| (s - 0.5).abs() < 1e-6));
        assert!(output[frames..].iter().all(|&s| s == 0.0));
    }
}
