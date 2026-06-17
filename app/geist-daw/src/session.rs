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

use crate::engine::{NUM_TRACKS, SEQ_ROWS, SEQ_STEPS, TRACK_BASE_MIDI};

// Studio session slot filename, distinct from the classic patch slot
const SESSION_FILE: &str = "geist-studio.gproj";
// Node tag carrying the global macros and per-track levels
const MACROS_NODE_KIND: &str = "geist-macros";

// Macro parameter ids on the macros node
const PARAM_CUTOFF: u32 = 0;
const PARAM_RESONANCE: u32 = 1;
const PARAM_GAIN: u32 = 2;
const PARAM_DELAY: u32 = 5;
const PARAM_REVERB: u32 = 6;
const PARAM_REVERB_MIX: u32 = 7;
// Amp ADSR param ids (attack/decay/sustain/release)
const PARAM_AMP_ENV_BASE: u32 = 8;
// Filter ADSR param ids (attack/decay/sustain/release)
const PARAM_FILTER_ENV_BASE: u32 = 12;
// Delay param ids
const PARAM_DELAY_TIME: u32 = 16;
const PARAM_DELAY_FEEDBACK: u32 = 17;
const PARAM_DELAY_MIX: u32 = 18;
// Oscillator param ids
const PARAM_OSC_MIX: u32 = 19;
const PARAM_OSC_B_SEMIS: u32 = 20;
// Per-track level param ids start here, one per track
const PARAM_TRACK_LEVEL_BASE: u32 = 100;
// Per-track pan param ids start here, one per track
const PARAM_TRACK_PAN_BASE: u32 = 200;

// Clip ids distinguishing the piano-roll clip from the step-grid clip
const PIANO_CLIP_ID: u64 = 0;
const STEP_CLIP_ID: u64 = 1;

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

// One track's persisted state: mix flags, level, step gates, and clip notes
#[derive(Clone, Debug, PartialEq)]
pub struct TrackSession {
    pub level: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    // Step gates that are on, as (row, step)
    pub gates: Vec<(u8, u8)>,
    pub notes: Vec<NoteSession>,
}

// The whole studio session, independent of the on-disk encoding
#[derive(Clone, Debug, PartialEq)]
pub struct StudioSession {
    pub bpm: f32,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub gain: f32,
    pub delay_on: bool,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub reverb_on: bool,
    pub reverb_mix: f32,
    // Oscillator A/B blend and osc B pitch offset
    pub osc_mix: f32,
    pub osc_b_semis: f32,
    // Amp/filter ADSR macros [attack, decay, sustain, release]
    pub amp_env: [f32; 4],
    pub filter_env: [f32; 4],
    pub tracks: Vec<TrackSession>,
}

impl StudioSession {
    // Encode this session as a project file
    fn to_project(&self) -> ProjectFile {
        let mut project = ProjectFile::new("Geist Studio Session");
        project.meta.tempo_bpm = self.bpm as f64;

        // Macros node: global filter/fx plus a level param per track
        let mut params = vec![
            ParamValue { id: PARAM_CUTOFF, value: self.cutoff_hz },
            ParamValue { id: PARAM_RESONANCE, value: self.resonance },
            ParamValue { id: PARAM_GAIN, value: self.gain },
            ParamValue { id: PARAM_DELAY, value: bool_to_f32(self.delay_on) },
            ParamValue { id: PARAM_DELAY_TIME, value: self.delay_time },
            ParamValue { id: PARAM_DELAY_FEEDBACK, value: self.delay_feedback },
            ParamValue { id: PARAM_DELAY_MIX, value: self.delay_mix },
            ParamValue { id: PARAM_REVERB, value: bool_to_f32(self.reverb_on) },
            ParamValue { id: PARAM_REVERB_MIX, value: self.reverb_mix },
            ParamValue { id: PARAM_OSC_MIX, value: self.osc_mix },
            ParamValue { id: PARAM_OSC_B_SEMIS, value: self.osc_b_semis },
        ];
        for (i, &value) in self.amp_env.iter().enumerate() {
            params.push(ParamValue { id: PARAM_AMP_ENV_BASE + i as u32, value });
        }
        for (i, &value) in self.filter_env.iter().enumerate() {
            params.push(ParamValue { id: PARAM_FILTER_ENV_BASE + i as u32, value });
        }
        for (track, state) in self.tracks.iter().enumerate() {
            params.push(ParamValue {
                id: PARAM_TRACK_LEVEL_BASE + track as u32,
                value: state.level,
            });
            params.push(ParamValue {
                id: PARAM_TRACK_PAN_BASE + track as u32,
                value: state.pan,
            });
        }
        project.graph.nodes.push(NodeEntry {
            id: 0,
            kind: MACROS_NODE_KIND.to_string(),
            params,
            state_blob: Vec::new(),
        });

        // One track entry per track: mix flags plus piano and step clips
        for (index, state) in self.tracks.iter().enumerate() {
            let base = base_midi(index);
            let piano_notes = state
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
            project.tracks.push(TrackEntry {
                id: index as u64,
                name: format!("Track {}", index + 1),
                muted: state.muted,
                soloed: state.soloed,
                clips: vec![
                    ClipEntry {
                        id: PIANO_CLIP_ID,
                        start_ticks: 0,
                        length_ticks: 0,
                        kind: ClipKind::Midi { notes: piano_notes },
                    },
                    ClipEntry {
                        id: STEP_CLIP_ID,
                        start_ticks: 0,
                        length_ticks: 0,
                        kind: ClipKind::Midi { notes: step_notes },
                    },
                ],
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
                match param.id {
                    PARAM_CUTOFF => session.cutoff_hz = param.value,
                    PARAM_RESONANCE => session.resonance = param.value,
                    PARAM_GAIN => session.gain = param.value,
                    PARAM_DELAY => session.delay_on = param.value >= 0.5,
                    PARAM_DELAY_TIME => session.delay_time = param.value,
                    PARAM_DELAY_FEEDBACK => session.delay_feedback = param.value,
                    PARAM_DELAY_MIX => session.delay_mix = param.value,
                    PARAM_REVERB => session.reverb_on = param.value >= 0.5,
                    PARAM_REVERB_MIX => session.reverb_mix = param.value,
                    PARAM_OSC_MIX => session.osc_mix = param.value,
                    PARAM_OSC_B_SEMIS => session.osc_b_semis = param.value,
                    id if (PARAM_AMP_ENV_BASE..PARAM_AMP_ENV_BASE + 4).contains(&id) => {
                        session.amp_env[(id - PARAM_AMP_ENV_BASE) as usize] = param.value;
                    }
                    id if (PARAM_FILTER_ENV_BASE..PARAM_FILTER_ENV_BASE + 4).contains(&id) => {
                        session.filter_env[(id - PARAM_FILTER_ENV_BASE) as usize] = param.value;
                    }
                    _ => {}
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
                let level_id = PARAM_TRACK_LEVEL_BASE + index as u32;
                if let Some(param) = node.params.iter().find(|p| p.id == level_id) {
                    state.level = param.value;
                }
                let pan_id = PARAM_TRACK_PAN_BASE + index as u32;
                if let Some(param) = node.params.iter().find(|p| p.id == pan_id) {
                    state.pan = param.value;
                }
            }
            state.notes.clear();
            state.gates.clear();
            let base = base_midi(index);
            for clip in &entry.clips {
                let ClipKind::Midi { notes } = &clip.kind else {
                    continue;
                };
                match clip.id {
                    PIANO_CLIP_ID => {
                        for note in notes {
                            state.notes.push(NoteSession {
                                pitch: note.pitch,
                                start_beats: ticks_to_beats(note.start_ticks),
                                len_beats: ticks_to_beats(note.length_ticks),
                                velocity: vel_to_f32(note.velocity),
                            });
                        }
                    }
                    STEP_CLIP_ID => {
                        for note in notes {
                            let row = note.pitch.saturating_sub(base);
                            let step = (note.start_ticks / STEP_TICKS) as u8;
                            if (row as usize) < SEQ_ROWS && (step as usize) < SEQ_STEPS {
                                state.gates.push((row, step));
                            }
                        }
                    }
                    _ => {}
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

// Path to the studio session slot, in the home directory when available
pub fn session_path() -> PathBuf {
    let dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join(SESSION_FILE)
}

// Write a session to an explicit path
pub fn save_to(session: &StudioSession, path: &Path) -> Result<(), ProjectError> {
    save_to_path(&session.to_project(), path)
}

// Read a session from an explicit path, falling back to `defaults` per field
pub fn load_from(
    defaults: &StudioSession,
    path: &Path,
) -> Result<StudioSession, ProjectError> {
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

    fn defaults() -> StudioSession {
        StudioSession {
            bpm: 120.0,
            cutoff_hz: 1_500.0,
            resonance: 0.9,
            gain: 1.0,
            delay_on: false,
            delay_time: 0.25,
            delay_feedback: 0.3,
            delay_mix: 0.3,
            reverb_on: false,
            reverb_mix: 0.3,
            osc_mix: 0.5,
            osc_b_semis: 0.0,
            amp_env: [0.005, 0.1, 0.8, 0.3],
            filter_env: [0.01, 0.2, 0.3, 0.3],
            tracks: (0..NUM_TRACKS)
                .map(|_| TrackSession {
                    level: 0.8,
                    pan: 0.0,
                    muted: false,
                    soloed: false,
                    gates: Vec::new(),
                    notes: Vec::new(),
                })
                .collect(),
        }
    }

    fn sample() -> StudioSession {
        let mut s = defaults();
        s.bpm = 140.0;
        s.cutoff_hz = 820.0;
        s.resonance = 2.0;
        s.gain = 0.7;
        s.delay_on = true;
        s.delay_time = 0.4;
        s.delay_feedback = 0.55;
        s.delay_mix = 0.45;
        s.reverb_on = true;
        s.reverb_mix = 0.6;
        s.osc_mix = 0.8;
        s.osc_b_semis = -12.0;
        s.amp_env = [0.02, 0.3, 0.6, 1.2];
        s.filter_env = [0.05, 0.4, 0.2, 0.8];
        s.tracks[0].level = 0.5;
        s.tracks[0].pan = -0.5;
        s.tracks[0].muted = true;
        s.tracks[1].pan = 0.75;
        s.tracks[1].soloed = true;
        s.tracks[1].gates = vec![(0, 0), (2, 4), (5, 12)];
        // Velocity 1.0 maps cleanly through the 0..127 MIDI range
        s.tracks[2].notes = vec![
            NoteSession { pitch: 60, start_beats: 0.0, len_beats: 1.0, velocity: 1.0 },
            NoteSession { pitch: 67, start_beats: 2.5, len_beats: 0.5, velocity: 1.0 },
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
    fn missing_file_is_an_error_not_a_panic() {
        let path = PathBuf::from("/no/such/dir/geist-studio-missing.gproj");
        assert!(load_from(&defaults(), &path).is_err());
    }
}
