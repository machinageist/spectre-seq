// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 8 evidence — the bounce panel's pre-render checks, tested without a window
// Notes: This repository has no GUI-driving harness and R4-8 adds none. What is testable is the
//   decision the button-disabled rule is made from, so it lives in a free function and is
//   asserted here rather than checked by hand.

use spectre_app::bounce_panel::{
    duration_label, validate_request, BouncePanel, BounceRequestError, BounceState,
};
use spectre_offline::bounce::max_frames;

const RATE: f64 = 48_000.0;

// E1
#[test]
fn an_empty_destination_is_refused_before_anything_renders() {
    assert_eq!(
        validate_request("", 1_024, max_frames(RATE)),
        Err(BounceRequestError::DestinationEmpty)
    );
    // Whitespace is empty; a path made only of spaces would create a file nobody can find
    assert_eq!(
        validate_request("   ", 1_024, max_frames(RATE)),
        Err(BounceRequestError::DestinationEmpty)
    );
}

// E1
#[test]
fn a_destination_in_a_missing_directory_is_refused() {
    let result = validate_request(
        "/definitely/not/a/real/directory/take.wav",
        1_024,
        max_frames(RATE),
    );
    assert!(matches!(
        result,
        Err(BounceRequestError::DestinationParentMissing(_))
    ));

    // A directory that does exist passes, and so does a bare filename with no directory at all
    let existing = std::env::temp_dir().join("take.wav");
    assert_eq!(
        validate_request(existing.to_str().unwrap(), 1_024, max_frames(RATE)),
        Ok(())
    );
    assert_eq!(
        validate_request("take.wav", 1_024, max_frames(RATE)),
        Ok(())
    );
}

// E3
#[test]
fn a_zero_or_over_ceiling_length_is_refused() {
    let existing = std::env::temp_dir().join("take.wav");
    let path = existing.to_str().unwrap();
    let ceiling = max_frames(RATE);

    assert_eq!(
        validate_request(path, 0, ceiling),
        Err(BounceRequestError::LengthZero)
    );
    assert_eq!(
        validate_request(path, ceiling + 1, ceiling),
        Err(BounceRequestError::LengthAboveCeiling {
            frames: ceiling + 1,
            maximum: ceiling,
        })
    );
    // The ceiling itself is allowed; an off-by-one here would refuse a legal 24-hour render
    assert_eq!(validate_request(path, ceiling, ceiling), Ok(()));
    assert_eq!(validate_request(path, 1, ceiling), Ok(()));
}

// The destination is checked before the length, so an empty field does not also shout about
// a length the user has not reached yet
#[test]
fn the_destination_is_reported_before_the_length() {
    assert_eq!(
        validate_request("", 0, max_frames(RATE)),
        Err(BounceRequestError::DestinationEmpty)
    );
}

#[test]
fn the_duration_readout_matches_the_frame_count() {
    assert_eq!(duration_label(0, RATE), "00:00.000");
    assert_eq!(duration_label(48_000, RATE), "00:01.000");
    assert_eq!(duration_label(48_000 * 61, RATE), "01:01.000");
    assert_eq!(duration_label(24, RATE), "00:00.001");
    // A rate of zero is a defect upstream, not a divide-by-zero here
    assert_eq!(duration_label(48_000, 0.0), "--:--.---");
}

// A fresh panel has started nothing. This is the state the headless smoke line reports
#[test]
fn a_fresh_panel_is_idle_with_no_report() {
    let panel = BouncePanel::default();
    assert_eq!(panel.state(), BounceState::Idle);
    assert_eq!(panel.state().as_str(), "idle");
    assert!(panel.report().is_none());
    assert_eq!(panel.progress(), (0, 0));
    // Default destination is empty, so the button is disabled until the user types one
    assert_eq!(
        validate_request(&panel.destination, panel.frames, max_frames(RATE)),
        Err(BounceRequestError::DestinationEmpty)
    );
}

// A full run through the panel's own worker: start, poll to completion, read the report
#[test]
fn a_panel_render_writes_a_file_and_reports_it() {
    let path = std::env::temp_dir().join("spectre-panel-bounce.wav");
    let _ = std::fs::remove_file(&path);

    let mut panel = BouncePanel::default();
    panel.destination = path.to_str().unwrap().to_string();
    panel.frames = 1_024;

    let config = spectre_offline::bounce::BounceConfig {
        sample_rate: RATE,
        frames: 1_024,
        block_frames: 256,
        log_block_hashes: false,
    };
    let model = spectre_app::AppModel::prototype();
    let values = model.device_parameter_snapshot().unwrap();
    let events = spectre_offline::fixture_events(1_024).to_vec();
    panel.start(config, values, events).unwrap();
    assert_eq!(panel.state(), BounceState::Running);

    // Poll the way the shell's repaint does, rather than joining, so this exercises the path
    // the UI actually takes
    for _ in 0..2_000 {
        panel.poll();
        if panel.state() != BounceState::Running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    assert_eq!(panel.state(), BounceState::Finished, "{}", panel.message);
    let report = panel.report().unwrap();
    assert_eq!(report.frames, 1_024);
    assert_eq!(report.blocks, 4);
    assert!(report.peak > 0.0);
    assert_eq!(report.contaminated_nodes, 0);
    assert_eq!(panel.progress(), (4, 4));

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes.len(), 44 + 1_024 * 2 * 4);
    std::fs::remove_file(&path).unwrap();
}

// A failed render leaves nothing behind. A truncated file is indistinguishable from a finished
// short one once the app is closed, which is the whole reason for the rule
#[test]
fn a_refused_render_leaves_no_partial_file() {
    let path = std::env::temp_dir().join("spectre-panel-refused.wav");
    let _ = std::fs::remove_file(&path);

    let mut panel = BouncePanel::default();
    panel.destination = path.to_str().unwrap().to_string();
    // A block size of zero is refused by BounceConfig validation, after the file is created
    let config = spectre_offline::bounce::BounceConfig {
        sample_rate: RATE,
        frames: 1_024,
        block_frames: 0,
        log_block_hashes: false,
    };
    let model = spectre_app::AppModel::prototype();
    let values = model.device_parameter_snapshot().unwrap();
    panel.start(config, values, Vec::new()).unwrap();

    for _ in 0..2_000 {
        panel.poll();
        if panel.state() != BounceState::Running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    assert_eq!(panel.state(), BounceState::Failed);
    assert!(!panel.message.is_empty());
    assert!(
        !path.exists(),
        "a failed render must remove its partial file"
    );
}
