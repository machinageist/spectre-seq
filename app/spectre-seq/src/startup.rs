// =============================================================================
// File: app/spectre-seq/src/startup.rs
// Layer: application binary
// Purpose: Parse launch options and resolve startup workflow configuration
// Status: Implemented; app-thread workflow files choose the initial UI state.
// Notes: File I/O and TOML parsing happen before the GUI starts, never on the
//        audio callback. Invalid optional profiles report diagnostics and fall
//        back to the last valid profile.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::fmt;
use std::path::{Path, PathBuf};

use spectre_config::loader::WorkflowSource;
use spectre_config::schema::WorkflowProfile;
use spectre_config::validate::ConfigDiagnostic;
use spectre_ui::state::UIState;

// Parsed process options that affect startup routing
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct StartupOptions {
    pub headless: bool,
    pub classic: bool,
    pub workflow_file: Option<PathBuf>,
}

// Diagnostic tagged with the startup source that produced it
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupWorkflowDiagnostic {
    pub source: WorkflowSource,
    pub diagnostic: ConfigDiagnostic,
}

impl fmt::Display for StartupWorkflowDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {}: {}",
            self.source, self.diagnostic.field, self.diagnostic.message
        )
    }
}

// Parse launch options; unknown args are ignored for forward-compatible flags
pub fn parse_args(args: impl IntoIterator<Item = String>) -> StartupOptions {
    let mut options = StartupOptions::default();
    let mut iter = args.into_iter();
    let _program = iter.next();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--headless" => options.headless = true,
            "--classic" => options.classic = true,
            "--workflow" => {
                if let Some(path) = iter.next() {
                    options.workflow_file = Some(PathBuf::from(path));
                }
            }
            _ => {
                if let Some(path) = arg.strip_prefix("--workflow=") {
                    options.workflow_file = Some(PathBuf::from(path));
                }
            }
        }
    }

    options
}

// Resolve the initial studio UI state from bundled, user, project, and explicit profiles
pub fn resolve_ui_state(options: &StartupOptions) -> (UIState, Vec<StartupWorkflowDiagnostic>) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home = std::env::var_os("HOME").map(PathBuf::from);
    resolve_ui_state_from_paths(
        bundled_default_workflow_path(),
        home.as_deref(),
        &cwd,
        options.workflow_file.as_deref(),
    )
}

// Resolve workflow candidates; later valid profiles override earlier ones
fn resolve_ui_state_from_paths(
    bundled_default: PathBuf,
    home: Option<&Path>,
    cwd: &Path,
    explicit: Option<&Path>,
) -> (UIState, Vec<StartupWorkflowDiagnostic>) {
    let mut candidates = vec![
        (
            WorkflowSource::BuiltInDefault,
            Ok(WorkflowProfile::default_profile()),
        ),
        (
            WorkflowSource::BundledProfile,
            spectre_config::loader::load_workflow_toml(&bundled_default),
        ),
    ];

    if let Some(home) = home {
        let user = home.join(".config/geist/workflows/default.toml");
        if user.exists() {
            candidates.push((
                WorkflowSource::UserProfile,
                spectre_config::loader::load_workflow_toml(&user),
            ));
        }
    }

    let project = cwd.join(".geist/workflow.toml");
    if project.exists() {
        candidates.push((
            WorkflowSource::ProjectOverride,
            spectre_config::loader::load_workflow_toml(&project),
        ));
    }

    if let Some(explicit) = explicit {
        candidates.push((
            WorkflowSource::ProjectOverride,
            spectre_config::loader::load_workflow_toml(explicit),
        ));
    }

    let (app, diagnostics) = spectre_ui::app::App::from_workflow_candidates(candidates);
    (
        app.state().clone(),
        diagnostics
            .into_iter()
            .map(|(source, diagnostic)| StartupWorkflowDiagnostic { source, diagnostic })
            .collect(),
    )
}

// Locate the bundled default workflow relative to this app crate
fn bundled_default_workflow_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/workflows/default.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_config::schema::LensId;
    use std::fs;

    #[test]
    fn parses_workflow_flags() {
        let options = parse_args([
            "spectre-seq".to_string(),
            "--workflow".to_string(),
            "custom.toml".to_string(),
            "--headless".to_string(),
        ]);

        assert!(options.headless);
        assert!(!options.classic);
        assert_eq!(options.workflow_file, Some(PathBuf::from("custom.toml")));

        let options = parse_args([
            "spectre-seq".to_string(),
            "--classic".to_string(),
            "--workflow=other.toml".to_string(),
        ]);
        assert!(options.classic);
        assert_eq!(options.workflow_file, Some(PathBuf::from("other.toml")));
    }

    #[test]
    fn explicit_workflow_overrides_bundled_default() {
        let root = temp_root("explicit-workflow");
        let explicit = root.join("mixing.toml");
        fs::create_dir_all(&root).unwrap();
        fs::copy(repo_root().join("assets/workflows/mixing.toml"), &explicit).unwrap();

        let (state, diagnostics) = resolve_ui_state_from_paths(
            repo_root().join("assets/workflows/default.toml"),
            None,
            &root,
            Some(&explicit),
        );

        assert_eq!(state.workflow().profile_id, "mixing");
        assert_eq!(state.active_lens(), LensId::Mix);
        assert!(diagnostics.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_project_workflow_keeps_bundled_default() {
        let root = temp_root("invalid-project-workflow");
        let project_dir = root.join(".geist");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(
            project_dir.join("workflow.toml"),
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

        let (state, diagnostics) = resolve_ui_state_from_paths(
            repo_root().join("assets/workflows/default.toml"),
            None,
            &root,
            None,
        );

        assert_eq!(state.workflow().profile_id, "default");
        assert_eq!(state.active_lens(), LensId::Arrange);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].source, WorkflowSource::ProjectOverride);
        let _ = fs::remove_dir_all(root);
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("spectre-seq-{name}-{}", std::process::id()))
    }
}
