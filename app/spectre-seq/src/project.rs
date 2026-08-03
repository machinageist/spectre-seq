// =============================================================================
// File: app/spectre-seq/src/project.rs
// Layer: application binary
// Purpose: Map the app's patch state to and from the spectre-project file format
// Status: Implemented; single-slot session save/load of the synth + fx patch.
// Notes: The on-disk schema lives in spectre-project; this is the app-side mapping
//        the crate intentionally leaves to callers. Tempo rides in ProjectMeta;
//        the patch is one NodeEntry whose params are keyed by stable ids. Disk
//        I/O only; never touched from the audio thread.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::{Path, PathBuf};

use spectre_project::prelude::{
    load_from_path, save_to_path, NodeEntry, ParamValue, ProjectError, ProjectFile,
};

// Stable parameter ids for the saved patch node
const PARAM_CUTOFF: u32 = 0;
const PARAM_RESONANCE: u32 = 1;
const PARAM_GAIN: u32 = 2;
const PARAM_UNISON: u32 = 3;
const PARAM_DETUNE: u32 = 4;
const PARAM_DELAY: u32 = 5;
const PARAM_REVERB: u32 = 6;
const PARAM_REVERB_MIX: u32 = 7;

// Node kind tag identifying the app's patch node in the project graph
const PATCH_NODE_KIND: &str = "geist-patch";
// Single session slot filename
const SESSION_FILE: &str = "geist-session.gproj";

// Savable snapshot of the app's parameters
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatchState {
    pub bpm: f32,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub gain: f32,
    pub unison_voices: usize,
    pub detune_cents: f32,
    pub delay_on: bool,
    pub reverb_on: bool,
    pub reverb_mix: f32,
}

impl PatchState {
    // Encode this patch as a project file
    fn to_project(self) -> ProjectFile {
        let mut project = ProjectFile::new("Geist Session");
        project.meta.tempo_bpm = self.bpm as f64;
        project.graph.nodes.push(NodeEntry {
            id: 0,
            kind: PATCH_NODE_KIND.to_string(),
            params: vec![
                ParamValue {
                    id: PARAM_CUTOFF,
                    value: self.cutoff_hz,
                },
                ParamValue {
                    id: PARAM_RESONANCE,
                    value: self.resonance,
                },
                ParamValue {
                    id: PARAM_GAIN,
                    value: self.gain,
                },
                ParamValue {
                    id: PARAM_UNISON,
                    value: self.unison_voices as f32,
                },
                ParamValue {
                    id: PARAM_DETUNE,
                    value: self.detune_cents,
                },
                ParamValue {
                    id: PARAM_DELAY,
                    value: bool_to_f32(self.delay_on),
                },
                ParamValue {
                    id: PARAM_REVERB,
                    value: bool_to_f32(self.reverb_on),
                },
                ParamValue {
                    id: PARAM_REVERB_MIX,
                    value: self.reverb_mix,
                },
            ],
            state_blob: Vec::new(),
        });
        project
    }

    // Merge a project file over this patch, keeping `self` for anything missing
    fn merge(self, project: &ProjectFile) -> PatchState {
        let mut patch = self;
        patch.bpm = project.meta.tempo_bpm as f32;
        if let Some(node) = project
            .graph
            .nodes
            .iter()
            .find(|node| node.kind == PATCH_NODE_KIND)
        {
            for param in &node.params {
                match param.id {
                    PARAM_CUTOFF => patch.cutoff_hz = param.value,
                    PARAM_RESONANCE => patch.resonance = param.value,
                    PARAM_GAIN => patch.gain = param.value,
                    PARAM_UNISON => patch.unison_voices = (param.value.round() as usize).max(1),
                    PARAM_DETUNE => patch.detune_cents = param.value,
                    PARAM_DELAY => patch.delay_on = param.value >= 0.5,
                    PARAM_REVERB => patch.reverb_on = param.value >= 0.5,
                    PARAM_REVERB_MIX => patch.reverb_mix = param.value,
                    _ => {}
                }
            }
        }
        patch
    }
}

// Encode a flag as a parameter value
fn bool_to_f32(flag: bool) -> f32 {
    if flag {
        1.0
    } else {
        0.0
    }
}

// Path to the single session slot, in the home directory when available
pub fn session_path() -> PathBuf {
    let dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join(SESSION_FILE)
}

// Write a patch to an explicit path
pub fn save_to(patch: &PatchState, path: &Path) -> Result<(), ProjectError> {
    save_to_path(&patch.to_project(), path)
}

// Read a patch from an explicit path, merging onto `current` for missing fields
pub fn load_from(current: &PatchState, path: &Path) -> Result<PatchState, ProjectError> {
    let project = load_from_path(path)?;
    Ok(current.merge(&project))
}

// Save to the session slot, returning the written path
pub fn save(patch: &PatchState) -> Result<PathBuf, ProjectError> {
    let path = session_path();
    save_to(patch, &path)?;
    Ok(path)
}

// Load from the session slot, merging onto `current`
pub fn load(current: &PatchState) -> Result<PatchState, ProjectError> {
    load_from(current, &session_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PatchState {
        PatchState {
            bpm: 140.0,
            cutoff_hz: 820.0,
            resonance: 2.0,
            gain: 0.7,
            unison_voices: 5,
            detune_cents: 22.0,
            delay_on: true,
            reverb_on: false,
            reverb_mix: 0.6,
        }
    }

    fn defaults() -> PatchState {
        PatchState {
            bpm: 120.0,
            cutoff_hz: 1_500.0,
            resonance: 0.9,
            gain: 1.0,
            unison_voices: 1,
            detune_cents: 0.0,
            delay_on: false,
            reverb_on: false,
            reverb_mix: 0.3,
        }
    }

    #[test]
    fn patch_round_trips_through_a_project_file() {
        let path = std::env::temp_dir().join(format!("geist-test-{}.gproj", std::process::id()));
        let patch = sample();
        save_to(&patch, &path).unwrap();
        let loaded = load_from(&defaults(), &path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(loaded, patch);
    }

    #[test]
    fn missing_file_is_an_error_not_a_panic() {
        let path = PathBuf::from("/no/such/dir/geist-missing.gproj");
        assert!(load_from(&defaults(), &path).is_err());
    }
}
