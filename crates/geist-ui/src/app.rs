// Author: Jeff
// Date: 2026-06-08
// Description: Top-level UI app state and command dispatch.
// Notes: App validates UI commands before changing disposable UI state.

use crate::commands::{command_from_intent, UICommand, UICommandError};
use crate::state::{SelectedObject, UIState, UIStateError};
use geist_config::loader::{load_workflow_toml, resolve_workflow, WorkflowSource};
use geist_config::validate::ConfigDiagnostic;
use std::path::Path;

// Minimal app shell for workflow-driven UI state
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct App {
    state: UIState,
    pending_app_intents: Vec<geist_config::commands::CommandIntent>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_state(state: UIState) -> Self {
        Self {
            state,
            pending_app_intents: Vec::new(),
        }
    }

    // Build from startup workflow candidates using built-in/user/project precedence
    pub fn from_workflow_candidates(
        candidates: impl IntoIterator<
            Item = (
                WorkflowSource,
                Result<geist_config::schema::WorkflowProfile, Vec<ConfigDiagnostic>>,
            ),
        >,
    ) -> (Self, Vec<(WorkflowSource, ConfigDiagnostic)>) {
        let (workflow, diagnostics) = resolve_workflow(candidates);
        (
            Self::with_state(UIState::from_workflow(workflow)),
            diagnostics,
        )
    }

    // Build from optional startup workflow files; invalid later files keep last valid
    pub fn from_workflow_files<P: AsRef<Path>>(
        files: impl IntoIterator<Item = (WorkflowSource, P)>,
    ) -> (Self, Vec<(WorkflowSource, ConfigDiagnostic)>) {
        Self::from_workflow_candidates(
            files
                .into_iter()
                .map(|(source, path)| (source, load_workflow_toml(path.as_ref()))),
        )
    }

    pub fn state(&self) -> &UIState {
        &self.state
    }

    pub fn pending_app_intents(&self) -> &[geist_config::commands::CommandIntent] {
        &self.pending_app_intents
    }

    pub fn drain_pending_app_intents(&mut self) -> Vec<geist_config::commands::CommandIntent> {
        self.pending_app_intents.drain(..).collect()
    }

    // Load and apply one workflow TOML file on the app/control side
    pub fn load_workflow_file(&mut self, path: &Path) -> Result<(), Vec<ConfigDiagnostic>> {
        match load_workflow_toml(path) {
            Ok(workflow) => {
                self.state.apply_workflow(workflow);
                Ok(())
            }
            Err(diagnostics) => Err(diagnostics),
        }
    }

    // Dispatch one UI command and keep app/project mutation behind typed intents
    pub fn dispatch(&mut self, command: UICommand) -> Result<(), UICommandError> {
        match command {
            UICommand::SwitchLens(lens) => self
                .state
                .switch_lens(lens)
                .map_err(|UIStateError::HiddenLens(hidden)| UICommandError::HiddenLens(hidden)),
            UICommand::ApplyWorkflow(workflow) => {
                self.state.apply_workflow(workflow);
                Ok(())
            }
            UICommand::LoadWorkflowFile(path) => self
                .load_workflow_file(&path)
                .map_err(UICommandError::InvalidWorkflow),
            UICommand::OpenCommandPalette => {
                self.state.set_command_palette_open(true);
                Ok(())
            }
            UICommand::CloseCommandPalette => {
                self.state.set_command_palette_open(false);
                Ok(())
            }
            UICommand::ExecuteAlias(alias) => self.dispatch_alias(&alias),
            UICommand::ExecuteIntent(intent) => {
                if !intent.is_declarative() {
                    return Err(UICommandError::InvalidIntent(intent.command));
                }
                self.pending_app_intents.push(intent);
                Ok(())
            }
            UICommand::SelectTrack(track_id) => {
                self.state.select_object(SelectedObject::Track(track_id));
                Ok(())
            }
            UICommand::ClearSelection => {
                self.state.clear_selection();
                Ok(())
            }
        }
    }

    fn dispatch_alias(&mut self, alias: &str) -> Result<(), UICommandError> {
        let intent = self
            .state
            .workflow()
            .commands
            .aliases
            .get(alias)
            .cloned()
            .ok_or_else(|| UICommandError::UnknownAlias(alias.to_string()))?;
        let command = command_from_intent(&intent)?;
        self.dispatch(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_config::commands::CommandIntent;
    use geist_config::schema::{LensId, WorkflowProfile};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn alias_can_switch_visible_lens() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow
            .commands
            .aliases
            .insert("Graph".to_string(), CommandIntent::new("switch_lens:build"));
        let mut app = App::with_state(UIState::from_workflow(workflow));

        app.dispatch(UICommand::ExecuteAlias("Graph".to_string()))
            .unwrap();

        assert_eq!(app.state().active_lens(), LensId::Build);
    }

    #[test]
    fn alias_to_app_intent_stays_pending_for_app_core() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow
            .commands
            .aliases
            .insert("Add Track".to_string(), CommandIntent::new("add_track"));
        let mut app = App::with_state(UIState::from_workflow(workflow));

        app.dispatch(UICommand::ExecuteAlias("Add Track".to_string()))
            .unwrap();

        assert_eq!(
            app.pending_app_intents(),
            &[CommandIntent::new("add_track")]
        );
    }

    #[test]
    fn unknown_alias_is_rejected() {
        let mut app = App::new();
        let error = app
            .dispatch(UICommand::ExecuteAlias("Nope".to_string()))
            .unwrap_err();
        assert_eq!(error, UICommandError::UnknownAlias("Nope".to_string()));
    }

    #[test]
    fn startup_candidates_apply_latest_valid_and_report_invalid_sources() {
        let mut bundled = WorkflowProfile::default_profile();
        bundled.profile_id = "bundled".to_string();
        bundled.startup_lens = LensId::Build;
        let mut project = WorkflowProfile::default_profile();
        project.profile_id = "project".to_string();
        project.startup_lens = LensId::Mix;

        let (app, diagnostics) = App::from_workflow_candidates([
            (
                WorkflowSource::BuiltInDefault,
                Ok(WorkflowProfile::default_profile()),
            ),
            (WorkflowSource::BundledProfile, Ok(bundled)),
            (
                WorkflowSource::UserProfile,
                Err(vec![ConfigDiagnostic {
                    field: "startup_lens",
                    message: "startup lens must be visible".to_string(),
                }]),
            ),
            (WorkflowSource::ProjectOverride, Ok(project)),
        ]);

        assert_eq!(app.state().workflow().profile_id, "project");
        assert_eq!(app.state().active_lens(), LensId::Mix);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, WorkflowSource::UserProfile);
    }

    #[test]
    fn startup_file_resolution_falls_back_to_last_valid_profile() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let bundled = repo_root.join("assets/workflows/modular.toml");
        let invalid = temp_workflow_path("invalid-startup-workflow.toml");
        fs::write(
            &invalid,
            r#"
version = 1
profile_id = "bad"
display_name = "Bad"
startup_lens = "build"

[lenses]
order = ["arrange", "build"]
visible = ["arrange"]
"#,
        )
        .unwrap();

        let (app, diagnostics) = App::from_workflow_files([
            (WorkflowSource::BundledProfile, bundled),
            (WorkflowSource::UserProfile, invalid.clone()),
        ]);

        assert_eq!(app.state().workflow().profile_id, "modular-builder");
        assert_eq!(app.state().active_lens(), LensId::Build);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].0, WorkflowSource::UserProfile);
        let _ = fs::remove_file(invalid);
    }

    #[test]
    fn load_workflow_file_applies_valid_profile() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = repo_root.join("assets/workflows/modular.toml");
        let mut app = App::new();

        app.load_workflow_file(&path).unwrap();

        assert_eq!(app.state().workflow().profile_id, "modular-builder");
        assert_eq!(app.state().active_lens(), LensId::Build);
    }

    #[test]
    fn load_workflow_command_applies_valid_profile() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = repo_root.join("assets/workflows/mixing.toml");
        let mut app = App::new();

        app.dispatch(UICommand::LoadWorkflowFile(path)).unwrap();

        assert_eq!(app.state().workflow().profile_id, "mixing");
        assert_eq!(app.state().active_lens(), LensId::Mix);
    }

    #[test]
    fn invalid_workflow_file_keeps_existing_profile() {
        let path = temp_workflow_path("invalid-workflow.toml");
        fs::write(
            &path,
            r#"
version = 1
profile_id = "bad"
display_name = "Bad"
startup_lens = "build"

[lenses]
order = ["arrange", "build"]
visible = ["arrange"]
"#,
        )
        .unwrap();
        let mut app = App::new();

        let diagnostics = app.load_workflow_file(&path).unwrap_err();

        assert_eq!(app.state().workflow().profile_id, "default");
        assert_eq!(app.state().active_lens(), LensId::Arrange);
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.field == "startup_lens"));
        let _ = fs::remove_file(path);
    }

    fn temp_workflow_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("geist-ui-{}-{name}", std::process::id()))
    }
}
