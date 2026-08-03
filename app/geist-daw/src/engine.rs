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

use std::sync::Arc;

use geist_audio_backend::prelude::{BlockProcessor, CpalBackend, Stream};
use spectre_core::context::ProcessContext;
use spectre_core::events::NoteEvent;
use spectre_core::transport::TransportSnapshot;
use geist_graph::node::AudioNode;
use geist_synth::prelude::SynthNode;
use geist_timeline::prelude::Transport;

use crate::control::{EngineCommand, EngineSink};
use crate::fx::FxChain;

// Most audio assets (recorded buffers) the engine holds at once. The store is
// pre-sized so registering an asset never allocates on the audio thread.
pub const MAX_AUDIO_ASSETS: usize = 64;

// A recorded audio buffer resident in the engine, shared with the recorder that
// produced it. Interleaved by `channels`.
#[derive(Clone)]
pub struct StoredAsset {
    pub samples: Arc<[f32]>,
    pub channels: u16,
}

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
                push_capped(
                    out,
                    NoteEvent::on(0, 0, self.base_midi + row as u8, STEP_VELOCITY),
                );
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
// Most placed clips one track's arrangement can hold; fixed for realtime safety
pub const MAX_CLIPS_PER_TRACK: usize = 64;
// Start-beat tolerance when matching a note for removal
const START_EPS: f32 = 1e-3;

// One timed note inside a clip, positioned relative to the clip start, with its
// current sounding state
#[derive(Copy, Clone)]
struct ClipNote {
    pitch: u8,
    start_beats: f32,
    len_beats: f32,
    velocity: f32,
    sounding: bool,
}

// One MIDI clip placed on the timeline. Notes are relative to the clip start and
// play once across the clip's span at absolute transport position.
struct ArrClip {
    id: u64,
    start_beats: f32,
    len_beats: f32,
    notes: Vec<ClipNote>,
}

impl ArrClip {
    // Empty clip; note capacity is fixed for realtime safety
    fn new(id: u64, start_beats: f32, len_beats: f32) -> Self {
        Self {
            id,
            start_beats,
            len_beats,
            notes: Vec::with_capacity(MAX_CLIP_NOTES),
        }
    }

    // Release every sounding note in this clip
    fn release(&mut self, out: &mut Vec<NoteEvent>) {
        for note in &mut self.notes {
            if note.sounding {
                push_capped(out, NoteEvent::off(0, 0, note.pitch));
                note.sounding = false;
            }
        }
    }
}

// One placed audio clip: a window into a stored asset, positioned on the
// timeline. `slot` indexes the engine's asset store, which carries the channel
// layout and sample length.
struct AudioArrClip {
    id: u64,
    start_beats: f32,
    len_beats: f32,
    slot: usize,
}

// A track's timeline: placed MIDI clips advanced by absolute transport beat plus
// placed audio clips mixed from the asset store. Level-based and positioned: a
// note sounds when the transport is inside its clip's span and the note's span.
// Robust to seeking, looping, and tempo changes.
pub struct Arrangement {
    clips: Vec<ArrClip>,
    audio: Vec<AudioArrClip>,
}

impl Arrangement {
    // Empty arrangement; clip capacity is fixed for realtime safety
    pub fn new() -> Self {
        Self {
            clips: Vec::with_capacity(MAX_CLIPS_PER_TRACK),
            audio: Vec::with_capacity(MAX_CLIPS_PER_TRACK),
        }
    }

    // Place an audio clip referencing an asset slot, if room remains
    fn add_audio_clip(&mut self, id: u64, start_beats: f32, len_beats: f32, slot: usize) {
        if self.audio.len() < MAX_CLIPS_PER_TRACK && !self.audio.iter().any(|c| c.id == id) {
            self.audio.push(AudioArrClip {
                id,
                start_beats,
                len_beats,
                slot,
            });
        }
    }

    // Mix every audio clip overlapping this block into the channel-major scratch.
    // Block-accurate: the clip's read offset is derived from the block-start beat.
    fn mix_audio(
        &self,
        snapshot: &TransportSnapshot,
        beat: f64,
        scratch: &mut [f32],
        frames: usize,
        channels: usize,
        assets: &[Option<StoredAsset>],
    ) {
        for clip in &self.audio {
            if clip.len_beats <= 0.0 {
                continue;
            }
            let start = clip.start_beats as f64;
            if beat < start || beat >= start + clip.len_beats as f64 {
                continue;
            }
            let Some(Some(asset)) = assets.get(clip.slot) else {
                continue;
            };
            let src_channels = (asset.channels as usize).max(1);
            let asset_frames = asset.samples.len() / src_channels;
            let offset = snapshot.beats_to_samples(beat - start) as i64;
            for f in 0..frames {
                let src_frame = offset + f as i64;
                if src_frame < 0 || src_frame as usize >= asset_frames {
                    continue;
                }
                let base = src_frame as usize * src_channels;
                for ch in 0..channels {
                    let src_ch = ch.min(src_channels - 1);
                    if let Some(&sample) = asset.samples.get(base + src_ch) {
                        scratch[ch * frames + f] += sample;
                    }
                }
            }
        }
    }

    // Add a placed clip unless the arrangement is full or the id already exists
    fn add_clip(&mut self, id: u64, start_beats: f32, len_beats: f32) {
        if self.clips.len() < MAX_CLIPS_PER_TRACK && !self.clips.iter().any(|c| c.id == id) {
            self.clips.push(ArrClip::new(id, start_beats, len_beats));
        }
    }

    // Move a clip's start position
    fn move_clip(&mut self, id: u64, start_beats: f32) {
        let start_beats = start_beats.max(0.0);
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            clip.start_beats = start_beats;
        }
        if let Some(clip) = self.audio.iter_mut().find(|c| c.id == id) {
            clip.start_beats = start_beats;
        }
    }

    // Resize a clip's length
    fn resize_clip(&mut self, id: u64, len_beats: f32) {
        let len_beats = len_beats.max(0.0);
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            clip.len_beats = len_beats;
        }
        if let Some(clip) = self.audio.iter_mut().find(|c| c.id == id) {
            clip.len_beats = len_beats;
        }
    }

    // Remove a clip by id (MIDI or audio), releasing any sounding notes
    fn remove_clip(&mut self, id: u64, out: &mut Vec<NoteEvent>) {
        if let Some(index) = self.clips.iter().position(|c| c.id == id) {
            self.clips[index].release(out);
            self.clips.swap_remove(index);
        }
        if let Some(index) = self.audio.iter().position(|c| c.id == id) {
            self.audio.swap_remove(index);
        }
    }

    // Add a note (relative to the clip start) to a clip, if room remains
    fn add_note(&mut self, id: u64, pitch: u8, start_beats: f32, len_beats: f32, velocity: f32) {
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            if clip.notes.len() < MAX_CLIP_NOTES {
                clip.notes.push(ClipNote {
                    pitch,
                    start_beats,
                    len_beats,
                    velocity,
                    sounding: false,
                });
            }
        }
    }

    // Remove the first note matching pitch+start within a clip, releasing it if sounding
    fn remove_note(&mut self, id: u64, pitch: u8, start_beats: f32, out: &mut Vec<NoteEvent>) {
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            if let Some(index) = clip
                .notes
                .iter()
                .position(|n| n.pitch == pitch && (n.start_beats - start_beats).abs() < START_EPS)
            {
                if clip.notes[index].sounding {
                    push_capped(out, NoteEvent::off(0, 0, pitch));
                }
                clip.notes.swap_remove(index);
            }
        }
    }

    // Drop every note in a clip, releasing any that are sounding
    fn clear_clip(&mut self, id: u64, out: &mut Vec<NoteEvent>) {
        if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
            clip.release(out);
            clip.notes.clear();
        }
    }

    // Trigger/release notes so the sounding set matches the absolute beat
    pub fn advance(&mut self, beat: f64, out: &mut Vec<NoteEvent>) {
        for clip in &mut self.clips {
            if clip.len_beats <= 0.0 {
                continue;
            }
            let start = clip.start_beats as f64;
            let within = beat >= start && beat < start + clip.len_beats as f64;
            let local = (beat - start) as f32;
            for note in &mut clip.notes {
                let should = within
                    && local >= note.start_beats
                    && local < note.start_beats + note.len_beats;
                if should && !note.sounding {
                    push_capped(out, NoteEvent::on(0, 0, note.pitch, note.velocity));
                    note.sounding = true;
                } else if !should && note.sounding {
                    push_capped(out, NoteEvent::off(0, 0, note.pitch));
                    note.sounding = false;
                }
            }
        }
    }

    // Release every sounding note across all clips, e.g. when the transport stops
    pub fn release(&mut self, out: &mut Vec<NoteEvent>) {
        for clip in &mut self.clips {
            clip.release(out);
        }
    }
}

impl Default for Arrangement {
    fn default() -> Self {
        Self::new()
    }
}

// A track's instrument patch: every synth macro is now per-track, so each track
// has an independent sound. Applied to the track's SynthNode each block.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Patch {
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub unison_voices: usize,
    pub detune_cents: f32,
    pub osc_mix: f32,
    pub osc_b_semis: f32,
    // [attack, decay, sustain, release]
    pub amp_env: [f32; 4],
    pub filter_env: [f32; 4],
}

impl Default for Patch {
    // Musical defaults matching the synth's startup voice
    fn default() -> Self {
        Self {
            cutoff_hz: DEFAULT_CUTOFF_HZ,
            resonance: DEFAULT_RESONANCE,
            unison_voices: DEFAULT_UNISON_VOICES,
            detune_cents: DEFAULT_DETUNE_CENTS,
            osc_mix: DEFAULT_OSC_MIX,
            osc_b_semis: DEFAULT_OSC_B_SEMIS,
            amp_env: DEFAULT_AMP_ENV,
            filter_env: DEFAULT_FILTER_ENV,
        }
    }
}

// One mixer track: an instrument, its patch, its own effects chain, its step
// pattern, its note clip, and mix state
pub struct Track {
    node: SynthNode,
    patch: Patch,
    fx: FxChain,
    sequencer: Sequencer,
    arrangement: Arrangement,
    level: f32,
    pan: f32,
    muted: bool,
    soloed: bool,
}

impl Track {
    // Build a track with its base note and seed pattern; node + fx still need
    // prepare. The per-track effects chain is sized for the stream block here.
    pub fn new(
        sample_rate_hz: u32,
        polyphony: usize,
        base_midi: u8,
        grid: Grid,
        channels: usize,
        block_frames: usize,
    ) -> Self {
        Self {
            node: SynthNode::new(sample_rate_hz as f32, polyphony),
            patch: Patch::default(),
            fx: FxChain::new(channels, block_frames, sample_rate_hz),
            sequencer: Sequencer::new(base_midi, grid),
            arrangement: Arrangement::new(),
            level: DEFAULT_TRACK_LEVEL,
            pan: 0.0,
            muted: false,
            soloed: false,
        }
    }

    // Prepare the track's instrument and effects chain for the stream
    pub fn prepare(&mut self, config: &spectre_core::config::AudioConfig) {
        self.node.prepare(config);
        self.fx.prepare(config);
    }

    // Push the patch macros into the instrument node for this block
    fn apply_patch(&mut self) {
        self.node
            .set_unison(self.patch.unison_voices, self.patch.detune_cents);
        self.node
            .set_filter(self.patch.cutoff_hz, self.patch.resonance);
        self.node.set_osc_mix(self.patch.osc_mix);
        self.node.set_osc_b_semitones(self.patch.osc_b_semis);
        let amp = self.patch.amp_env;
        self.node.set_amp_env(amp[0], amp[1], amp[2], amp[3]);
        let flt = self.patch.filter_env;
        self.node.set_filter_env(flt[0], flt[1], flt[2], flt[3]);
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
    // Master output gain applied post-mix
    gain: f32,
    // Per-track note events, preallocated so process_block never allocates
    track_events: Vec<Vec<NoteEvent>>,
    // One track's rendered block, summed into the master with its level
    scratch: Vec<f32>,
    // Recorded audio buffers, indexed by slot; pre-sized to avoid audio-thread alloc
    audio_assets: Vec<Option<StoredAsset>>,
}

impl SynthProcessor {
    // Assemble the processor; each track owns its prepared instrument and fx
    pub fn new(
        tracks: Vec<Track>,
        sample_rate_hz: u32,
        block_len: usize,
        sink: EngineSink,
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
            gain: DEFAULT_GAIN,
            track_events,
            scratch: vec![0.0; block_len],
            audio_assets: (0..MAX_AUDIO_ASSETS).map(|_| None).collect(),
        }
    }
}

impl BlockProcessor for SynthProcessor {
    // Drain commands, mix every track, run master fx, and publish the peak
    fn process_block(
        &mut self,
        _input: &[f32],
        output: &mut [f32],
        channels: usize,
        frames: usize,
    ) {
        let Self {
            tracks,
            sample_rate_hz,
            sink,
            transport,
            gain,
            track_events,
            scratch,
            audio_assets,
        } = self;

        for events in track_events.iter_mut() {
            events.clear();
        }

        // Move any newly recorded buffers into the asset store (no allocation:
        // the Arc is built on the UI thread and only its pointer is stored here)
        while let Ok(asset) = sink.assets.pop() {
            if let Some(slot) = audio_assets.get_mut(asset.slot) {
                *slot = Some(StoredAsset {
                    samples: asset.samples,
                    channels: asset.channels,
                });
            }
        }

        // Translate queued UI commands into per-track events, transport, and macros
        while let Ok(command) = sink.commands.pop() {
            match command {
                EngineCommand::NoteOn {
                    track,
                    key,
                    velocity,
                } => {
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
                            track.arrangement.release(events);
                        }
                    }
                }
                EngineCommand::SetBpm(bpm) => {
                    transport.tempo_map_mut().set_tempo(0.0, bpm as f64);
                }
                EngineCommand::SetCutoff { track, hz } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.cutoff_hz = hz;
                    }
                }
                EngineCommand::SetResonance { track, resonance } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.resonance = resonance;
                    }
                }
                EngineCommand::SetGain(value) => *gain = value,
                EngineCommand::SetDelay { track, on } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.fx.set_delay(on);
                    }
                }
                EngineCommand::SetDelayTime { track, seconds } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.fx.set_delay_time(seconds);
                    }
                }
                EngineCommand::SetDelayFeedback { track, feedback } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.fx.set_delay_feedback(feedback);
                    }
                }
                EngineCommand::SetDelayMix { track, mix } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.fx.set_delay_mix(mix);
                    }
                }
                EngineCommand::SetReverb { track, on } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.fx.set_reverb(on);
                    }
                }
                EngineCommand::SetReverbMix { track, mix } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.fx.set_reverb_mix(mix);
                    }
                }
                EngineCommand::SetUnisonVoices { track, voices } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.unison_voices = voices;
                    }
                }
                EngineCommand::SetDetune { track, cents } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.detune_cents = cents;
                    }
                }
                EngineCommand::SetOscMix { track, mix } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.osc_mix = mix;
                    }
                }
                EngineCommand::SetOscBSemis { track, semis } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.osc_b_semis = semis;
                    }
                }
                EngineCommand::SetAmpEnv {
                    track,
                    attack,
                    decay,
                    sustain,
                    release,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.amp_env = [attack, decay, sustain, release];
                    }
                }
                EngineCommand::SetFilterEnv {
                    track,
                    attack,
                    decay,
                    sustain,
                    release,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.patch.filter_env = [attack, decay, sustain, release];
                    }
                }
                EngineCommand::SetCell {
                    track,
                    step,
                    row,
                    on,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.sequencer.set_cell(step as usize, row as usize, on);
                    }
                }
                EngineCommand::ClearPattern { track } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.sequencer.clear();
                    }
                }
                EngineCommand::AddClip {
                    track,
                    id,
                    start_beats,
                    len_beats,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.arrangement.add_clip(id, start_beats, len_beats);
                    }
                }
                EngineCommand::MoveClip {
                    track,
                    id,
                    start_beats,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.arrangement.move_clip(id, start_beats);
                    }
                }
                EngineCommand::ResizeClip {
                    track,
                    id,
                    len_beats,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.arrangement.resize_clip(id, len_beats);
                    }
                }
                EngineCommand::RemoveClip { track, id } => {
                    if let (Some(t), Some(events)) = (
                        tracks.get_mut(track as usize),
                        track_events.get_mut(track as usize),
                    ) {
                        t.arrangement.remove_clip(id, events);
                    }
                }
                EngineCommand::AddClipNote {
                    track,
                    clip,
                    pitch,
                    start_beats,
                    len_beats,
                    velocity,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.arrangement
                            .add_note(clip, pitch, start_beats, len_beats, velocity);
                    }
                }
                EngineCommand::RemoveClipNote {
                    track,
                    clip,
                    pitch,
                    start_beats,
                } => {
                    if let (Some(t), Some(events)) = (
                        tracks.get_mut(track as usize),
                        track_events.get_mut(track as usize),
                    ) {
                        t.arrangement.remove_note(clip, pitch, start_beats, events);
                    }
                }
                EngineCommand::ClearClip { track, clip } => {
                    if let (Some(t), Some(events)) = (
                        tracks.get_mut(track as usize),
                        track_events.get_mut(track as usize),
                    ) {
                        t.arrangement.clear_clip(clip, events);
                    }
                }
                EngineCommand::AddAudioClip {
                    track,
                    id,
                    start_beats,
                    len_beats,
                    slot,
                } => {
                    if let Some(t) = tracks.get_mut(track as usize) {
                        t.arrangement
                            .add_audio_clip(id, start_beats, len_beats, slot);
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
                track.arrangement.advance(beat, events);
            }
            track.apply_patch();

            // Render the track into scratch, then run its own effects chain in
            // place, then free the borrow before summing
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
            // Mix any audio clips on this track over the synth output, then fx
            if rolling {
                track.arrangement.mix_audio(
                    &snapshot,
                    beat,
                    scratch,
                    frames,
                    channels,
                    audio_assets,
                );
            }
            track.fx.process(scratch, frames);
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

// Live engine handle: keeps the backend and streams alive while audio runs
pub struct Engine {
    // Held so the device stays open for the stream's lifetime
    _backend: CpalBackend,
    stream: Box<dyn Stream>,
    // Held so the capture device stays open while recording is possible
    _input: Option<Box<dyn Stream>>,
    sample_rate_hz: u32,
    channels: u16,
}

impl Engine {
    // Wrap the running backend, output stream, and optional input stream
    pub fn new(
        backend: CpalBackend,
        stream: Box<dyn Stream>,
        input: Option<Box<dyn Stream>>,
        sample_rate_hz: u32,
        channels: u16,
    ) -> Self {
        Self {
            _backend: backend,
            stream,
            _input: input,
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
    use spectre_core::config::AudioConfig;
    use spectre_core::events::NoteEventKind;

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
            let mut track = Track::new(
                sample_rate_hz,
                8,
                TRACK_BASE_MIDI[index],
                default_grid_for(index),
                channels as usize,
                block as usize,
            );
            track.prepare(&cfg);
            tracks.push(track);
        }
        let (control, sink) = control_plane(NUM_TRACKS);
        let proc = SynthProcessor::new(
            tracks,
            sample_rate_hz,
            block_len,
            sink,
            rolling,
            DEFAULT_BPM,
        );
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
        assert!(
            control.position_beats() > 0.0,
            "playhead position did not advance"
        );
    }

    #[test]
    fn play_command_starts_the_sequencer() {
        let (mut control, mut proc, len) = processor(false);
        let mut output = vec![0.0f32; len];
        for _ in 0..4 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            output.iter().all(|&s| s == 0.0),
            "stopped transport should be silent"
        );

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
        assert!(
            output.iter().all(|&s| s == 0.0),
            "expected silence before any note"
        );

        control.send(EngineCommand::NoteOn {
            track: 0,
            key: 69,
            velocity: 1.0,
        });
        let mut energy = 0.0f32;
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
            energy += output.iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(energy > 0.01, "note-on command produced no audio");
        assert!(control.level() > 0.0, "meter did not register output");
    }

    #[test]
    fn placed_clip_triggers_only_inside_its_span() {
        // A clip placed at beat 8 holding a note at local 0 must be silent before
        // beat 8 and trigger once the transport reaches it.
        let mut arr = Arrangement::new();
        arr.add_clip(1, 8.0, 4.0);
        arr.add_note(1, 60, 0.0, 1.0, 1.0);
        let mut out = Vec::with_capacity(MAX_BLOCK_EVENTS);
        arr.advance(0.0, &mut out);
        assert!(out.is_empty(), "clip sounded before its start beat");
        arr.advance(8.0, &mut out);
        assert!(out
            .iter()
            .any(|e| e.kind == NoteEventKind::On && e.key == 60));
        out.clear();
        // Past the note's local end it releases
        arr.advance(9.5, &mut out);
        assert!(out
            .iter()
            .any(|e| e.kind == NoteEventKind::Off && e.key == 60));
    }

    #[test]
    fn moving_a_clip_shifts_its_trigger_beat() {
        let mut arr = Arrangement::new();
        arr.add_clip(1, 0.0, 4.0);
        arr.add_note(1, 64, 0.0, 1.0, 1.0);
        let mut out = Vec::with_capacity(MAX_BLOCK_EVENTS);
        // At beat 0 it triggers; move it to beat 16 and beat 0 goes silent
        arr.advance(0.0, &mut out);
        assert!(out
            .iter()
            .any(|e| e.kind == NoteEventKind::On && e.key == 64));
        out.clear();
        arr.release(&mut out);
        out.clear();
        arr.move_clip(1, 16.0);
        arr.advance(0.0, &mut out);
        assert!(
            out.is_empty(),
            "moved clip still triggered at the old position"
        );
        arr.advance(16.0, &mut out);
        assert!(out
            .iter()
            .any(|e| e.kind == NoteEventKind::On && e.key == 64));
    }

    #[test]
    fn two_clips_on_one_track_both_play() {
        let mut arr = Arrangement::new();
        arr.add_clip(1, 0.0, 4.0);
        arr.add_note(1, 60, 0.0, 1.0, 1.0);
        arr.add_clip(2, 8.0, 4.0);
        arr.add_note(2, 67, 0.0, 1.0, 1.0);
        let mut out = Vec::with_capacity(MAX_BLOCK_EVENTS);
        arr.advance(0.0, &mut out);
        assert!(out
            .iter()
            .any(|e| e.kind == NoteEventKind::On && e.key == 60));
        out.clear();
        arr.advance(8.0, &mut out);
        assert!(out
            .iter()
            .any(|e| e.kind == NoteEventKind::On && e.key == 67));
    }

    #[test]
    fn arrangement_stays_within_capacity() {
        let mut arr = Arrangement::new();
        let clip_cap = arr.clips.capacity();
        // Overfill clips
        for id in 0..(MAX_CLIPS_PER_TRACK as u64 + 10) {
            arr.add_clip(id, 0.0, 4.0);
        }
        assert_eq!(arr.clips.len(), MAX_CLIPS_PER_TRACK);
        assert_eq!(arr.clips.capacity(), clip_cap, "clip list grew its buffer");
        // Overfill one clip's notes
        let note_cap = arr.clips[0].notes.capacity();
        for i in 0..(MAX_CLIP_NOTES + 10) {
            arr.add_note(0, 60, i as f32 * 0.01, 0.1, 1.0);
        }
        assert_eq!(arr.clips[0].notes.len(), MAX_CLIP_NOTES);
        assert_eq!(
            arr.clips[0].notes.capacity(),
            note_cap,
            "clip notes grew the buffer"
        );
    }

    #[test]
    fn audio_clip_mixes_its_samples_at_position() {
        // A mono ramp asset placed at beat 0 should land verbatim in scratch
        let mut arr = Arrangement::new();
        let samples: Arc<[f32]> = (0..8).map(|i| i as f32).collect::<Vec<_>>().into();
        let assets = vec![Some(StoredAsset {
            samples,
            channels: 1,
        })];
        arr.add_audio_clip(1, 0.0, 4.0, 0);
        let snap = TransportSnapshot::stopped(48_000);
        let mut scratch = vec![0.0f32; 4];
        arr.mix_audio(&snap, 0.0, &mut scratch, 4, 1, &assets);
        assert_eq!(scratch, vec![0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn audio_clip_is_silent_outside_its_span() {
        let mut arr = Arrangement::new();
        let samples: Arc<[f32]> = vec![1.0f32; 8].into();
        let assets = vec![Some(StoredAsset {
            samples,
            channels: 1,
        })];
        arr.add_audio_clip(1, 0.0, 1.0, 0);
        let snap = TransportSnapshot::stopped(48_000);
        let mut scratch = vec![0.0f32; 4];
        // Beat well past the clip's one-beat span mixes nothing
        arr.mix_audio(&snap, 10.0, &mut scratch, 4, 1, &assets);
        assert!(scratch.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn audio_clip_move_and_resize_follow_arrangement_edits() {
        let mut arr = Arrangement::new();
        arr.add_audio_clip(7, 1.0, 2.0, 0);

        arr.move_clip(7, 4.0);
        arr.resize_clip(7, 8.0);

        assert_eq!(arr.audio[0].start_beats, 4.0);
        assert_eq!(arr.audio[0].len_beats, 8.0);
    }

    #[test]
    fn clip_command_sounds_on_an_empty_track() {
        // Track 0 carries no step pattern, so any sound proves the clip played
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        control.send(EngineCommand::AddClip {
            track: 0,
            id: 1,
            start_beats: 0.0,
            len_beats: 4.0,
        });
        control.send(EngineCommand::AddClipNote {
            track: 0,
            clip: 1,
            pitch: 60,
            start_beats: 0.0,
            len_beats: 4.0,
            velocity: 1.0,
        });
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            control.track_level(0) > 0.0,
            "clip note did not sound on track 0"
        );
    }

    #[test]
    fn processor_does_not_grow_under_command_traffic() {
        // The audio thread must be allocation-free: internal buffer capacities
        // stay put and per-track clips never exceed their fixed cap, regardless
        // of how much command churn the UI produces.
        let (mut control, mut proc, len) = processor(true);
        let frames = len / 2;
        let mut output = vec![0.0f32; len];

        // A placed clip on track 2 receives the note churn below
        control.send(EngineCommand::AddClip {
            track: 2,
            id: 1,
            start_beats: 0.0,
            len_beats: 16.0,
        });

        // Warm up so buffers reach their working size
        proc.process_block(&[], &mut output, 2, frames);
        let event_caps: Vec<usize> = proc.track_events.iter().map(|e| e.capacity()).collect();
        let scratch_cap = proc.scratch.capacity();
        let clip_list_caps: Vec<usize> = proc
            .tracks
            .iter()
            .map(|t| t.arrangement.clips.capacity())
            .collect();
        let note_cap = proc.tracks[2].arrangement.clips[0].notes.capacity();

        for i in 0..4000u32 {
            let key = 60 + (i % 12) as u8;
            let beat = (i % 16) as f32;
            control.send(EngineCommand::NoteOn {
                track: 0,
                key,
                velocity: 1.0,
            });
            control.send(EngineCommand::NoteOff { track: 0, key });
            control.send(EngineCommand::AddClipNote {
                track: 2,
                clip: 1,
                pitch: key,
                start_beats: beat,
                len_beats: 1.0,
                velocity: 1.0,
            });
            control.send(EngineCommand::RemoveClipNote {
                track: 2,
                clip: 1,
                pitch: key,
                start_beats: beat,
            });
            control.send(EngineCommand::SetCutoff {
                track: 0,
                hz: 400.0 + (i % 800) as f32,
            });
            control.send(EngineCommand::SetBpm(100.0 + (i % 60) as f32));
            proc.process_block(&[], &mut output, 2, frames);
        }

        for (events, &cap) in proc.track_events.iter().zip(event_caps.iter()) {
            assert_eq!(events.capacity(), cap, "track_events buffer grew");
        }
        assert_eq!(proc.scratch.capacity(), scratch_cap, "scratch buffer grew");
        for (track, &cap) in proc.tracks.iter().zip(clip_list_caps.iter()) {
            assert_eq!(track.arrangement.clips.capacity(), cap, "clip list grew");
        }
        let clip = &proc.tracks[2].arrangement.clips[0];
        assert!(
            clip.notes.len() <= MAX_CLIP_NOTES,
            "clip note count unbounded"
        );
        assert_eq!(clip.notes.capacity(), note_cap, "clip notes buffer grew");
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
    fn per_track_delay_rings_only_its_own_track() {
        // Each track now owns its effects chain: a delay on track 0 must leave a
        // ringing tail on track 0's meter while a track with no delay stays silent.
        let (mut control, mut proc, len) = processor(false);
        let frames = len / 2;
        let mut output = vec![0.0f32; len];
        control.send(EngineCommand::SetDelay { track: 0, on: true });
        control.send(EngineCommand::SetDelayTime {
            track: 0,
            seconds: 0.05,
        });
        control.send(EngineCommand::SetDelayFeedback {
            track: 0,
            feedback: 0.6,
        });
        control.send(EngineCommand::SetDelayMix { track: 0, mix: 0.8 });
        control.send(EngineCommand::NoteOn {
            track: 0,
            key: 60,
            velocity: 1.0,
        });
        for _ in 0..4 {
            proc.process_block(&[], &mut output, 2, frames);
        }
        control.send(EngineCommand::NoteOff { track: 0, key: 60 });
        // Run long past the dry note so only the delay feedback can still ring
        let mut tail = 0.0f32;
        for _ in 0..200 {
            proc.process_block(&[], &mut output, 2, frames);
            tail += control.track_level(0);
        }
        assert!(tail > 0.0, "track 0's own delay produced no tail");
        assert_eq!(
            control.track_level(2),
            0.0,
            "track 2 has no delay or notes; stays silent"
        );
    }

    #[test]
    fn per_track_amp_env_is_independent() {
        // Each track now owns its patch: a long release on track 2 must outlast a
        // near-instant release on track 0 for the same gesture.
        let (mut control, mut proc, len) = processor(false);
        let frames = len / 2;
        let mut output = vec![0.0f32; len];
        control.send(EngineCommand::SetAmpEnv {
            track: 0,
            attack: 0.001,
            decay: 0.01,
            sustain: 0.0,
            release: 0.005,
        });
        control.send(EngineCommand::SetAmpEnv {
            track: 2,
            attack: 0.001,
            decay: 0.01,
            sustain: 1.0,
            release: 2.0,
        });
        control.send(EngineCommand::NoteOn {
            track: 0,
            key: 60,
            velocity: 1.0,
        });
        control.send(EngineCommand::NoteOn {
            track: 2,
            key: 72,
            velocity: 1.0,
        });
        for _ in 0..4 {
            proc.process_block(&[], &mut output, 2, frames);
        }
        control.send(EngineCommand::NoteOff { track: 0, key: 60 });
        control.send(EngineCommand::NoteOff { track: 2, key: 72 });
        let mut t0 = 0.0f32;
        let mut t2 = 0.0f32;
        for _ in 0..20 {
            proc.process_block(&[], &mut output, 2, frames);
            t0 += control.track_level(0);
            t2 += control.track_level(2);
        }
        assert!(
            t2 > 0.0 && t2 > t0 * 4.0,
            "track 2's long release should outlast track 0's: t0={t0} t2={t2}"
        );
    }

    #[test]
    fn track_pan_steers_energy_between_channels() {
        // Track 1 carries the riff; pan it hard left and the right should empty
        let (mut control, mut proc, len) = processor(true);
        let frames = len / 2;
        let mut output = vec![0.0f32; len];
        control.send(EngineCommand::SetTrackPan {
            track: 1,
            pan: -1.0,
        });
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, frames);
            left += output[..frames].iter().map(|s| s.abs()).sum::<f32>();
            right += output[frames..].iter().map(|s| s.abs()).sum::<f32>();
        }
        assert!(left > 0.0, "left channel should carry the panned track");
        assert!(
            right < left * 0.01,
            "hard-left pan leaked right: {right} vs {left}"
        );
    }

    #[test]
    fn gain_command_scales_output() {
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            output.iter().any(|&s| s != 0.0),
            "transport should be sounding"
        );

        control.send(EngineCommand::SetGain(0.0));
        for _ in 0..2 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            output.iter().all(|&s| s == 0.0),
            "gain=0 did not silence output"
        );
    }

    #[test]
    fn muting_the_playing_track_silences_it() {
        // Only track 1 carries the seeded pattern
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            output.iter().any(|&s| s != 0.0),
            "track 1 should be sounding"
        );

        control.send(EngineCommand::SetTrackMute { track: 1, on: true });
        for _ in 0..2 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            output.iter().all(|&s| s == 0.0),
            "muted track still audible"
        );
    }

    #[test]
    fn soloing_an_empty_track_silences_the_mix() {
        let (mut control, mut proc, len) = processor(true);
        let mut output = vec![0.0f32; len];
        for _ in 0..8 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            output.iter().any(|&s| s != 0.0),
            "track 1 should be sounding"
        );

        // Track 0 has no pattern; soloing it mutes everything else
        control.send(EngineCommand::SetTrackSolo { track: 0, on: true });
        for _ in 0..2 {
            proc.process_block(&[], &mut output, 2, len / 2);
        }
        assert!(
            output.iter().all(|&s| s == 0.0),
            "solo did not isolate the empty track"
        );
    }
}
