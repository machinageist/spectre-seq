// Author: Jeff
// Date: 2026-06-08
// Description: End-to-end workflow profile rendering coverage.
// Notes: Bundled config must affect UI frame planning without touching project/audio truth.

use spectre_config::loader::load_workflow_toml;
use spectre_config::schema::{LensId, PanelId};
use spectre_ui::egui_renderer::EguiRenderer;
use spectre_ui::prelude::UIState;
use spectre_ui::renderer::{PanelSlot, Renderer};
use spectre_ui::views::workspace_surface_from_frame;
use spectre_ui::widgets::workspace_widgets_from_surface;
use std::path::PathBuf;

#[test]
fn bundled_modular_profile_drives_render_frame() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = repo_root.join("assets/workflows/modular.toml");
    let workflow = load_workflow_toml(&path).expect("bundled modular workflow should validate");
    let state = UIState::from_workflow(workflow);
    let mut renderer = EguiRenderer::new();

    let frame = renderer.render(&state);

    assert_eq!(frame.workflow_id, "modular-builder");
    assert_eq!(frame.active_lens, LensId::Build);
    assert_eq!(frame.lens_tabs[0].lens, LensId::Build);
    assert!(frame.lens_tabs[0].active);
    assert!(frame.panels.iter().any(|placement| {
        placement.slot == PanelSlot::Bottom && placement.panel == PanelId::ModulationOverview
    }));
    assert_eq!(
        frame.main_view.empty_actions,
        vec![
            "add_source",
            "add_processor",
            "add_modulator",
            "open_browser"
        ]
    );
}

#[test]
fn bundled_modular_profile_drives_view_surface() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = repo_root.join("assets/workflows/modular.toml");
    let workflow = load_workflow_toml(&path).expect("bundled modular workflow should validate");
    let state = UIState::from_workflow(workflow);
    let mut renderer = EguiRenderer::new();

    let frame = renderer.render(&state);
    let surface = workspace_surface_from_frame(&frame);

    assert_eq!(surface.workflow_id, "modular-builder");
    assert_eq!(surface.main.purpose, "Build and understand sound flow.");
    assert_eq!(surface.lens_tabs[0], "Build*");
    assert_eq!(surface.main.empty_actions[0].label, "Add Source");
    assert_eq!(surface.main.empty_actions[2].command, "add_modulator");
}

#[test]
fn bundled_modular_profile_drives_widget_inputs() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = repo_root.join("assets/workflows/modular.toml");
    let workflow = load_workflow_toml(&path).expect("bundled modular workflow should validate");
    let state = UIState::from_workflow(workflow);
    let mut renderer = EguiRenderer::new();

    let frame = renderer.render(&state);
    let surface = workspace_surface_from_frame(&frame);
    let widgets = workspace_widgets_from_surface(&surface);

    assert_eq!(widgets.workflow_id, "modular-builder");
    assert_eq!(widgets.lens_tabs[0].command, "switch_lens:build");
    assert!(widgets.lens_tabs[0].active);
    assert!(widgets
        .panels
        .iter()
        .any(|panel| panel.title == "Modulation Overview"));
    assert_eq!(widgets.main.empty_actions[0].label, "Add Source");
    assert_eq!(widgets.main.empty_actions[2].command, "add_modulator");
}
