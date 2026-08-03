// =============================================================================
// File: crates/spectre-project/src/serialize.rs
// Layer: project persistence
// Purpose: CBOR encode/decode of the project file and the crate error type
// Status: Implemented; byte and path round-trips over ciborium.
// Notes: ProjectError is the shared crate error. Encoding is compact CBOR;
//        unknown fields are skipped on decode, giving forward tolerance.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::fmt;
use std::path::Path;

use crate::schema::ProjectFile;

// Shared error for every persistence operation in the crate
#[derive(Debug)]
pub enum ProjectError {
    Io(std::io::Error),
    Encode(String),
    Decode(String),
    UnsupportedVersion { found: u32, max: u32 },
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::Io(e) => write!(f, "i/o error: {e}"),
            ProjectError::Encode(e) => write!(f, "encode error: {e}"),
            ProjectError::Decode(e) => write!(f, "decode error: {e}"),
            ProjectError::UnsupportedVersion { found, max } => {
                write!(
                    f,
                    "project schema version {found} is newer than supported {max}"
                )
            }
        }
    }
}

impl std::error::Error for ProjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProjectError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProjectError {
    fn from(e: std::io::Error) -> Self {
        ProjectError::Io(e)
    }
}

// Encode a project to a compact CBOR byte buffer
pub fn to_cbor(project: &ProjectFile) -> Result<Vec<u8>, ProjectError> {
    let mut buf = Vec::new();
    ciborium::into_writer(project, &mut buf).map_err(|e| ProjectError::Encode(e.to_string()))?;
    Ok(buf)
}

// Decode a project from a CBOR byte buffer
pub fn from_cbor(bytes: &[u8]) -> Result<ProjectFile, ProjectError> {
    ciborium::from_reader(bytes).map_err(|e| ProjectError::Decode(e.to_string()))
}

// Encode and write a project to disk
pub fn save_to_path(project: &ProjectFile, path: impl AsRef<Path>) -> Result<(), ProjectError> {
    let bytes = to_cbor(project)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

// Read and decode a project from disk
pub fn load_from_path(path: impl AsRef<Path>) -> Result<ProjectFile, ProjectError> {
    let bytes = std::fs::read(path)?;
    from_cbor(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        AutomationLaneEntry, BreakpointEntry, ClipEntry, ClipKind, Connection, CurveKind,
        NodeEntry, NoteEntry, ParamValue, TrackEntry,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    // Build a project exercising every payload variant
    fn sample_project() -> ProjectFile {
        let mut p = ProjectFile::new("round trip");
        p.graph.nodes.push(NodeEntry {
            id: 1,
            kind: "spectre-synth".into(),
            params: vec![ParamValue { id: 7, value: 0.5 }],
            state_blob: vec![0xDE, 0xAD, 0xBE, 0xEF],
        });
        p.graph.connections.push(Connection {
            src_node: 1,
            src_port: 0,
            dst_node: 2,
            dst_port: 0,
        });
        p.tracks.push(TrackEntry {
            id: 10,
            name: "lead".into(),
            clips: vec![ClipEntry {
                id: 100,
                start_ticks: 0,
                length_ticks: 960,
                kind: ClipKind::Midi {
                    notes: vec![NoteEntry {
                        pitch: 64,
                        velocity: 110,
                        start_ticks: 0,
                        length_ticks: 240,
                        channel: 1,
                    }],
                },
            }],
            muted: false,
            soloed: true,
        });
        p.automation.push(AutomationLaneEntry {
            target_node: 1,
            target_param: 7,
            points: vec![
                BreakpointEntry {
                    pos_ticks: 0,
                    value: 0.0,
                    curve: CurveKind::Linear,
                },
                BreakpointEntry {
                    pos_ticks: 480,
                    value: 1.0,
                    curve: CurveKind::Smooth,
                },
            ],
        });
        p
    }

    // Build a unique scratch path under the system temp dir
    fn temp_path(tag: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("geist_{tag}_{nanos}.geist"))
    }

    #[test]
    fn cbor_round_trip_is_lossless() {
        let project = sample_project();
        let bytes = to_cbor(&project).unwrap();
        let decoded = from_cbor(&bytes).unwrap();
        assert_eq!(project, decoded);
    }

    #[test]
    fn state_blob_survives_as_bytes() {
        let project = sample_project();
        let decoded = from_cbor(&to_cbor(&project).unwrap()).unwrap();
        assert_eq!(
            decoded.graph.nodes[0].state_blob,
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
    }

    #[test]
    fn path_round_trip_matches() {
        let project = sample_project();
        let path = temp_path("roundtrip");
        save_to_path(&project, &path).unwrap();
        let loaded = load_from_path(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(project, loaded);
    }

    #[test]
    fn decode_rejects_garbage() {
        let err = from_cbor(&[0xFF, 0x00, 0x13, 0x37]).unwrap_err();
        assert!(matches!(err, ProjectError::Decode(_)));
    }

    #[test]
    fn missing_file_is_io_error() {
        let err = load_from_path("/nonexistent/geist/path.geist").unwrap_err();
        assert!(matches!(err, ProjectError::Io(_)));
    }
}
