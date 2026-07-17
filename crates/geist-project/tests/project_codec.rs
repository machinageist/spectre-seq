// Author: Jeff
// Date: 2026-07-16
// Description: R1 fixture-driven acceptance tests for the versioned JSON project codec
// Notes: Byte stability is scoped to rewriting this checked-in fixture with the pinned serde stack

use geist_project::{from_bytes, to_bytes, ProjectError, MAX_READABLE_SCHEMA, SCHEMA_VERSION};
use serde_json::{json, Value};

const GOLDEN: &[u8] = include_bytes!("fixtures/r1-canonical.json");

fn fixture_value() -> Value {
    serde_json::from_slice(GOLDEN).expect("checked-in fixture must be JSON")
}

fn encoded_fixture_value(mut edit: impl FnMut(&mut Value)) -> Vec<u8> {
    let mut value = fixture_value();
    edit(&mut value);
    serde_json::to_vec(&value).expect("fixture mutation must remain encodable")
}

#[test]
fn canonical_fixture_decodes_with_representative_r1_state() {
    let envelope = from_bytes(GOLDEN).expect("canonical fixture must decode");

    assert_eq!(envelope.schema_version, SCHEMA_VERSION);
    assert_eq!(envelope.project.id.raw(), 1_311_768_467_463_790_320);
    assert_eq!(envelope.project.name, "R1 Canonical Fixture");
    assert_eq!(envelope.project.tempo_map.segments().len(), 2);
    assert_eq!(envelope.project.tempo_map.segments()[1].bpm, 96.5);
    let meter_map = envelope
        .project
        .meter_map()
        .expect("fixture meter map must be valid")
        .expect("fixture must carry meter state");
    assert_eq!(meter_map.changes().len(), 2);
    assert_eq!(meter_map.changes()[1].signature.numerator(), 7);
    assert_eq!(meter_map.changes()[1].signature.denominator(), 8);
    assert!(envelope.project.transport.loop_enabled);
    assert_eq!(envelope.project.transport.position.0, 72_000);
    assert_eq!(
        envelope
            .project
            .transport
            .loop_region
            .expect("fixture loop region")
            .end
            .0,
        192_000
    );
}

#[test]
fn canonical_fixture_rewrite_is_byte_stable() {
    let decoded = from_bytes(GOLDEN).expect("canonical fixture must decode");
    let rewritten = to_bytes(&decoded).expect("canonical fixture must encode");

    assert_eq!(rewritten, GOLDEN);
}

#[test]
fn canonical_fixture_has_an_exact_decode_encode_decode_round_trip() {
    let decoded = from_bytes(GOLDEN).expect("canonical fixture must decode");
    let encoded = to_bytes(&decoded).expect("canonical fixture must encode");
    let decoded_again = from_bytes(&encoded).expect("rewritten fixture must decode");

    assert_eq!(decoded_again, decoded);
}

#[test]
fn canonical_fixture_unknown_fields_survive_rewrite() {
    let decoded = from_bytes(GOLDEN).expect("canonical fixture must decode");
    let rewritten = to_bytes(&decoded).expect("canonical fixture must encode");
    let value: Value = serde_json::from_slice(&rewritten).expect("rewrite must remain JSON");

    assert_eq!(
        value["project"]["future_session"],
        json!({"color": "ultraviolet", "launch_quantization": 8})
    );
    assert_eq!(
        value["r1_extension"],
        json!({"enabled": true, "revision": 3})
    );
}

#[test]
fn canonical_fixture_with_newer_schema_is_rejected() {
    let bytes = encoded_fixture_value(|value| {
        value["schema_version"] = json!(MAX_READABLE_SCHEMA + 1);
    });

    assert!(matches!(
        from_bytes(&bytes),
        Err(ProjectError::SchemaTooNew {
            found,
            max_readable: MAX_READABLE_SCHEMA,
        }) if found == MAX_READABLE_SCHEMA + 1
    ));
}

#[test]
fn canonical_fixture_with_invalid_tempo_map_is_rejected() {
    let bytes = encoded_fixture_value(|value| {
        value["project"]["tempo_map"]["segments"][1]["start"] = json!(0);
    });

    assert!(matches!(
        from_bytes(&bytes),
        Err(ProjectError::InvalidProject("tempo map is invalid"))
    ));
}

#[test]
fn canonical_fixture_with_invalid_meter_is_rejected() {
    let bytes = encoded_fixture_value(|value| {
        value["project"]["meter_map"][1]["signature"]["denominator"] = json!(3);
    });

    assert!(matches!(
        from_bytes(&bytes),
        Err(ProjectError::InvalidProject("meter map is invalid"))
    ));
}

#[test]
fn canonical_fixture_with_invalid_loop_is_rejected() {
    let bytes = encoded_fixture_value(|value| {
        value["project"]["transport"]["loop_region"]["end"] = json!(48_000);
    });

    assert!(matches!(
        from_bytes(&bytes),
        Err(ProjectError::InvalidProject("loop region is invalid"))
    ));
}
