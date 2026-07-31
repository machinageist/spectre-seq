// Author: Jeff
// Date: 2026-06-08
// Description: UI state derived from project selection and workflow configuration.
// Notes: UI state is disposable view state; project and audio truth live outside widgets.

use geist_config::schema::{LayoutConfig, LensId, WorkflowProfile};

// Renderer-facing UI state snapshot
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UIState {
    workflow: WorkflowProfile,
    active_lens: LensId,
    focused_pane: WorkspacePane,
    left_panel_open: bool,
    right_panel_open: bool,
    selected_object: Option<SelectedObject>,
    command_palette_open: bool,
}

// Stable keyboard-focus targets in the tiled studio shell
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspacePane {
    Transport,
    Left,
    Main,
    Right,
    Monitor,
}

// UI-only selection anchor used to choose contextual actions
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedObject {
    Track(String),
    Clip(String),
    Node(String),
    Cable(String),
    Parameter(String),
    ModulationRoute(String),
}

impl UIState {
    // Build UI state from the default workflow profile
    pub fn new() -> Self {
        Self::from_workflow(WorkflowProfile::default_profile())
    }

    // Build studio UI state: every lens visible, opening on the synth/effects lens
    pub fn studio() -> Self {
        let mut workflow = WorkflowProfile::default_profile();
        let all = vec![
            LensId::Arrange,
            LensId::Build,
            LensId::Shape,
            LensId::Mix,
            LensId::Browser,
            LensId::Modulation,
        ];
        workflow.lenses.order = all.clone();
        workflow.lenses.visible = all;
        workflow.startup_lens = LensId::Shape;
        Self::from_workflow(workflow)
    }

    // Build UI state from a validated workflow profile
    pub fn from_workflow(workflow: WorkflowProfile) -> Self {
        let active_lens = workflow.startup_lens;
        let left_panel_open = workflow.layout.left_panel.is_some();
        let right_panel_open = workflow.layout.right_panel.is_some();
        Self {
            workflow,
            active_lens,
            focused_pane: WorkspacePane::Main,
            left_panel_open,
            right_panel_open,
            selected_object: None,
            command_palette_open: false,
        }
    }

    // Replace the active workflow and move to its startup lens
    pub fn apply_workflow(&mut self, workflow: WorkflowProfile) {
        self.active_lens = workflow.startup_lens;
        self.left_panel_open = workflow.layout.left_panel.is_some();
        self.right_panel_open = workflow.layout.right_panel.is_some();
        self.focused_pane = WorkspacePane::Main;
        self.workflow = workflow;
    }

    // Switch to a visible lens from the active workflow
    pub fn switch_lens(&mut self, lens: LensId) -> Result<(), UIStateError> {
        if !self.workflow.lenses.visible.contains(&lens) {
            return Err(UIStateError::HiddenLens(lens));
        }
        self.active_lens = lens;
        Ok(())
    }

    pub fn workflow(&self) -> &WorkflowProfile {
        &self.workflow
    }

    pub fn active_lens(&self) -> LensId {
        self.active_lens
    }

    pub fn focused_pane(&self) -> WorkspacePane {
        self.focused_pane
    }

    pub fn focus_pane(&mut self, pane: WorkspacePane) {
        self.focused_pane = pane;
    }

    pub fn left_panel_open(&self) -> bool {
        self.left_panel_open
    }

    pub fn right_panel_open(&self) -> bool {
        self.right_panel_open
    }

    pub fn toggle_left_panel(&mut self) {
        self.left_panel_open = self.layout().left_panel.is_some() && !self.left_panel_open;
        if !self.left_panel_open && self.focused_pane == WorkspacePane::Left {
            self.focused_pane = WorkspacePane::Main;
        }
    }

    pub fn toggle_right_panel(&mut self) {
        self.right_panel_open = self.layout().right_panel.is_some() && !self.right_panel_open;
        if !self.right_panel_open && self.focused_pane == WorkspacePane::Right {
            self.focused_pane = WorkspacePane::Main;
        }
    }

    pub fn focus_horizontally(&mut self, right: bool) {
        self.focused_pane = match (self.focused_pane, right) {
            (WorkspacePane::Main, false) if self.left_panel_open => WorkspacePane::Left,
            (WorkspacePane::Main, true) if self.right_panel_open => WorkspacePane::Right,
            (WorkspacePane::Left, true) | (WorkspacePane::Right, false) => WorkspacePane::Main,
            (pane, _) => pane,
        };
    }

    // Move through the current vertical tile stack without depending on pixels
    pub fn cycle_pane_focus(&mut self, forward: bool) {
        self.focused_pane = match (self.focused_pane, forward) {
            (WorkspacePane::Transport, true) | (WorkspacePane::Monitor, false) => {
                WorkspacePane::Main
            }
            (WorkspacePane::Main, true) => WorkspacePane::Monitor,
            (WorkspacePane::Monitor, true) => WorkspacePane::Transport,
            (WorkspacePane::Transport, false) => WorkspacePane::Monitor,
            (WorkspacePane::Main, false) => WorkspacePane::Transport,
            (WorkspacePane::Left | WorkspacePane::Right, _) => WorkspacePane::Main,
        };
    }

    pub fn visible_lenses(&self) -> &[LensId] {
        &self.workflow.lenses.visible
    }

    pub fn layout(&self) -> &LayoutConfig {
        &self.workflow.layout
    }

    pub fn selected_object(&self) -> Option<&SelectedObject> {
        self.selected_object.as_ref()
    }

    pub fn select_object(&mut self, selection: SelectedObject) {
        self.selected_object = Some(selection);
    }

    pub fn clear_selection(&mut self) {
        self.selected_object = None;
    }

    pub fn command_palette_open(&self) -> bool {
        self.command_palette_open
    }

    pub fn set_command_palette_open(&mut self, open: bool) {
        self.command_palette_open = open;
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UIStateError {
    HiddenLens(LensId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_config::schema::{LensConfig, WorkflowProfile};

    #[test]
    fn default_state_uses_default_workflow_startup_lens() {
        let state = UIState::new();
        assert_eq!(state.workflow().profile_id, "default");
        assert_eq!(state.active_lens(), LensId::Arrange);
    }

    #[test]
    fn studio_state_shows_every_lens_opening_on_shape() {
        let state = UIState::studio();
        assert_eq!(state.visible_lenses().len(), 6);
        assert_eq!(state.active_lens(), LensId::Shape);
        // Every visible lens is switchable
        for &lens in state.visible_lenses() {
            let mut s = state.clone();
            assert!(s.switch_lens(lens).is_ok());
        }
    }

    #[test]
    fn applying_workflow_sets_startup_lens() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.profile_id = "modular".to_string();
        workflow.startup_lens = LensId::Build;

        let mut state = UIState::new();
        state.apply_workflow(workflow);

        assert_eq!(state.workflow().profile_id, "modular");
        assert_eq!(state.active_lens(), LensId::Build);
    }

    #[test]
    fn hidden_lens_switch_is_rejected() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.lenses = LensConfig {
            order: vec![LensId::Arrange, LensId::Build],
            visible: vec![LensId::Arrange],
        };

        let mut state = UIState::from_workflow(workflow);
        assert_eq!(
            state.switch_lens(LensId::Build),
            Err(UIStateError::HiddenLens(LensId::Build))
        );
        assert_eq!(state.active_lens(), LensId::Arrange);
    }

    #[test]
    fn pane_focus_cycles_through_the_vertical_tile_stack() {
        let mut state = UIState::new();
        assert_eq!(state.focused_pane(), WorkspacePane::Main);
        state.cycle_pane_focus(true);
        assert_eq!(state.focused_pane(), WorkspacePane::Monitor);
        state.cycle_pane_focus(true);
        assert_eq!(state.focused_pane(), WorkspacePane::Transport);
        state.cycle_pane_focus(false);
        assert_eq!(state.focused_pane(), WorkspacePane::Monitor);
    }

    #[test]
    fn side_panels_focus_and_collapse_without_stranding_focus() {
        let mut state = UIState::new();
        state.focus_horizontally(false);
        assert_eq!(state.focused_pane(), WorkspacePane::Left);
        state.toggle_left_panel();
        assert!(!state.left_panel_open());
        assert_eq!(state.focused_pane(), WorkspacePane::Main);
        state.focus_horizontally(true);
        assert_eq!(state.focused_pane(), WorkspacePane::Right);
    }
}
