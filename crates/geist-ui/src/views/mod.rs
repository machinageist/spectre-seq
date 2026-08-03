// Author: Jeff
// Date: 2026-06-08
// Description: Renderer-neutral view surfaces built from workflow-derived frame plans.
// Notes: Views consume RenderFrame data and emit no project mutations directly.

pub mod arrangement;
pub mod browser;
pub mod mixer;
pub mod modulation;
pub mod node_graph;
pub mod piano_roll;
pub mod plugin_rack;
pub mod step_sequencer;

use crate::renderer::{PanelPlacement, RenderFrame};
use spectre_config::schema::{Density, LensId};

// Complete renderer-neutral surface for the current UI frame
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSurface {
    pub workflow_id: String,
    pub density: Density,
    pub lens: LensId,
    pub lens_tabs: Vec<String>,
    pub panels: Vec<PanelPlacement>,
    pub main: LensSurface,
    pub context_actions: Vec<ActionChip>,
    pub command_palette_open: bool,
}

// Active lens content assembled from the configured frame plan
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensSurface {
    pub lens: LensId,
    pub title: String,
    pub purpose: &'static str,
    pub empty_actions: Vec<ActionChip>,
}

// Small visible command affordance; renderers may draw it as a chip/button/menu item
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionChip {
    pub label: String,
    pub command: String,
}

// Build the surface model concrete renderers consume
pub fn workspace_surface_from_frame(frame: &RenderFrame) -> WorkspaceSurface {
    WorkspaceSurface {
        workflow_id: frame.workflow_id.clone(),
        density: frame.density,
        lens: frame.active_lens,
        lens_tabs: frame
            .lens_tabs
            .iter()
            .map(|tab| {
                if tab.active {
                    format!("{}*", lens_label(tab.lens))
                } else {
                    lens_label(tab.lens).to_string()
                }
            })
            .collect(),
        panels: frame.panels.clone(),
        main: surface_for_lens(frame),
        context_actions: action_chips(&frame.context_actions),
        command_palette_open: frame.command_palette_open,
    }
}

fn surface_for_lens(frame: &RenderFrame) -> LensSurface {
    match frame.active_lens {
        LensId::Arrange => arrangement::surface(&frame.main_view),
        LensId::Build => node_graph::surface(&frame.main_view),
        LensId::Shape => plugin_rack::surface(&frame.main_view),
        LensId::Mix => mixer::surface(&frame.main_view),
        LensId::Browser => browser::surface(&frame.main_view),
        LensId::Modulation => modulation::surface(&frame.main_view),
    }
}

pub(crate) fn action_chips(commands: &[String]) -> Vec<ActionChip> {
    commands
        .iter()
        .map(|command| ActionChip {
            label: label_from_command(command),
            command: command.clone(),
        })
        .collect()
}

pub(crate) fn label_from_command(command: &str) -> String {
    command
        .split(['_', ':'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn lens_label(lens: LensId) -> &'static str {
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
    use crate::renderer::frame_from_state;
    use crate::state::{SelectedObject, UIState};
    use spectre_config::schema::{ContextShelfConfig, LensId, WorkflowProfile};

    #[test]
    fn workspace_surface_uses_configured_lens_and_empty_actions() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.startup_lens = LensId::Build;
        workflow.graph.empty_actions = vec!["add_source".to_string(), "open_browser".to_string()];
        let state = UIState::from_workflow(workflow);
        let frame = frame_from_state(&state);

        let surface = workspace_surface_from_frame(&frame);

        assert_eq!(surface.lens, LensId::Build);
        assert_eq!(surface.main.purpose, "Build and understand sound flow.");
        assert_eq!(surface.main.empty_actions[0].label, "Add Source");
        assert_eq!(surface.main.empty_actions[1].command, "open_browser");
    }

    #[test]
    fn workspace_surface_uses_configured_context_actions() {
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

        let surface = workspace_surface_from_frame(&frame);

        assert_eq!(surface.context_actions[0].label, "Add Effect");
        assert_eq!(surface.context_actions[1].command, "show_graph_branch");
    }
}
