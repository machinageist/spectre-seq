// Author: Jeff
// Date: 2026-06-08
// Description: Validation coverage for bundled workflow profile TOML files.
// Notes: Bundled profiles must remain safe examples for creator-authored workflows.

use spectre_config::loader::load_workflow_toml;
use std::path::PathBuf;

#[test]
fn bundled_workflow_profiles_parse_and_validate() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in ["default", "modular", "songwriting", "mixing", "performance"] {
        let path = repo_root
            .join("assets/workflows")
            .join(format!("{name}.toml"));
        let profile = load_workflow_toml(&path).unwrap_or_else(|diagnostics| {
            panic!("{name}.toml failed validation: {diagnostics:#?}");
        });
        assert_eq!(profile.version, 1);
        assert!(!profile.profile_id.trim().is_empty());
    }
}
