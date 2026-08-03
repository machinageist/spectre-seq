// Author: Jeff
// Date: 2026-06-08
// Description: Deterministic egui renderer adapter scaffold.
// Notes: This adapter consumes workflow-backed UI state without owning project truth.

use crate::renderer::{frame_from_state, RenderFrame, Renderer};
use crate::state::UIState;
use crate::views::{workspace_surface_from_frame, WorkspaceSurface};
use crate::widgets::{workspace_widgets_from_surface, WorkspaceWidgets};

// Placeholder renderer until egui/eframe is added as a concrete dependency
#[derive(Clone, Debug, Default)]
pub struct EguiRenderer {
    last_frame: Option<RenderFrame>,
    last_surface: Option<WorkspaceSurface>,
    last_widgets: Option<WorkspaceWidgets>,
}

impl EguiRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_frame(&self) -> Option<&RenderFrame> {
        self.last_frame.as_ref()
    }

    pub fn last_surface(&self) -> Option<&WorkspaceSurface> {
        self.last_surface.as_ref()
    }

    pub fn last_widgets(&self) -> Option<&WorkspaceWidgets> {
        self.last_widgets.as_ref()
    }

    // Build concrete widget inputs from workflow-backed UI state without drawing yet
    pub fn render_widgets(&mut self, state: &UIState) -> WorkspaceWidgets {
        let frame = self.render(state);
        let surface = workspace_surface_from_frame(&frame);
        let widgets = workspace_widgets_from_surface(&surface);
        self.last_surface = Some(surface);
        self.last_widgets = Some(widgets.clone());
        widgets
    }
}

impl Renderer for EguiRenderer {
    fn render(&mut self, state: &UIState) -> RenderFrame {
        let frame = frame_from_state(state);
        self.last_frame = Some(frame.clone());
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_config::schema::{LensId, PanelId, WorkflowProfile};

    #[test]
    fn renderer_frame_tracks_active_workflow_lens() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.profile_id = "performance".to_string();
        workflow.startup_lens = LensId::Mix;
        let state = UIState::from_workflow(workflow);
        let mut renderer = EguiRenderer::new();

        let frame = renderer.render(&state);

        assert_eq!(frame.workflow_id, "performance");
        assert_eq!(frame.active_lens, LensId::Mix);
        assert_eq!(renderer.last_frame(), Some(&frame));
    }

    #[test]
    fn renderer_widgets_track_workflow_layout() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.startup_lens = LensId::Build;
        workflow.layout.bottom_panel = Some(PanelId::ModulationOverview);
        let state = UIState::from_workflow(workflow);
        let mut renderer = EguiRenderer::new();

        let widgets = renderer.render_widgets(&state);

        assert_eq!(widgets.lens_tabs[0].label, "Arrange");
        assert!(widgets
            .lens_tabs
            .iter()
            .any(|tab| tab.label == "Build" && tab.active));
        assert!(widgets
            .panels
            .iter()
            .any(|panel| panel.title == "Modulation Overview"));
        assert!(renderer.last_surface().is_some());
        assert_eq!(renderer.last_widgets(), Some(&widgets));
    }
}
