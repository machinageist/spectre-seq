// Author: Jeff
// Date: 2026-07-12
// Description: Contract tests for deterministic offline harness reports
// Notes: R0 proves reproducible project loading before R2 adds audio rendering

use geist_offline::{default_project, inspect_project, render_vertical_slice};
use geist_project::to_bytes;

#[test]
fn default_project_report_is_deterministic() {
    let bytes = to_bytes(&default_project()).unwrap();
    let first = inspect_project(&bytes).unwrap();
    let second = inspect_project(&bytes).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.schema_version, 1);
    assert_eq!(first.project_name, "Untitled");
    assert_eq!(first.tempo_segment_count, 1);
    assert_eq!(first.transport_position_samples, 0);
}

#[test]
fn invalid_project_is_reported_without_panicking() {
    let error = inspect_project(b"not a project").unwrap_err();
    assert!(error.contains("malformed project"));
}

#[test]
fn native_device_chain_renders_deterministically() {
    let first = render_vertical_slice(48_000.0, 4_096).unwrap();
    let second = render_vertical_slice(48_000.0, 4_096).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.frames, 4_096);
    assert_eq!(first.channels, 2);
    assert_ne!(first.hash, 0);
    assert!(first.peak > 0.0 && first.peak <= 1.0);
}
