// Author: Jeff
// Date: 2026-08-24
// Description: Ordered project track model with stable identity and mixer state
// Notes: App-thread only, never callback-reachable. Levels are carried by the accepted device
//   descriptors rather than a new fader law, so no numeric range is invented here.

use crate::clip::{ClipError, MidiClip, TrackClips};
use serde::{Deserialize, Serialize};
use spectre_core::ObjectId;
use spectre_dsp::{GAIN_PARAMETERS, GLOAM_DEPTH, GLOAM_PARAMETERS, PULSE_PARAMETERS};

// Maximum tracks a v1 project may sum into the master bus
//
// Rationale row in docs/01-requirements/requirements-ledger.md (PROD-003, decision 16).
//
// **Raised 16 -> 32 on 2026-09-06, and the reason the old value existed is gone.** It was
// derived from summing: one SumBus cannot declare more than MAX_SUM_BUSES input buses, because
// PlanStep carries its input map as a fixed array and RT-001 forbids a heap collection on the
// render path. `routing::build_sum_tree` now sums through a tree of such nodes, so the number of
// TRACKS is no longer bounded by the fan-in of any one node.
//
// What still bounds it is note delivery, and only that: `RenderBridge` builds one block's note
// inputs in a fixed stack array of `spectre_graph::MAX_FLAT_INPUTS` entries, which admits one
// primary note node plus 31 clip voices. Thirty-two instrument tracks is exactly that array, so
// this is a derived number rather than a chosen one. Removing the array is what removes the cap
pub const MAX_TRACKS: usize = 32;

// The one instrument kind a v1 track may host. R4-6 replaces the variant; the slot stays
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackInstrument {
    Pulse,
    // R4-9 made the R4-6 synth reachable from a track. Until then Filament rendered in the
    // offline harness and appeared on the Build surface but was in no plan ./spectre ran, which
    // is what NEXT.md slice 6 recorded as open. R4's exit row asks for one original synth in the
    // product, not one that exists beside it
    Filament,
}

// The one insert kind a v1 track may host, mirroring TrackInstrument's shape. R4-6 shipped
// Gloam and R4-9's spec requires one per track; before this slot existed the effect was
// constructed nowhere, so a stored gloam parameter reached no render
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackEffect {
    Gloam,
}

// A track's insert: the device and its headline parameter, stored the way instrument and
// instrument_level are. Gloam's depth is the stored one because it is the control that decides
// whether the insert is audible at all -- its descriptor default is 0.0, fully dry -- and it is
// the parameter r4-qa-protocol.md row 3 drags. Damp and track take their descriptor defaults
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrackInsert {
    effect: TrackEffect,
    depth: f32,
}

impl TrackInsert {
    // Create an insert with its depth clamped through the effect's own descriptor
    pub fn new(effect: TrackEffect, depth: f32) -> Self {
        Self {
            effect,
            depth: GLOAM_PARAMETERS[GLOAM_DEPTH].clamp(depth),
        }
    }

    pub fn effect(&self) -> TrackEffect {
        self.effect
    }

    pub fn depth(&self) -> f32 {
        self.depth
    }
}

// App-thread track failure; every variant leaves the model unmutated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackError {
    BlankName,
    TrackLimit { limit: usize },
    UnknownTrack(ObjectId),
    DuplicateId(ObjectId),
    IndexOutOfRange { index: usize, len: usize },
}

impl std::fmt::Display for TrackError {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlankName => formatter.write_str("track name must not be blank"),
            Self::TrackLimit { limit } => {
                write!(formatter, "a project may hold at most {limit} tracks")
            }
            Self::UnknownTrack(id) => write!(formatter, "unknown track {}", id.raw()),
            Self::DuplicateId(id) => write!(formatter, "track id {} is already in use", id.raw()),
            Self::IndexOutOfRange { index, len } => {
                write!(formatter, "index {index} is out of range for {len} tracks")
            }
        }
    }
}

impl std::error::Error for TrackError {}

// One project track: identity, name, one instrument slot, and mixer state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    id: ObjectId,
    name: String,
    instrument: TrackInstrument,
    instrument_level: f32,
    level: f32,
    muted: bool,
    soloed: bool,
    // Clip placements on this track's timeline. Held here rather than in a parallel app-side
    // table so R4-7 persists one thing: the track model already travels into the document whole
    #[serde(default)]
    clips: TrackClips,
    // The insert this track's signal passes through between instrument and track gain. Absent
    // keeps the R4-4 path byte-for-byte and node-for-node as it was, so a project written before
    // the slot existed serializes identically (CORE-003) and rebuilds the same node IDs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    insert: Option<TrackInsert>,
}

impl Track {
    // Create a validated track; the caller owns ID allocation from the project IdGen
    pub fn new(id: ObjectId, name: &str, instrument: TrackInstrument) -> Result<Self, TrackError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(TrackError::BlankName);
        }
        Ok(Self {
            id,
            name: name.to_string(),
            instrument,
            // No new range is invented: the instrument's own descriptor owns its default
            instrument_level: PULSE_PARAMETERS[0].default(),
            // The fader's range is the accepted gain descriptor's range, and unity is its default
            level: GAIN_PARAMETERS[0].default(),
            muted: false,
            soloed: false,
            clips: TrackClips::new(),
            insert: None,
        })
    }

    pub fn clips(&self) -> &TrackClips {
        &self.clips
    }

    pub fn clips_mut(&mut self) -> &mut TrackClips {
        &mut self.clips
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn instrument(&self) -> TrackInstrument {
        self.instrument
    }

    pub fn instrument_level(&self) -> f32 {
        self.instrument_level
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn is_soloed(&self) -> bool {
        self.soloed
    }

    // Rename in place; a blank name leaves the track unchanged
    pub fn set_name(&mut self, name: &str) -> Result<(), TrackError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(TrackError::BlankName);
        }
        self.name = name.to_string();
        Ok(())
    }

    // Clamp through the accepted gain descriptor rather than a fader law of our own
    pub fn set_level(&mut self, level: f32) {
        self.level = GAIN_PARAMETERS[0].clamp(level);
    }

    // Clamp through the instrument's own descriptor
    pub fn set_instrument_level(&mut self, level: f32) {
        self.instrument_level = PULSE_PARAMETERS[0].clamp(level);
    }

    pub fn insert(&self) -> Option<TrackInsert> {
        self.insert
    }

    // Clamp through the effect's own descriptor
    pub fn set_insert_depth(&mut self, depth: f32) {
        if let Some(insert) = self.insert.as_mut() {
            insert.depth = GLOAM_PARAMETERS[GLOAM_DEPTH].clamp(depth);
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn set_soloed(&mut self, soloed: bool) {
        self.soloed = soloed;
    }
}

// Ordered track collection plus the master bus level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackList {
    tracks: Vec<Track>,
    // Project-scoped clip table. A clip is placed by identity and may be placed more than once,
    // so the material lives here once and the placements reference it
    #[serde(default)]
    clips: Vec<MidiClip>,
    master_level: f32,
    // Advances on every edit that changes the compiled graph's shape: push, insert, remove,
    // reorder, and instrument change. Level, mute, solo, and rename do not advance it, because
    // they reach a live plan over the parameter lane instead. Session-local, so a reload does
    // not inherit a stale revision and R4-7 persists nothing about it
    #[serde(skip)]
    structure_revision: u64,
}

// Equality is over document content only, for the reason TrackClips states: structure_revision
// is #[serde(skip)], so including it would make a reloaded list compare unequal to its original
impl PartialEq for TrackList {
    fn eq(&self, other: &Self) -> bool {
        self.tracks == other.tracks
            && self.clips == other.clips
            && self.master_level == other.master_level
    }
}

impl Default for TrackList {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackList {
    // Empty list; master level is the gain descriptor's default, so no new constant appears
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            clips: Vec::new(),
            master_level: GAIN_PARAMETERS[0].default(),
            structure_revision: 0,
        }
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn master_level(&self) -> f32 {
        self.master_level
    }

    // Clamp through the accepted gain descriptor
    pub fn set_master_level(&mut self, level: f32) {
        self.master_level = GAIN_PARAMETERS[0].clamp(level);
    }

    pub fn index_of(&self, id: ObjectId) -> Option<usize> {
        self.tracks.iter().position(|track| track.id == id)
    }

    pub fn get(&self, id: ObjectId) -> Option<&Track> {
        self.tracks.iter().find(|track| track.id == id)
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|track| track.id == id)
    }

    // Append; refuses past MAX_TRACKS and refuses a duplicate ObjectId (CORE-001)
    pub fn push(&mut self, track: Track) -> Result<(), TrackError> {
        self.insert(self.tracks.len(), track)
    }

    // Insert at an absolute index; the same two refusals apply
    pub fn insert(&mut self, index: usize, track: Track) -> Result<(), TrackError> {
        if self.tracks.len() >= MAX_TRACKS {
            return Err(TrackError::TrackLimit { limit: MAX_TRACKS });
        }
        if index > self.tracks.len() {
            return Err(TrackError::IndexOutOfRange {
                index,
                len: self.tracks.len(),
            });
        }
        if self.index_of(track.id).is_some() {
            return Err(TrackError::DuplicateId(track.id));
        }
        self.tracks.insert(index, track);
        self.structure_revision += 1;
        Ok(())
    }

    // Remove by identity, returning the track whole so an undo can reinsert it unchanged
    pub fn remove(&mut self, id: ObjectId) -> Result<Track, TrackError> {
        let index = self.index_of(id).ok_or(TrackError::UnknownTrack(id))?;
        let track = self.tracks.remove(index);
        self.structure_revision += 1;
        Ok(track)
    }

    // Move one track to an absolute index, preserving identity and every field. Returns the
    // index it came from, so an undo can put it back exactly
    pub fn reorder(&mut self, id: ObjectId, to_index: usize) -> Result<usize, TrackError> {
        let from = self.index_of(id).ok_or(TrackError::UnknownTrack(id))?;
        if to_index >= self.tracks.len() {
            return Err(TrackError::IndexOutOfRange {
                index: to_index,
                len: self.tracks.len(),
            });
        }
        if from != to_index {
            let track = self.tracks.remove(from);
            self.tracks.insert(to_index, track);
            self.structure_revision += 1;
        }
        Ok(from)
    }

    // Replace one track's instrument; changes the graph's shape, so the revision advances
    pub fn set_instrument(
        &mut self,
        id: ObjectId,
        instrument: TrackInstrument,
    ) -> Result<(), TrackError> {
        let track = self.get_mut(id).ok_or(TrackError::UnknownTrack(id))?;
        if track.instrument != instrument {
            track.instrument = instrument;
            self.structure_revision += 1;
        }
        Ok(())
    }

    // Replace one track's insert; changes the graph's shape, so the revision advances for the
    // same reason set_instrument does -- adding or removing the node is a rebuild, not a
    // parameter edit. Changing only the depth of an existing insert is NOT a shape change
    pub fn set_insert(
        &mut self,
        id: ObjectId,
        insert: Option<TrackInsert>,
    ) -> Result<(), TrackError> {
        let track = self.get_mut(id).ok_or(TrackError::UnknownTrack(id))?;
        let shape_changed = track.insert.map(|slot| slot.effect) != insert.map(|slot| slot.effect);
        track.insert = insert;
        if shape_changed {
            self.structure_revision += 1;
        }
        Ok(())
    }

    // Where one track's parameters sit inside the ordering build_track_graph's
    // TrackPathNodes::parameter_targets emits: per track, instrument level, then the insert depth
    // WHERE THE TRACK HAS ONE, then track gain; master last.
    //
    // Defined here, on the list, because the layout is a fact about which tracks declare inserts
    // and every caller already holds the list. It used to be `index * 2` in spectre-app, which
    // silently addressed the wrong parameter the moment a stride stopped being two
    pub fn instrument_target_index(&self, track_index: usize) -> usize {
        self.targets_before(track_index)
    }

    // None where the track declares no insert
    pub fn insert_target_index(&self, track_index: usize) -> Option<usize> {
        self.tracks
            .get(track_index)?
            .insert()
            .map(|_| self.targets_before(track_index) + 1)
    }

    pub fn gain_target_index(&self, track_index: usize) -> usize {
        let insert = usize::from(
            self.tracks
                .get(track_index)
                .is_some_and(|track| track.insert().is_some()),
        );
        self.targets_before(track_index) + 1 + insert
    }

    pub fn master_target_index(&self) -> usize {
        self.targets_before(self.tracks.len())
    }

    // How many targets precede the given track, counting each earlier track's own insert
    fn targets_before(&self, track_index: usize) -> usize {
        let upto = track_index.min(self.tracks.len());
        let inserts = self.tracks[..upto]
            .iter()
            .filter(|track| track.insert().is_some())
            .count();
        track_index * 2 + inserts
    }

    pub fn any_soloed(&self) -> bool {
        self.tracks.iter().any(|track| track.soloed)
    }

    // level x mute x solo, clamped through the accepted gain descriptor; None for an unknown
    // track. Defined once here and used by the graph builder, the offline harness, and the app,
    // so the three cannot disagree about what a fader position means
    pub fn effective_gain(&self, id: ObjectId) -> Option<f32> {
        let track = self.get(id)?;
        let any_soloed = self.any_soloed();
        let muted = if track.muted { 0.0 } else { 1.0 };
        let solo = if any_soloed && !track.soloed {
            0.0
        } else {
            1.0
        };
        Some(GAIN_PARAMETERS[0].clamp(track.level * muted * solo))
    }

    pub fn structure_revision(&self) -> u64 {
        self.structure_revision
    }

    pub fn clips(&self) -> &[MidiClip] {
        &self.clips
    }

    pub fn clip(&self, id: ObjectId) -> Option<&MidiClip> {
        self.clips.iter().find(|clip| clip.id() == id)
    }

    pub fn clip_mut(&mut self, id: ObjectId) -> Option<&mut MidiClip> {
        self.clips.iter_mut().find(|clip| clip.id() == id)
    }

    // Add clip material to the project. Refuses a duplicate identity, which CORE-001 requires;
    // the clip's own length bound is enforced by MidiClip::new before it reaches here
    pub fn add_clip(&mut self, clip: MidiClip) -> Result<(), ClipError> {
        if self.clip(clip.id()).is_some() {
            return Err(ClipError::DuplicateId(clip.id()));
        }
        self.clips.push(clip);
        Ok(())
    }

    // Total placements across every track, which is what a bake would walk
    pub fn placement_count(&self) -> usize {
        self.tracks.iter().map(|track| track.clips().len()).sum()
    }
}
