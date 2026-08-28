// Author: Jeff
// Date: 2026-08-28
// Description: R4 slice 7 structural guard — the render path never reaches the filesystem
// Notes: Same technique as crates/spectre-audio/tests/rt_guard.rs's module scan. Two assertions,
//   not one, because the boundary has two halves and only the second can actually fire today:
//   spectre-app does depend on spectre-project, so a manifest scan guards nothing there.

use std::path::Path;

const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");

fn read(relative: &str) -> String {
    let path = Path::new(MANIFEST).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

// U12
#[test]
fn render_path_crates_do_not_depend_on_the_project_crate() {
    for crate_name in [
        "spectre-audio",
        "spectre-graph",
        "spectre-dsp",
        "spectre-core",
    ] {
        let manifest = read(&format!("../{crate_name}/Cargo.toml"));
        assert!(
            !manifest.contains("spectre-project"),
            "{crate_name} must not depend on spectre-project: the crate that owns the document \
             is not on the callback path"
        );
    }
}

// U12b — the half that can fire
#[test]
fn the_apps_render_closure_module_names_no_persistence_api() {
    let engine = read("../spectre-app/src/engine.rs");
    for forbidden in [
        "save_project_atomic",
        "load_project",
        "to_bytes",
        "from_bytes",
        "ProjectEnvelope",
        "ProjectDoc",
        "std::fs",
    ] {
        assert!(
            !engine.contains(forbidden),
            "engine.rs builds the render closure and must not name {forbidden}"
        );
    }
    // Positive control: the file really was read and the scan is looking at the right module
    assert!(engine.contains("build_track_graph"));
}
