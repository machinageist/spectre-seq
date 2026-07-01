// =============================================================================
// File: app/geist-daw/src/session.rs
// Layer: application binary
// Purpose: Map the full studio session to and from the geist-project file format
// Status: Implemented; round-trips transport, mixer, macros, step grids, clips.
// Notes: StudioSession is the app's plain intermediate; conversion to the on-disk
//        ProjectFile is the app layer's job (the crate leaves it to callers).
//        Macros and per-track levels ride on one graph node's params; mute/solo
//        and patterns ride on TrackEntry clips (piano notes and step gates as
//        MIDI). Disk I/O only; never touched from the audio thread.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::{Path, PathBuf};

use geist_project::prelude::{
    load_from_path, save_to_path, ClipEntry, ClipKind, NodeEntry, NoteEntry, ParamValue,
    ProjectError, ProjectFile, TrackEntry,
};

use crate::control::{FxKind, LfoDestination, FX_CHAIN_MAX};
use crate::engine::{NUM_TRACKS, SEQ_ROWS, SEQ_STEPS, TRACK_BASE_MIDI};

// Studio session slot filename, distinct from the classic patch slot
const SESSION_FILE: &str = "geist-studio.gproj";
// Node tag carrying the global macros and per-track levels
const MACROS_NODE_KIND: &str = "geist-macros";

// Master gain param id on the macros node (the one remaining global macro)
const PARAM_GAIN: u32 = 2;
// Global arrangement loop region (macros-node params, not per-track)
const PARAM_LOOP_ENABLED: u32 = 3;
const PARAM_LOOP_START: u32 = 4;
const PARAM_LOOP_END: u32 = 5;
// Per-track param id bases on the macros node; each holds one value per track at
// base + track, except envelopes which hold four values at base + track*4 + i.
const PARAM_TRACK_LEVEL_BASE: u32 = 100;
const PARAM_TRACK_PAN_BASE: u32 = 200;
const PARAM_TRACK_CUTOFF_BASE: u32 = 300;
const PARAM_TRACK_RESONANCE_BASE: u32 = 310;
const PARAM_TRACK_DELAY_BASE: u32 = 320;
const PARAM_TRACK_DELAY_TIME_BASE: u32 = 330;
const PARAM_TRACK_DELAY_FEEDBACK_BASE: u32 = 340;
const PARAM_TRACK_DELAY_MIX_BASE: u32 = 350;
const PARAM_TRACK_REVERB_BASE: u32 = 360;
const PARAM_TRACK_REVERB_MIX_BASE: u32 = 370;
const PARAM_TRACK_OSC_MIX_BASE: u32 = 380;
const PARAM_TRACK_OSC_B_SEMIS_BASE: u32 = 390;
// Four ids per track: base + track*4 + i
const PARAM_TRACK_AMP_ENV_BASE: u32 = 400;
const PARAM_TRACK_FILTER_ENV_BASE: u32 = 420;
// Oscillator A coarse, A/B fine tuning, and FM index (clear of the env ranges)
const PARAM_TRACK_OSC_A_SEMIS_BASE: u32 = 440;
const PARAM_TRACK_OSC_A_CENTS_BASE: u32 = 450;
const PARAM_TRACK_OSC_B_CENTS_BASE: u32 = 460;
const PARAM_TRACK_FM_BASE: u32 = 470;
const PARAM_TRACK_VOICES_BASE: u32 = 480;
// Legacy character-FX params from the pre-chain format. Load-only.
const PARAM_TRACK_FX_ON_BASE: u32 = 600;
const PARAM_TRACK_FX_PARAM_BASE: u32 = 620;
const PARAM_TRACK_FX_CHAIN_KIND_BASE: u32 = 700;
const PARAM_TRACK_FX_CHAIN_INSTANCE_BASE: u32 = 730;
const PARAM_TRACK_FX_CHAIN_ON_BASE: u32 = 760;
const PARAM_TRACK_FX_CHAIN_PARAM_BASE: u32 = 800;
const FX_COUNT: usize = 4;
const FX_PARAMS: usize = 4;
const FX_DEFAULTS: [[f32; FX_PARAMS]; FX_COUNT] = [
    [2.0, 0.7, 1.0, 0.0],
    [0.5, 1.0, 0.5, 0.5],
    [0.3, 2.0, 0.5, 0.5],
    [0.8, 4.0, 0.5, 0.0],
];
const PARAM_TRACK_LFO_RATE_BASE: u32 = 490;
const PARAM_TRACK_LFO_DEPTH_BASE: u32 = 500;
const PARAM_TRACK_LFO_DEST_BASE: u32 = 510;

// Reserved clip id for a track's step grid, kept clear of arrangement clip ids
const STEP_CLIP_ID: u64 = u64::MAX;

// Reserved clip-id block for session-launcher slots, just below STEP_CLIP_ID.
// Scene s of a track encodes as SESSION_CLIP_ID_BASE + s.
const SESSION_CLIP_ID_BASE: u64 = u64::MAX - crate::engine::MAX_SCENES as u64 - 1;
// Fixed length of a session-launcher clip in beats (mirrors studio SESSION_CLIP_LEN)
const SESSION_SLOT_LEN_BEATS: f32 = 4.0;

// Scene index if `id` falls in the reserved session-slot block, else None
fn session_scene_of(id: u64) -> Option<u8> {
    let scenes = crate::engine::MAX_SCENES as u64;
    if id >= SESSION_CLIP_ID_BASE && id < SESSION_CLIP_ID_BASE + scenes {
        Some((id - SESSION_CLIP_ID_BASE) as u8)
    } else {
        None
    }
}

// Tick grid: musical ticks per beat, and per step (sixteenths)
const TICKS_PER_BEAT: u64 = 960;
const STEP_TICKS: u64 = TICKS_PER_BEAT / 4;

// One piano-roll note in beats, as the UI models it
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteSession {
    pub pitch: u8,
    pub start_beats: f32,
    pub len_beats: f32,
    pub velocity: f32,
}

// One placed timeline clip with its id, position, length, and clip-relative notes
#[derive(Clone, Debug, PartialEq)]
pub struct ClipSession {
    pub id: u64,
    pub start_beats: f32,
    pub len_beats: f32,
    pub notes: Vec<NoteSession>,
}

// One saved session-launcher slot: a scene index and its looping notes. Slot
// length is the fixed SESSION_SLOT_LEN_BEATS, so it is not stored per slot.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionSlotSession {
    pub scene: u8,
    pub notes: Vec<NoteSession>,
}

// One ordered character-FX instance in a saved track chain
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FxSession {
    pub kind: FxKind,
    pub instance: u8,
    pub on: bool,
    pub params: [f32; FX_PARAMS],
}

// One track's persisted state: its full instrument patch, its effects chain,
// mix flags, step gates, and clip notes. Every synth/fx macro is per-track.
#[derive(Clone, Debug, PartialEq)]
pub struct TrackSession {
    pub level: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub delay_on: bool,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub reverb_on: bool,
    pub reverb_mix: f32,
    // Oscillator A/B blend, coarse/fine pitch per osc, FM index, and voices
    pub osc_mix: f32,
    pub osc_b_semis: f32,
    pub osc_a_semis: f32,
    pub osc_a_cents: f32,
    pub osc_b_cents: f32,
    pub fm_amount: f32,
    pub polyphony: usize,
    pub lfo_rate_hz: f32,
    pub lfo_depth: f32,
    pub lfo_dest: LfoDestination,
    // Ordered duplicate-capable character effects; authoritative persisted FX state
    pub fx_chain: Vec<FxSession>,
    // Amp/filter ADSR macros [attack, decay, sustain, release]
    pub amp_env: [f32; 4],
    pub filter_env: [f32; 4],
    // Step gates that are on, as (row, step)
    pub gates: Vec<(u8, u8)>,
    // Placed timeline clips on this track
    pub clips: Vec<ClipSession>,
    // Session-launcher slots (scene + looping notes) for this track
    pub session_slots: Vec<SessionSlotSession>,
}

// The whole studio session, independent of the on-disk encoding. The transport
// tempo, master gain, and arrangement loop are global; everything else per-track.
#[derive(Clone, Debug, PartialEq)]
pub struct StudioSession {
    pub bpm: f32,
    pub gain: f32,
    pub loop_enabled: bool,
    pub loop_start_beats: f32,
    pub loop_end_beats: f32,
    pub tracks: Vec<TrackSession>,
}

impl StudioSession {
    // Encode this session as a project file
    fn to_project(&self) -> ProjectFile {
        let mut project = ProjectFile::new("Geist Studio Session");
        project.meta.tempo_bpm = self.bpm as f64;

        // Macros node: master gain, the global loop region, plus a full per-track
        // patch and fx block
        let mut params = vec![
            ParamValue {
                id: PARAM_GAIN,
                value: self.gain,
            },
            ParamValue {
                id: PARAM_LOOP_ENABLED,
                value: if self.loop_enabled { 1.0 } else { 0.0 },
            },
            ParamValue {
                id: PARAM_LOOP_START,
                value: self.loop_start_beats,
            },
            ParamValue {
                id: PARAM_LOOP_END,
                value: self.loop_end_beats,
            },
        ];
        for (track, state) in self.tracks.iter().enumerate() {
            let t = track as u32;
            params.push(ParamValue {
                id: PARAM_TRACK_LEVEL_BASE + t,
                value: state.level,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_PAN_BASE + t,
                value: state.pan,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_CUTOFF_BASE + t,
                value: state.cutoff_hz,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_RESONANCE_BASE + t,
                value: state.resonance,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_DELAY_BASE + t,
                value: bool_to_f32(state.delay_on),
            });
            params.push(ParamValue {
                id: PARAM_TRACK_DELAY_TIME_BASE + t,
                value: state.delay_time,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_DELAY_FEEDBACK_BASE + t,
                value: state.delay_feedback,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_DELAY_MIX_BASE + t,
                value: state.delay_mix,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_REVERB_BASE + t,
                value: bool_to_f32(state.reverb_on),
            });
            params.push(ParamValue {
                id: PARAM_TRACK_REVERB_MIX_BASE + t,
                value: state.reverb_mix,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_OSC_MIX_BASE + t,
                value: state.osc_mix,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_OSC_B_SEMIS_BASE + t,
                value: state.osc_b_semis,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_OSC_A_SEMIS_BASE + t,
                value: state.osc_a_semis,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_OSC_A_CENTS_BASE + t,
                value: state.osc_a_cents,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_OSC_B_CENTS_BASE + t,
                value: state.osc_b_cents,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_FM_BASE + t,
                value: state.fm_amount,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_VOICES_BASE + t,
                value: state.polyphony as f32,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_LFO_RATE_BASE + t,
                value: state.lfo_rate_hz,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_LFO_DEPTH_BASE + t,
                value: state.lfo_depth,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_LFO_DEST_BASE + t,
                value: lfo_dest_to_f32(state.lfo_dest),
            });
            for (slot, fx) in state.fx_chain.iter().take(FX_CHAIN_MAX).enumerate() {
                let s = slot as u32;
                params.push(ParamValue {
                    id: PARAM_TRACK_FX_CHAIN_KIND_BASE + t * FX_CHAIN_MAX as u32 + s,
                    value: fx_kind_to_f32(fx.kind),
                });
                params.push(ParamValue {
                    id: PARAM_TRACK_FX_CHAIN_INSTANCE_BASE + t * FX_CHAIN_MAX as u32 + s,
                    value: fx.instance as f32,
                });
                params.push(ParamValue {
                    id: PARAM_TRACK_FX_CHAIN_ON_BASE + t * FX_CHAIN_MAX as u32 + s,
                    value: bool_to_f32(fx.on),
                });
                for p in 0..FX_PARAMS {
                    params.push(ParamValue {
                        id: PARAM_TRACK_FX_CHAIN_PARAM_BASE
                            + t * (FX_CHAIN_MAX * FX_PARAMS) as u32
                            + (slot * FX_PARAMS) as u32
                            + p as u32,
                        value: fx.params[p],
                    });
                }
            }
            for (i, &value) in state.amp_env.iter().enumerate() {
                params.push(ParamValue {
                    id: PARAM_TRACK_AMP_ENV_BASE + t * 4 + i as u32,
                    value,
                });
            }
            for (i, &value) in state.filter_env.iter().enumerate() {
                params.push(ParamValue {
                    id: PARAM_TRACK_FILTER_ENV_BASE + t * 4 + i as u32,
                    value,
                });
            }
        }
        project.graph.nodes.push(NodeEntry {
            id: 0,
            kind: MACROS_NODE_KIND.to_string(),
            params,
            state_blob: Vec::new(),
        });

        // One track entry per track: mix flags, the step grid, and timeline clips
        for (index, state) in self.tracks.iter().enumerate() {
            let base = base_midi(index);
            let step_notes = state
                .gates
                .iter()
                .map(|&(row, step)| NoteEntry {
                    pitch: base.saturating_add(row),
                    velocity: 100,
                    start_ticks: step as u64 * STEP_TICKS,
                    length_ticks: STEP_TICKS,
                    channel: 0,
                })
                .collect();
            let mut clips = vec![ClipEntry {
                id: STEP_CLIP_ID,
                start_ticks: 0,
                length_ticks: 0,
                kind: ClipKind::Midi { notes: step_notes },
            }];
            for clip in &state.clips {
                let notes = clip
                    .notes
                    .iter()
                    .map(|note| NoteEntry {
                        pitch: note.pitch,
                        velocity: vel_to_u8(note.velocity),
                        start_ticks: beats_to_ticks(note.start_beats),
                        length_ticks: beats_to_ticks(note.len_beats),
                        channel: 0,
                    })
                    .collect();
                clips.push(ClipEntry {
                    id: clip.id,
                    start_ticks: beats_to_ticks(clip.start_beats),
                    length_ticks: beats_to_ticks(clip.len_beats),
                    kind: ClipKind::Midi { notes },
                });
            }
            // Session-launcher slots ride along as reserved-id MIDI clips
            for slot in &state.session_slots {
                if slot.scene as usize >= crate::engine::MAX_SCENES {
                    continue;
                }
                let notes = slot
                    .notes
                    .iter()
                    .map(|note| NoteEntry {
                        pitch: note.pitch,
                        velocity: vel_to_u8(note.velocity),
                        start_ticks: beats_to_ticks(note.start_beats),
                        length_ticks: beats_to_ticks(note.len_beats),
                        channel: 0,
                    })
                    .collect();
                clips.push(ClipEntry {
                    id: SESSION_CLIP_ID_BASE + slot.scene as u64,
                    start_ticks: 0,
                    length_ticks: beats_to_ticks(SESSION_SLOT_LEN_BEATS),
                    kind: ClipKind::Midi { notes },
                });
            }
            project.tracks.push(TrackEntry {
                id: index as u64,
                name: format!("Track {}", index + 1),
                muted: state.muted,
                soloed: state.soloed,
                clips,
            });
        }
        project
    }

    // Decode a session from a project file, falling back to `defaults` per field
    fn from_project(project: &ProjectFile, defaults: &StudioSession) -> StudioSession {
        let mut session = defaults.clone();
        session.bpm = project.meta.tempo_bpm as f32;

        let macros = project
            .graph
            .nodes
            .iter()
            .find(|node| node.kind == MACROS_NODE_KIND);
        if let Some(node) = macros {
            for param in &node.params {
                if param.id == PARAM_GAIN {
                    session.gain = param.value;
                }
                if param.id == PARAM_LOOP_ENABLED {
                    session.loop_enabled = param.value >= 0.5;
                }
                if param.id == PARAM_LOOP_START {
                    session.loop_start_beats = param.value;
                }
                if param.id == PARAM_LOOP_END {
                    session.loop_end_beats = param.value;
                }
            }
        }

        for entry in &project.tracks {
            let index = entry.id as usize;
            let Some(state) = session.tracks.get_mut(index) else {
                continue;
            };
            state.muted = entry.muted;
            state.soloed = entry.soloed;
            if let Some(node) = macros {
                let t = index as u32;
                let read = |base: u32| {
                    node.params
                        .iter()
                        .find(|p| p.id == base + t)
                        .map(|p| p.value)
                };
                if let Some(v) = read(PARAM_TRACK_LEVEL_BASE) {
                    state.level = v;
                }
                if let Some(v) = read(PARAM_TRACK_PAN_BASE) {
                    state.pan = v;
                }
                if let Some(v) = read(PARAM_TRACK_CUTOFF_BASE) {
                    state.cutoff_hz = v;
                }
                if let Some(v) = read(PARAM_TRACK_RESONANCE_BASE) {
                    state.resonance = v;
                }
                if let Some(v) = read(PARAM_TRACK_DELAY_BASE) {
                    state.delay_on = v >= 0.5;
                }
                if let Some(v) = read(PARAM_TRACK_DELAY_TIME_BASE) {
                    state.delay_time = v;
                }
                if let Some(v) = read(PARAM_TRACK_DELAY_FEEDBACK_BASE) {
                    state.delay_feedback = v;
                }
                if let Some(v) = read(PARAM_TRACK_DELAY_MIX_BASE) {
                    state.delay_mix = v;
                }
                if let Some(v) = read(PARAM_TRACK_REVERB_BASE) {
                    state.reverb_on = v >= 0.5;
                }
                if let Some(v) = read(PARAM_TRACK_REVERB_MIX_BASE) {
                    state.reverb_mix = v;
                }
                if let Some(v) = read(PARAM_TRACK_OSC_MIX_BASE) {
                    state.osc_mix = v;
                }
                if let Some(v) = read(PARAM_TRACK_OSC_B_SEMIS_BASE) {
                    state.osc_b_semis = v;
                }
                if let Some(v) = read(PARAM_TRACK_OSC_A_SEMIS_BASE) {
                    state.osc_a_semis = v;
                }
                if let Some(v) = read(PARAM_TRACK_OSC_A_CENTS_BASE) {
                    state.osc_a_cents = v;
                }
                if let Some(v) = read(PARAM_TRACK_OSC_B_CENTS_BASE) {
                    state.osc_b_cents = v;
                }
                if let Some(v) = read(PARAM_TRACK_FM_BASE) {
                    state.fm_amount = v;
                }
                if let Some(v) = read(PARAM_TRACK_VOICES_BASE) {
                    state.polyphony = v.round().max(1.0) as usize;
                }
                if let Some(v) = read(PARAM_TRACK_LFO_RATE_BASE) {
                    state.lfo_rate_hz = v;
                }
                if let Some(v) = read(PARAM_TRACK_LFO_DEPTH_BASE) {
                    state.lfo_depth = v;
                }
                if let Some(v) = read(PARAM_TRACK_LFO_DEST_BASE) {
                    state.lfo_dest = lfo_dest_from_f32(v);
                }
                let mut legacy_fx_on = [false; FX_COUNT];
                let mut legacy_fx_param = FX_DEFAULTS;
                let mut legacy_fx_seen = false;
                for fx in 0..FX_COUNT {
                    let on_id = PARAM_TRACK_FX_ON_BASE + t * FX_COUNT as u32 + fx as u32;
                    if let Some(param) = node.params.iter().find(|p| p.id == on_id) {
                        legacy_fx_on[fx] = param.value >= 0.5;
                        legacy_fx_seen = true;
                    }
                    for (p, value) in legacy_fx_param[fx].iter_mut().enumerate() {
                        let id = PARAM_TRACK_FX_PARAM_BASE
                            + t * (FX_COUNT * FX_PARAMS) as u32
                            + (fx * FX_PARAMS) as u32
                            + p as u32;
                        if let Some(param) = node.params.iter().find(|p2| p2.id == id) {
                            *value = param.value;
                            legacy_fx_seen = true;
                        }
                    }
                }
                let mut chain = Vec::new();
                for slot in 0..FX_CHAIN_MAX {
                    let s = slot as u32;
                    let kind_id = PARAM_TRACK_FX_CHAIN_KIND_BASE + t * FX_CHAIN_MAX as u32 + s;
                    let Some(kind) = node
                        .params
                        .iter()
                        .find(|p| p.id == kind_id)
                        .map(|p| fx_kind_from_f32(p.value))
                    else {
                        continue;
                    };
                    let instance_id =
                        PARAM_TRACK_FX_CHAIN_INSTANCE_BASE + t * FX_CHAIN_MAX as u32 + s;
                    let on_id = PARAM_TRACK_FX_CHAIN_ON_BASE + t * FX_CHAIN_MAX as u32 + s;
                    let instance = node
                        .params
                        .iter()
                        .find(|p| p.id == instance_id)
                        .map(|p| p.value.round().clamp(0.0, u8::MAX as f32) as u8)
                        .unwrap_or(0);
                    let on = node
                        .params
                        .iter()
                        .find(|p| p.id == on_id)
                        .map(|p| p.value >= 0.5)
                        .unwrap_or(false);
                    let mut params = [0.0; FX_PARAMS];
                    for (p, value) in params.iter_mut().enumerate() {
                        let id = PARAM_TRACK_FX_CHAIN_PARAM_BASE
                            + t * (FX_CHAIN_MAX * FX_PARAMS) as u32
                            + (slot * FX_PARAMS) as u32
                            + p as u32;
                        if let Some(param) = node.params.iter().find(|p2| p2.id == id) {
                            *value = param.value;
                        }
                    }
                    chain.push(FxSession {
                        kind,
                        instance,
                        on,
                        params,
                    });
                }
                if !chain.is_empty() {
                    state.fx_chain = chain;
                } else if legacy_fx_seen {
                    apply_legacy_fx_to_chain(&mut state.fx_chain, legacy_fx_on, legacy_fx_param);
                }
                for i in 0..4u32 {
                    let amp_id = PARAM_TRACK_AMP_ENV_BASE + t * 4 + i;
                    if let Some(param) = node.params.iter().find(|p| p.id == amp_id) {
                        state.amp_env[i as usize] = param.value;
                    }
                    let flt_id = PARAM_TRACK_FILTER_ENV_BASE + t * 4 + i;
                    if let Some(param) = node.params.iter().find(|p| p.id == flt_id) {
                        state.filter_env[i as usize] = param.value;
                    }
                }
            }
            state.clips.clear();
            state.gates.clear();
            state.session_slots.clear();
            let base = base_midi(index);
            for clip in &entry.clips {
                let ClipKind::Midi { notes } = &clip.kind else {
                    continue;
                };
                if clip.id == STEP_CLIP_ID {
                    for note in notes {
                        let row = note.pitch.saturating_sub(base);
                        let step = (note.start_ticks / STEP_TICKS) as u8;
                        if (row as usize) < SEQ_ROWS && (step as usize) < SEQ_STEPS {
                            state.gates.push((row, step));
                        }
                    }
                } else if let Some(scene) = session_scene_of(clip.id) {
                    let slot_notes = notes
                        .iter()
                        .map(|note| NoteSession {
                            pitch: note.pitch,
                            start_beats: ticks_to_beats(note.start_ticks),
                            len_beats: ticks_to_beats(note.length_ticks),
                            velocity: vel_to_f32(note.velocity),
                        })
                        .collect();
                    state.session_slots.push(SessionSlotSession {
                        scene,
                        notes: slot_notes,
                    });
                } else {
                    let clip_notes = notes
                        .iter()
                        .map(|note| NoteSession {
                            pitch: note.pitch,
                            start_beats: ticks_to_beats(note.start_ticks),
                            len_beats: ticks_to_beats(note.length_ticks),
                            velocity: vel_to_f32(note.velocity),
                        })
                        .collect();
                    state.clips.push(ClipSession {
                        id: clip.id,
                        start_beats: ticks_to_beats(clip.start_ticks),
                        len_beats: ticks_to_beats(clip.length_ticks),
                        notes: clip_notes,
                    });
                }
            }
        }
        session
    }
}

// Base MIDI note for a track's step rows, clamped to the known tracks
fn base_midi(track: usize) -> u8 {
    TRACK_BASE_MIDI
        .get(track)
        .copied()
        .unwrap_or(TRACK_BASE_MIDI[NUM_TRACKS - 1])
}

fn beats_to_ticks(beats: f32) -> u64 {
    (beats.max(0.0) * TICKS_PER_BEAT as f32).round() as u64
}

fn ticks_to_beats(ticks: u64) -> f32 {
    ticks as f32 / TICKS_PER_BEAT as f32
}

fn vel_to_u8(velocity: f32) -> u8 {
    (velocity.clamp(0.0, 1.0) * 127.0).round() as u8
}

fn vel_to_f32(velocity: u8) -> f32 {
    velocity as f32 / 127.0
}

fn bool_to_f32(flag: bool) -> f32 {
    if flag {
        1.0
    } else {
        0.0
    }
}

fn lfo_dest_to_f32(dest: LfoDestination) -> f32 {
    match dest {
        LfoDestination::Cutoff => 0.0,
        LfoDestination::Pitch => 1.0,
        LfoDestination::Fm => 2.0,
    }
}

fn lfo_dest_from_f32(value: f32) -> LfoDestination {
    match value.round() as i32 {
        1 => LfoDestination::Pitch,
        2 => LfoDestination::Fm,
        _ => LfoDestination::Cutoff,
    }
}

fn fx_kind_to_f32(kind: FxKind) -> f32 {
    match kind {
        FxKind::Distortion => 0.0,
        FxKind::Phaser => 1.0,
        FxKind::Flanger => 2.0,
        FxKind::Chorus => 3.0,
        FxKind::Eq => 4.0,
        FxKind::Saturator => 5.0,
    }
}

fn fx_kind_from_f32(value: f32) -> FxKind {
    match value.round() as i32 {
        1 => FxKind::Phaser,
        2 => FxKind::Flanger,
        3 => FxKind::Chorus,
        4 => FxKind::Eq,
        5 => FxKind::Saturator,
        _ => FxKind::Distortion,
    }
}

#[cfg(test)]
fn default_fx_chain() -> Vec<FxSession> {
    [
        (FxKind::Distortion, FX_DEFAULTS[0]),
        (FxKind::Phaser, FX_DEFAULTS[1]),
        (FxKind::Flanger, FX_DEFAULTS[2]),
        (FxKind::Chorus, FX_DEFAULTS[3]),
    ]
    .into_iter()
    .map(|(kind, params)| FxSession {
        kind,
        instance: 0,
        on: false,
        params,
    })
    .collect()
}

fn apply_legacy_fx_to_chain(
    chain: &mut [FxSession],
    legacy_on: [bool; FX_COUNT],
    legacy_params: [[f32; FX_PARAMS]; FX_COUNT],
) {
    for fx in chain.iter_mut().filter(|fx| fx.instance == 0) {
        let index = match fx.kind {
            FxKind::Distortion => 0,
            FxKind::Phaser => 1,
            FxKind::Flanger => 2,
            FxKind::Chorus => 3,
            // EQ and Saturator never existed in the legacy per-kind format
            FxKind::Eq | FxKind::Saturator => continue,
        };
        fx.on = legacy_on[index];
        fx.params = legacy_params[index];
    }
}

// Path to the studio session slot, in the home directory when available
pub fn session_path() -> PathBuf {
    let dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join(SESSION_FILE)
}

// Directory for recorded audio takes, created on demand beside the session
pub fn recordings_dir() -> PathBuf {
    let dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("geist-recordings");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

// Write a session to an explicit path
pub fn save_to(session: &StudioSession, path: &Path) -> Result<(), ProjectError> {
    save_to_path(&session.to_project(), path)
}

// Read a session from an explicit path, falling back to `defaults` per field
pub fn load_from(defaults: &StudioSession, path: &Path) -> Result<StudioSession, ProjectError> {
    let project = load_from_path(path)?;
    Ok(StudioSession::from_project(&project, defaults))
}

// Save to the studio slot, returning the written path
pub fn save(session: &StudioSession) -> Result<PathBuf, ProjectError> {
    let path = session_path();
    save_to(session, &path)?;
    Ok(path)
}

// Load from the studio slot, falling back to `defaults` per field
pub fn load(defaults: &StudioSession) -> Result<StudioSession, ProjectError> {
    load_from(defaults, &session_path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::DEFAULT_VOICES;

    fn track_defaults() -> TrackSession {
        TrackSession {
            level: 0.8,
            pan: 0.0,
            muted: false,
            soloed: false,
            cutoff_hz: 1_500.0,
            resonance: 0.9,
            delay_on: false,
            delay_time: 0.25,
            delay_feedback: 0.3,
            delay_mix: 0.3,
            reverb_on: false,
            reverb_mix: 0.3,
            osc_mix: 0.5,
            osc_b_semis: 0.0,
            osc_a_semis: 0.0,
            osc_a_cents: 0.0,
            osc_b_cents: 0.0,
            fm_amount: 0.0,
            polyphony: DEFAULT_VOICES,
            lfo_rate_hz: 2.0,
            lfo_depth: 0.0,
            lfo_dest: LfoDestination::Cutoff,
            fx_chain: default_fx_chain(),
            amp_env: [0.005, 0.1, 0.8, 0.3],
            filter_env: [0.01, 0.2, 0.3, 0.3],
            gates: Vec::new(),
            clips: Vec::new(),
            session_slots: Vec::new(),
        }
    }

    fn defaults() -> StudioSession {
        StudioSession {
            bpm: 120.0,
            gain: 1.0,
            loop_enabled: false,
            loop_start_beats: 0.0,
            loop_end_beats: 16.0,
            tracks: (0..NUM_TRACKS).map(|_| track_defaults()).collect(),
        }
    }

    fn sample() -> StudioSession {
        let mut s = defaults();
        s.bpm = 140.0;
        s.gain = 0.7;
        s.loop_enabled = true;
        s.loop_start_beats = 4.0;
        s.loop_end_beats = 12.0;
        // Each track gets a distinct patch + fx to prove per-track persistence
        s.tracks[0].level = 0.5;
        s.tracks[0].pan = -0.5;
        s.tracks[0].muted = true;
        s.tracks[0].cutoff_hz = 820.0;
        s.tracks[0].resonance = 2.0;
        s.tracks[0].delay_on = true;
        s.tracks[0].delay_time = 0.4;
        s.tracks[0].delay_feedback = 0.55;
        s.tracks[0].delay_mix = 0.45;
        s.tracks[0].amp_env = [0.02, 0.3, 0.6, 1.2];
        s.tracks[1].pan = 0.75;
        s.tracks[1].soloed = true;
        s.tracks[1].reverb_on = true;
        s.tracks[1].reverb_mix = 0.6;
        s.tracks[1].osc_mix = 0.8;
        s.tracks[1].osc_b_semis = -12.0;
        s.tracks[1].osc_a_semis = 7.0;
        s.tracks[1].osc_a_cents = -15.0;
        s.tracks[1].osc_b_cents = 22.0;
        s.tracks[1].fm_amount = 1.5;
        s.tracks[1].polyphony = 6;
        s.tracks[1].lfo_rate_hz = 5.5;
        s.tracks[1].lfo_depth = 0.75;
        s.tracks[1].lfo_dest = LfoDestination::Fm;
        // Duplicate-capable chain is the authoritative character-FX state.
        s.tracks[1].fx_chain = vec![
            FxSession {
                kind: FxKind::Distortion,
                instance: 0,
                on: true,
                params: [6.0, 0.4, 0.8, 0.0],
            },
            FxSession {
                kind: FxKind::Distortion,
                instance: 1,
                on: true,
                params: [12.0, 0.2, 0.5, 0.0],
            },
            FxSession {
                kind: FxKind::Chorus,
                instance: 0,
                on: true,
                params: [1.2, 5.0, 0.6, 0.0],
            },
        ];
        s.tracks[1].filter_env = [0.05, 0.4, 0.2, 0.8];
        s.tracks[1].gates = vec![(0, 0), (2, 4), (5, 12)];
        // Velocity 1.0 maps cleanly through the 0..127 MIDI range
        s.tracks[2].clips = vec![
            ClipSession {
                id: 1,
                start_beats: 0.0,
                len_beats: 8.0,
                notes: vec![
                    NoteSession {
                        pitch: 60,
                        start_beats: 0.0,
                        len_beats: 1.0,
                        velocity: 1.0,
                    },
                    NoteSession {
                        pitch: 67,
                        start_beats: 2.5,
                        len_beats: 0.5,
                        velocity: 1.0,
                    },
                ],
            },
            ClipSession {
                id: 2,
                start_beats: 16.0,
                len_beats: 4.0,
                notes: vec![NoteSession {
                    pitch: 72,
                    start_beats: 1.0,
                    len_beats: 2.0,
                    velocity: 1.0,
                }],
            },
        ];
        // Session-launcher slots: one empty (created, no notes) and one with notes
        s.tracks[0].session_slots = vec![
            SessionSlotSession {
                scene: 0,
                notes: vec![
                    NoteSession {
                        pitch: 48,
                        start_beats: 0.0,
                        len_beats: 1.0,
                        velocity: 1.0,
                    },
                    NoteSession {
                        pitch: 55,
                        start_beats: 2.0,
                        len_beats: 0.5,
                        velocity: 1.0,
                    },
                ],
            },
            SessionSlotSession {
                scene: 3,
                notes: Vec::new(),
            },
        ];
        s
    }

    #[test]
    fn session_round_trips_through_a_project_file() {
        let path = std::env::temp_dir().join(format!("geist-studio-{}.gproj", std::process::id()));
        let session = sample();
        save_to(&session, &path).unwrap();
        let loaded = load_from(&defaults(), &path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(loaded, session);
    }

    #[test]
    fn session_launcher_slots_round_trip() {
        let session = sample();
        let project = session.to_project();
        let loaded = StudioSession::from_project(&project, &defaults());
        // Track 0 had a slot at scene 0 (two notes) and an empty slot at scene 3
        let slots = &loaded.tracks[0].session_slots;
        assert_eq!(slots.len(), 2, "both created slots must persist");
        assert_eq!(slots[0].scene, 0);
        assert_eq!(slots[0].notes.len(), 2, "slot notes must persist");
        assert_eq!(slots[1].scene, 3);
        assert!(
            slots[1].notes.is_empty(),
            "an empty-but-created slot still persists"
        );
        // Session slots must not leak into arrangement clips or other tracks
        assert!(loaded.tracks[1].session_slots.is_empty());
        assert!(loaded.tracks[0].clips.is_empty());
    }

    #[test]
    fn arrangement_loop_round_trips() {
        let session = sample();
        let project = session.to_project();
        let loaded = StudioSession::from_project(&project, &defaults());
        assert!(loaded.loop_enabled, "loop enable must persist");
        assert_eq!(loaded.loop_start_beats, 4.0);
        assert_eq!(loaded.loop_end_beats, 12.0);
    }

    #[test]
    fn project_persists_character_fx_chain_without_legacy_parallel_state() {
        let session = sample();
        let project = session.to_project();
        let macros = project
            .graph
            .nodes
            .iter()
            .find(|node| node.kind == MACROS_NODE_KIND)
            .expect("macros node");

        assert!(
            macros.params.iter().any(|param| {
                (PARAM_TRACK_FX_CHAIN_KIND_BASE..PARAM_TRACK_FX_CHAIN_INSTANCE_BASE)
                    .contains(&param.id)
            }),
            "chain slot params should be persisted"
        );
        assert!(
            !macros.params.iter().any(|param| {
                (PARAM_TRACK_FX_ON_BASE..PARAM_TRACK_FX_PARAM_BASE).contains(&param.id)
                    || (PARAM_TRACK_FX_PARAM_BASE..PARAM_TRACK_FX_CHAIN_KIND_BASE)
                        .contains(&param.id)
            }),
            "legacy per-kind fx_on/fx_param params must not be written alongside fx_chain"
        );
    }

    #[test]
    fn legacy_per_kind_character_fx_params_load_into_default_chain() {
        let defaults = defaults();
        let mut project = defaults.to_project();
        let macros = project
            .graph
            .nodes
            .iter_mut()
            .find(|node| node.kind == MACROS_NODE_KIND)
            .expect("macros node");
        macros.params.retain(|param| {
            !(PARAM_TRACK_FX_CHAIN_KIND_BASE..PARAM_TRACK_FX_CHAIN_PARAM_BASE + 64)
                .contains(&param.id)
        });
        let track = 1u32;
        macros.params.push(ParamValue {
            id: PARAM_TRACK_FX_ON_BASE + track * FX_COUNT as u32,
            value: 1.0,
        });
        for (param, value) in [6.0, 0.4, 0.8, 0.0].into_iter().enumerate() {
            macros.params.push(ParamValue {
                id: PARAM_TRACK_FX_PARAM_BASE
                    + track * (FX_COUNT * FX_PARAMS) as u32
                    + param as u32,
                value,
            });
        }

        let loaded = StudioSession::from_project(&project, &defaults);
        let distortion = loaded.tracks[track as usize]
            .fx_chain
            .iter()
            .find(|fx| fx.kind == FxKind::Distortion && fx.instance == 0)
            .expect("default distortion slot");

        assert!(distortion.on);
        assert_eq!(distortion.params, [6.0, 0.4, 0.8, 0.0]);
    }

    #[test]
    fn missing_file_is_an_error_not_a_panic() {
        let path = PathBuf::from("/no/such/dir/geist-studio-missing.gproj");
        assert!(load_from(&defaults(), &path).is_err());
    }
}
