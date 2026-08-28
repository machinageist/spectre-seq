// Author: Jeff
// Date: 2026-07-11
// Description: Versioned project document with schema gate and unknown-field preservation
// Notes: CORE-001 (IDs), CORE-003 (versioned envelope, forward-field preservation). Schema 2 adds
//   the first persisted object collections; the filesystem half of CORE-004 lives in fs.rs

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use spectre_core::{MeterMap, ObjectId, TempoMap, Transport};
use spectre_dsp::{GAIN_PARAMETERS, PULSE_PARAMETERS};
use std::collections::HashSet;

pub mod clip;
pub mod command;
pub mod fs;
pub mod routing;
pub mod track;

pub use clip::{
    ClipError, ClipNote, ClipPlacement, MidiClip, TrackClips, MAX_CLIPS_PER_TRACK,
    MAX_CLIP_LENGTH_TICKS, MAX_NOTES_PER_CLIP,
};
pub use fs::{
    load_project, save_project_atomic, LoadError, SaveError, SaveReceipt, SaveStage, TargetState,
    MAX_PROJECT_FILE_BYTES, SAVE_TEMP_NAME_ATTEMPTS,
};
pub use routing::{build_track_graph, track_device_factory, RoutingError, TrackPathNodes};
pub use track::{Track, TrackError, TrackInstrument, TrackList, MAX_TRACKS};

// Current schema version written by this build. Schema identifiers, not numeric limits, so
// PROD-003 does not govern them. The bump to 2 is required rather than cosmetic: schema 2 adds
// persisted object collections, and a build that cannot render them must refuse the file rather
// than show the musician an empty project it would then let them overwrite
pub const SCHEMA_VERSION: u32 = 2;

// Newest schema this build can read
pub const MAX_READABLE_SCHEMA: u32 = 2;

// Envelope wrapping every persisted project
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectEnvelope {
    pub schema_version: u32,
    pub project: ProjectDoc,
    // Unknown top-level fields from newer writers survive a rewrite
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

// Minimal R1 project document, extended at schema 2 with the first persisted collections.
// Every added field is #[serde(default)] so a schema-1 file still decodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDoc {
    pub id: ObjectId,
    pub name: String,
    pub tempo_map: TempoMap,
    pub transport: Transport,
    // Splitmix64 generator position; 0 means a schema-1 document that carried none.
    // Each schema-2 field is skipped when it carries nothing, so reading a schema-1 document and
    // rewriting it reproduces it byte for byte rather than injecting four empty fields into it.
    // That is CORE-003's forward preservation applied backwards, and
    // canonical_fixture_rewrite_is_byte_stable is the gate that holds it
    #[serde(default, skip_serializing_if = "is_zero")]
    pub id_gen_state: u64,
    // First persisted object collection. R4-4's own TrackList, not a parallel document type: it
    // already derives Serialize/Deserialize, already owns order, and already lives in this crate.
    // A second track type would mean a hand-written mapping and two places to forget a field
    #[serde(default, skip_serializing_if = "carries_no_tracks")]
    pub tracks: TrackList,
    // Device instances and their app-thread parameter values
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<DeviceDoc>,
    // Restored working context; never engine or render state
    #[serde(default, skip_serializing_if = "carries_no_view")]
    pub view: ViewDoc,
    // Unknown document fields from newer writers survive a rewrite
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

// True when the list holds nothing a file needs to carry. TrackList's PartialEq compares every
// persisted field and deliberately excludes only the #[serde(skip)] revision, so this stays
// correct when a field is added: the same impl is where the new field would go
fn carries_no_tracks(tracks: &TrackList) -> bool {
    *tracks == TrackList::default()
}

fn carries_no_view(view: &ViewDoc) -> bool {
    *view == ViewDoc::default()
}

// One persisted device instance, addressed by stable ID and by canonical key
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceDoc {
    pub id: ObjectId,
    pub key: String,
    pub parameters: Vec<ParameterDoc>,
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

// One persisted parameter value; range validation belongs to whoever owns the descriptor
// catalogue, which this crate is not. The value is the BASE value and nothing else — when
// automation lands under PROD-002 it gets its own fields rather than overloading this one
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDoc {
    pub id: ObjectId,
    pub key: String,
    pub value: f32,
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

// Working context restored on open. The document schema must not depend on the UI crate, so the
// four lens variants are duplicated here rather than imported; spectre-app owns the conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ViewDoc {
    pub lens: LensDoc,
    pub selected_track: Option<ObjectId>,
    pub selected_device: Option<ObjectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LensDoc {
    #[default]
    Arrange,
    Build,
    Shape,
    Mix,
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
    validate_envelope(&envelope).map_err(|error| ProjectError::InvalidProject(error.reason))?;
    Ok(envelope)
}

// Semantic invalidity, distinct from encoding failure and from I/O failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationError {
    pub reason: &'static str,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for ValidationError {}

// The one semantic validator: called after decode and before encode. Public because
// save_project_atomic must run it before it touches a destination — a project that cannot be
// read back must never reach the disk in the first place
pub fn validate_envelope(envelope: &ProjectEnvelope) -> Result<(), ValidationError> {
    let fail = |reason: &'static str| Err(ValidationError { reason });

    // Schema 1's five rules, unchanged and in their original order
    if envelope.project.id.raw() == 0 {
        return fail("object ID must be nonzero");
    }
    if TempoMap::new(envelope.project.tempo_map.segments().to_vec()).is_err() {
        return fail("tempo map is invalid");
    }
    if envelope.project.meter_map().is_err() {
        return fail("meter map is invalid");
    }
    let transport = envelope.project.transport;
    if transport.loop_enabled && transport.loop_region.is_none() {
        return fail("enabled loop requires a loop region");
    }
    if let Some(region) = transport.loop_region {
        if region.end.0 <= region.start.0 {
            return fail("loop region is invalid");
        }
    }

    let document = &envelope.project;

    // Rule 1 — project-wide unique object IDs. TrackList::insert's runtime duplicate refusal
    // does NOT cover this: the derived Deserialize builds the vector directly and calls no
    // method on it. Nonzero comes free from ObjectId's hand-written Deserialize
    let mut seen = HashSet::new();
    if !seen.insert(document.id.raw()) {
        return fail("object IDs must be unique within a project");
    }
    for track in document.tracks.tracks() {
        if !seen.insert(track.id().raw()) {
            return fail("object IDs must be unique within a project");
        }
    }
    for device in &document.devices {
        if !seen.insert(device.id.raw()) {
            return fail("object IDs must be unique within a project");
        }
        for parameter in &device.parameters {
            if !seen.insert(parameter.id.raw()) {
                return fail("object IDs must be unique within a project");
            }
        }
    }

    // Rule 2 — referential integrity of the view. A dangling selection is invalid, not
    // silently dropped: dropping it would restore a project the musician did not save
    if let Some(id) = document.view.selected_track {
        if !document.tracks.tracks().iter().any(|t| t.id() == id) {
            return fail("selected track is not in the project");
        }
    }
    if let Some(id) = document.view.selected_device {
        if !document.devices.iter().any(|d| d.id == id) {
            return fail("selected device is not in the project");
        }
    }

    // Rule 3 — the Track invariants deserialization bypasses. Track's mutators clamp and
    // Track::new refuses a blank name, but #[derive(Deserialize)] calls none of them, so a
    // hand-edited file reaches the model through a door the constructors close. No new number
    // is introduced here: every bound named is an already-accepted one re-checked on a path
    // that skipped it
    if document.tracks.len() > MAX_TRACKS {
        return fail("project holds more tracks than the accepted maximum");
    }
    let gain = GAIN_PARAMETERS[0];
    let instrument = PULSE_PARAMETERS[0];
    if !in_descriptor_range(
        document.tracks.master_level(),
        gain.minimum(),
        gain.maximum(),
    ) {
        return fail("master level is outside the accepted gain range");
    }
    for track in document.tracks.tracks() {
        if track.name().trim().is_empty() {
            return fail("track name must not be blank");
        }
        if !in_descriptor_range(track.level(), gain.minimum(), gain.maximum()) {
            return fail("track level is outside the accepted gain range");
        }
        if !in_descriptor_range(
            track.instrument_level(),
            instrument.minimum(),
            instrument.maximum(),
        ) {
            return fail("track instrument level is outside the accepted range");
        }
    }

    // Rule 4 — finite parameter values and non-empty keys. Not hygiene: JSON has no literal
    // for NaN or infinity, so a non-finite value cannot survive an honest encode, and whichever
    // way the encoder resolves it the file no longer means what the project meant. Validating
    // before encoding turns that into a refusal with the destination never opened
    for device in &document.devices {
        if device.key.trim().is_empty() {
            return fail("device key must not be empty");
        }
        let mut keys = HashSet::new();
        for parameter in &device.parameters {
            if parameter.key.trim().is_empty() {
                return fail("parameter key must not be empty");
            }
            if !parameter.value.is_finite() {
                return fail("parameter value must be finite");
            }
            if !keys.insert(parameter.key.as_str()) {
                return fail("parameter keys must be unique within a device");
            }
        }
    }

    Ok(())
}

// Inclusive descriptor-range membership that also refuses non-finite input. Written as its own
// predicate because `!(value < min || value > max)` is true for NaN and a bare comparison chain
// would let one through
fn in_descriptor_range(value: f32, minimum: f32, maximum: f32) -> bool {
    value.is_finite() && value >= minimum && value <= maximum
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::IdGen;

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
                id_gen_state: ids.state(),
                tracks: TrackList::new(),
                devices: Vec::new(),
                view: ViewDoc::default(),
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
