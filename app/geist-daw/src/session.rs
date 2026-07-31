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
    hash_bytes, load_from_path, save_to_path, AssetMap, AssetRef, ClipEntry, ClipKind, NodeEntry,
    NoteEntry, ParamValue, ProjectError, ProjectFile, TrackEntry,
};

use crate::engine::{NUM_TRACKS, SEQ_ROWS, SEQ_STEPS, TRACK_BASE_MIDI};

// Studio session slot filename, distinct from the classic patch slot
const SESSION_FILE: &str = "geist-studio.gproj";
// Node tag carrying the global macros and per-track levels
const MACROS_NODE_KIND: &str = "geist-macros";

// Master gain param id on the macros node (the one remaining global macro)
const PARAM_GAIN: u32 = 2;
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

// Reserved clip id for a track's step grid, kept clear of arrangement clip ids
const STEP_CLIP_ID: u64 = u64::MAX;

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

// One placed audio clip: its id, position, length, and the absolute path to the
// recorded take. On disk the take is referenced by a project asset (relative
// path plus content hash); the runtime keeps the resolved absolute path.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioClipSession {
    pub id: u64,
    pub start_beats: f32,
    pub len_beats: f32,
    pub wav_path: PathBuf,
    // True only when the file exists and matches its persisted content hash
    pub verified: bool,
    // Original persisted reference retained so offline clips can be saved losslessly
    pub asset_ref: Option<AssetRef>,
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
    // Oscillator A/B blend and osc B pitch offset
    pub osc_mix: f32,
    pub osc_b_semis: f32,
    // Amp/filter ADSR macros [attack, decay, sustain, release]
    pub amp_env: [f32; 4],
    pub filter_env: [f32; 4],
    // Step gates that are on, as (row, step)
    pub gates: Vec<(u8, u8)>,
    // Placed timeline clips on this track
    pub clips: Vec<ClipSession>,
    // Placed audio clips on this track, referencing recorded takes
    pub audio_clips: Vec<AudioClipSession>,
}

// The whole studio session, independent of the on-disk encoding. Only the
// transport tempo and master gain are global; everything else is per-track.
#[derive(Clone, Debug, PartialEq)]
pub struct StudioSession {
    pub bpm: f32,
    pub gain: f32,
    pub tracks: Vec<TrackSession>,
}

impl StudioSession {
    // Encode this session and package its takes beside `project_path`.
    fn to_project(&self, project_path: &Path) -> Result<ProjectFile, ProjectError> {
        let base_dir = base_dir_of(project_path);
        let mut project = ProjectFile::new("Geist Studio Session");
        project.meta.tempo_bpm = self.bpm as f64;

        // Seed the registry with unavailable references so arrangement edits can
        // be saved without discarding or rewriting offline media metadata.
        let offline_assets = self
            .tracks
            .iter()
            .flat_map(|track| &track.audio_clips)
            .filter(|clip| !clip.verified)
            .filter_map(|clip| {
                clip.asset_ref.as_ref().map(|asset| {
                    let mut preserved = asset.clone();
                    if !Path::new(&preserved.relative_path).is_absolute()
                        && base_dir.join(&preserved.relative_path) != clip.wav_path
                    {
                        preserved.relative_path = clip.wav_path.to_string_lossy().into_owned();
                    }
                    preserved
                })
            })
            .collect();
        let mut assets = AssetMap::from_refs(offline_assets);

        // Macros node: master gain plus a full per-track patch and fx block
        let mut params = vec![ParamValue { id: PARAM_GAIN, value: self.gain }];
        for (track, state) in self.tracks.iter().enumerate() {
            let t = track as u32;
            params.push(ParamValue { id: PARAM_TRACK_LEVEL_BASE + t, value: state.level });
            params.push(ParamValue { id: PARAM_TRACK_PAN_BASE + t, value: state.pan });
            params.push(ParamValue { id: PARAM_TRACK_CUTOFF_BASE + t, value: state.cutoff_hz });
            params.push(ParamValue { id: PARAM_TRACK_RESONANCE_BASE + t, value: state.resonance });
            params.push(ParamValue { id: PARAM_TRACK_DELAY_BASE + t, value: bool_to_f32(state.delay_on) });
            params.push(ParamValue { id: PARAM_TRACK_DELAY_TIME_BASE + t, value: state.delay_time });
            params.push(ParamValue { id: PARAM_TRACK_DELAY_FEEDBACK_BASE + t, value: state.delay_feedback });
            params.push(ParamValue { id: PARAM_TRACK_DELAY_MIX_BASE + t, value: state.delay_mix });
            params.push(ParamValue { id: PARAM_TRACK_REVERB_BASE + t, value: bool_to_f32(state.reverb_on) });
            params.push(ParamValue { id: PARAM_TRACK_REVERB_MIX_BASE + t, value: state.reverb_mix });
            params.push(ParamValue { id: PARAM_TRACK_OSC_MIX_BASE + t, value: state.osc_mix });
            params.push(ParamValue { id: PARAM_TRACK_OSC_B_SEMIS_BASE + t, value: state.osc_b_semis });
            for (i, &value) in state.amp_env.iter().enumerate() {
                params.push(ParamValue { id: PARAM_TRACK_AMP_ENV_BASE + t * 4 + i as u32, value });
            }
            for (i, &value) in state.filter_env.iter().enumerate() {
                params.push(ParamValue { id: PARAM_TRACK_FILTER_ENV_BASE + t * 4 + i as u32, value });
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
            // Package verified takes; preserve unavailable references unchanged.
            for clip in &state.audio_clips {
                let asset_index = if clip.verified {
                    let bytes = std::fs::read(&clip.wav_path)?;
                    let relative = take_relative_path(project_path, &hash_bytes(&bytes));
                    let destination = base_dir.join(&relative);
                    let already_packaged =
                        matches!(std::fs::read(&destination), Ok(existing) if existing == bytes);
                    if !already_packaged {
                        if let Some(parent) = destination.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&destination, &bytes)?;
                    }
                    assets.register(relative.to_string_lossy(), &bytes)
                } else {
                    let asset = clip.asset_ref.as_ref().ok_or_else(|| {
                        ProjectError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("offline take has no asset reference: {}", clip.wav_path.display()),
                        ))
                    })?;
                    assets.index_of_hash(&asset.content_hash).ok_or_else(|| {
                        ProjectError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("offline take asset is not indexed: {}", clip.wav_path.display()),
                        ))
                    })?
                };
                clips.push(ClipEntry {
                    id: clip.id,
                    start_ticks: beats_to_ticks(clip.start_beats),
                    length_ticks: beats_to_ticks(clip.len_beats),
                    kind: ClipKind::Audio { asset_index, offset_ticks: 0, gain_db: 0.0 },
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
        project.assets = assets.as_refs().to_vec();
        Ok(project)
    }

    // Decode a session from a project file, falling back to `defaults` per field.
    // `base_dir` is the directory the project file lives in; asset references
    // resolve against it into absolute take paths.
    fn from_project(
        project: &ProjectFile,
        defaults: &StudioSession,
        project_path: &Path,
    ) -> StudioSession {
        let base_dir = base_dir_of(project_path);
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
                let read = |base: u32| node.params.iter().find(|p| p.id == base + t).map(|p| p.value);
                if let Some(v) = read(PARAM_TRACK_LEVEL_BASE) { state.level = v; }
                if let Some(v) = read(PARAM_TRACK_PAN_BASE) { state.pan = v; }
                if let Some(v) = read(PARAM_TRACK_CUTOFF_BASE) { state.cutoff_hz = v; }
                if let Some(v) = read(PARAM_TRACK_RESONANCE_BASE) { state.resonance = v; }
                if let Some(v) = read(PARAM_TRACK_DELAY_BASE) { state.delay_on = v >= 0.5; }
                if let Some(v) = read(PARAM_TRACK_DELAY_TIME_BASE) { state.delay_time = v; }
                if let Some(v) = read(PARAM_TRACK_DELAY_FEEDBACK_BASE) { state.delay_feedback = v; }
                if let Some(v) = read(PARAM_TRACK_DELAY_MIX_BASE) { state.delay_mix = v; }
                if let Some(v) = read(PARAM_TRACK_REVERB_BASE) { state.reverb_on = v >= 0.5; }
                if let Some(v) = read(PARAM_TRACK_REVERB_MIX_BASE) { state.reverb_mix = v; }
                if let Some(v) = read(PARAM_TRACK_OSC_MIX_BASE) { state.osc_mix = v; }
                if let Some(v) = read(PARAM_TRACK_OSC_B_SEMIS_BASE) { state.osc_b_semis = v; }
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
            state.audio_clips.clear();
            state.gates.clear();
            let base = base_midi(index);
            for clip in &entry.clips {
                match &clip.kind {
                    ClipKind::Midi { notes } if clip.id == STEP_CLIP_ID => {
                        for note in notes {
                            let row = note.pitch.saturating_sub(base);
                            let step = (note.start_ticks / STEP_TICKS) as u8;
                            if (row as usize) < SEQ_ROWS && (step as usize) < SEQ_STEPS {
                                state.gates.push((row, step));
                            }
                        }
                    }
                    ClipKind::Midi { notes } => {
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
                    // Resolve the asset reference to an absolute take path; a
                    // missing or dangling reference drops the clip.
                    ClipKind::Audio { asset_index, .. } => {
                        let Some(asset) = project.assets.get(*asset_index) else {
                            continue;
                        };
                        let expected_path = base_dir.join(&asset.relative_path);
                        let wav_path = if asset_matches(&expected_path, asset) {
                            expected_path
                        } else {
                            find_matching_take(project_path, asset).unwrap_or(expected_path)
                        };
                        let verified = asset_matches(&wav_path, asset);
                        state.audio_clips.push(AudioClipSession {
                            id: clip.id,
                            start_beats: ticks_to_beats(clip.start_ticks),
                            len_beats: ticks_to_beats(clip.length_ticks),
                            wav_path,
                            verified,
                            asset_ref: Some(asset.clone()),
                        });
                    }
                    ClipKind::Automation { .. } => {}
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

// Relative directory containing one project's external assets
fn project_assets_dir_name(project_path: &Path) -> String {
    let stem = project_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("geist-project");
    format!("{stem}.assets")
}

// Content-addressed path for one packaged WAV take
fn take_relative_path(project_path: &Path, content_hash: &str) -> PathBuf {
    PathBuf::from(project_assets_dir_name(project_path))
        .join("Takes")
        .join(format!("{content_hash}.wav"))
}

// Verify one candidate without accepting a same-name or same-size substitute
pub fn asset_matches(path: &Path, asset: &AssetRef) -> bool {
    std::fs::read(path).is_ok_and(|bytes| {
        (asset.size_bytes == 0 || bytes.len() as u64 == asset.size_bytes)
            && hash_bytes(&bytes) == asset.content_hash
    })
}

// Search only this project's managed asset tree for a moved exact-hash match
fn find_matching_take(project_path: &Path, asset: &AssetRef) -> Option<PathBuf> {
    let root = base_dir_of(project_path).join(project_assets_dir_name(project_path));
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(dir).ok()?.filter_map(Result::ok).collect();
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() && asset_matches(&entry.path(), asset) {
                return Some(entry.path());
            }
        }
    }
    None
}

// Path to the studio session slot, in the home directory when available
pub fn session_path() -> PathBuf {
    let dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join(SESSION_FILE)
}

// Directory for recorded audio takes owned by the studio session
pub fn recordings_dir() -> PathBuf {
    let path = session_path();
    let dir = base_dir_of(&path)
        .join(project_assets_dir_name(&path))
        .join("Takes");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

// Directory a project file lives in, for resolving relative asset paths
fn base_dir_of(path: &Path) -> &Path {
    path.parent().unwrap_or_else(|| Path::new("."))
}

// Write a session to an explicit path
pub fn save_to(session: &StudioSession, path: &Path) -> Result<(), ProjectError> {
    save_to_path(&session.to_project(path)?, path)
}

// Read a session from an explicit path, falling back to `defaults` per field
pub fn load_from(
    defaults: &StudioSession,
    path: &Path,
) -> Result<StudioSession, ProjectError> {
    let project = load_from_path(path)?;
    Ok(StudioSession::from_project(&project, defaults, path))
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
            amp_env: [0.005, 0.1, 0.8, 0.3],
            filter_env: [0.01, 0.2, 0.3, 0.3],
            gates: Vec::new(),
            clips: Vec::new(),
            audio_clips: Vec::new(),
        }
    }

    fn defaults() -> StudioSession {
        StudioSession {
            bpm: 120.0,
            gain: 1.0,
            tracks: (0..NUM_TRACKS).map(|_| track_defaults()).collect(),
        }
    }

    fn sample() -> StudioSession {
        let mut s = defaults();
        s.bpm = 140.0;
        s.gain = 0.7;
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
        s.tracks[1].filter_env = [0.05, 0.4, 0.2, 0.8];
        s.tracks[1].gates = vec![(0, 0), (2, 4), (5, 12)];
        // Velocity 1.0 maps cleanly through the 0..127 MIDI range
        s.tracks[2].clips = vec![
            ClipSession {
                id: 1,
                start_beats: 0.0,
                len_beats: 8.0,
                notes: vec![
                    NoteSession { pitch: 60, start_beats: 0.0, len_beats: 1.0, velocity: 1.0 },
                    NoteSession { pitch: 67, start_beats: 2.5, len_beats: 0.5, velocity: 1.0 },
                ],
            },
            ClipSession {
                id: 2,
                start_beats: 16.0,
                len_beats: 4.0,
                notes: vec![NoteSession { pitch: 72, start_beats: 1.0, len_beats: 2.0, velocity: 1.0 }],
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
    fn missing_file_is_an_error_not_a_panic() {
        let path = PathBuf::from("/no/such/dir/geist-studio-missing.gproj");
        assert!(load_from(&defaults(), &path).is_err());
    }

    #[test]
    fn audio_clip_is_packaged_beside_the_project() {
        let root = std::env::temp_dir().join(format!("geist-audio-project-{}", std::process::id()));
        let source_dir = root.join("capture");
        let project_dir = root.join("project");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::create_dir_all(&project_dir).unwrap();
        let wav = source_dir.join("take.wav");
        std::fs::write(&wav, b"riff-take-bytes").unwrap();
        let path = project_dir.join("song.gproj");

        let mut session = defaults();
        session.tracks[0].audio_clips = vec![AudioClipSession {
            id: 5,
            start_beats: 4.0,
            len_beats: 2.5,
            wav_path: wav.clone(),
            verified: true,
            asset_ref: None,
        }];

        save_to(&session, &path).unwrap();
        let loaded = load_from(&defaults(), &path).unwrap();

        let loaded_clip = &loaded.tracks[0].audio_clips[0];
        assert_ne!(loaded_clip.wav_path, wav);
        assert!(loaded_clip.wav_path.starts_with(&project_dir));
        assert!(loaded_clip.verified);
        assert_eq!(std::fs::read(&loaded_clip.wav_path).unwrap(), b"riff-take-bytes");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn missing_audio_take_fails_save() {
        let root = std::env::temp_dir().join(format!("geist-missing-take-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("song.gproj");
        let mut session = defaults();
        session.tracks[0].audio_clips = vec![AudioClipSession {
            id: 9,
            start_beats: 0.0,
            len_beats: 1.0,
            wav_path: root.join("missing.wav"),
            verified: true,
            asset_ref: None,
        }];

        assert!(save_to(&session, &path).is_err());
        assert!(!path.exists());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn changed_audio_take_loads_as_offline() {
        let root = std::env::temp_dir().join(format!("geist-changed-take-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source.wav");
        let project_path = root.join("song.gproj");
        std::fs::write(&source, b"original-audio").unwrap();
        let mut session = defaults();
        session.tracks[0].audio_clips = vec![AudioClipSession {
            id: 10,
            start_beats: 2.0,
            len_beats: 4.0,
            wav_path: source,
            verified: true,
            asset_ref: None,
        }];
        save_to(&session, &project_path).unwrap();
        let loaded = load_from(&defaults(), &project_path).unwrap();
        let packaged = loaded.tracks[0].audio_clips[0].wav_path.clone();
        std::fs::write(&packaged, b"changed-audio").unwrap();

        let mut changed = load_from(&defaults(), &project_path).unwrap();
        assert_eq!(changed.tracks[0].audio_clips.len(), 1);
        assert!(!changed.tracks[0].audio_clips[0].verified);
        let original_ref = changed.tracks[0].audio_clips[0].asset_ref.clone();

        changed.bpm = 123.0;
        save_to(&changed, &project_path).unwrap();
        let resaved = load_from(&defaults(), &project_path).unwrap();
        assert_eq!(resaved.bpm, 123.0);
        assert!(!resaved.tracks[0].audio_clips[0].verified);
        assert_eq!(resaved.tracks[0].audio_clips[0].asset_ref, original_ref);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn moved_take_is_recovered_by_hash_inside_project_assets() {
        let root = std::env::temp_dir().join(format!("geist-moved-take-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = root.join("source.wav");
        let project_path = root.join("song.gproj");
        std::fs::write(&source, b"recoverable-audio").unwrap();
        let mut session = defaults();
        session.tracks[0].audio_clips = vec![AudioClipSession {
            id: 11,
            start_beats: 1.0,
            len_beats: 2.0,
            wav_path: source,
            verified: true,
            asset_ref: None,
        }];
        save_to(&session, &project_path).unwrap();
        let packaged = load_from(&defaults(), &project_path).unwrap().tracks[0].audio_clips[0]
            .wav_path
            .clone();
        let recovered = root.join("song.assets/Recovered/moved.wav");
        std::fs::create_dir_all(recovered.parent().unwrap()).unwrap();
        std::fs::rename(&packaged, &recovered).unwrap();

        let loaded = load_from(&defaults(), &project_path).unwrap();
        assert!(loaded.tracks[0].audio_clips[0].verified);
        assert_eq!(loaded.tracks[0].audio_clips[0].wav_path, recovered);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn relink_candidate_requires_exact_size_and_hash() {
        let path = std::env::temp_dir().join(format!("geist-relink-candidate-{}", std::process::id()));
        std::fs::write(&path, b"candidate-audio").unwrap();
        let asset = AssetRef {
            relative_path: "missing.wav".to_string(),
            content_hash: hash_bytes(b"candidate-audio"),
            size_bytes: b"candidate-audio".len() as u64,
        };
        assert!(asset_matches(&path, &asset));
        std::fs::write(&path, b"wrong-candidate").unwrap();
        assert!(!asset_matches(&path, &asset));
        std::fs::remove_file(path).ok();
    }
}
