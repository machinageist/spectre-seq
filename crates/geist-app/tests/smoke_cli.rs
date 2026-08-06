// Author: Jeff
// Date: 2026-07-12
// Description: Process-level smoke test for the Geist app launch target
// Notes: --smoke-test validates startup without opening a native window

use std::process::Command;

#[test]
fn smoke_mode_reports_launchable_prototype() {
    let output = Command::new(env!("CARGO_BIN_EXE_geist-app"))
        .arg("--smoke-test")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Geist prototype ready"));
    assert!(stdout.contains("lens=Arrange"));
    assert!(stdout.contains("tracks=1"));
    assert!(stdout.contains("selected_device=Pulse(pulse)"));
}
