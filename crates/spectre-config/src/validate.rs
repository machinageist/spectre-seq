// Author: Jeff
// Date: 2026-06-08
// Description: Validation rules for workflow configuration snapshots.
// Notes: Invalid config falls back before UI state changes reach the renderer.

use crate::commands::CommandIntent;
use crate::keybindings::validate_keybindings;
use crate::schema::{WorkflowProfile, CURRENT_WORKFLOW_VERSION};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigDiagnostic {
    pub field: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigReport {
    pub diagnostics: Vec<ConfigDiagnostic>,
}

impl ConfigReport {
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn push(&mut self, field: &'static str, message: impl Into<String>) {
        self.diagnostics.push(ConfigDiagnostic {
            field,
            message: message.into(),
        });
    }
}

// Validate a complete immutable workflow profile before it can become active
pub fn validate_workflow(profile: &WorkflowProfile) -> ConfigReport {
    let mut report = ConfigReport::default();

    if profile.version != CURRENT_WORKFLOW_VERSION {
        report.push("version", "unsupported workflow schema version");
    }
    if profile.profile_id.trim().is_empty() {
        report.push("profile_id", "profile id must not be empty");
    }
    if profile.display_name.trim().is_empty() {
        report.push("display_name", "display name must not be empty");
    }
    if profile.lenses.order.is_empty() {
        report.push("lenses.order", "at least one lens must be ordered");
    }
    if profile.lenses.visible.is_empty() {
        report.push("lenses.visible", "at least one lens must be visible");
    }

    let order: BTreeSet<_> = profile.lenses.order.iter().copied().collect();
    let visible: BTreeSet<_> = profile.lenses.visible.iter().copied().collect();

    if order.len() != profile.lenses.order.len() {
        report.push("lenses.order", "lens order must not contain duplicates");
    }
    if visible.len() != profile.lenses.visible.len() {
        report.push(
            "lenses.visible",
            "visible lenses must not contain duplicates",
        );
    }
    if !visible.contains(&profile.startup_lens) {
        report.push("startup_lens", "startup lens must be visible");
    }
    for lens in &visible {
        if !order.contains(lens) {
            report.push(
                "lenses.visible",
                "visible lens must also appear in lens order",
            );
        }
    }

    if !validate_keybindings(&profile.keybindings) {
        report.push(
            "keybindings",
            "keybindings must map non-empty keys to non-empty commands",
        );
    }

    for (alias, intent) in &profile.commands.aliases {
        if alias.trim().is_empty() {
            report.push("commands.aliases", "alias labels must not be empty");
        }
        validate_command_intent(intent, &mut report);
    }

    for (target, shelf) in &profile.context_shelf {
        if target.trim().is_empty() {
            report.push("context_shelf", "context shelf target must not be empty");
        }
        if shelf.actions.iter().any(|action| action.trim().is_empty()) {
            report.push(
                "context_shelf.actions",
                "context shelf actions must not be empty",
            );
        }
    }

    for action in &profile.graph.empty_actions {
        if action.trim().is_empty() {
            report.push(
                "graph.empty_actions",
                "empty-state actions must not be empty",
            );
        }
    }

    for template in &profile.templates {
        if template.name.trim().is_empty() {
            report.push("templates.name", "template names must not be empty");
        }
        if template
            .args
            .iter()
            .any(|(key, value)| key.trim().is_empty() || value.trim().is_empty())
        {
            report.push("templates.args", "template args must not be empty");
        }
    }

    report
}

fn validate_command_intent(intent: &CommandIntent, report: &mut ConfigReport) {
    if !intent.is_declarative() {
        report.push(
            "commands.aliases",
            "command intents must be declarative typed commands, not shell paths",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{LensId, WorkflowProfile};

    #[test]
    fn default_profile_validates() {
        let profile = WorkflowProfile::default_profile();
        assert!(validate_workflow(&profile).is_ok());
    }

    #[test]
    fn rejects_hidden_startup_lens() {
        let mut profile = WorkflowProfile::default_profile();
        profile.startup_lens = LensId::Modulation;
        profile
            .lenses
            .visible
            .retain(|lens| *lens != LensId::Modulation);
        let report = validate_workflow(&profile);
        assert!(!report.is_ok());
        assert!(report.diagnostics.iter().any(|d| d.field == "startup_lens"));
    }

    #[test]
    fn rejects_shell_like_alias() {
        let mut profile = WorkflowProfile::default_profile();
        profile.commands.aliases.insert(
            "Bad".to_string(),
            CommandIntent::new("shell:rm -rf /tmp/session"),
        );
        let report = validate_workflow(&profile);
        assert!(!report.is_ok());
    }

    #[test]
    fn rejects_blank_template_names_and_args() {
        let mut profile = WorkflowProfile::default_profile();
        profile.templates.push(crate::templates::TemplateRef {
            name: " ".to_string(),
            kind: crate::templates::TemplateKind::Track,
            args: [("".to_string(), "lead".to_string())].into(),
        });

        let report = validate_workflow(&profile);

        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.field == "templates.name"));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.field == "templates.args"));
    }
}
