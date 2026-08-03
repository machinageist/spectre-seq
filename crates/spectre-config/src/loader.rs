// Author: Jeff
// Date: 2026-06-08
// Description: Workflow profile loading and fallback resolution.
// Notes: Loading happens off the audio callback and returns validated snapshots.

use crate::schema::WorkflowProfile;
use crate::validate::{validate_workflow, ConfigDiagnostic};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowSource {
    BuiltInDefault,
    BundledProfile,
    UserProfile,
    ProjectOverride,
}

// Parse and validate one TOML workflow file
pub fn load_workflow_toml(path: &Path) -> Result<WorkflowProfile, Vec<ConfigDiagnostic>> {
    let text = fs::read_to_string(path).map_err(|err| {
        vec![ConfigDiagnostic {
            field: "path",
            message: format!("could not read workflow config: {err}"),
        }]
    })?;

    let profile: WorkflowProfile = toml::from_str(&text).map_err(|err| {
        vec![ConfigDiagnostic {
            field: "toml",
            message: format!("could not parse workflow config: {err}"),
        }]
    })?;

    let report = validate_workflow(&profile);
    if report.is_ok() {
        Ok(profile)
    } else {
        Err(report.diagnostics)
    }
}

// Apply source precedence and fall back to the last valid profile
pub fn resolve_workflow(
    candidates: impl IntoIterator<
        Item = (
            WorkflowSource,
            Result<WorkflowProfile, Vec<ConfigDiagnostic>>,
        ),
    >,
) -> (WorkflowProfile, Vec<(WorkflowSource, ConfigDiagnostic)>) {
    let mut active = WorkflowProfile::default_profile();
    let mut diagnostics = Vec::new();

    for (source, candidate) in candidates {
        match candidate {
            Ok(profile) => active = profile,
            Err(errors) => diagnostics.extend(errors.into_iter().map(|error| (source, error))),
        }
    }

    (active, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{LensId, WorkflowProfile};

    #[test]
    fn later_valid_candidate_wins() {
        let mut user = WorkflowProfile::default_profile();
        user.profile_id = "user".to_string();
        user.startup_lens = LensId::Build;

        let (active, diagnostics) = resolve_workflow([
            (
                WorkflowSource::BuiltInDefault,
                Ok(WorkflowProfile::default_profile()),
            ),
            (WorkflowSource::UserProfile, Ok(user)),
        ]);

        assert_eq!(active.profile_id, "user");
        assert_eq!(active.startup_lens, LensId::Build);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn invalid_later_candidate_keeps_last_valid() {
        let mut valid = WorkflowProfile::default_profile();
        valid.profile_id = "valid".to_string();

        let (active, diagnostics) = resolve_workflow([
            (WorkflowSource::UserProfile, Ok(valid)),
            (
                WorkflowSource::ProjectOverride,
                Err(vec![ConfigDiagnostic {
                    field: "startup_lens",
                    message: "startup lens must be visible".to_string(),
                }]),
            ),
        ]);

        assert_eq!(active.profile_id, "valid");
        assert_eq!(diagnostics.len(), 1);
    }
}
