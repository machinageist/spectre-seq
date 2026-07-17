// Author: Jeff
// Date: 2026-07-11
// Description: Versioned minimal project document with schema gate and unknown-field preservation
// Notes: CORE-001 (IDs), CORE-003 (versioned envelope, forward-field preservation); atomic writes arrive in R5

use geist_core::{MeterMap, ObjectId, TempoMap, Transport};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub mod command;

// Current schema version written by this build
pub const SCHEMA_VERSION: u32 = 1;

// Newest schema this build can read
pub const MAX_READABLE_SCHEMA: u32 = 1;

// Envelope wrapping every persisted project
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectEnvelope {
    pub schema_version: u32,
    pub project: ProjectDoc,
    // Unknown top-level fields from newer writers survive a rewrite
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

// Minimal R1 project document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDoc {
    pub id: ObjectId,
    pub name: String,
    pub tempo_map: TempoMap,
    pub transport: Transport,
    // Unknown document fields from newer writers survive a rewrite
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

impl ProjectDoc {
    // Decode optional meter state carried by the minimal R1 document
    pub fn meter_map(&self) -> Result<Option<MeterMap>, ProjectError> {
        self.unknown
            .get("meter_map")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|_| ProjectError::InvalidProject("meter map is invalid"))
    }
}

// Load failures with actionable variants
#[derive(Debug)]
pub enum ProjectError {
    SchemaTooNew { found: u32, max_readable: u32 },
    InvalidProject(&'static str),
    Malformed(serde_json::Error),
}

impl std::fmt::Display for ProjectError {
    // Render a user-facing failure description
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::SchemaTooNew {
                found,
                max_readable,
            } => write!(
                f,
                "project schema {found} is newer than this build can read (max {max_readable})"
            ),
            ProjectError::InvalidProject(reason) => write!(f, "invalid project: {reason}"),
            ProjectError::Malformed(e) => write!(f, "malformed project: {e}"),
        }
    }
}

impl std::error::Error for ProjectError {}

// Serialize an envelope to the development codec
pub fn to_bytes(envelope: &ProjectEnvelope) -> Result<Vec<u8>, ProjectError> {
    serde_json::to_vec_pretty(envelope).map_err(ProjectError::Malformed)
}

// Deserialize and schema-gate an envelope
pub fn from_bytes(bytes: &[u8]) -> Result<ProjectEnvelope, ProjectError> {
    // Peek the version before full decode so newer schemas fail cleanly
    #[derive(Deserialize)]
    struct VersionPeek {
        schema_version: u32,
    }
    let peek: VersionPeek = serde_json::from_slice(bytes).map_err(ProjectError::Malformed)?;
    if peek.schema_version > MAX_READABLE_SCHEMA {
        return Err(ProjectError::SchemaTooNew {
            found: peek.schema_version,
            max_readable: MAX_READABLE_SCHEMA,
        });
    }
    let envelope: ProjectEnvelope =
        serde_json::from_slice(bytes).map_err(ProjectError::Malformed)?;
    validate(&envelope)?;
    Ok(envelope)
}

// Revalidate invariants that derived serde decoding cannot enforce
fn validate(envelope: &ProjectEnvelope) -> Result<(), ProjectError> {
    if envelope.project.id.raw() == 0 {
        return Err(ProjectError::InvalidProject("object ID must be nonzero"));
    }
    TempoMap::new(envelope.project.tempo_map.segments().to_vec())
        .map_err(|_| ProjectError::InvalidProject("tempo map is invalid"))?;
    envelope.project.meter_map()?;
    let transport = envelope.project.transport;
    if transport.loop_enabled && transport.loop_region.is_none() {
        return Err(ProjectError::InvalidProject(
            "enabled loop requires a loop region",
        ));
    }
    if let Some(region) = transport.loop_region {
        if region.end.0 <= region.start.0 {
            return Err(ProjectError::InvalidProject("loop region is invalid"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::IdGen;

    // Helper minimal envelope
    fn envelope() -> ProjectEnvelope {
        let mut ids = IdGen::new(11);
        ProjectEnvelope {
            schema_version: SCHEMA_VERSION,
            project: ProjectDoc {
                id: ids.next_id(),
                name: "Test".into(),
                tempo_map: TempoMap::constant(120.0).unwrap(),
                transport: Transport::new(),
                unknown: Map::new(),
            },
            unknown: Map::new(),
        }
    }

    // Round trip preserves the document and its identity
    #[test]
    fn round_trip_preserves_document() {
        let env = envelope();
        let bytes = to_bytes(&env).unwrap();
        let back = from_bytes(&bytes).unwrap();
        assert_eq!(back, env);
    }

    // Unknown fields written by a newer minor writer survive a rewrite
    #[test]
    fn unknown_fields_survive_rewrite() {
        let env = envelope();
        let mut value = serde_json::to_value(&env).unwrap();
        value["future_feature"] = serde_json::json!({ "enabled": true });
        value["project"]["future_track_kind"] = serde_json::json!("granular");
        let bytes = serde_json::to_vec(&value).unwrap();

        let loaded = from_bytes(&bytes).unwrap();
        let rewritten = to_bytes(&loaded).unwrap();
        let reread: Value = serde_json::from_slice(&rewritten).unwrap();
        assert_eq!(reread["future_feature"]["enabled"], Value::Bool(true));
        assert_eq!(
            reread["project"]["future_track_kind"],
            Value::String("granular".into())
        );
    }

    // A newer schema version fails cleanly, never silently rewrites
    #[test]
    fn newer_schema_is_rejected() {
        let env = envelope();
        let mut value = serde_json::to_value(&env).unwrap();
        value["schema_version"] = serde_json::json!(99);
        let bytes = serde_json::to_vec(&value).unwrap();
        match from_bytes(&bytes) {
            Err(ProjectError::SchemaTooNew {
                found: 99,
                max_readable,
            }) => {
                assert_eq!(max_readable, MAX_READABLE_SCHEMA);
            }
            other => panic!("expected SchemaTooNew, got {other:?}"),
        }
    }

    // Garbage input reports malformed, not panic
    #[test]
    fn malformed_input_is_an_error() {
        assert!(matches!(
            from_bytes(b"not json"),
            Err(ProjectError::Malformed(_))
        ));
    }

    // Structurally valid JSON cannot bypass core invariants
    #[test]
    fn invalid_core_values_are_rejected() {
        let env = envelope();

        let mut zero_id = serde_json::to_value(&env).unwrap();
        zero_id["project"]["id"] = serde_json::json!(0);
        assert!(from_bytes(&serde_json::to_vec(&zero_id).unwrap()).is_err());

        let mut empty_tempo = serde_json::to_value(&env).unwrap();
        empty_tempo["project"]["tempo_map"]["segments"] = serde_json::json!([]);
        assert!(from_bytes(&serde_json::to_vec(&empty_tempo).unwrap()).is_err());

        let mut invalid_loop = serde_json::to_value(&env).unwrap();
        invalid_loop["project"]["transport"]["loop_region"] =
            serde_json::json!({ "start": 20, "end": 10 });
        invalid_loop["project"]["transport"]["loop_enabled"] = serde_json::json!(true);
        assert!(from_bytes(&serde_json::to_vec(&invalid_loop).unwrap()).is_err());
    }
}
