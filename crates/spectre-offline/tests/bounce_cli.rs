// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 8 evidence — the --bounce CLI round trip and the hash it must print
// Notes: The pinned hash is the live callback path's, asserted against the same literal in
//   crates/spectre-audio/tests/bounce_equivalence.rs. Two binaries agreeing with one written
//   constant is what makes this a round trip rather than a restatement: the constant was
//   produced by the live path, so a CLI that renders differently cannot match it.

use std::process::Command;

// The frame-major streaming hash of 4,096 frames of the fixture at 48 kHz in 256-frame blocks.
// Produced by the live RenderBridge path, not by the bounce; see the shared-constant note above
const LIVE_PATH_HASH: u64 = 6_709_076_177_999_973_021;

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
    assert_eq!(report["hash"].as_u64(), Some(LIVE_PATH_HASH));
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
