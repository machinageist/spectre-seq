// =============================================================================
// File: app/geist-daw/src/control.rs
// Layer: application binary
// Purpose: Lock-free control plane between the UI thread and the audio thread
// Status: Implemented; note/transport commands plus an atomic level meter.
// Notes: Commands flow UI -> audio over an rtrb SPSC ring; the meter flows
//        audio -> UI as a single atomic. Neither side blocks, so the audio
//        thread stays realtime-safe. Param/macro commands extend this enum.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use rtrb::{Consumer, Producer, RingBuffer};

// Default depth of the command ring; one block rarely drains this many
const COMMAND_CAPACITY: usize = 256;
// Depth of the scope sample ring; sized to outpace the UI frame rate
const SCOPE_CAPACITY: usize = 8_192;
// Keep every Nth output sample for the scope, thinning the audio-rate stream
const SCOPE_DECIMATION: usize = 4;
// Width of the rolling scope window the UI displays
const SCOPE_VIEW_LEN: usize = 1_024;

// One instruction from the UI thread to the audio thread
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EngineCommand {
    // Start a note on a track at a MIDI key and normalized velocity
    NoteOn { track: u8, key: u8, velocity: f32 },
    // Release a note on a track at a MIDI key
    NoteOff { track: u8, key: u8 },
    // Silence every sounding voice
    AllNotesOff,
    // Start or stop the transport (drives the tempo-synced sequencer)
    SetPlaying(bool),
    // Set the transport tempo in beats per minute
    SetBpm(f32),
    // Set a track's base filter cutoff in hertz
    SetCutoff { track: u8, hz: f32 },
    // Set a track's filter resonance
    SetResonance { track: u8, resonance: f32 },
    // Set the master output gain
    SetGain(f32),
    // Toggle a track's delay effect
    SetDelay { track: u8, on: bool },
    // Set a track's delay time in seconds (both channels)
    SetDelayTime { track: u8, seconds: f32 },
    // Set a track's delay feedback amount
    SetDelayFeedback { track: u8, feedback: f32 },
    // Set a track's delay dry/wet mix
    SetDelayMix { track: u8, mix: f32 },
    // Toggle a track's reverb effect
    SetReverb { track: u8, on: bool },
    // Set a track's reverb dry/wet mix
    SetReverbMix { track: u8, mix: f32 },
    // Set a track's oscillator A unison voice count
    SetUnisonVoices { track: u8, voices: usize },
    // Set a track's oscillator A unison detune spread in cents
    SetDetune { track: u8, cents: f32 },
    // Set a track's oscillator A/B blend (0 = sine, 1 = saw)
    SetOscMix { track: u8, mix: f32 },
    // Set a track's oscillator B pitch offset in semitones
    SetOscBSemis { track: u8, semis: f32 },
    // Set a track's amplitude ADSR (attack/decay/release seconds, sustain 0..1)
    SetAmpEnv { track: u8, attack: f32, decay: f32, sustain: f32, release: f32 },
    // Set a track's filter ADSR (attack/decay/release seconds, sustain 0..1)
    SetFilterEnv { track: u8, attack: f32, decay: f32, sustain: f32, release: f32 },
    // Toggle one step-sequencer cell on a track
    SetCell { track: u8, step: u8, row: u8, on: bool },
    // Clear a track's whole step pattern
    ClearPattern { track: u8 },
    // Place a new MIDI clip on a track's timeline
    AddClip { track: u8, id: u64, start_beats: f32, len_beats: f32 },
    // Move a placed clip's start position
    MoveClip { track: u8, id: u64, start_beats: f32 },
    // Resize a placed clip's length
    ResizeClip { track: u8, id: u64, len_beats: f32 },
    // Remove a placed clip from a track's timeline
    RemoveClip { track: u8, id: u64 },
    // Add a timed note (relative to the clip start) to a clip
    AddClipNote { track: u8, clip: u64, pitch: u8, start_beats: f32, len_beats: f32, velocity: f32 },
    // Remove the matching note (pitch + start) from a clip
    RemoveClipNote { track: u8, clip: u64, pitch: u8, start_beats: f32 },
    // Clear all notes from a clip
    ClearClip { track: u8, clip: u64 },
    // Set a track's mixer level
    SetTrackLevel { track: u8, level: f32 },
    // Set a track's stereo pan in [-1, 1] (left to right)
    SetTrackPan { track: u8, pan: f32 },
    // Mute or unmute a track
    SetTrackMute { track: u8, on: bool },
    // Solo or unsolo a track
    SetTrackSolo { track: u8, on: bool },
}

// Lock-free latest-value output meter shared across the thread boundary
// The audio thread stores each block's peak; the UI polls it without blocking
#[derive(Debug, Default)]
pub struct LevelMeter {
    peak: AtomicU32,
}

impl LevelMeter {
    // Build a meter reading zero until the first block runs
    pub fn new() -> Self {
        Self::default()
    }

    // Publish the latest block peak from the audio thread
    pub fn store(&self, peak: f32) {
        self.peak.store(peak.to_bits(), Ordering::Relaxed);
    }

    // Read the most recent block peak from the UI thread
    pub fn read(&self) -> f32 {
        f32::from_bits(self.peak.load(Ordering::Relaxed))
    }
}

// Lock-free latest transport position in beats, published audio -> UI
#[derive(Debug, Default)]
pub struct BeatClock {
    beats: AtomicU64,
}

impl BeatClock {
    pub fn new() -> Self {
        Self::default()
    }

    // Publish the current transport beat position from the audio thread
    pub fn store(&self, beats: f64) {
        self.beats.store(beats.to_bits(), Ordering::Relaxed);
    }

    // Read the latest transport beat position from the UI thread
    pub fn read(&self) -> f64 {
        f64::from_bits(self.beats.load(Ordering::Relaxed))
    }
}

// UI-thread handle: sends commands, reads the meters/clock, and drains the scope
pub struct EngineControl {
    commands: Producer<EngineCommand>,
    meter: Arc<LevelMeter>,
    // One post-fader meter per mixer track, parallel to the engine's tracks
    track_meters: Arc<Vec<LevelMeter>>,
    // Latest transport beat position for the UI playhead
    position: Arc<BeatClock>,
    scope: Consumer<f32>,
    // Rolling display window the UI redraws each frame
    scope_view: Vec<f32>,
}

// Audio-thread handle: drains commands, publishes the meters, clock, and scope
pub struct EngineSink {
    pub commands: Consumer<EngineCommand>,
    pub meter: Arc<LevelMeter>,
    pub track_meters: Arc<Vec<LevelMeter>>,
    pub position: Arc<BeatClock>,
    scope: Producer<f32>,
}

// Build the paired UI and audio ends of the control plane for `num_tracks`
pub fn control_plane(num_tracks: usize) -> (EngineControl, EngineSink) {
    let (command_tx, command_rx) = RingBuffer::new(COMMAND_CAPACITY);
    let (scope_tx, scope_rx) = RingBuffer::new(SCOPE_CAPACITY);
    let meter = Arc::new(LevelMeter::new());
    let track_meters = Arc::new((0..num_tracks).map(|_| LevelMeter::new()).collect::<Vec<_>>());
    let position = Arc::new(BeatClock::new());
    (
        EngineControl {
            commands: command_tx,
            meter: Arc::clone(&meter),
            track_meters: Arc::clone(&track_meters),
            position: Arc::clone(&position),
            scope: scope_rx,
            scope_view: Vec::with_capacity(SCOPE_VIEW_LEN),
        },
        EngineSink {
            commands: command_rx,
            meter,
            track_meters,
            position,
            scope: scope_tx,
        },
    )
}

impl EngineSink {
    // Publish a decimated copy of the block's mono waveform for the scope
    // Wait-free: samples are dropped if the UI falls behind, never blocked
    pub fn push_scope(&mut self, mono: &[f32]) {
        let mut index = 0;
        while index < mono.len() {
            let _ = self.scope.push(mono[index]);
            index += SCOPE_DECIMATION;
        }
    }
}

impl EngineControl {
    // Enqueue a command; dropped (returns false) only if the ring is saturated
    pub fn send(&mut self, command: EngineCommand) -> bool {
        self.commands.push(command).is_ok()
    }

    // Current output peak for meters and scopes
    pub fn level(&self) -> f32 {
        self.meter.read()
    }

    // Latest post-fader peak for one track; zero for an out-of-range index
    pub fn track_level(&self, track: usize) -> f32 {
        self.track_meters.get(track).map(LevelMeter::read).unwrap_or(0.0)
    }

    // Latest transport position in beats for the UI playhead
    pub fn position_beats(&self) -> f64 {
        self.position.read()
    }

    // Drain newly published scope samples into the rolling display window
    pub fn update_scope(&mut self) {
        while let Ok(sample) = self.scope.pop() {
            self.scope_view.push(sample);
        }
        let len = self.scope_view.len();
        if len > SCOPE_VIEW_LEN {
            self.scope_view.drain(0..len - SCOPE_VIEW_LEN);
        }
    }

    // The latest scope window, oldest sample first
    pub fn scope_view(&self) -> &[f32] {
        &self.scope_view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_round_trips_across_the_boundary() {
        let (control, sink) = control_plane(3);
        assert_eq!(control.level(), 0.0);
        sink.meter.store(0.75);
        assert_eq!(control.level(), 0.75);
    }

    #[test]
    fn track_meters_round_trip_and_clamp_out_of_range() {
        let (control, sink) = control_plane(3);
        assert_eq!(control.track_level(0), 0.0);
        sink.track_meters[1].store(0.4);
        assert_eq!(control.track_level(1), 0.4);
        // An index past the track count reads as silence, never panics
        assert_eq!(control.track_level(9), 0.0);
    }

    #[test]
    fn beat_clock_round_trips_across_the_boundary() {
        let (control, sink) = control_plane(2);
        assert_eq!(control.position_beats(), 0.0);
        sink.position.store(4.25);
        assert_eq!(control.position_beats(), 4.25);
    }

    #[test]
    fn scope_publishes_decimated_samples() {
        let (mut control, mut sink) = control_plane(3);
        sink.push_scope(&[0.5f32; 64]);
        control.update_scope();
        let view = control.scope_view();
        // 64 samples decimated by 4 yields 16 points
        assert_eq!(view.len(), 16);
        assert!(view.iter().all(|&s| s == 0.5));
    }

    #[test]
    fn commands_arrive_in_order() {
        let (mut control, mut sink) = control_plane(3);
        assert!(control.send(EngineCommand::NoteOn { track: 0, key: 60, velocity: 1.0 }));
        assert!(control.send(EngineCommand::NoteOff { track: 0, key: 60 }));
        assert_eq!(
            sink.commands.pop().ok(),
            Some(EngineCommand::NoteOn { track: 0, key: 60, velocity: 1.0 })
        );
        assert_eq!(
            sink.commands.pop().ok(),
            Some(EngineCommand::NoteOff { track: 0, key: 60 })
        );
        assert!(sink.commands.pop().is_err());
    }
}
