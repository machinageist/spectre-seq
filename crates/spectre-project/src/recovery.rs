// Author: Jeff
// Date: 2026-08-29
// Description: Recovery from an unclean exit — offered and described, never applied silently
// Notes: R5's exit evidence asks that recovery "restores work after an unclean exit and states
//   what was and was not recovered". Both halves are load-bearing.
//
//   Nothing here applies recovery on its own. A musician who is shown a project must know whether
//   they are looking at what they saved or what a crash recovered, and the only way to guarantee
//   that is to make acceptance an explicit act. `inspect` reports; `accept` and `decline` change
//   things, and each says exactly what it did.
//
//   The comparison is deliberately shallow and named field by field rather than being a
//   structural diff. A diff engine over the whole document is a larger thing than this milestone
//   asks for, and a shallow comparison that says "three more tracks, same tempo" is what a
//   musician needs to decide. What it must never do is claim equality it did not check, so
//   `Difference` carries only the fields it actually compared.

use crate::fs::{load_project, save_project_atomic, LoadError};
use crate::journal::{discard_autosave, read_autosave, AutosaveError};
use crate::ProjectEnvelope;
use std::path::Path;

// One named way the autosaved state differs from the saved one. Every variant carries both sides,
// so a caller renders the difference without re-deriving it
#[derive(Debug, Clone, PartialEq)]
pub enum Difference {
    ProjectName { saved: String, autosaved: String },
    TrackCount { saved: usize, autosaved: usize },
    ClipCount { saved: usize, autosaved: usize },
    DeviceCount { saved: usize, autosaved: usize },
    TempoSegments { saved: usize, autosaved: usize },
}

// What inspecting a project's recovery state found
#[derive(Debug)]
pub enum Recovery {
    // No sidecar for this project. The ordinary case after a clean quit
    Nothing,
    // A sidecar exists and its content is identical on every field compared. Kept distinct from
    // `Nothing` because the sidecar still needs discarding, and because "we found unsaved work and
    // it matched" is a different thing to report than "there was none"
    Redundant,
    // Unsaved work exists and differs
    Available(Box<RecoveryOffer>),
}

// Unsaved work, the saved state it would replace, and the differences actually compared
#[derive(Debug)]
pub struct RecoveryOffer {
    pub saved: ProjectEnvelope,
    pub autosaved: ProjectEnvelope,
    // Empty is impossible here: an offer with no differences is reported as `Redundant`
    pub differences: Vec<Difference>,
}

impl RecoveryOffer {
    // One line per difference, phrased so a caller can show it without interpreting it
    pub fn describe(&self) -> Vec<String> {
        self.differences
            .iter()
            .map(|difference| match difference {
                Difference::ProjectName { saved, autosaved } => {
                    format!("name: saved \"{saved}\", unsaved \"{autosaved}\"")
                }
                Difference::TrackCount { saved, autosaved } => {
                    format!("tracks: saved {saved}, unsaved {autosaved}")
                }
                Difference::ClipCount { saved, autosaved } => {
                    format!("clips: saved {saved}, unsaved {autosaved}")
                }
                Difference::DeviceCount { saved, autosaved } => {
                    format!("devices: saved {saved}, unsaved {autosaved}")
                }
                Difference::TempoSegments { saved, autosaved } => {
                    format!("tempo segments: saved {saved}, unsaved {autosaved}")
                }
            })
            .collect()
    }

    // The fields this offer did NOT compare, stated so a caller cannot present the difference list
    // as exhaustive. R5's exit evidence asks what was and was NOT recovered, and this is the half
    // that is easy to omit
    pub fn uncompared(&self) -> &'static [&'static str] {
        &[
            "clip note content",
            "track mute, solo, level, and instrument",
            "device parameter values",
            "view context",
            "unknown fields preserved from newer writers",
        ]
    }
}

// Look for unsaved work beside a project. Reads only; changes nothing
pub fn inspect(project: &Path) -> Result<Recovery, AutosaveError> {
    let saved = match load_project(project) {
        Ok(envelope) => envelope,
        Err(LoadError::Open { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            // No project means nothing to recover INTO. A sidecar without its project is not
            // this function's to interpret, and silently adopting it would invent a project
            return Ok(Recovery::Nothing);
        }
        Err(source) => return Err(AutosaveError::Load(source)),
    };
    let Some(autosaved) = read_autosave(project, saved.project.id)? else {
        return Ok(Recovery::Nothing);
    };

    let differences = compare(&saved, &autosaved);
    if differences.is_empty() {
        return Ok(Recovery::Redundant);
    }
    Ok(Recovery::Available(Box::new(RecoveryOffer {
        saved,
        autosaved,
        differences,
    })))
}

// Accept unsaved work: write it over the project through the qualified atomic save, then discard
// the sidecar. The order matters and is not interchangeable -- discarding first would lose the
// work if the save then failed
pub fn accept(project: &Path, offer: &RecoveryOffer) -> Result<(), AutosaveError> {
    save_project_atomic(project, &offer.autosaved).map_err(AutosaveError::Save)?;
    discard_autosave(project)
}

// Decline unsaved work: the project is left exactly as it was and the sidecar is removed. The
// project file is not opened, because declining is not an edit
pub fn decline(project: &Path) -> Result<(), AutosaveError> {
    discard_autosave(project)
}

// Compare the fields this milestone claims to compare, and no others
fn compare(saved: &ProjectEnvelope, autosaved: &ProjectEnvelope) -> Vec<Difference> {
    let mut differences = Vec::new();
    if saved.project.name != autosaved.project.name {
        differences.push(Difference::ProjectName {
            saved: saved.project.name.clone(),
            autosaved: autosaved.project.name.clone(),
        });
    }
    let (saved_tracks, autosaved_tracks) =
        (saved.project.tracks.len(), autosaved.project.tracks.len());
    if saved_tracks != autosaved_tracks {
        differences.push(Difference::TrackCount {
            saved: saved_tracks,
            autosaved: autosaved_tracks,
        });
    }
    let clips = |envelope: &ProjectEnvelope| {
        envelope
            .project
            .tracks
            .tracks()
            .iter()
            .map(|track| track.clips().placements().len())
            .sum::<usize>()
    };
    let (saved_clips, autosaved_clips) = (clips(saved), clips(autosaved));
    if saved_clips != autosaved_clips {
        differences.push(Difference::ClipCount {
            saved: saved_clips,
            autosaved: autosaved_clips,
        });
    }
    if saved.project.devices.len() != autosaved.project.devices.len() {
        differences.push(Difference::DeviceCount {
            saved: saved.project.devices.len(),
            autosaved: autosaved.project.devices.len(),
        });
    }
    let (saved_tempo, autosaved_tempo) = (
        saved.project.tempo_map.segments().len(),
        autosaved.project.tempo_map.segments().len(),
    );
    if saved_tempo != autosaved_tempo {
        differences.push(Difference::TempoSegments {
            saved: saved_tempo,
            autosaved: autosaved_tempo,
        });
    }
    differences
}
