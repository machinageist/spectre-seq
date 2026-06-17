// =============================================================================
// File: crates/geist-project/src/schema.rs
// Layer: project persistence
// Purpose: Versioned ProjectFile DTO tree and SCHEMA_VERSION
// Status: Implemented; decoupled serde data model for the whole project.
// Notes: These DTOs are the stable on-disk shape, deliberately separate from the
//        live engine types. serde(default) on fields lets older files load and
//        unknown fields are skipped, giving forward and backward tolerance.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use serde::{Deserialize, Serialize};

// On-disk format version; bump the major component on a breaking change
pub const SCHEMA_VERSION: u32 = 1;

// Musical defaults for a fresh project
const DEFAULT_TEMPO_BPM: f64 = 120.0;
const DEFAULT_TIME_SIG_NUM: u16 = 4;
const DEFAULT_TIME_SIG_DEN: u16 = 4;
const DEFAULT_SAMPLE_RATE_HZ: u32 = 48_000;

// Root of a saved project; everything needed to reconstruct DAW state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub schema_version: u32,
    #[serde(default)]
    pub meta: ProjectMeta,
    #[serde(default)]
    pub graph: GraphTopology,
    #[serde(default)]
    pub tracks: Vec<TrackEntry>,
    #[serde(default)]
    pub automation: Vec<AutomationLaneEntry>,
    #[serde(default)]
    pub assets: Vec<AssetRef>,
}

impl ProjectFile {
    // Build an empty project at the current schema version
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            meta: ProjectMeta {
                name: name.into(),
                ..ProjectMeta::default()
            },
            graph: GraphTopology::default(),
            tracks: Vec::new(),
            automation: Vec::new(),
            assets: Vec::new(),
        }
    }
}

// Project-level metadata and transport defaults
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectMeta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub created_unix: u64,
    #[serde(default)]
    pub modified_unix: u64,
    #[serde(default = "default_tempo")]
    pub tempo_bpm: f64,
    #[serde(default = "default_time_sig_num")]
    pub time_sig_num: u16,
    #[serde(default = "default_time_sig_den")]
    pub time_sig_den: u16,
    #[serde(default = "default_sample_rate")]
    pub sample_rate_hz: u32,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        Self {
            name: String::new(),
            created_unix: 0,
            modified_unix: 0,
            tempo_bpm: DEFAULT_TEMPO_BPM,
            time_sig_num: DEFAULT_TIME_SIG_NUM,
            time_sig_den: DEFAULT_TIME_SIG_DEN,
            sample_rate_hz: DEFAULT_SAMPLE_RATE_HZ,
        }
    }
}

// Defaults referenced by serde(default = ...) for individual fields
fn default_tempo() -> f64 {
    DEFAULT_TEMPO_BPM
}
fn default_time_sig_num() -> u16 {
    DEFAULT_TIME_SIG_NUM
}
fn default_time_sig_den() -> u16 {
    DEFAULT_TIME_SIG_DEN
}
fn default_sample_rate() -> u32 {
    DEFAULT_SAMPLE_RATE_HZ
}

// Processing graph: nodes and the connections between their ports
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphTopology {
    #[serde(default)]
    pub nodes: Vec<NodeEntry>,
    #[serde(default)]
    pub connections: Vec<Connection>,
}

// One node: its type tag, parameter snapshot, and opaque plugin state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeEntry {
    pub id: u64,
    pub kind: String,
    #[serde(default)]
    pub params: Vec<ParamValue>,
    // Opaque CLAP/internal state blob; empty when the node has none
    // serde_bytes encodes it as a compact CBOR byte string, not an int array
    #[serde(default, with = "serde_bytes")]
    pub state_blob: Vec<u8>,
}

// A single parameter value addressed by its stable id
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParamValue {
    pub id: u32,
    pub value: f32,
}

// Directed connection from one node's output port to another's input port
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub src_node: u64,
    pub src_port: u32,
    pub dst_node: u64,
    pub dst_port: u32,
}

// A track: an ordered lane of clips with mix flags
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrackEntry {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub clips: Vec<ClipEntry>,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub soloed: bool,
}

// A clip placed on the timeline at a tick position with a typed payload
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipEntry {
    pub id: u64,
    pub start_ticks: u64,
    pub length_ticks: u64,
    pub kind: ClipKind,
}

// Clip payload variants: audio sample, MIDI notes, or an automation lane
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClipKind {
    Audio {
        asset_index: usize,
        #[serde(default)]
        offset_ticks: u64,
        #[serde(default)]
        gain_db: f32,
    },
    Midi {
        #[serde(default)]
        notes: Vec<NoteEntry>,
    },
    Automation {
        lane_index: usize,
    },
}

// A MIDI note within a clip, positioned relative to the clip start
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteEntry {
    pub pitch: u8,
    pub velocity: u8,
    pub start_ticks: u64,
    pub length_ticks: u64,
    #[serde(default)]
    pub channel: u8,
}

// An automation lane targeting one parameter on one node
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutomationLaneEntry {
    pub target_node: u64,
    pub target_param: u32,
    #[serde(default)]
    pub points: Vec<BreakpointEntry>,
}

// One automation breakpoint: a value at a tick with a segment shape
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BreakpointEntry {
    pub pos_ticks: u64,
    pub value: f32,
    #[serde(default)]
    pub curve: CurveKind,
}

// Interpolation shape from a breakpoint to the next
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveKind {
    #[default]
    Linear,
    Exponential,
    Logarithmic,
    Smooth,
    Step,
}

// Reference to an external audio file by relative path and content hash
// Files are never embedded; portability is the user's responsibility
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRef {
    pub relative_path: String,
    pub content_hash: String,
    #[serde(default)]
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_carries_current_version_and_defaults() {
        let p = ProjectFile::new("untitled");
        assert_eq!(p.schema_version, SCHEMA_VERSION);
        assert_eq!(p.meta.name, "untitled");
        assert_eq!(p.meta.tempo_bpm, DEFAULT_TEMPO_BPM);
        assert_eq!(p.meta.time_sig_num, DEFAULT_TIME_SIG_NUM);
        assert_eq!(p.meta.sample_rate_hz, DEFAULT_SAMPLE_RATE_HZ);
        assert!(p.tracks.is_empty());
        assert!(p.graph.nodes.is_empty());
    }

    #[test]
    fn meta_default_fills_musical_values() {
        let m = ProjectMeta::default();
        assert_eq!(m.tempo_bpm, 120.0);
        assert_eq!((m.time_sig_num, m.time_sig_den), (4, 4));
    }

    #[test]
    fn curve_kind_defaults_to_linear() {
        assert_eq!(CurveKind::default(), CurveKind::Linear);
    }

    #[test]
    fn clip_kinds_construct() {
        let audio = ClipEntry {
            id: 1,
            start_ticks: 0,
            length_ticks: 960,
            kind: ClipKind::Audio {
                asset_index: 0,
                offset_ticks: 0,
                gain_db: -3.0,
            },
        };
        let midi = ClipEntry {
            id: 2,
            start_ticks: 960,
            length_ticks: 480,
            kind: ClipKind::Midi {
                notes: vec![NoteEntry {
                    pitch: 60,
                    velocity: 100,
                    start_ticks: 0,
                    length_ticks: 240,
                    channel: 0,
                }],
            },
        };
        assert_ne!(audio.kind, midi.kind);
    }
}
