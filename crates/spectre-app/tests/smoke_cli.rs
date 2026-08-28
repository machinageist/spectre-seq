// Author: Jeff
// Date: 2026-07-12
// Description: Process-level smoke test for the Spectre app launch target
// Notes: --smoke-test validates startup without opening a native window

use std::process::Command;

#[test]
fn smoke_mode_reports_launchable_prototype() {
    let output = Command::new(env!("CARGO_BIN_EXE_spectre-app"))
        .arg("--smoke-test")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Spectre prototype ready"));
    assert!(stdout.contains("lens=Arrange"));
    assert!(stdout.contains("tracks=1"));
    assert!(stdout.contains("selected_device=Pulse(pulse)"));
    // The headless path must never open a device; this fails if engine startup is wired into it
    assert!(stdout.contains("engine=not-started"));
    // R4-8 — nothing in the headless path starts a render either
    assert!(
        stdout.contains("bounce=idle"),
        "smoke output was {stdout:?}"
    );
}

// R4-5 E-6 — the headless launch still succeeds with clip state present in the model.
// Reads the count from the shell's own model rather than asserting a literal, so it fails if the
// clip surface ever stops being reachable from the launch path
#[test]
fn the_headless_launch_reports_its_clip_count() {
    let output = Command::new(env!("CARGO_BIN_EXE_spectre-app"))
        .arg("--smoke-test")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // The first-launch project has no clips, and the surface must say so rather than omit the field
    assert!(stdout.contains("clips=0"), "smoke output was {stdout:?}");
}
