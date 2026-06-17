// =============================================================================
// File: app/geist-daw/src/studio.rs
// Layer: application binary
// Purpose: Front-end on the geist-ui studio shell, bridged to the audio engine
// Status: Implemented; lens shell (mixer/rack/graph/arrange/browser) over the
//         real engine, playable from an on-screen and computer keyboard.
// Notes: The UI owns no audio truth. A SessionModel mirrors the engine; views
//        mutate that mirror and this layer diffs it each frame, emitting only the
//        EngineCommands whose values changed. Live notes bypass the diff and are
//        sent straight from the keyboards. Monitoring (scope/spectrum/master
//        meter) is pulled from the lock-free control plane into the mirror.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use eframe::egui;
use geist_config::commands::CommandIntent;
use geist_config::templates::{TemplateKind, TemplateRef};
use std::collections::HashMap;

use geist_ui::model::{
    BrowserItem, BrowserModel, ChannelStrip, Clip, EffectSlot, GraphModel, GraphNode, Lane, Note,
    ParamSpec, Port, RackModel, SessionModel, StepPattern, StepSequencerModel, TimelineModel,
};
use geist_ui::shell::draw_studio;
use geist_ui::state::UIState;
use geist_ui::theme::{self, SignalKind};
use geist_ui::widgets::{KeyEvent, Keyboard, Taper};

use crate::control::{EngineCommand, EngineControl};
use crate::engine::{
    default_grid_for, Engine, DEFAULT_AMP_ENV, DEFAULT_FILTER_ENV, DEFAULT_OSC_B_SEMIS,
    DEFAULT_OSC_MIX, NUM_TRACKS, SEQ_ROWS, SEQ_STEPS, TRACK_BASE_MIDI,
};
use crate::session::{self, ClipSession, NoteSession, StudioSession, TrackSession};

// On-screen keyboard spans two octaves from C3
const KEYBOARD_BASE_MIDI: u8 = 48;
const KEYBOARD_KEYS: usize = 25;
// Velocity for notes played live from either keyboard
const UI_VELOCITY: f32 = 0.9;

// Engine startup values, mirrored so the first diff is a no-op
const DEFAULT_BPM: f32 = 120.0;
const DEFAULT_CUTOFF_HZ: f32 = 1_500.0;
const DEFAULT_RESONANCE: f32 = 0.9;
const DEFAULT_GAIN: f32 = 1.0;
const DEFAULT_REVERB_MIX: f32 = 0.3;
const DEFAULT_TRACK_LEVEL: f32 = 0.8;
// Delay defaults, matching the engine's DelayNode startup state
const DEFAULT_DELAY_TIME: f32 = 0.25;
const DEFAULT_DELAY_FEEDBACK: f32 = 0.3;
const DEFAULT_DELAY_MIX: f32 = 0.3;

// Stable rack slot order so the diff can read params back by index
const SLOT_OSC: usize = 0;
const SLOT_FILTER: usize = 1;
const SLOT_AMP_ENV: usize = 2;
const SLOT_FILTER_ENV: usize = 3;
const SLOT_DELAY: usize = 4;
const SLOT_REVERB: usize = 5;
// Oscillator slot parameter order
const OSC_SHAPE: usize = 0;
const OSC_B_SEMIS: usize = 1;
// Filter slot parameter order
const FILTER_CUTOFF: usize = 0;
const FILTER_RESO: usize = 1;
// Envelope slot parameter order (attack/decay/sustain/release)
const ENV_ATTACK: usize = 0;
const ENV_DECAY: usize = 1;
const ENV_SUSTAIN: usize = 2;
const ENV_RELEASE: usize = 3;
// Delay slot parameter order
const DELAY_TIME: usize = 0;
const DELAY_FEEDBACK: usize = 1;
const DELAY_MIX: usize = 2;
// Reverb slot parameter order
const REVERB_MIX: usize = 0;
// Envelope time knob range in seconds
const ENV_TIME_MAX: f32 = 4.0;
// Delay time knob range in seconds
const DELAY_TIME_MAX: f32 = 1.5;
// Oscillator B pitch-offset knob range in semitones
const OSC_B_SEMIS_RANGE: f32 = 24.0;

// Resolution of the on-the-fly spectrum analyzer drawn in the monitor strip
const SPECTRUM_BINS: usize = 48;
// Gain applied to raw DFT magnitudes so quiet material is still visible
const SPECTRUM_SCALE: f32 = 8.0;

// Computer-keyboard to MIDI map: one octave from C4, tracker-style
const COMPUTER_KEYS: [(egui::Key, u8); 13] = [
    (egui::Key::Z, 60),
    (egui::Key::S, 61),
    (egui::Key::X, 62),
    (egui::Key::D, 63),
    (egui::Key::C, 64),
    (egui::Key::V, 65),
    (egui::Key::G, 66),
    (egui::Key::B, 67),
    (egui::Key::H, 68),
    (egui::Key::N, 69),
    (egui::Key::J, 70),
    (egui::Key::M, 71),
    (egui::Key::Comma, 72),
];

// Last values sent to the engine, used to emit only what changed each frame.
// Every synth/fx macro is per-track now; only transport and master gain are global.
struct EngineMirror {
    playing: bool,
    bpm: f32,
    gain: f32,
    cutoff_hz: [f32; NUM_TRACKS],
    resonance: [f32; NUM_TRACKS],
    delay_on: [bool; NUM_TRACKS],
    delay_time: [f32; NUM_TRACKS],
    delay_feedback: [f32; NUM_TRACKS],
    delay_mix: [f32; NUM_TRACKS],
    reverb_on: [bool; NUM_TRACKS],
    reverb_mix: [f32; NUM_TRACKS],
    // Oscillator A/B blend and osc B pitch offset
    osc_mix: [f32; NUM_TRACKS],
    osc_b_semis: [f32; NUM_TRACKS],
    // Amp/filter ADSR macros [attack, decay, sustain, release]
    amp_env: [[f32; 4]; NUM_TRACKS],
    filter_env: [[f32; 4]; NUM_TRACKS],
    track_level: [f32; NUM_TRACKS],
    track_pan: [f32; NUM_TRACKS],
    track_muted: [bool; NUM_TRACKS],
    track_soloed: [bool; NUM_TRACKS],
}

impl EngineMirror {
    // Seed with the engine's startup state so nothing is re-sent on frame one
    fn initial() -> Self {
        Self {
            playing: false,
            bpm: DEFAULT_BPM,
            gain: DEFAULT_GAIN,
            cutoff_hz: [DEFAULT_CUTOFF_HZ; NUM_TRACKS],
            resonance: [DEFAULT_RESONANCE; NUM_TRACKS],
            delay_on: [false; NUM_TRACKS],
            delay_time: [DEFAULT_DELAY_TIME; NUM_TRACKS],
            delay_feedback: [DEFAULT_DELAY_FEEDBACK; NUM_TRACKS],
            delay_mix: [DEFAULT_DELAY_MIX; NUM_TRACKS],
            reverb_on: [false; NUM_TRACKS],
            reverb_mix: [DEFAULT_REVERB_MIX; NUM_TRACKS],
            osc_mix: [DEFAULT_OSC_MIX; NUM_TRACKS],
            osc_b_semis: [DEFAULT_OSC_B_SEMIS; NUM_TRACKS],
            amp_env: [DEFAULT_AMP_ENV; NUM_TRACKS],
            filter_env: [DEFAULT_FILTER_ENV; NUM_TRACKS],
            track_level: [DEFAULT_TRACK_LEVEL; NUM_TRACKS],
            track_pan: [0.0; NUM_TRACKS],
            track_muted: [false; NUM_TRACKS],
            track_soloed: [false; NUM_TRACKS],
        }
    }
}

// Studio front-end: the lens shell over the engine, played live
pub struct StudioApp {
    // Held so the audio stream stays open for the window's lifetime
    _engine: Engine,
    control: EngineControl,
    state: UIState,
    session: SessionModel,
    mirror: EngineMirror,
    // Per-track effects/instrument racks; session.rack reflects the selected one
    track_racks: Vec<RackModel>,
    // Which track session.rack currently reflects
    rack_track: usize,
    // Per-clip note content, keyed by engine clip id; clip-centric piano roll
    clip_notes: HashMap<u64, Vec<Note>>,
    // Last-synced per-clip notes, for diffing edits to the engine
    clip_notes_mirror: HashMap<u64, Vec<Note>>,
    // Which clip the shared piano-roll model currently reflects
    piano_clip: Option<u64>,
    // Last-synced timeline clip placements, for diffing to the engine
    timeline_mirror: Vec<Clip>,
    // Monotonic allocator for engine clip ids (new view clips arrive as id 0)
    next_clip_id: u64,
    // Last-synced step patterns, mirroring the engine grids for diffing
    step_mirror: Vec<StepPattern>,
    // Per-key held state for the on-screen keyboard and the computer keyboard
    kb_held: Vec<bool>,
    computer_held: [bool; COMPUTER_KEYS.len()],
    // Last save/load result shown under the keyboard
    status: String,
}

impl StudioApp {
    // Wrap a running engine and seed a workflow-derived studio shell
    pub fn with_ui_state(engine: Engine, control: EngineControl, state: UIState) -> Self {
        let mut session = initial_session();
        append_workflow_templates(&mut session.browser, &state.workflow().templates);
        // Mirror starts equal to the seeded grids so frame one emits nothing
        let step_mirror = session.step_seq.tracks.clone();
        // Each track starts from the same default rack; session.rack reflects track 0
        let track_racks = vec![session.rack.clone(); NUM_TRACKS];
        Self {
            _engine: engine,
            control,
            state,
            session,
            mirror: EngineMirror::initial(),
            track_racks,
            rack_track: 0,
            clip_notes: HashMap::new(),
            clip_notes_mirror: HashMap::new(),
            piano_clip: None,
            timeline_mirror: Vec::new(),
            next_clip_id: 1,
            step_mirror,
            kb_held: vec![false; KEYBOARD_KEYS],
            computer_held: [false; COMPUTER_KEYS.len()],
            status: String::new(),
        }
    }

    // Snapshot the full session from engine-mirrored state and the pattern mirrors
    fn to_session(&self) -> StudioSession {
        let tracks = (0..NUM_TRACKS)
            .map(|track| {
                let gates = self
                    .session
                    .step_seq
                    .tracks
                    .get(track)
                    .map(gates_of)
                    .unwrap_or_default();
                // This track's placed clips with their note content (skip unassigned)
                let clips = self
                    .session
                    .timeline
                    .clips
                    .iter()
                    .filter(|c| c.lane == track && c.id != 0)
                    .map(|c| ClipSession {
                        id: c.id,
                        start_beats: c.start_beats,
                        len_beats: c.len_beats,
                        notes: self
                            .clip_notes
                            .get(&c.id)
                            .map(|ns| {
                                ns.iter()
                                    .map(|n| NoteSession {
                                        pitch: n.pitch,
                                        start_beats: n.start_beats,
                                        len_beats: n.len_beats,
                                        velocity: n.velocity,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                    .collect();
                TrackSession {
                    level: self.mirror.track_level[track],
                    pan: self.mirror.track_pan[track],
                    muted: self.mirror.track_muted[track],
                    soloed: self.mirror.track_soloed[track],
                    cutoff_hz: self.mirror.cutoff_hz[track],
                    resonance: self.mirror.resonance[track],
                    delay_on: self.mirror.delay_on[track],
                    delay_time: self.mirror.delay_time[track],
                    delay_feedback: self.mirror.delay_feedback[track],
                    delay_mix: self.mirror.delay_mix[track],
                    reverb_on: self.mirror.reverb_on[track],
                    reverb_mix: self.mirror.reverb_mix[track],
                    osc_mix: self.mirror.osc_mix[track],
                    osc_b_semis: self.mirror.osc_b_semis[track],
                    amp_env: self.mirror.amp_env[track],
                    filter_env: self.mirror.filter_env[track],
                    gates,
                    clips,
                }
            })
            .collect();
        StudioSession {
            bpm: self.mirror.bpm,
            gain: self.mirror.gain,
            tracks,
        }
    }

    // Apply a loaded session to the engine, the session model, and the mirrors so
    // the next frame's diffs are quiet.
    fn apply_session(&mut self, loaded: StudioSession) {
        // Transport tempo and master gain are the only global macros
        self.control.send(EngineCommand::SetBpm(loaded.bpm));
        self.control.send(EngineCommand::SetGain(loaded.gain));
        self.mirror.bpm = loaded.bpm;
        self.mirror.gain = loaded.gain;
        self.session.transport.bpm = loaded.bpm;
        if let Some(master) = self.session.mixer.channels.get_mut(NUM_TRACKS) {
            master.level = loaded.gain;
        }

        for (track, state) in loaded.tracks.iter().enumerate().take(NUM_TRACKS) {
            let t = track as u8;

            // Mix flags to the engine, mirror, and strip
            self.control.send(EngineCommand::SetTrackLevel { track: t, level: state.level });
            self.control.send(EngineCommand::SetTrackPan { track: t, pan: state.pan });
            self.control.send(EngineCommand::SetTrackMute { track: t, on: state.muted });
            self.control.send(EngineCommand::SetTrackSolo { track: t, on: state.soloed });
            self.mirror.track_level[track] = state.level;
            self.mirror.track_pan[track] = state.pan;
            self.mirror.track_muted[track] = state.muted;
            self.mirror.track_soloed[track] = state.soloed;
            if let Some(strip) = self.session.mixer.channels.get_mut(track) {
                strip.level = state.level;
                strip.pan = state.pan;
                strip.muted = state.muted;
                strip.soloed = state.soloed;
            }

            // Per-track patch + fx to the engine and the mirror
            self.control.send(EngineCommand::SetCutoff { track: t, hz: state.cutoff_hz });
            self.control.send(EngineCommand::SetResonance { track: t, resonance: state.resonance });
            self.control.send(EngineCommand::SetDelay { track: t, on: state.delay_on });
            self.control.send(EngineCommand::SetDelayTime { track: t, seconds: state.delay_time });
            self.control.send(EngineCommand::SetDelayFeedback { track: t, feedback: state.delay_feedback });
            self.control.send(EngineCommand::SetDelayMix { track: t, mix: state.delay_mix });
            self.control.send(EngineCommand::SetReverb { track: t, on: state.reverb_on });
            self.control.send(EngineCommand::SetReverbMix { track: t, mix: state.reverb_mix });
            self.control.send(EngineCommand::SetOscMix { track: t, mix: state.osc_mix });
            self.control.send(EngineCommand::SetOscBSemis { track: t, semis: state.osc_b_semis });
            self.control.send(EngineCommand::SetAmpEnv {
                track: t,
                attack: state.amp_env[ENV_ATTACK],
                decay: state.amp_env[ENV_DECAY],
                sustain: state.amp_env[ENV_SUSTAIN],
                release: state.amp_env[ENV_RELEASE],
            });
            self.control.send(EngineCommand::SetFilterEnv {
                track: t,
                attack: state.filter_env[ENV_ATTACK],
                decay: state.filter_env[ENV_DECAY],
                sustain: state.filter_env[ENV_SUSTAIN],
                release: state.filter_env[ENV_RELEASE],
            });
            self.mirror.cutoff_hz[track] = state.cutoff_hz;
            self.mirror.resonance[track] = state.resonance;
            self.mirror.delay_on[track] = state.delay_on;
            self.mirror.delay_time[track] = state.delay_time;
            self.mirror.delay_feedback[track] = state.delay_feedback;
            self.mirror.delay_mix[track] = state.delay_mix;
            self.mirror.reverb_on[track] = state.reverb_on;
            self.mirror.reverb_mix[track] = state.reverb_mix;
            self.mirror.osc_mix[track] = state.osc_mix;
            self.mirror.osc_b_semis[track] = state.osc_b_semis;
            self.mirror.amp_env[track] = state.amp_env;
            self.mirror.filter_env[track] = state.filter_env;

            // Reflect the patch into this track's rack so the Shape view matches
            if let Some(rack) = self.track_racks.get_mut(track) {
                set_rack_from_track(rack, state);
            }

            // Rebuild the step grid
            self.control.send(EngineCommand::ClearPattern { track: t });
            if let Some(pattern) = self.session.step_seq.tracks.get_mut(track) {
                pattern.clear();
                for &(row, step) in &state.gates {
                    pattern.set(row as usize, step as usize, true);
                    self.control.send(EngineCommand::SetCell { track: t, step, row, on: true });
                }
                self.step_mirror[track] = pattern.clone();
            }
        }

        // Tear down every known clip in the engine, then rebuild from the load
        for clip in &self.session.timeline.clips {
            self.control.send(EngineCommand::RemoveClip { track: clip.lane as u8, id: clip.id });
        }
        self.session.timeline.clips.clear();
        self.session.timeline.selected = None;
        self.clip_notes.clear();
        self.clip_notes_mirror.clear();
        let mut max_id = 0u64;
        for (track, state) in loaded.tracks.iter().enumerate().take(NUM_TRACKS) {
            let t = track as u8;
            for clip in &state.clips {
                self.control.send(EngineCommand::AddClip {
                    track: t,
                    id: clip.id,
                    start_beats: clip.start_beats,
                    len_beats: clip.len_beats,
                });
                let notes: Vec<Note> = clip
                    .notes
                    .iter()
                    .map(|n| Note {
                        pitch: n.pitch,
                        start_beats: n.start_beats,
                        len_beats: n.len_beats,
                        velocity: n.velocity,
                    })
                    .collect();
                for note in &notes {
                    self.control.send(EngineCommand::AddClipNote {
                        track: t,
                        clip: clip.id,
                        pitch: note.pitch,
                        start_beats: note.start_beats,
                        len_beats: note.len_beats,
                        velocity: note.velocity,
                    });
                }
                self.session.timeline.clips.push(Clip {
                    id: clip.id,
                    lane: track,
                    name: format!("Clip {}", clip.id),
                    start_beats: clip.start_beats,
                    len_beats: clip.len_beats,
                    kind: geist_ui::theme::SignalKind::Note,
                });
                self.clip_notes.insert(clip.id, notes.clone());
                self.clip_notes_mirror.insert(clip.id, notes);
                max_id = max_id.max(clip.id);
            }
        }
        self.timeline_mirror = self.session.timeline.clips.clone();
        self.next_clip_id = max_id + 1;
        self.piano_clip = None;
        self.session.piano.notes.clear();

        // Bind the visible rack to the selected track
        let shown = self.session.mixer.selected.min(NUM_TRACKS - 1);
        self.session.rack = self.track_racks[shown].clone();
        self.rack_track = shown;
    }

    // Send one note transition to the engine on the mixer-selected track
    fn note_event(&mut self, ev: KeyEvent) {
        let track = self.session.mixer.selected.min(NUM_TRACKS - 1) as u8;
        let command = if ev.down {
            EngineCommand::NoteOn { track, key: ev.midi, velocity: UI_VELOCITY }
        } else {
            EngineCommand::NoteOff { track, key: ev.midi }
        };
        self.control.send(command);
    }

    // Poll the computer keyboard and play mapped notes, edge-detected per key
    fn handle_computer_keys(&mut self, ctx: &egui::Context) {
        let down = ctx.input(|i| COMPUTER_KEYS.map(|(key, _)| i.keys_down.contains(&key)));
        for (index, (_, midi)) in COMPUTER_KEYS.iter().enumerate() {
            let now = down[index];
            if now != self.computer_held[index] {
                self.note_event(KeyEvent { midi: *midi, down: now });
                self.computer_held[index] = now;
            }
        }
    }

    // Pull the latest scope/spectrum/master-meter into the mirror session.
    // Reuses the scope/spectrum buffers (clear keeps capacity) so the per-frame
    // monitor path allocates nothing after warmup.
    fn sync_monitor(&mut self) {
        self.control.update_scope();
        let scope = &mut self.session.scope.samples;
        scope.clear();
        scope.extend_from_slice(self.control.scope_view());
        spectrum_into(&self.session.scope.samples, &mut self.session.spectrum.bins);

        // Per-track strips read their own post-fader peak from the engine
        for track in 0..NUM_TRACKS {
            let peak = self.control.track_level(track).clamp(0.0, 1.5);
            if let Some(strip) = self.session.mixer.channels.get_mut(track) {
                strip.peak = peak;
                strip.rms = (peak * 0.85).clamp(0.0, 1.5);
            }
        }
        // The trailing master strip reads the summed output peak
        let level = self.control.level().clamp(0.0, 1.5);
        if let Some(master) = self.session.mixer.channels.get_mut(NUM_TRACKS) {
            master.peak = level;
            master.rms = (level * 0.85).clamp(0.0, 1.5);
        }

        // Mirror the real transport position from the engine for the playhead
        self.session.transport.position_beats = self.control.position_beats();
    }

    // Diff timeline clip placements to the engine: assign ids to view-created
    // clips (id 0), emit Move/Resize on edits, re-home on a lane change, and
    // Remove deleted clips. A clip's lane is its track.
    fn sync_timeline(&mut self) {
        // Assign engine ids to clips the arrangement view just created
        for clip in &mut self.session.timeline.clips {
            if clip.id == 0 {
                clip.id = self.next_clip_id;
                self.next_clip_id += 1;
                self.control.send(EngineCommand::AddClip {
                    track: clip.lane as u8,
                    id: clip.id,
                    start_beats: clip.start_beats,
                    len_beats: clip.len_beats,
                });
                self.clip_notes.insert(clip.id, Vec::new());
                self.clip_notes_mirror.insert(clip.id, Vec::new());
            }
        }

        let current = self.session.timeline.clips.clone();
        for clip in &current {
            match self.timeline_mirror.iter().find(|m| m.id == clip.id).cloned() {
                Some(prev) if prev.lane != clip.lane => {
                    // Lane change re-homes the clip on another track, notes and all
                    self.control.send(EngineCommand::RemoveClip { track: prev.lane as u8, id: clip.id });
                    self.control.send(EngineCommand::AddClip {
                        track: clip.lane as u8,
                        id: clip.id,
                        start_beats: clip.start_beats,
                        len_beats: clip.len_beats,
                    });
                    if let Some(notes) = self.clip_notes.get(&clip.id).cloned() {
                        for note in &notes {
                            self.control.send(EngineCommand::AddClipNote {
                                track: clip.lane as u8,
                                clip: clip.id,
                                pitch: note.pitch,
                                start_beats: note.start_beats,
                                len_beats: note.len_beats,
                                velocity: note.velocity,
                            });
                        }
                    }
                }
                Some(prev) => {
                    if (prev.start_beats - clip.start_beats).abs() > 1e-4 {
                        self.control.send(EngineCommand::MoveClip {
                            track: clip.lane as u8,
                            id: clip.id,
                            start_beats: clip.start_beats,
                        });
                    }
                    if (prev.len_beats - clip.len_beats).abs() > 1e-4 {
                        self.control.send(EngineCommand::ResizeClip {
                            track: clip.lane as u8,
                            id: clip.id,
                            len_beats: clip.len_beats,
                        });
                    }
                }
                None => {} // freshly id-assigned clips were Added above
            }
        }

        // Clips present before but gone now were deleted
        for prev in self.timeline_mirror.clone() {
            if !current.iter().any(|c| c.id == prev.id) {
                self.control.send(EngineCommand::RemoveClip { track: prev.lane as u8, id: prev.id });
                self.clip_notes.remove(&prev.id);
                self.clip_notes_mirror.remove(&prev.id);
                if self.piano_clip == Some(prev.id) {
                    self.piano_clip = None;
                }
            }
        }
        self.timeline_mirror = current;
    }

    // Bind the piano roll to the selected timeline clip and diff its note edits to
    // that clip in the engine. Notes are clip-relative. With no selection the roll
    // is empty.
    fn sync_clip_notes(&mut self) {
        let selected = self
            .session
            .timeline
            .selected_clip()
            .map(|c| (c.id, c.lane, c.len_beats));
        let Some((id, lane, len)) = selected else {
            if self.piano_clip.is_some() {
                self.session.piano.notes.clear();
                self.piano_clip = None;
            }
            return;
        };
        if self.piano_clip != Some(id) {
            // Show the newly selected clip's notes
            self.session.piano.notes = self.clip_notes.get(&id).cloned().unwrap_or_default();
            self.session.piano.length_beats = len.max(1.0);
            self.piano_clip = Some(id);
            return;
        }

        let current = self.session.piano.notes.clone();
        let mirror = self.clip_notes_mirror.get(&id).cloned().unwrap_or_default();
        for note in &current {
            if !mirror.iter().any(|m| same_note(m, note)) {
                self.control.send(EngineCommand::AddClipNote {
                    track: lane as u8,
                    clip: id,
                    pitch: note.pitch,
                    start_beats: note.start_beats,
                    len_beats: note.len_beats,
                    velocity: note.velocity,
                });
            }
        }
        for note in &mirror {
            if !current.iter().any(|c| same_note(c, note)) {
                self.control.send(EngineCommand::RemoveClipNote {
                    track: lane as u8,
                    clip: id,
                    pitch: note.pitch,
                    start_beats: note.start_beats,
                });
            }
        }
        self.clip_notes.insert(id, current.clone());
        self.clip_notes_mirror.insert(id, current);
    }

    // Push step-grid edits to the engine: a wholesale clear becomes one
    // ClearPattern, otherwise each changed gate becomes a SetCell.
    fn sync_steps(&mut self) {
        for track in 0..NUM_TRACKS {
            let current = match self.session.step_seq.tracks.get(track) {
                Some(pattern) => pattern.clone(),
                None => continue,
            };
            let mirror = self.step_mirror[track].clone();
            if current.is_empty() && !mirror.is_empty() {
                self.control.send(EngineCommand::ClearPattern { track: track as u8 });
            } else {
                for row in 0..current.rows {
                    for step in 0..current.steps {
                        let on = current.cell(row, step);
                        if on != mirror.cell(row, step) {
                            self.control.send(EngineCommand::SetCell {
                                track: track as u8,
                                step: step as u8,
                                row: row as u8,
                                on,
                            });
                        }
                    }
                }
            }
            self.step_mirror[track] = current;
        }
    }

    // Emit an EngineCommand for every mirror value a view changed this frame
    fn emit_engine_diff(&mut self) {
        // Transport
        let playing = self.session.transport.playing;
        if playing != self.mirror.playing {
            self.control.send(EngineCommand::SetPlaying(playing));
            self.mirror.playing = playing;
        }
        let bpm = self.session.transport.bpm;
        if bpm != self.mirror.bpm {
            self.control.send(EngineCommand::SetBpm(bpm));
            self.mirror.bpm = bpm;
        }

        // Master gain lives on the trailing "Master" mixer strip
        if let Some(master) = self.session.mixer.channels.get(NUM_TRACKS) {
            let gain = master.level;
            if gain != self.mirror.gain {
                self.control.send(EngineCommand::SetGain(gain));
                self.mirror.gain = gain;
            }
        }

        // Per-track level/mute/solo
        for track in 0..NUM_TRACKS {
            if let Some(channel) = self.session.mixer.channels.get(track) {
                if channel.level != self.mirror.track_level[track] {
                    self.control.send(EngineCommand::SetTrackLevel {
                        track: track as u8,
                        level: channel.level,
                    });
                    self.mirror.track_level[track] = channel.level;
                }
                if channel.pan != self.mirror.track_pan[track] {
                    self.control.send(EngineCommand::SetTrackPan {
                        track: track as u8,
                        pan: channel.pan,
                    });
                    self.mirror.track_pan[track] = channel.pan;
                }
                if channel.muted != self.mirror.track_muted[track] {
                    self.control.send(EngineCommand::SetTrackMute {
                        track: track as u8,
                        on: channel.muted,
                    });
                    self.mirror.track_muted[track] = channel.muted;
                }
                if channel.soloed != self.mirror.track_soloed[track] {
                    self.control.send(EngineCommand::SetTrackSolo {
                        track: track as u8,
                        on: channel.soloed,
                    });
                    self.mirror.track_soloed[track] = channel.soloed;
                }
            }
        }
    }

    // Keep the Shape rack bound to the selected track and push its edits to that
    // track's instrument + effects chain. Like the piano roll, the shared rack
    // model reflects one track at a time; switching tracks reloads that track's
    // rack, and within a track each changed param emits a per-track command.
    fn sync_rack(&mut self) {
        let track = self.session.mixer.selected.min(NUM_TRACKS - 1);
        if track != self.rack_track {
            // Persist any edits to the old track, then show the newly selected one
            self.track_racks[self.rack_track] = self.session.rack.clone();
            self.session.rack = self.track_racks[track].clone();
            self.rack_track = track;
            return;
        }
        let t = track as u8;

        // Filter slot params drive this track's filter
        if let Some(slot) = self.session.rack.slots.get(SLOT_FILTER) {
            if let Some(cutoff) = slot.params.get(FILTER_CUTOFF).map(|p| p.value) {
                if cutoff != self.mirror.cutoff_hz[track] {
                    self.control.send(EngineCommand::SetCutoff { track: t, hz: cutoff });
                    self.mirror.cutoff_hz[track] = cutoff;
                }
            }
            if let Some(reso) = slot.params.get(FILTER_RESO).map(|p| p.value) {
                if reso != self.mirror.resonance[track] {
                    self.control.send(EngineCommand::SetResonance { track: t, resonance: reso });
                    self.mirror.resonance[track] = reso;
                }
            }
        }

        // Delay/reverb enable follows the slot bypass; their params follow the knobs
        if let Some(slot) = self.session.rack.slots.get(SLOT_DELAY) {
            let on = !slot.bypassed;
            if on != self.mirror.delay_on[track] {
                self.control.send(EngineCommand::SetDelay { track: t, on });
                self.mirror.delay_on[track] = on;
            }
            if let Some(time) = slot.params.get(DELAY_TIME).map(|p| p.value) {
                if time != self.mirror.delay_time[track] {
                    self.control.send(EngineCommand::SetDelayTime { track: t, seconds: time });
                    self.mirror.delay_time[track] = time;
                }
            }
            if let Some(fbk) = slot.params.get(DELAY_FEEDBACK).map(|p| p.value) {
                if fbk != self.mirror.delay_feedback[track] {
                    self.control.send(EngineCommand::SetDelayFeedback { track: t, feedback: fbk });
                    self.mirror.delay_feedback[track] = fbk;
                }
            }
            if let Some(mix) = slot.params.get(DELAY_MIX).map(|p| p.value) {
                if mix != self.mirror.delay_mix[track] {
                    self.control.send(EngineCommand::SetDelayMix { track: t, mix });
                    self.mirror.delay_mix[track] = mix;
                }
            }
        }
        if let Some(slot) = self.session.rack.slots.get(SLOT_REVERB) {
            let on = !slot.bypassed;
            if on != self.mirror.reverb_on[track] {
                self.control.send(EngineCommand::SetReverb { track: t, on });
                self.mirror.reverb_on[track] = on;
            }
            if let Some(mix) = slot.params.get(REVERB_MIX).map(|p| p.value) {
                if mix != self.mirror.reverb_mix[track] {
                    self.control.send(EngineCommand::SetReverbMix { track: t, mix });
                    self.mirror.reverb_mix[track] = mix;
                }
            }
        }

        // Oscillator shape (sine/saw blend) and osc B pitch offset
        if let Some(slot) = self.session.rack.slots.get(SLOT_OSC) {
            if let Some(shape) = slot.params.get(OSC_SHAPE).map(|p| p.value) {
                if shape != self.mirror.osc_mix[track] {
                    self.control.send(EngineCommand::SetOscMix { track: t, mix: shape });
                    self.mirror.osc_mix[track] = shape;
                }
            }
            if let Some(semis) = slot.params.get(OSC_B_SEMIS).map(|p| p.value) {
                if semis != self.mirror.osc_b_semis[track] {
                    self.control.send(EngineCommand::SetOscBSemis { track: t, semis });
                    self.mirror.osc_b_semis[track] = semis;
                }
            }
        }

        // Amp/filter envelopes drive this track's ADSR
        if let Some(env) = self.session.rack.slots.get(SLOT_AMP_ENV).and_then(env_of) {
            if env != self.mirror.amp_env[track] {
                self.control.send(EngineCommand::SetAmpEnv {
                    track: t,
                    attack: env[ENV_ATTACK],
                    decay: env[ENV_DECAY],
                    sustain: env[ENV_SUSTAIN],
                    release: env[ENV_RELEASE],
                });
                self.mirror.amp_env[track] = env;
            }
        }
        if let Some(env) = self.session.rack.slots.get(SLOT_FILTER_ENV).and_then(env_of) {
            if env != self.mirror.filter_env[track] {
                self.control.send(EngineCommand::SetFilterEnv {
                    track: t,
                    attack: env[ENV_ATTACK],
                    decay: env[ENV_DECAY],
                    sustain: env[ENV_SUSTAIN],
                    release: env[ENV_RELEASE],
                });
                self.mirror.filter_env[track] = env;
            }
        }

        // Persist the selected track's edits back into its stored rack
        self.track_racks[track] = self.session.rack.clone();
    }
}

impl eframe::App for StudioApp {
    // eframe 0.34 hands a root Ui; the shell composes panels via show_inside
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_computer_keys(&ctx);
        self.sync_monitor();

        // Playable keyboard pinned to the very bottom, below the monitor strip
        egui::Panel::bottom("geist_keyboard")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Play Z S X D C V G B H N J M ,  ·  notes go to the selected mixer track")
                        .small()
                        .color(theme::TEXT_MUTED),
                );
                let events = Keyboard::new(&mut self.kb_held)
                    .base_midi(KEYBOARD_BASE_MIDI)
                    .keys(KEYBOARD_KEYS)
                    .show(ui);
                for ev in events {
                    self.note_event(ev);
                }
                if !self.status.is_empty() {
                    ui.label(egui::RichText::new(&self.status).small().color(theme::TEXT_MUTED));
                }
            });

        // The studio shell: transport + lens tabs, lens content, monitor strip
        let response = draw_studio(ui, &mut self.state, &mut self.session);

        // Reflect any view edits to the engine, then keep meters animating
        self.emit_engine_diff();
        self.sync_rack();
        self.sync_timeline();
        self.sync_clip_notes();
        self.sync_steps();

        // Session persistence, after the syncs so the snapshot is current
        if response.save_requested {
            let snapshot = self.to_session();
            self.status = match session::save(&snapshot) {
                Ok(path) => format!("Saved {}", path.display()),
                Err(err) => format!("Save failed: {err}"),
            };
        }
        if response.load_requested {
            let fallback = self.to_session();
            match session::load(&fallback) {
                Ok(loaded) => {
                    self.apply_session(loaded);
                    self.status = "Loaded session".to_string();
                }
                Err(err) => self.status = format!("Load failed: {err}"),
            }
        }

        ctx.request_repaint();
    }
}

// Build the mirror session that backs the views, matching engine startup state
fn initial_session() -> SessionModel {
    let mut session = SessionModel::default();
    session.transport.bpm = DEFAULT_BPM;

    // Mixer: one strip per track plus a trailing master strip the meter reads
    for track in 0..NUM_TRACKS {
        let mut strip = ChannelStrip::new(format!("Track {}", track + 1));
        strip.level = DEFAULT_TRACK_LEVEL;
        strip.inserts = vec!["Synth".into()];
        session.mixer.channels.push(strip);
    }
    let mut master = ChannelStrip::new("Master");
    master.level = DEFAULT_GAIN;
    session.mixer.channels.push(master);
    // Open the keyboard/mixer on the first instrument track
    session.mixer.selected = 0;

    // Rack: the visible, modifiable signal chain mapped to engine controls.
    // Order matches the SLOT_* indices the diff reads back.
    let oscillator = EffectSlot::new(
        "Oscillator",
        vec![
            ParamSpec::new("Shape", DEFAULT_OSC_MIX, 0.0, 1.0),
            ParamSpec::new("B Semi", DEFAULT_OSC_B_SEMIS, -OSC_B_SEMIS_RANGE, OSC_B_SEMIS_RANGE),
        ],
    );
    let filter = EffectSlot::new(
        "Filter",
        vec![
            ParamSpec::new("Cutoff", DEFAULT_CUTOFF_HZ, 20.0, 18_000.0)
                .unit("Hz")
                .taper(Taper::Logarithmic),
            ParamSpec::new("Reso", DEFAULT_RESONANCE, 0.5, 6.0),
        ],
    );
    let amp_env = EffectSlot::new("Amp Env", env_params(DEFAULT_AMP_ENV));
    let filter_env = EffectSlot::new("Filter Env", env_params(DEFAULT_FILTER_ENV));
    let mut delay = EffectSlot::new(
        "Delay",
        vec![
            ParamSpec::new("Time", DEFAULT_DELAY_TIME, 0.01, DELAY_TIME_MAX)
                .unit("s")
                .taper(Taper::Logarithmic),
            ParamSpec::new("Fbk", DEFAULT_DELAY_FEEDBACK, 0.0, 0.95),
            ParamSpec::new("Mix", DEFAULT_DELAY_MIX, 0.0, 1.0),
        ],
    );
    delay.bypassed = true;
    let mut reverb = EffectSlot::new(
        "Reverb",
        vec![ParamSpec::new("Mix", DEFAULT_REVERB_MIX, 0.0, 1.0)],
    );
    reverb.bypassed = true;
    session.rack.push(oscillator);
    session.rack.push(filter);
    session.rack.push(amp_env);
    session.rack.push(filter_env);
    session.rack.push(delay);
    session.rack.push(reverb);
    session.rack.selected = Some(SLOT_OSC);

    // Step sequencer: mirror the engine's seeded per-track grids
    session.step_seq = StepSequencerModel {
        tracks: (0..NUM_TRACKS).map(engine_step_pattern).collect(),
        selected: 0,
    };

    // Graph: a representative view of the engine's signal path (visual for now)
    session.graph = engine_graph();

    // Empty editing surfaces, sized to the mix
    session.piano.length_beats = 16.0;
    session.timeline = TimelineModel {
        lanes: (0..NUM_TRACKS)
            .map(|track| Lane { name: format!("Track {}", track + 1) })
            .collect(),
        clips: Vec::new(),
        length_beats: 32.0,
        selected: None,
    };

    session.browser = browser_catalog();
    session
}

// Build a step pattern mirroring the engine's seeded grid for one track
fn engine_step_pattern(track: usize) -> StepPattern {
    let grid = default_grid_for(track);
    let mut pattern = StepPattern::new(SEQ_ROWS, SEQ_STEPS, TRACK_BASE_MIDI[track]);
    for (row, steps) in grid.iter().enumerate() {
        for (step, &on) in steps.iter().enumerate() {
            pattern.set(row, step, on);
        }
    }
    pattern
}

// A representative node-graph view of the fixed engine chain
fn engine_graph() -> GraphModel {
    let audio_in = |name: &str| Port { name: name.into(), kind: SignalKind::Audio };
    let audio_out = || Port { name: "Out".into(), kind: SignalKind::Audio };
    GraphModel {
        nodes: vec![
            GraphNode {
                id: 1,
                name: "Synth".into(),
                pos: (40.0, 70.0),
                inputs: vec![Port { name: "Pitch".into(), kind: SignalKind::Note }],
                outputs: vec![audio_out()],
            },
            GraphNode {
                id: 2,
                name: "Filter".into(),
                pos: (220.0, 70.0),
                inputs: vec![audio_in("In"), Port { name: "Cutoff".into(), kind: SignalKind::Cv }],
                outputs: vec![audio_out()],
            },
            GraphNode {
                id: 3,
                name: "Delay".into(),
                pos: (400.0, 70.0),
                inputs: vec![audio_in("In")],
                outputs: vec![audio_out()],
            },
            GraphNode {
                id: 4,
                name: "Reverb".into(),
                pos: (560.0, 70.0),
                inputs: vec![audio_in("In")],
                outputs: vec![audio_out()],
            },
            GraphNode {
                id: 5,
                name: "Master".into(),
                pos: (720.0, 70.0),
                inputs: vec![audio_in("In")],
                outputs: Vec::new(),
            },
        ],
        cables: vec![
            cable(1, 2),
            cable(2, 3),
            cable(3, 4),
            cable(4, 5),
        ],
    }
}

// One audio cable from a node's first output to the next node's first input
fn cable(from: u64, to: u64) -> geist_ui::model::Cable {
    geist_ui::model::Cable {
        from_node: from,
        from_port: 0,
        to_node: to,
        to_port: 0,
        kind: SignalKind::Audio,
    }
}

// The browser catalog of the engine's built-in instrument and effects
fn browser_catalog() -> BrowserModel {
    BrowserModel {
        items: vec![
            BrowserItem::new("Geist Synth", "Instrument", SignalKind::Note),
            BrowserItem::new("Filter", "Effect", SignalKind::Audio),
            BrowserItem::new("Delay", "Effect", SignalKind::Audio),
            BrowserItem::new("Reverb", "Effect", SignalKind::Audio),
        ],
        query: String::new(),
        selected: None,
    }
}

// Add workflow-defined templates to the browser as declarative insert commands
fn append_workflow_templates(browser: &mut BrowserModel, templates: &[TemplateRef]) {
    for template in templates {
        browser.items.push(template_browser_item(template));
    }
}

// Convert one workflow template into a searchable browser item
fn template_browser_item(template: &TemplateRef) -> BrowserItem {
    let mut intent = CommandIntent::new("instantiate_template");
    intent.args.insert("name".to_string(), template.name.clone());
    intent
        .args
        .insert("kind".to_string(), template_kind_name(template.kind).to_string());
    intent.args.extend(template.args.clone());
    BrowserItem::new(
        template.name.clone(),
        format!("{} Template", template_kind_label(template.kind)),
        template_signal_kind(template.kind),
    )
    .with_intent(intent)
}

fn template_signal_kind(kind: TemplateKind) -> SignalKind {
    match kind {
        TemplateKind::Project => SignalKind::Note,
        TemplateKind::Track => SignalKind::Note,
        TemplateKind::Rack => SignalKind::Audio,
        TemplateKind::Graph => SignalKind::Audio,
        TemplateKind::Modulation => SignalKind::Cv,
    }
}

fn template_kind_label(kind: TemplateKind) -> &'static str {
    match kind {
        TemplateKind::Project => "Project",
        TemplateKind::Track => "Track",
        TemplateKind::Rack => "Rack",
        TemplateKind::Graph => "Graph",
        TemplateKind::Modulation => "Modulation",
    }
}

fn template_kind_name(kind: TemplateKind) -> &'static str {
    match kind {
        TemplateKind::Project => "project",
        TemplateKind::Track => "track",
        TemplateKind::Rack => "rack",
        TemplateKind::Graph => "graph",
        TemplateKind::Modulation => "modulation",
    }
}

// Two piano-roll notes are the same slot when pitch and start beat coincide
fn same_note(a: &Note, b: &Note) -> bool {
    a.pitch == b.pitch && (a.start_beats - b.start_beats).abs() < 1e-3
}

// Build the four ParamSpecs for an envelope slot from [a, d, s, r]
fn env_params(env: [f32; 4]) -> Vec<ParamSpec> {
    vec![
        ParamSpec::new("A", env[ENV_ATTACK], 0.001, ENV_TIME_MAX).unit("s").taper(Taper::Logarithmic),
        ParamSpec::new("D", env[ENV_DECAY], 0.001, ENV_TIME_MAX).unit("s").taper(Taper::Logarithmic),
        ParamSpec::new("S", env[ENV_SUSTAIN], 0.0, 1.0),
        ParamSpec::new("R", env[ENV_RELEASE], 0.001, ENV_TIME_MAX).unit("s").taper(Taper::Logarithmic),
    ]
}

// Write a loaded track's patch + fx into its rack slots so the Shape view matches
fn set_rack_from_track(rack: &mut RackModel, state: &TrackSession) {
    if let Some(slot) = rack.slots.get_mut(SLOT_OSC) {
        if let Some(p) = slot.params.get_mut(OSC_SHAPE) {
            p.value = state.osc_mix;
        }
        if let Some(p) = slot.params.get_mut(OSC_B_SEMIS) {
            p.value = state.osc_b_semis;
        }
    }
    if let Some(slot) = rack.slots.get_mut(SLOT_FILTER) {
        if let Some(p) = slot.params.get_mut(FILTER_CUTOFF) {
            p.value = state.cutoff_hz;
        }
        if let Some(p) = slot.params.get_mut(FILTER_RESO) {
            p.value = state.resonance;
        }
    }
    if let Some(slot) = rack.slots.get_mut(SLOT_AMP_ENV) {
        set_env_slot(slot, state.amp_env);
    }
    if let Some(slot) = rack.slots.get_mut(SLOT_FILTER_ENV) {
        set_env_slot(slot, state.filter_env);
    }
    if let Some(slot) = rack.slots.get_mut(SLOT_DELAY) {
        slot.bypassed = !state.delay_on;
        if let Some(p) = slot.params.get_mut(DELAY_TIME) {
            p.value = state.delay_time;
        }
        if let Some(p) = slot.params.get_mut(DELAY_FEEDBACK) {
            p.value = state.delay_feedback;
        }
        if let Some(p) = slot.params.get_mut(DELAY_MIX) {
            p.value = state.delay_mix;
        }
    }
    if let Some(slot) = rack.slots.get_mut(SLOT_REVERB) {
        slot.bypassed = !state.reverb_on;
        if let Some(p) = slot.params.get_mut(REVERB_MIX) {
            p.value = state.reverb_mix;
        }
    }
}

// Write [a, d, s, r] into an envelope slot's first four params
fn set_env_slot(slot: &mut EffectSlot, env: [f32; 4]) {
    for (i, &value) in env.iter().enumerate() {
        if let Some(param) = slot.params.get_mut(i) {
            param.value = value;
        }
    }
}

// Read an envelope slot's four params as [a, d, s, r]
fn env_of(slot: &EffectSlot) -> Option<[f32; 4]> {
    if slot.params.len() >= 4 {
        Some([
            slot.params[ENV_ATTACK].value,
            slot.params[ENV_DECAY].value,
            slot.params[ENV_SUSTAIN].value,
            slot.params[ENV_RELEASE].value,
        ])
    } else {
        None
    }
}

// The (row, step) gates that are on in a step pattern
fn gates_of(pattern: &geist_ui::model::StepPattern) -> Vec<(u8, u8)> {
    let mut gates = Vec::new();
    for row in 0..pattern.rows {
        for step in 0..pattern.steps {
            if pattern.cell(row, step) {
                gates.push((row as u8, step as u8));
            }
        }
    }
    gates
}

// Naive DFT magnitude at integer harmonics of the window, scaled for display.
// Writes into `bins`, reusing its capacity (clear keeps it) to avoid per-frame
// allocation on the monitor path.
fn spectrum_into(samples: &[f32], bins: &mut Vec<f32>) {
    bins.clear();
    let n = samples.len();
    if n < 2 {
        bins.resize(SPECTRUM_BINS, 0.0);
        return;
    }
    for bin in 1..=SPECTRUM_BINS {
        let freq = bin as f32;
        let mut re = 0.0f32;
        let mut im = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let phase = std::f32::consts::TAU * freq * i as f32 / n as f32;
            re += s * phase.cos();
            im -= s * phase.sin();
        }
        bins.push(((re * re + im * im).sqrt() / n as f32 * SPECTRUM_SCALE).clamp(0.0, 1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_matches_initial_session_so_first_diff_is_quiet() {
        // The seeded session must agree with the mirror's startup values
        // session.rack reflects the selected track (track 0) at startup
        let session = initial_session();
        let mirror = EngineMirror::initial();
        let master = session.mixer.channels.get(NUM_TRACKS).unwrap();
        assert_eq!(master.level, mirror.gain);
        let filter = &session.rack.slots[SLOT_FILTER];
        assert_eq!(filter.params[FILTER_CUTOFF].value, mirror.cutoff_hz[0]);
        assert_eq!(filter.params[FILTER_RESO].value, mirror.resonance[0]);
        assert_eq!(!session.rack.slots[SLOT_DELAY].bypassed, mirror.delay_on[0]);
        assert_eq!(!session.rack.slots[SLOT_REVERB].bypassed, mirror.reverb_on[0]);
        assert_eq!(session.rack.slots[SLOT_REVERB].params[REVERB_MIX].value, mirror.reverb_mix[0]);
        assert_eq!(env_of(&session.rack.slots[SLOT_AMP_ENV]), Some(mirror.amp_env[0]));
        assert_eq!(env_of(&session.rack.slots[SLOT_FILTER_ENV]), Some(mirror.filter_env[0]));
        let delay = &session.rack.slots[SLOT_DELAY];
        assert_eq!(delay.params[DELAY_TIME].value, mirror.delay_time[0]);
        assert_eq!(delay.params[DELAY_FEEDBACK].value, mirror.delay_feedback[0]);
        assert_eq!(delay.params[DELAY_MIX].value, mirror.delay_mix[0]);
        let osc = &session.rack.slots[SLOT_OSC];
        assert_eq!(osc.params[OSC_SHAPE].value, mirror.osc_mix[0]);
        assert_eq!(osc.params[OSC_B_SEMIS].value, mirror.osc_b_semis[0]);
        assert_eq!(session.transport.bpm, mirror.bpm);
    }

    #[test]
    fn session_has_a_strip_per_track_plus_master() {
        let session = initial_session();
        assert_eq!(session.mixer.channels.len(), NUM_TRACKS + 1);
        assert_eq!(session.mixer.channels[NUM_TRACKS].name, "Master");
    }

    #[test]
    fn step_grid_mirrors_the_seeded_engine_pattern() {
        // The engine seeds the mid track with the riff; the rest start empty
        let session = initial_session();
        assert_eq!(session.step_seq.tracks.len(), NUM_TRACKS);
        assert!(!session.step_seq.tracks[1].is_empty(), "track 1 carries the riff");
        assert!(session.step_seq.tracks[0].is_empty(), "track 0 starts empty");
    }

    #[test]
    fn workflow_templates_become_browser_insert_intents() {
        let mut browser = browser_catalog();
        let template = TemplateRef {
            name: "Macro Mod Lane".to_string(),
            kind: TemplateKind::Modulation,
            args: [("target".to_string(), "selected_parameter".to_string())].into(),
        };

        append_workflow_templates(&mut browser, &[template]);

        let item = browser.items.last().unwrap();
        assert_eq!(item.name, "Macro Mod Lane");
        assert_eq!(item.category, "Modulation Template");
        assert_eq!(item.kind, SignalKind::Cv);
        assert_eq!(item.intent.command, "instantiate_template");
        assert_eq!(item.intent.args.get("kind").unwrap(), "modulation");
        assert_eq!(
            item.intent.args.get("target").unwrap(),
            "selected_parameter"
        );
    }

    #[test]
    fn silence_yields_no_spectrum() {
        let mut bins = Vec::new();
        spectrum_into(&[0.0; 256], &mut bins);
        assert_eq!(bins.len(), SPECTRUM_BINS);
        assert!(bins.iter().all(|&b| b == 0.0));
    }

    #[test]
    fn a_tone_lights_a_spectrum_bin() {
        // A pure sine at harmonic 4 of the window should energize a bin
        let n = 256usize;
        let samples: Vec<f32> = (0..n)
            .map(|i| (std::f32::consts::TAU * 4.0 * i as f32 / n as f32).sin())
            .collect();
        let mut bins = Vec::new();
        spectrum_into(&samples, &mut bins);
        assert!(bins.iter().any(|&b| b > 0.1), "expected a lit bin");
    }

    #[test]
    fn spectrum_reuses_its_buffer_without_reallocating() {
        // After the first fill, refills must not grow the buffer's capacity
        let samples = [0.5f32; 256];
        let mut bins = Vec::new();
        spectrum_into(&samples, &mut bins);
        let cap = bins.capacity();
        for _ in 0..100 {
            spectrum_into(&samples, &mut bins);
        }
        assert_eq!(bins.capacity(), cap, "spectrum buffer should be reused");
        assert_eq!(bins.len(), SPECTRUM_BINS);
    }
}
