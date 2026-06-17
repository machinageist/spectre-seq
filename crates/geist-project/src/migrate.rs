// =============================================================================
// File: crates/geist-project/src/migrate.rs
// Layer: project persistence
// Purpose: Forward-migrate older project files up to the current schema
// Status: Implemented; ordered step engine plus a load-and-migrate helper.
// Notes: STEP_FOR maps a source version to the step that lifts it one version.
//        The engine is generic over a target version and lookup so it is fully
//        testable before a second schema version exists. Newer files are
//        rejected; equal-version files pass through untouched.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::Path;

use crate::schema::{ProjectFile, SCHEMA_VERSION};
use crate::serialize::{load_from_path, ProjectError};

// One migration step lifts a project from version N to version N + 1
type MigrationStep = fn(&mut ProjectFile);

// Real migration table: returns the step that migrates `from` to `from + 1`
// Empty while only one schema version exists; add arms as the schema evolves
#[allow(clippy::match_single_binding)] // kept as a table for future version arms
fn step_for(from: u32) -> Option<MigrationStep> {
    match from {
        // 1 => Some(v1_to_v2),
        _ => None,
    }
}

// Migrate a decoded project to the current schema version
pub fn migrate(project: ProjectFile) -> Result<ProjectFile, ProjectError> {
    migrate_to(project, SCHEMA_VERSION, step_for)
}

// Load a project from disk and migrate it to the current schema version
pub fn load_and_migrate(path: impl AsRef<Path>) -> Result<ProjectFile, ProjectError> {
    migrate(load_from_path(path)?)
}

// Drive a project up to `target` by applying ordered steps from `lookup`
// Rejects files newer than the target and gaps with no available step
fn migrate_to(
    mut project: ProjectFile,
    target: u32,
    lookup: impl Fn(u32) -> Option<MigrationStep>,
) -> Result<ProjectFile, ProjectError> {
    if project.schema_version > target {
        return Err(ProjectError::UnsupportedVersion {
            found: project.schema_version,
            max: target,
        });
    }
    while project.schema_version < target {
        let from = project.schema_version;
        let Some(step) = lookup(from) else {
            return Err(ProjectError::UnsupportedVersion {
                found: from,
                max: target,
            });
        };
        step(&mut project);
        project.schema_version = from + 1;
    }
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_passes_through_unchanged() {
        let project = ProjectFile::new("keep");
        let out = migrate(project.clone()).unwrap();
        assert_eq!(out, project);
    }

    #[test]
    fn newer_than_current_is_rejected() {
        let mut project = ProjectFile::new("future");
        project.schema_version = SCHEMA_VERSION + 1;
        let err = migrate(project).unwrap_err();
        match err {
            ProjectError::UnsupportedVersion { found, max } => {
                assert_eq!(found, SCHEMA_VERSION + 1);
                assert_eq!(max, SCHEMA_VERSION);
            }
            other => panic!("expected UnsupportedVersion, got {other:?}"),
        }
    }

    #[test]
    fn engine_applies_ordered_steps_to_target() {
        fn bump_a(p: &mut ProjectFile) {
            p.meta.name.push('a');
        }
        fn bump_b(p: &mut ProjectFile) {
            p.meta.name.push('b');
        }
        let lookup = |from: u32| -> Option<MigrationStep> {
            match from {
                1 => Some(bump_a),
                2 => Some(bump_b),
                _ => None,
            }
        };
        let mut project = ProjectFile::new("");
        project.schema_version = 1;
        let out = migrate_to(project, 3, lookup).unwrap();
        assert_eq!(out.schema_version, 3);
        assert_eq!(out.meta.name, "ab"); // steps ran in order 1->2->3
    }

    #[test]
    fn engine_rejects_a_gap_with_no_step() {
        let lookup = |_: u32| -> Option<MigrationStep> { None };
        let mut project = ProjectFile::new("");
        project.schema_version = 1;
        let err = migrate_to(project, 2, lookup).unwrap_err();
        assert!(matches!(err, ProjectError::UnsupportedVersion { .. }));
    }
}
