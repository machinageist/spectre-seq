// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 8 evidence — the --bounce CLI round trip and the hash it must print
// Notes: The expected hash is computed in this process rather than pinned as a literal. It used
//   to be a constant produced on macOS, but device math reaches the platform's libm through tanh
//   and powf, so that number was per-platform and failed on Linux while every rendered sample
//   still agreed. The round trip survives the change: this file proves the CLI subprocess equals
//   an in-process bounce, and crates/spectre-audio/tests/bounce_equivalence.rs proves that same
//   bounce equals the live callback path, so a CLI that renders differently from the live path
//   still cannot pass both.

use spectre_core::ObjectId;
use spectre_dsp::{
    DeviceParameterSnapshot, GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS,
};
use spectre_offline::bounce::{bounce_report, BounceConfig};
use spectre_offline::{fixture, fixture_events};
use std::process::Command;

// The CLI's own fixed device identities, restated because main.rs keeps them private. A
// generated identity here would describe a different device set than the binary under test
const PULSE_DEVICE: u64 = 10;
const PULSE_LEVEL: u64 = 11;
const GAIN_DEVICE: u64 = 20;
const GAIN_VALUE: u64 = 21;
const SATURATOR_DEVICE: u64 = 30;
const SATURATOR_DRIVE: u64 = 31;
const SATURATOR_MIX: u64 = 32;

fn id(raw: u64) -> ObjectId {
    ObjectId::from_raw(raw).unwrap()
}

// The fixture's four parameters in the identity shape DeviceValues::from_snapshot accepts
fn fixture_snapshot() -> Vec<DeviceParameterSnapshot> {
    vec![
        DeviceParameterSnapshot::new(
            id(PULSE_DEVICE),
            "pulse",
            id(PULSE_LEVEL),
            PULSE_PARAMETERS[0],
            fixture::FIXTURE_PULSE_LEVEL,
        ),
        DeviceParameterSnapshot::new(
            id(GAIN_DEVICE),
            "gain",
            id(GAIN_VALUE),
            GAIN_PARAMETERS[0],
            fixture::FIXTURE_GAIN,
        ),
        DeviceParameterSnapshot::new(
            id(SATURATOR_DEVICE),
            "saturator",
            id(SATURATOR_DRIVE),
            SATURATOR_PARAMETERS[0],
            fixture::FIXTURE_SATURATOR_DRIVE,
        ),
        DeviceParameterSnapshot::new(
            id(SATURATOR_DEVICE),
            "saturator",
            id(SATURATOR_MIX),
            SATURATOR_PARAMETERS[1],
            fixture::FIXTURE_SATURATOR_MIX,
        ),
    ]
}

// The hash the same render produces in this process, for the geometry the CLI is invoked with.
// Falsified on both arguments that can move it: 2,048 frames and 44,100 Hz each fail the
// assertion below. block_frames does NOT move it, and that is the render's property rather than
// a gap here — a clean render folds the same samples in the same order however it is chunked,
// which is exactly what a_contaminated_render_is_not_identical_across_block_sizes scopes
fn expected_hash(frames: usize, sample_rate: f64, block_frames: usize) -> u64 {
    bounce_report(
        BounceConfig {
            sample_rate,
            frames,
            block_frames,
            log_block_hashes: false,
        },
        &fixture_snapshot(),
        &fixture_events(frames),
    )
    .unwrap()
    .hash
}

fn bounce(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_spectre-offline"))
        .args(arguments)
        .output()
        .unwrap()
}

// 20
#[test]
fn the_bounce_cli_prints_the_live_paths_hash() {
    let output = bounce(&[
        "--bounce", "--frames", "4096", "--rate", "48000", "--block", "256",
    ]);
    assert!(output.status.success(), "{:?}", output.status);

    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        report["hash"].as_u64(),
        Some(expected_hash(4_096, 48_000.0, 256))
    );
    assert_eq!(report["frames"].as_u64(), Some(4_096));
    assert_eq!(report["blocks"].as_u64(), Some(16));
    assert_eq!(report["channels"].as_u64(), Some(2));
    assert_eq!(report["contaminated_nodes"].as_u64(), Some(0));
    // A silent render would print a stable hash too
    assert!(report["peak"].as_f64().unwrap() > 0.0);
    // The per-block log costs a vector nobody reads unless it is asked for
    assert_eq!(report["block_hashes"].as_array().map(Vec::len), Some(0));
}

#[test]
fn the_bounce_cli_writes_a_wav_whose_length_matches_its_report() {
    let path = std::env::temp_dir().join("spectre-bounce-cli.wav");
    let _ = std::fs::remove_file(&path);
    let output = bounce(&[
        "--bounce",
        "--frames",
        "1024",
        "--block",
        "256",
        "--out",
        path.to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let bytes = std::fs::read(&path).unwrap();
    // 44-byte header plus 1,024 frames x 2 channels x 4 bytes
    assert_eq!(bytes.len(), 44 + 1_024 * 2 * 4);
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    std::fs::remove_file(&path).unwrap();
}

// A refusal is a refusal at the process boundary too, not a zero-length file and exit 0
#[test]
fn the_bounce_cli_refuses_a_length_past_the_ceiling() {
    let output = bounce(&["--bounce", "--frames", "999999999999", "--rate", "48000"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("ceiling"), "stderr was {stderr:?}");
}

#[test]
fn bounce_without_a_length_is_refused_rather_than_guessed() {
    let output = bounce(&["--bounce"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--frames"), "stderr was {stderr:?}");
}

// The pre-existing modes must still behave exactly as they did
#[test]
fn the_self_test_mode_is_unchanged() {
    let output = bounce(&["--self-test"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(report["schema_version"].is_number());
    assert!(report["project_name"].is_string());
}
