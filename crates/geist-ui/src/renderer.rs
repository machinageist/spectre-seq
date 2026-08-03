// Author: Jeff
// Date: 2026-06-08
// Description: Renderer abstraction driven by UIState and workflow configuration.
// Notes: Renderers read UI state and emit commands; they do not own DAW truth.

use crate::commands::UICommand;
use crate::state::{SelectedObject, UIState, WorkspacePane};
use spectre_config::schema::{Density, LensId, PanelEdge, PanelId};

// Swappable renderer boundary for egui now and wgpu later
pub trait Renderer {
    fn render(&mut self, state: &UIState) -> RenderFrame;
}

// Renderer-neutral frame plan derived from the active workflow profile
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderFrame {
    pub workflow_id: String,
    pub density: Density,
    pub transport_edge: PanelEdge,
    pub active_lens: LensId,
    pub focused_pane: WorkspacePane,
    pub lens_tabs: Vec<LensTab>,
    pub panels: Vec<PanelPlacement>,
    pub main_view: ViewPlan,
    pub context_actions: Vec<String>,
    pub command_palette_open: bool,
    pub emitted_commands: Vec<UICommand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensTab {
    pub lens: LensId,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PanelPlacement {
    pub slot: PanelSlot,
    pub panel: PanelId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelSlot {
    Left,
    Right,
    Bottom,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewPlan {
    pub lens: LensId,
    pub title: &'static str,
    pub empty_actions: Vec<String>,
}

// Build a deterministic frame plan from workflow-backed UI state
pub fn frame_from_state(state: &UIState) -> RenderFrame {
    let workflow = state.workflow();
    let layout = state.layout();

    RenderFrame {
        workflow_id: workflow.profile_id.clone(),
        density: layout.density,
        transport_edge: layout.transport,
        active_lens: state.active_lens(),
        focused_pane: state.focused_pane(),
        lens_tabs: state
            .visible_lenses()
            .iter()
            .copied()
            .map(|lens| LensTab {
                lens,
                active: lens == state.active_lens(),
            })
            .collect(),
        panels: panel_placements(state),
        main_view: ViewPlan {
            lens: state.active_lens(),
            title: lens_title(state.active_lens()),
            empty_actions: empty_actions_for_lens(state),
        },
        context_actions: context_actions_for_selection(state),
        command_palette_open: state.command_palette_open(),
        emitted_commands: Vec::new(),
    }
}

fn panel_placements(state: &UIState) -> Vec<PanelPlacement> {
    let layout = state.layout();
    [
        (PanelSlot::Left, layout.left_panel),
        (PanelSlot::Right, layout.right_panel),
        (PanelSlot::Bottom, layout.bottom_panel),
    ]
    .into_iter()
    .filter_map(|(slot, panel)| panel.map(|panel| PanelPlacement { slot, panel }))
    .collect()
}

fn empty_actions_for_lens(state: &UIState) -> Vec<String> {
    match state.active_lens() {
        LensId::Build => state.workflow().graph.empty_actions.clone(),
        LensId::Arrange => vec![
            "add_track".to_string(),
            "add_instrument".to_string(),
            "add_sample".to_string(),
            "open_browser".to_string(),
        ],
        LensId::Mix => vec!["add_audio_track".to_string(), "add_bus".to_string()],
        LensId::Shape => vec!["pin_parameter".to_string(), "add_effect".to_string()],
        LensId::Browser => vec!["search".to_string(), "show_favorites".to_string()],
        LensId::Modulation => vec!["add_modulator".to_string(), "show_routes".to_string()],
    }
}

fn context_actions_for_selection(state: &UIState) -> Vec<String> {
    let Some(selection) = state.selected_object() else {
        return Vec::new();
    };
    let key = match selection {
        SelectedObject::Track(_) => "track",
        SelectedObject::Clip(_) => "clip",
        SelectedObject::Node(_) => "node",
        SelectedObject::Cable(_) => "cable",
        SelectedObject::Parameter(_) => "parameter",
        SelectedObject::ModulationRoute(_) => "modulation_route",
    };
    state
        .workflow()
        .context_shelf
        .get(key)
        .map(|shelf| shelf.actions.clone())
        .unwrap_or_default()
}

fn lens_title(lens: LensId) -> &'static str {
    match lens {
        LensId::Arrange => "Arrange",
        LensId::Build => "Build",
        LensId::Shape => "Shape",
        LensId::Mix => "Mix",
        LensId::Browser => "Browser",
        LensId::Modulation => "Modulation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{SelectedObject, UIState};
    use spectre_config::schema::{ContextShelfConfig, LensId, PanelId, WorkflowProfile};

    #[test]
    fn frame_uses_workflow_lens_order_and_layout() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.profile_id = "modular".to_string();
        workflow.startup_lens = LensId::Build;
        workflow.lenses.visible = vec![LensId::Build, LensId::Shape, LensId::Mix];
        workflow.layout.left_panel = Some(PanelId::Browser);
        workflow.layout.right_panel = Some(PanelId::ContextShelf);
        workflow.layout.bottom_panel = Some(PanelId::ModulationOverview);

        let state = UIState::from_workflow(workflow);
        let frame = frame_from_state(&state);

        assert_eq!(frame.workflow_id, "modular");
        assert_eq!(frame.active_lens, LensId::Build);
        assert_eq!(frame.focused_pane, WorkspacePane::Main);
        assert_eq!(frame.lens_tabs.len(), 3);
        assert_eq!(frame.lens_tabs[0].lens, LensId::Build);
        assert!(frame.lens_tabs[0].active);
        assert_eq!(frame.panels.len(), 3);
        assert_eq!(
            frame.main_view.empty_actions,
            state.workflow().graph.empty_actions
        );
    }

    #[test]
    fn frame_uses_configured_context_shelf_actions_for_selection() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.context_shelf.insert(
            "track".to_string(),
            ContextShelfConfig {
                actions: vec!["add_effect".to_string(), "show_graph_branch".to_string()],
            },
        );
        let mut state = UIState::from_workflow(workflow);
        state.select_object(SelectedObject::Track("track-1".to_string()));

        let frame = frame_from_state(&state);

        assert_eq!(
            frame.context_actions,
            vec!["add_effect".to_string(), "show_graph_branch".to_string()]
        );
    }
}
