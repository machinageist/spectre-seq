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
    selected_object: Option<SelectedObject>,
    command_palette_open: bool,
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
        Self {
            workflow,
            active_lens,
            selected_object: None,
            command_palette_open: false,
        }
    }

    // Replace the active workflow and move to its startup lens
    pub fn apply_workflow(&mut self, workflow: WorkflowProfile) {
        self.active_lens = workflow.startup_lens;
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
}
