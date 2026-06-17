// =============================================================================
// File: app/geist-daw/src/engine.rs
// Layer: application binary
// Purpose: Demo sequencer, block processor, and the running engine handle
// Status: Implemented; drives one SynthNode from a looping step sequence.
// Notes: The block processor runs on cpal's audio thread, so it never allocates:
//        the event buffer is preallocated and only cleared/refilled per block.
//        Steps are quantized to whole blocks, so every note lands at offset 0.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_audio_backend::prelude::{BlockProcessor, CpalBackend, Stream};
use geist_core::context::ProcessContext;
use geist_core::events::NoteEvent;
use geist_graph::node::AudioNode;
use geist_synth::prelude::SynthNode;
use geist_timeline::prelude::Transport;

use crate::control::{EngineCommand, EngineSink};
use crate::fx::FxChain;

// Sequencer steps per beat (sixteenth notes)
const STEPS_PER_BEAT: f64 = 4.0;
// Note velocity for every triggered step
const STEP_VELOCITY: f32 = 0.9;
// Pitch rows: one chromatic octave from the base note
pub const SEQ_ROWS: usize = 13;
// Steps per pattern: one bar of sixteenths
pub const SEQ_STEPS: usize = 16;
// MIDI note at row 0 (the lowest row), C4
pub const SEQ_BASE_MIDI: u8 = 60;

// Pattern grid: grid[row][step] is true when that note plays on that step
pub type Grid = [[bool; SEQ_STEPS]; SEQ_ROWS];

// Default seed pattern so the sequencer sounds musical before any editing
pub fn default_grid() -> Grid {
    let mut grid = [[false; SEQ_STEPS]; SEQ_ROWS];
    // One note per step: a rising/falling riff over a pentatonic-ish shape
    const RIFF: [usize; SEQ_STEPS] = [0, 2, 4, 7, 9, 7, 4, 2, 0, 2, 4, 7, 9, 12, 9, 4];
    let mut step = 0;
    while step < SEQ_STEPS {
        grid[RIFF[step]][step] = true;
        step += 1;
    }
    grid
}

// An all-off pattern grid
pub fn empty_grid() -> Grid {
    [[false; SEQ_STEPS]; SEQ_ROWS]
}

// Number of mixer tracks
pub const NUM_TRACKS: usize = 3;
// Base MIDI note for each track: an octave below, at, and above the base
pub const TRACK_BASE_MIDI: [u8; NUM_TRACKS] =
    [SEQ_BASE_MIDI - 12, SEQ_BASE_MIDI, SEQ_BASE_MIDI + 12];

// Seed the mid track with the riff; the others start empty
pub fn default_grid_for(track: usize) -> Grid {
    if track == 1 {
        default_grid()
    } else {
        empty_grid()
    }
}

// Tempo-synced step sequencer reading columns from the transport beat position
pub struct Sequencer {
    grid: Grid,
    // MIDI note at row 0 for this track
    base_midi: u8,
    // Absolute column index last triggered, -1 before the first
    last_step: i64,
    // Which rows are currently sounding, released when the column changes
    sounding: [bool; SEQ_ROWS],
}

impl Sequencer {
    // Build a sequencer over a pattern at a given base note
    pub fn new(base_midi: u8, grid: Grid) -> Self {
        Self {
            grid,
            base_midi,
            last_step: -1,
            sounding: [false; SEQ_ROWS],
        }
    }

    // Toggle one cell on or off
    pub fn set_cell(&mut self, step: usize, row: usize, on: bool) {
        if step < SEQ_STEPS && row < SEQ_ROWS {
            self.grid[row][step] = on;
        }
    }

    // Clear the whole pattern
    pub fn clear(&mut self) {
        self.grid = [[false; SEQ_STEPS]; SEQ_ROWS];
    }

    // Trigger a column when `beat` crosses into a new one; allocation-free
    pub fn advance_to_beat(&mut self, beat: f64, out: &mut Vec<NoteEvent>) {
        let absolute = (beat * STEPS_PER_BEAT).floor() as i64;
        if absolute == self.last_step {
            return;
        }
        let column = absolute.rem_euclid(SEQ_STEPS as i64) as usize;
        // Release the previous column, then trigger the new one
        self.release_sounding(out);
        for row in 0..SEQ_ROWS {
            if self.grid[row][column] {
                push_capped(out, NoteEvent::on(0, 0, self.base_midi + row as u8, STEP_VELOCITY));
                self.sounding[row] = true;
            }
        }
        self.last_step = absolute;
    }

    // Stop all sounding rows and rearm so the next play retriggers from column 0
    pub fn release(&mut self, out: &mut Vec<NoteEvent>) {
        self.release_sounding(out);
        self.last_step = -1;
    }

    // Emit note-offs for every currently sounding row
    fn release_sounding(&mut self, out: &mut Vec<NoteEvent>) {
        for row in 0..SEQ_ROWS {
            if self.sounding[row] {
                push_capped(out, NoteEvent::off(0, 0, self.base_midi + row as u8));
                self.sounding[row] = false;
            }
        }
    }
}

// Balance pan gain for one output channel: unity at center, linear taper.
// Channel 0 is left, channel 1 is right; further channels pass at unity.
fn pan_gain(channel: usize, channels: usize, pan: f32) -> f32 {
    if channels < 2 {
        return 1.0;
    }
    match channel {
        0 => (1.0 - pan).clamp(0.0, 1.0),
        1 => (1.0 + pan).clamp(0.0, 1.0),
        _ => 1.0,
    }
}

// Push only while the preallocated buffer has room; keeps process() alloc-free
fn push_capped(out: &mut Vec<NoteEvent>, event: NoteEvent) {
    if out.len() < out.capacity() {
        out.push(event);
    }
}

// Most note events one block can carry from UI commands plus the demo
const MAX_BLOCK_EVENTS: usize = 64;
// Startup filter macro, matching the synth's default voice patch
const DEFAULT_CUTOFF_HZ: f32 = 1_500.0;
const DEFAULT_RESONANCE: f32 = 0.9;
// Startup master gain (unity)
const DEFAULT_GAIN: f32 = 1.0;
// Startup oscillator unison: a single voice, no detune
const DEFAULT_UNISON_VOICES: usize = 1;
const DEFAULT_DETUNE_CENTS: f32 = 0.0;
// Startup oscillator blend (sine/saw) and osc B pitch offset, matching OscStack
pub const DEFAULT_OSC_MIX: f32 = 0.5;
pub const DEFAULT_OSC_B_SEMIS: f32 = 0.0;
// Startup amp/filter ADSR, matching the voice's musical defaults
// [attack, decay, sustain, release]
pub const DEFAULT_AMP_ENV: [f32; 4] = [0.005, 0.1, 0.8, 0.3];
pub const DEFAULT_FILTER_ENV: [f32; 4] = [0.01, 0.2, 0.3, 0.3];
// Startup transport tempo
pub const DEFAULT_BPM: f64 = 120.0;

// Default per-track mixer level
const DEFAULT_TRACK_LEVEL: f32 = 0.8;

// Most notes one clip can hold; sized so add/remove/clear stay allocation-free
pub const MAX_CLIP_NOTES: usize = 256;
// Default clip loop length, matching the UI piano roll
pub const DEFAULT_CLIP_LEN_BEATS: f32 = 16.0;
// Start-beat tolerance when matching a note for removal
const START_EPS: f32 = 1e-3;

// One timed note in a looping clip, with its current sounding state
#[derive(Copy, Clone)]
struct ClipNote {
    pitch: u8,
    start_beats: f32,
    len_beats: f32,
    velocity: f32,
    sounding: bool,
}

// A looping piano-roll clip: arbitrary timed notes played against the transport
// Level-based: each block, notes whose span contains the loop phase sound; the
// rest are released. Robust to looping, seeking, and tempo changes.
pub struct NoteClip {
    notes: Vec<ClipNote>,
    length_beats: f32,
}

impl NoteClip {
    // Empty clip of a given loop length; capacity is fixed for realtime safety
    pub fn new(length_beats: f32) -> Self {
        Self {
            notes: Vec::with_capacity(MAX_CLIP_NOTES),
            length_beats,
        }
    }

    // Add a note unless the clip is full (keeps the buffer from reallocating)
    pub fn add(&mut self, pitch: u8, start_beats: f32, len_beats: f32, velocity: f32) {
        if self.notes.len() < MAX_CLIP_NOTES {
            self.notes.push(ClipNote {
                pitch,
                start_beats,
                len_beats,
                velocity,
                sounding: false,
            });
        }
    }

    // Remove the first note matching pitch+start, releasing it if it was sounding
    pub fn remove(&mut self, pitch: u8, start_beats: f32, out: &mut Vec<NoteEvent>) {
        if let Some(index) = self
            .notes
            .iter()
            .position(|n| n.pitch == pitch && (n.start_beats - start_beats).abs() < START_EPS)
        {
            if self.notes[index].sounding {
                push_capped(out, NoteEvent::off(0, 0, pitch));
            }
            self.notes.swap_remove(index);
        }
    }

    // Drop every note, releasing any that are sounding
    pub fn clear(&mut self, out: &mut Vec<NoteEvent>) {
        for note in &self.notes {
            if note.sounding {
                push_capped(out, NoteEvent::off(0, 0, note.pitch));
            }
        }
        self.notes.clear();
    }

    // Trigger/release notes so the sounding set matches the loop phase at `beat`
    pub fn advance_to_beat(&mut self, beat: f64, out: &mut Vec<NoteEvent>) {
        if self.length_beats <= 0.0 {
            return;
        }
        let phase = beat.rem_euclid(self.length_beats as f64) as f32;
        for note in &mut self.notes {
            let should = phase >= note.start_beats && phase < note.start_beats + note.len_beats;
            if should && !note.sounding {
                push_capped(out, NoteEvent::on(0, 0, note.pitch, note.velocity));
                note.sounding = true;
            } else if !should && note.sounding {
                push_capped(out, NoteEvent::off(0, 0, note.pitch));
                note.sounding = false;
            }
        }
    }

    // Release every sounding note, e.g. when the transport stops
    pub fn release(&mut self, out: &mut Vec<NoteEvent>) {
        for note in &mut self.notes {
            if note.sounding {
                push_capped(out, NoteEvent::off(0, 0, note.pitch));
                note.sounding = false;
            }
        }
    }
}

// One mixer track: an instrument, its step pattern, its note clip, and mix state
pub struct Track {
    node: SynthNode,
    sequencer: Sequencer,
    clip: NoteClip,
    level: f32,
    pan: f32,
    muted: bool,
    soloed: bool,
}

impl Track {
    // Build a track with its base note and seed pattern; node still needs prepare
    pub fn new(sample_rate_hz: u32, polyphony: usize, base_midi: u8, grid: Grid) -> Self {
        Self {
            node: SynthNode::new(sample_rate_hz as f32, polyphony),
            sequencer: Sequencer::new(base_midi, grid),
            clip: NoteClip::new(DEFAULT_CLIP_LEN_BEATS),
            level: DEFAULT_TRACK_LEVEL,
            pan: 0.0,
            muted: false,
            soloed: false,
        }
    }

    // Prepare the track's instrument for the stream
    pub fn prepare(&mut self, config: &geist_core::config::AudioConfig) {
        self.node.prepare(config);
    }
}

// Block processor: drains UI commands, mixes the tracks, and publishes the meter
pub struct SynthProcessor {
    tracks: Vec<Track>,
    sample_rate_hz: u32,
    // Audio-thread end of the control plane
    sink: EngineSink,
    // Transport driving the tempo-synced sequencers
    transport: Transport,
    // Macro filter base cutoff/resonance, applied to every track each block
    cutoff_hz: f32,
    resonance: f32,
    // Oscillator A unison voices and detune, applied to every track each block
    unison_voices: usize,
    detune_cents: f32,
    // Oscillator A/B blend and osc B pitch offset, applied to every track each block
    osc_mix: f32,
    osc_b_semis: f32,
    // Amp/filter ADSR macros [attack, decay, sustain, release], applied per block
    amp_env: [f32; 4],
    filter_env: [f32; 4],
    // Master output gain applied post-effects
    gain: f32,
    // Post-synth effects chain
    fx: FxChain,
    // Per-track note events, preallocated so process_block never allocates
    track_events: Vec<Vec<NoteEvent>>,
    // One track's rendered block, summed into the master with its level
    scratch: Vec<f32>,
}

impl SynthProcessor {
    // Assemble the processor; tracks and fx must already be prepared
    pub fn new(
        tracks: Vec<Track>,
        sample_rate_hz: u32,
        block_len: usize,
        sink: EngineSink,
        fx: FxChain,
        rolling: bool,
        bpm: f64,
    ) -> Self {
        let mut transport = Transport::new(sample_rate_hz, bpm);
        if rolling {
            transport.play();
        }
        let track_events = (0..tracks.len())
            .map(|_| Vec::with_capacity(MAX_BLOCK_EVENTS))
            .collect();
        Self {
            tracks,
            sample_rate_hz,
            sink,
            transport,
            cutoff_hz: DEFAULT_CUTOFF_HZ,
            resonance: DEFAULT_RESONANCE,
            unison_voices: DEFAULT_UNISON_VOICES,
            detune_cents: DEFAULT_DETUNE_CENTS,
            osc_mix: DEFAULT_OSC_MIX,
            osc_b_semis: DEFAULT_OSC_B_SEMIS,
            amp_env: DEFAULT_AMP_ENV,
            filter_env: DEFAULT_FILTER_ENV,
            gain: DEFAULT_GAIN,
            fx,
            track_events,
            scratch: vec![0.0; block_len],
        }
    }
}

impl BlockProcessor for SynthProcessor {
    // Drain commands, mix every track, run master fx, and publish the peak
    fn process_block(&mut self, _input: &[f32], output: &mut [f32], channels: usize, frames: usize) {
        let Self {
            tracks,
            sample_rate_hz,
            sink,
            transport,
            cutoff_hz,
            resonance,
            unison_voices,
            detune_cents,
            osc_mix,
            osc_b_semis,
            amp_env,
            filter_env,
            gain,
            fx,
            track_events,
            scratch,
        } = self;

        for events in track_events.iter_mut() {
            events.clear();
        }

        // Translate queued UI commands into per-track events, transport, and macros
        while let Ok(command) = sink.commands.pop() {
            match command {
                EngineCommand::NoteOn { track, key, velocity } => {
                    if let Some(events) = track_events.get_mut(track as usize) {
                        push_capped(events, NoteEvent::on(0, 0, key, velocity));
                    }
                }
                EngineCommand::NoteOff { track, key } => {
                    if let Some(events) = track_events.get_mut(track as usize) {
                        push_capped(events, NoteEvent::off(0, 0, key));
                    }
                }
                EngineCommand::AllNotesOff => {
                    for track in tracks.iter_mut() {
                        track.node.reset();
                    }
                }
                EngineCommand::SetPlaying(on) => {
                    if on {
                        transport.play();
                    } else {
                        transport.stop();
                        for (track, events) in tracks.iter_mut().zip(track_events.iter_mut()) {
                            track.sequencer.release(events);
                            track.clip.release(events);
                        }
                    }
                }
                EngineCommand::SetBpm(bpm) => {
                    transport.tempo_map_mut().set_tempo(0.0, bpm as f64);
                }
                EngineCommand::SetCutoff(hz) => *cutoff_hz = hz,
                EngineCommand::SetResonance(res) => *resonance = res,
                EngineCommand::SetGain(value) => *gain = value,
                EngineCommand::SetDelay(on) => fx.set_delay(on),
                EngineCommand::SetDelayTime(seconds) => fx.set_delay_time(seconds),
                EngineCommand::SetDelayFeedback(feedback) => fx.set_delay_feedback(feedback),
                EngineCommand::SetDelayMix(mix) => fx.set_delay_mix(mix),
                EngineCommand::SetReverb(on) => fx.set_reverb(on),
                EngineCommand::SetReverbMix(mix) => fx.set_reverb_mix(mix),
                EngineCommand::SetUnisonVoices(voices) => *unison_voices = voices,
                EngineCommand::SetDetune(cents) => *detune_cents = cents,
                EngineCommand::SetOscMix(mix) => *osc_mix = mix,
                EngineCommand::SetOscBSemis(semis) => *osc_b_semis = semis,
                EngineCommand::SetAmpEnv { attack, decay, sustain, release } => {
                    *amp_env = [attack, decay, sustain, release];
                }
                EngineCommand::SetFilterEnv { attack, decay, sustain, release } => {
                    *filter_env = [attack, decay, sustain, release];
                }
                EngineCommand::SetCell { track, step, row, on } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.sequencer.set_cell(step as usize, row as usize, on);
                    }
                }
                EngineCommand::ClearPattern { track } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.sequencer.clear();
                    }
                }
                EngineCommand::AddNote { track, pitch, start_beats, len_beats, velocity } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.clip.add(pitch, start_beats, len_beats, velocity);
                    }
                }
                EngineCommand::RemoveNote { track, pitch, start_beats } => {
                    if let (Some(t), Some(events)) =
                        (tracks.get_mut(track as usize), track_events.get_mut(track as usize))
                    {
                        t.clip.remove(pitch, start_beats, events);
                    }
                }
                EngineCommand::ClearNotes { track } => {
                    if let (Some(t), Some(events)) =
                        (tracks.get_mut(track as usize), track_events.get_mut(track as usize))
                    {
                        t.clip.clear(events);
                    }
                }
                EngineCommand::SetTrackLevel { track, level } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.level = level;
                    }
                }
                EngineCommand::SetTrackPan { track, pan } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.pan = pan.clamp(-1.0, 1.0);
                    }
                }
                EngineCommand::SetTrackMute { track, on } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.muted = on;
                    }
                }
                EngineCommand::SetTrackSolo { track, on } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.soloed = on;
                    }
                }
            }
        }

        let rolling = transport.is_rolling();
        let beat = transport.beat_position();
        let snapshot = transport.snapshot();
        let any_solo = tracks.iter().any(|t| t.soloed);

        // Start the master from silence and sum each audible track into it
        output.fill(0.0);
        for (index, track) in tracks.iter_mut().enumerate() {
            let events = &mut track_events[index];
            if rolling {
                track.sequencer.advance_to_beat(beat, events);
                track.clip.advance_to_beat(beat, events);
            }
            track.node.set_unison(*unison_voices, *detune_cents);
            track.node.set_filter(*cutoff_hz, *resonance);
            track.node.set_osc_mix(*osc_mix);
            track.node.set_osc_b_semitones(*osc_b_semis);
            track.node.set_amp_env(amp_env[0], amp_env[1], amp_env[2], amp_env[3]);
            track.node.set_filter_env(filter_env[0], filter_env[1], filter_env[2], filter_env[3]);

            // Render the track into scratch, then free the borrow before summing
            {
                let mut ctx = ProcessContext::new(
                    frames,
                    *sample_rate_hz,
                    &[],
                    scratch,
                    events,
                    &[],
                    snapshot,
                );
                track.node.process(&mut ctx);
            }
            // Sum the audible contribution per channel, panned, capturing the peak
            let audible = !track.muted && (!any_solo || track.soloed);
            let mut track_peak = 0.0f32;
            if audible {
                let level = track.level;
                let pan = track.pan;
                for ch in 0..channels {
                    let gain = pan_gain(ch, channels, pan) * level;
                    let start = ch * frames;
                    let out_ch = &mut output[start..start + frames];
                    let src_ch = &scratch[start..start + frames];
                    for (out, &sample) in out_ch.iter_mut().zip(src_ch.iter()) {
                        let contribution = sample * gain;
                        *out += contribution;
                        track_peak = track_peak.max(contribution.abs());
                    }
                }
            }
            if let Some(meter) = sink.track_meters.get(index) {
                meter.store(track_peak);
            }
        }
        fx.process(output, frames);

        // Advance the transport past this block for the next one
        if rolling {
            transport.advance(frames as u64);
        }

        // Publish the transport position for the UI playhead
        sink.position.store(transport.beat_position());

        // Apply master gain, then publish this block's output peak
        let mut peak = 0.0f32;
        for sample in output.iter_mut() {
            *sample *= *gain;
            peak = peak.max(sample.abs());
        }
        sink.meter.store(peak);

        // Publish channel 0 (channel-major) to the scope
        sink.push_scope(&output[..frames]);
    }
}

// Live engine handle: keeps the backend and stream alive while audio runs
pub struct Engine {
    // Held so the device stays open for the stream's lifetime
    _backend: CpalBackend,
    stream: Box<dyn Stream>,
    sample_rate_hz: u32,
    channels: u16,
}

impl Engine {
    // Wrap the running backend and stream
    pub fn new(backend: CpalBackend, stream: Box<dyn Stream>, sample_rate_hz: u32, channels: u16) -> Self {
        Self {
            _backend: backend,
            stream,
            sample_rate_hz,
            channels,
        }
    }

    // Buffer xruns observed since the stream started
    pub fn xruns(&self) -> u64 {
        self.stream.xruns()
    }

    // Active stream sample rate
    pub fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    // Active output channel count
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::control_plane;
    use geist_core::config::AudioConfig;
    use geist_core::events::NoteEventKind;

    fn config(sample_rate_hz: u32, block: u32, channels: u16) -> AudioConfig {
        AudioConfig::new(sample_rate_hz, block, 0, channels).unwrap()
    }

    // Build a stopped/rolling multi-track processor for the given stream shape
    fn processor(rolling: bool) -> (crate::control::EngineControl, SynthProcessor, usize) {
        let sample_rate_hz = 48_000;
        let block = 256u32;
        let channels = 2u16;
        let block_len = channels as usize * block as usize;
        let cfg = config(sample_rate_hz, block, channels);
        let mut tracks = Vec::new();
        for index in 0..NUM_TRACKS {
            let mut track =
                Track::new(sample_rate_hz, 8, TRACK_BASE_MIDI[index], default_grid_for(index));
            track.prepare(&cfg);
            tracks.push(track);
        }
        let (control, sink) = control_plane(NUM_TRACKS);
        let fx = FxChain::new(channels as usize, block as usize, sample_rate_hz);
        let proc =
            SynthProcessor::new(tracks, sample_rate_hz, block_len, sink, fx, rolling, DEFAULT_BPM);
        (control, proc, block_len)
    }

    #[test]
    fn default_pattern_triggers_on_the_first_column() {
        let mut seq = Sequencer::new(SEQ_BASE_MIDI, default_grid());
        let mut events = Vec::with_capacity(MAX_BLOCK_EVENTS);
        seq.advance_to_beat(0.0, &mut events);
        assert!(
            events.iter().any(|e| e.kind == NoteEventKind::On),
            "first column should trigger a note"
        );
    }

    #[test]
    fn toggling_a_cell_plays_that_note() {
        let mut seq = Sequencer::new(SEQ_BASE_MIDI, default_grid());
        seq.clear();
        seq.set_cell(0, 5, true); // column 0, row 5
        let mut events = Vec::with_capacity(MAX_BLOCK_EVENTS);
        seq.advance_to_beat(0.0, &mut events);
        let ons: Vec<_> = events
            .iter()
            .filter(|e| e.kind == NoteEventKind::On)
            .collect();
        assert_eq!(ons.len(), 1);
        assert_eq!(ons[0].key, SEQ_BASE_MIDI + 5);
    }

    #[test]
    fn crossing_columns_releases_then_triggers() {
        let mut seq = Sequencer::new(SEQ_BASE_MIDI, default_grid());
        seq.clear();
        seq.set_cell(0, 3, true);
        seq.set_cell(1, 7, true);
        let mut events = Vec::with_capacity(MAX_BLOCK_EVENTS);
        seq.advance_to_beat(0.0, &mut events); // column 0: on row 3
        events.clear();
        // STEPS_PER_BEAT is 4, so beat 0.25 enters column 1
        seq.advance_to_beat(0.25, &mut events);
        assert!(events
            .iter()
            .any(|e| e.kind == NoteEventKind::Off && e.key == SEQ_BASE_MIDI + 3));
        assert!(events
            .iter()
            .any(|e| e.kind == NoteEventKind::On && e.key == SEQ_BASE_MIDI + 7));
    }

    #[test]
    fn advance_reuses_capacity_without_allocating() {
        let mut seq = Sequencer::new(SEQ_BASE_MIDI, default_grid());
        let mut events = Vec::with_capacity(MAX_BLOCK_EVENTS);
        let cap = events.capacity();
        let mut beat = 0.0;
        for _ in 0..64 {
            events.clear();
            seq.advance_to_beat(beat, &mut events);
            beat += 0.25;
        }
        assert_eq!(events.capacity(), cap, "sequencer must not grow the buffer");
    }

    #[test]
    fn rolling_transport_renders_audio() {
        let (_control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        let mut energy = 0.0f32;
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
            energy += output.iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(energy > 0.01, "rolling transport produced no audio");
    }

    #[test]
    fn rolling_transport_advances_published_position() {
        let (control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        assert_eq!(control.position_beats(), 0.0);
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(control.position_beats() > 0.0, "playhead position did not advance");
    }

    #[test]
    fn play_command_starts_the_sequencer() {
        let (mut control, mut proc, len) = processor(false);
        let mut output = vec![0.0f32; len];
        for _ in 0..4 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(output.iter().all(|&s| s == 0.0), "stopped transport should be silent");

        control.send(EngineCommand::SetPlaying(true));
        let mut energy = 0.0f32;
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
            energy += output.iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(energy > 0.01, "play command did not start the sequencer");
    }

    #[test]
    fn note_on_command_drives_the_synth_and_meter() {
        let (mut control, mut proc, len) = processor(false);
        let mut output = vec![0.0f32; len];
        proc.process_block(&[], &mut output, 2, len / 2);
        assert!(output.iter().all(|&s| s == 0.0), "expected silence before any note");

        control.send(EngineCommand::NoteOn { track: 0, key: 69, velocity: 1.0 });
        let mut energy = 0.0f32;
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
            energy += output.iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(energy > 0.01, "note-on command produced no audio");
        assert!(control.level() > 0.0, "meter did not register output");
    }

    #[test]
    fn clip_triggers_and_releases_across_the_loop() {
        let mut clip = NoteClip::new(4.0);
        clip.add(60, 0.0, 1.0, 1.0);
        let mut out = Vec::with_capacity(MAX_BLOCK_EVENTS);
        clip.advance_to_beat(0.0, &mut out);
        assert!(out.iter().any(|e| e.kind == NoteEventKind::On && e.key == 60));
        out.clear();
        // Phase past the note's end releases it
        clip.advance_to_beat(2.0, &mut out);
        assert!(out.iter().any(|e| e.kind == NoteEventKind::Off && e.key == 60));
        out.clear();
        // Looping back into the note retriggers it
        clip.advance_to_beat(4.0, &mut out);
        assert!(out.iter().any(|e| e.kind == NoteEventKind::On && e.key == 60));
    }

    #[test]
    fn clip_remove_releases_a_sounding_note() {
        let mut clip = NoteClip::new(4.0);
        clip.add(64, 0.0, 2.0, 1.0);
        let mut out = Vec::with_capacity(MAX_BLOCK_EVENTS);
        clip.advance_to_beat(0.0, &mut out);
        out.clear();
        clip.remove(64, 0.0, &mut out);
        assert!(out.iter().any(|e| e.kind == NoteEventKind::Off && e.key == 64));
    }

    #[test]
    fn clip_add_stays_within_capacity() {
        let mut clip = NoteClip::new(16.0);
        let cap = clip.notes.capacity();
        for i in 0..(MAX_CLIP_NOTES + 10) {
            clip.add(60, i as f32 * 0.01, 0.1, 1.0);
        }
        assert_eq!(clip.notes.len(), MAX_CLIP_NOTES);
        assert_eq!(clip.notes.capacity(), cap, "clip must not grow its buffer");
    }

    #[test]
    fn add_note_command_sounds_on_an_empty_track() {
        // Track 0 carries no step pattern, so any sound proves the clip played
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        control.send(EngineCommand::AddNote {
            track: 0,
            pitch: 60,
            start_beats: 0.0,
            len_beats: 4.0,
            velocity: 1.0,
        });
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(control.track_level(0) > 0.0, "clip note did not sound on track 0");
    }

    #[test]
    fn processor_does_not_grow_under_command_traffic() {
        // The audio thread must be allocation-free: internal buffer capacities
        // stay put and per-track clips never exceed their fixed cap, regardless
        // of how much command churn the UI produces.
        let (mut control, mut proc, len) = processor(true);
        let frames = len / 2;
        let mut output = vec![0.0f32; len];

        // Warm up so buffers reach their working size
        proc.process_block(&[], &mut output, 2, frames);
        let event_caps: Vec<usize> = proc.track_events.iter().map(|e| e.capacity()).collect();
        let scratch_cap = proc.scratch.capacity();
        let clip_caps: Vec<usize> = proc.tracks.iter().map(|t| t.clip.notes.capacity()).collect();

        for i in 0..4000u32 {
            let key = 60 + (i % 12) as u8;
            let beat = (i % 16) as f32;
            control.send(EngineCommand::NoteOn { track: 0, key, velocity: 1.0 });
            control.send(EngineCommand::NoteOff { track: 0, key });
            control.send(EngineCommand::AddNote {
                track: 2,
                pitch: key,
                start_beats: beat,
                len_beats: 1.0,
                velocity: 1.0,
            });
            control.send(EngineCommand::RemoveNote { track: 2, pitch: key, start_beats: beat });
            control.send(EngineCommand::SetCutoff(400.0 + (i % 800) as f32));
            control.send(EngineCommand::SetBpm(100.0 + (i % 60) as f32));
            proc.process_block(&[], &mut output, 2, frames);
        }

        for (events, &cap) in proc.track_events.iter().zip(event_caps.iter()) {
            assert_eq!(events.capacity(), cap, "track_events buffer grew");
        }
        assert_eq!(proc.scratch.capacity(), scratch_cap, "scratch buffer grew");
        for (track, &cap) in proc.tracks.iter().zip(clip_caps.iter()) {
            assert!(track.clip.notes.len() <= MAX_CLIP_NOTES, "clip note count unbounded");
            assert_eq!(track.clip.notes.capacity(), cap, "clip buffer grew");
        }
    }

    #[test]
    fn per_track_meter_follows_the_sounding_track() {
        // Only track 1 carries the seeded pattern
        let (control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(control.track_level(1) > 0.0, "seeded track should meter");
        assert_eq!(control.track_level(0), 0.0, "silent track should read zero");
    }

    #[test]
    fn track_pan_steers_energy_between_channels() {
        // Track 1 carries the riff; pan it hard left and the right should empty
        let (mut control, mut proc, len) = processor(true);
        let frames = len / 2;
        let mut output = vec![0.0f32; len];
        control.send(EngineCommand::SetTrackPan { track: 1, pan: -1.0 });
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, frames);
            left += output[..frames].iter().map(|s| s.abs()).sum::<f32>();
            right += output[frames..].iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(left > 0.0, "left channel should carry the panned track");
        assert!(right < left * 0.01, "hard-left pan leaked right: {right} vs {left}");
    }

    #[test]
    fn gain_command_scales_output() {
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(output.iter().any(|&s| s != 0.0), "transport should be sounding");

        control.send(EngineCommand::SetGain(0.0));
        for _ in 0..2 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(output.iter().all(|&s| s == 0.0), "gain=0 did not silence output");
    }

    #[test]
    fn muting_the_playing_track_silences_it() {
        // Only track 1 carries the seeded pattern
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(output.iter().any(|&s| s != 0.0), "track 1 should be sounding");

        control.send(EngineCommand::SetTrackMute { track: 1, on: true });
        for _ in 0..2 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(output.iter().all(|&s| s == 0.0), "muted track still audible");
    }

    #[test]
    fn soloing_an_empty_track_silences_the_mix() {
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(output.iter().any(|&s| s != 0.0), "track 1 should be sounding");

        // Track 0 has no pattern; soloing it mutes everything else
        control.send(EngineCommand::SetTrackSolo { track: 0, on: true });
        for _ in 0..2 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(output.iter().all(|&s| s == 0.0), "solo did not isolate the empty track");
    }
}
