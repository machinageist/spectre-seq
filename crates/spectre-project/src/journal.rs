// Author: Jeff
// Date: 2026-08-29
// Description: Journaled autosave to a sidecar, per decision 14's adopted model
// Notes: Decision 14 adopts "journaled autosave to sidecar + atomic rename saves" without
//   specifying the journal's shape, so this file states the choice rather than leaving it implied.
//
//   THE SIDECAR IS A PROJECT ENVELOPE, written by save_project_atomic. Not an append-only
//   operation log: a log must be parsed after a crash that may have torn its last record, so
//   recovery would need record framing and a truncation policy before it could read anything --
//   new failure modes introduced to save work a snapshot already saves. And not a bespoke wrapper
//   record either, because the envelope already carries the project identity recovery needs to
//   match a sidecar to its project. Writing the same type the project uses means the sidecar
//   inherits R5-1's crash qualification, CORE-003's schema gate, and the semantic validator
//   whole, rather than by resemblance.
//
//   Stated cost: each autosave rewrites the whole project rather than appending a delta.
//
//   The project file is NEVER opened by this module. That is the property the model rests on --
//   an autosave that could damage the saved project would be worse than no autosave.

use crate::fs::{load_project, save_project_atomic, LoadError, SaveError};
use crate::ProjectEnvelope;
use spectre_core::ObjectId;
use std::path::{Path, PathBuf};

// Appended to the project's own file name. A visible sibling rather than a hidden file, so the
// musician sees the thing protecting their work, and distinct from fs.rs's "spectre-tmp"
// temporaries, which a crash may also leave in the same directory
const JOURNAL_SUFFIX: &str = "autosave";

// Where the sidecar for a project lives. A pure function of the project path, so recovery finds
// it without having been told where the last autosave went
pub fn journal_path(project: &Path) -> PathBuf {
    let mut name = project
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".into());
    name.push('.');
    name.push_str(JOURNAL_SUFFIX);
    project.with_file_name(name)
}

// Write current state to the project's sidecar. The project file is not opened.
//
// Returns the sidecar path so a caller reports it rather than reconstructing it
pub fn write_autosave(
    project: &Path,
    snapshot: &ProjectEnvelope,
) -> Result<PathBuf, AutosaveError> {
    let sidecar = journal_path(project);
    save_project_atomic(&sidecar, snapshot).map_err(AutosaveError::Save)?;
    Ok(sidecar)
}

// Read a project's sidecar, if it holds one for THIS project.
//
// `Ok(None)` means no sidecar, which is ordinary rather than an error. A sidecar whose project id
// differs is also `Ok(None)`: it is not this project's unsaved work, and reporting it as
// corruption would make an unrelated stale file look like damage
pub fn read_autosave(
    project: &Path,
    project_id: ObjectId,
) -> Result<Option<ProjectEnvelope>, AutosaveError> {
    let sidecar = journal_path(project);
    match load_project(&sidecar) {
        Ok(envelope) if envelope.project.id == project_id => Ok(Some(envelope)),
        Ok(_) => Ok(None),
        Err(LoadError::Open { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(source) => Err(AutosaveError::Load(source)),
    }
}

// Remove a project's sidecar. Absent is success: discarding what is not there is what the caller
// asked for, and reporting it would make every ordinary save path handle an error meaning nothing
pub fn discard_autosave(project: &Path) -> Result<(), AutosaveError> {
    let sidecar = journal_path(project);
    match std::fs::remove_file(&sidecar) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(AutosaveError::Io {
            path: sidecar,
            source,
        }),
    }
}

// Autosave failures, kept separate from SaveError so a caller can distinguish "the project failed
// to save" from "the autosave failed" -- different things to tell a musician
#[derive(Debug)]
pub enum AutosaveError {
    Save(SaveError),
    Load(LoadError),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl std::fmt::Display for AutosaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Save(source) => write!(f, "autosave could not be written: {source}"),
            Self::Load(source) => write!(f, "autosave could not be read: {source}"),
            Self::Io { path, source } => {
                write!(f, "autosave i/o failed at {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for AutosaveError {}
