// Author: Jeff
// Date: 2026-06-08
// Description: UI command types emitted by widgets and workflow bindings.
// Notes: Commands are typed app intents; workflow aliases cannot bypass validation.

use geist_config::commands::CommandIntent;
use geist_config::schema::{LensId, WorkflowProfile};
use geist_config::validate::ConfigDiagnostic;
use std::path::PathBuf;

// Typed UI command boundary between widgets and the app layer
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UICommand {
    SwitchLens(LensId),
    ApplyWorkflow(WorkflowProfile),
    LoadWorkflowFile(PathBuf),
    OpenCommandPalette,
    CloseCommandPalette,
    ExecuteAlias(String),
    ExecuteIntent(CommandIntent),
    SelectTrack(String),
    ClearSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UICommandError {
    UnknownAlias(String),
    InvalidIntent(String),
    HiddenLens(LensId),
    InvalidWorkflow(Vec<ConfigDiagnostic>),
}

// Convert a declarative command intent into a typed UI command when it is UI-local
pub fn command_from_intent(intent: &CommandIntent) -> Result<UICommand, UICommandError> {
    if !intent.is_declarative() {
        return Err(UICommandError::InvalidIntent(intent.command.clone()));
    }

    match intent.command.as_str() {
        "open_command_palette" => Ok(UICommand::OpenCommandPalette),
        command if command.starts_with("switch_lens:") => {
            let lens_name = command.trim_start_matches("switch_lens:");
            let lens = parse_lens_id(lens_name)
                .ok_or_else(|| UICommandError::InvalidIntent(intent.command.clone()))?;
            Ok(UICommand::SwitchLens(lens))
        }
        _ => Ok(UICommand::ExecuteIntent(intent.clone())),
    }
}

fn parse_lens_id(value: &str) -> Option<LensId> {
    match value {
        "arrange" => Some(LensId::Arrange),
        "build" => Some(LensId::Build),
        "shape" => Some(LensId::Shape),
        "mix" => Some(LensId::Mix),
        "browser" => Some(LensId::Browser),
        "modulation" => Some(LensId::Modulation),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_switch_lens_intent() {
        let command = command_from_intent(&CommandIntent::new("switch_lens:build")).unwrap();
        assert_eq!(command, UICommand::SwitchLens(LensId::Build));
    }

    #[test]
    fn keeps_non_ui_intents_typed_for_app_dispatch() {
        let command = command_from_intent(&CommandIntent::new("add_track")).unwrap();
        assert_eq!(
            command,
            UICommand::ExecuteIntent(CommandIntent::new("add_track"))
        );
    }

    #[test]
    fn rejects_shell_like_intent() {
        let error = command_from_intent(&CommandIntent::new("shell:open /tmp")).unwrap_err();
        assert_eq!(
            error,
            UICommandError::InvalidIntent("shell:open /tmp".to_string())
        );
    }
}
