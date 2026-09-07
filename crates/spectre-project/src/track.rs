// Author: Jeff
// Date: 2026-08-24
// Description: Ordered project track model with stable identity and mixer state
// Notes: App-thread only, never callback-reachable. Levels are carried by the accepted device
//   descriptors rather than a new fader law, so no numeric range is invented here.

use crate::clip::{ClipError, ClipNote, ClipPlacement, MidiClip, TrackClips};
use serde::{Deserialize, Serialize};
use spectre_core::{BeatTicks, ObjectId};
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

// Maximum effects one track may chain between its instrument and its gain
//
// A chain is serial, so every node in it has exactly one stereo input bus and none of the
// graph's fan-in bounds apply. The limit therefore exists for a different reason: a project file
// is untrusted input, and an unbounded chain length would let one track demand unbounded graph
// nodes and unbounded preallocated buffers at compile time. 64 is Spectre's own arithmetic --
// at MAX_TRACKS it admits 2 048 effect nodes, which is far past any musical use and still a
// bounded allocation the app thread can refuse
pub const MAX_CHAIN_DEVICES: usize = 64;

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
    ChainLimit { limit: usize },
    ChainIndex { index: usize, len: usize },
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
            Self::ChainLimit { limit } => {
                write!(formatter, "a track may chain at most {limit} effects")
            }
            Self::ChainIndex { index, len } => {
                write!(
                    formatter,
                    "effect position {index} is outside a chain of {len}"
                )
            }
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
    // Schema-2's single insert slot. Read on decode and moved into `inserts` by the schema 3
    // migration, then never written again. Retained as a field rather than deleted because
    // deserialization is the one path that bypasses every constructor, and a schema-2 file on
    // disk still carries this key
    // The serde key stays "insert" -- it is what schema-2 files on disk carry. Renaming the
    // Rust field without this made every existing effect vanish at decode, because Track has no
    // unknown-field map to catch it: serde simply ignored a key it no longer recognized
    #[serde(rename = "insert", default, skip_serializing_if = "Option::is_none")]
    legacy_insert: Option<TrackInsert>,
    // The ordered effect chain this track's signal passes through between instrument and track
    // gain. Empty keeps the R4-4 path byte-for-byte and node-for-node as it was, so a project
    // written before any effect existed serializes identically (CORE-003) and rebuilds the same
    // node IDs. Length is bounded only by MAX_CHAIN_DEVICES, which exists so a project cannot
    // demand unbounded graph nodes, not because the render path needs a fixed array: a chain is
    // serial, so every node in it has exactly one input bus
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    inserts: Vec<TrackInsert>,
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
            legacy_insert: None,
            inserts: Vec::new(),
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

    // The track's effect chain, in signal order
    pub fn inserts(&self) -> &[TrackInsert] {
        &self.inserts
    }

    // The schema-2 accessor, kept because most callers still mean "the first effect". It reads
    // the chain rather than a second field, so the two cannot disagree
    pub fn insert(&self) -> Option<TrackInsert> {
        self.inserts.first().copied()
    }

    // Append one effect to the end of the chain; refuses past MAX_CHAIN_DEVICES
    pub fn push_insert(&mut self, insert: TrackInsert) -> Result<(), TrackError> {
        if self.inserts.len() >= MAX_CHAIN_DEVICES {
            return Err(TrackError::ChainLimit {
                limit: MAX_CHAIN_DEVICES,
            });
        }
        self.inserts.push(insert);
        Ok(())
    }

    // Insert at a position, so a chain can be built in any order; refuses past the end
    pub fn insert_at(&mut self, index: usize, insert: TrackInsert) -> Result<(), TrackError> {
        if index > self.inserts.len() {
            return Err(TrackError::ChainIndex {
                index,
                len: self.inserts.len(),
            });
        }
        if self.inserts.len() >= MAX_CHAIN_DEVICES {
            return Err(TrackError::ChainLimit {
                limit: MAX_CHAIN_DEVICES,
            });
        }
        self.inserts.insert(index, insert);
        Ok(())
    }

    // Remove by position, returning the effect whole so an undo can put it back unchanged
    pub fn remove_insert(&mut self, index: usize) -> Result<TrackInsert, TrackError> {
        if index >= self.inserts.len() {
            return Err(TrackError::ChainIndex {
                index,
                len: self.inserts.len(),
            });
        }
        Ok(self.inserts.remove(index))
    }

    // Move one effect to another position. Order is the signal path, so this is a real edit
    pub fn reorder_insert(&mut self, from: usize, to: usize) -> Result<(), TrackError> {
        if from >= self.inserts.len() || to >= self.inserts.len() {
            return Err(TrackError::ChainIndex {
                index: from.max(to),
                len: self.inserts.len(),
            });
        }
        let moved = self.inserts.remove(from);
        self.inserts.insert(to, moved);
        Ok(())
    }

    // Clamp through the effect's own descriptor
    pub fn set_insert_depth(&mut self, depth: f32) {
        self.set_insert_depth_at(0, depth);
    }

    pub fn set_insert_depth_at(&mut self, index: usize, depth: f32) {
        if let Some(insert) = self.inserts.get_mut(index) {
            insert.depth = GLOAM_PARAMETERS[GLOAM_DEPTH].clamp(depth);
        }
    }

    // Move a decoded schema-2 slot into the chain. Called by the schema 3 migration and by the
    // validator's repair-free check; idempotent, so running it twice changes nothing
    pub(crate) fn adopt_legacy_insert(&mut self) {
        if let Some(slot) = self.legacy_insert.take() {
            if self.inserts.is_empty() {
                self.inserts.push(slot);
            }
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

    // Mutable access for the schema migration only. Not public: every product edit goes through
    // a command so it is reversible, and a public mutable slice would be the hole in that
    pub(crate) fn tracks_mut(&mut self) -> &mut [Track] {
        &mut self.tracks
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
        let shape_changed =
            track.insert().map(|slot| slot.effect) != insert.map(|slot| slot.effect);
        track.inserts = insert.into_iter().collect();
        if shape_changed {
            self.structure_revision += 1;
        }
        Ok(())
    }

    // Clip and note authoring, on the list because the clip TABLE and the per-track placements
    // both live here and an edit usually touches both. R4-5 recorded that clip undo was
    // impossible "because EditHistory mutates ProjectDoc and ProjectDoc has no clip field until
    // slice 7" -- slice 7 landed, clips travel inside this list, and EditScope carries it, so
    // that blocker is gone rather than merely old
    pub fn remove_clip(&mut self, id: ObjectId) -> Result<MidiClip, ClipError> {
        // Refused while any track still places it: removing the clip a placement names would
        // leave a placement pointing at nothing, which no later edit could repair
        if self
            .tracks
            .iter()
            .any(|track| track.clips().placements().iter().any(|p| p.clip() == id))
        {
            return Err(ClipError::UnknownClip(id));
        }
        let index = self
            .clips
            .iter()
            .position(|clip| clip.id() == id)
            .ok_or(ClipError::UnknownClip(id))?;
        Ok(self.clips.remove(index))
    }

    // Place a clip on a track. The length comes from the clip table rather than the caller, so a
    // placement cannot claim a span its clip does not have
    pub fn insert_placement(
        &mut self,
        track: ObjectId,
        placement: ClipPlacement,
    ) -> Result<(), ClipError> {
        let length = self
            .clip(placement.clip())
            .ok_or(ClipError::UnknownClip(placement.clip()))?
            .length();
        let track = self
            .get_mut(track)
            .ok_or(ClipError::UnknownPlacement(placement.id()))?;
        track.clips_mut().insert(placement, length)?;
        Ok(())
    }

    pub fn remove_placement(
        &mut self,
        track: ObjectId,
        id: ObjectId,
    ) -> Result<ClipPlacement, ClipError> {
        self.get_mut(track)
            .ok_or(ClipError::UnknownPlacement(id))?
            .clips_mut()
            .remove(id)
    }

    // Move a placement along its track's timeline. Remove-then-insert rather than an in-place
    // start edit, because the non-overlap invariant must be rechecked against the new span and
    // TrackClips::insert is the one place that check lives. A refused move restores the original
    pub fn move_placement(
        &mut self,
        track: ObjectId,
        id: ObjectId,
        start: BeatTicks,
    ) -> Result<BeatTicks, ClipError> {
        let existing = self.remove_placement(track, id)?;
        let previous = existing.start();
        let mut moved = existing;
        moved.set_start(start)?;
        match self.insert_placement(track, moved) {
            Ok(()) => Ok(previous),
            Err(error) => {
                // Put it back exactly where it was; a refused move must change nothing
                self.insert_placement(track, existing)
                    .expect("the original placement fitted a moment ago");
                Err(error)
            }
        }
    }

    // Note authoring. Returns the index the note landed at, which is what an undo addresses
    pub fn insert_note(&mut self, clip: ObjectId, note: ClipNote) -> Result<usize, ClipError> {
        self.clip_mut(clip)
            .ok_or(ClipError::UnknownClip(clip))?
            .insert_note(note)
    }

    pub fn remove_note(&mut self, clip: ObjectId, index: usize) -> Result<ClipNote, ClipError> {
        self.clip_mut(clip)
            .ok_or(ClipError::UnknownClip(clip))?
            .remove_note(index)
    }

    // Chain edits, on the list rather than on Track, because adding or removing a node changes
    // the compiled graph's shape and only the list owns the revision that says so. Depth is the
    // exception and deliberately so: it travels the parameter lane and rebuilds nothing
    pub fn insert_effect(
        &mut self,
        id: ObjectId,
        position: usize,
        insert: TrackInsert,
    ) -> Result<(), TrackError> {
        let track = self.get_mut(id).ok_or(TrackError::UnknownTrack(id))?;
        track.insert_at(position, insert)?;
        self.structure_revision += 1;
        Ok(())
    }

    // Returns the effect whole, so an undo puts back what was there rather than a rebuild of it
    pub fn remove_effect(
        &mut self,
        id: ObjectId,
        position: usize,
    ) -> Result<TrackInsert, TrackError> {
        let track = self.get_mut(id).ok_or(TrackError::UnknownTrack(id))?;
        let removed = track.remove_insert(position)?;
        self.structure_revision += 1;
        Ok(removed)
    }

    // Order is the signal path, so moving an effect changes what the track sounds like
    pub fn move_effect(&mut self, id: ObjectId, from: usize, to: usize) -> Result<(), TrackError> {
        let track = self.get_mut(id).ok_or(TrackError::UnknownTrack(id))?;
        track.reorder_insert(from, to)?;
        self.structure_revision += 1;
        Ok(())
    }

    // A parameter edit, not a shape change: the revision deliberately does not advance
    pub fn set_effect_depth(
        &mut self,
        id: ObjectId,
        position: usize,
        depth: f32,
    ) -> Result<f32, TrackError> {
        let track = self.get_mut(id).ok_or(TrackError::UnknownTrack(id))?;
        let previous = track
            .inserts()
            .get(position)
            .ok_or(TrackError::ChainIndex {
                index: position,
                len: track.inserts().len(),
            })?
            .depth();
        track.set_insert_depth_at(position, depth);
        Ok(previous)
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

    // Where one PARAMETER of one device sits in the target list. Every device now contributes one
    // target per descriptor rather than one in total, so a caller must say which parameter it
    // means; `position` is the descriptor's own index
    pub fn instrument_parameter_index(&self, track_index: usize, position: usize) -> Option<usize> {
        let track = self.tracks.get(track_index)?;
        (position < crate::routing::instrument_parameters(track.instrument()).len())
            .then(|| self.targets_before(track_index) + position)
    }

    pub fn insert_parameter_index(
        &self,
        track_index: usize,
        chain_position: usize,
        parameter: usize,
    ) -> Option<usize> {
        let track = self.tracks.get(track_index)?;
        let slot = track.inserts().get(chain_position)?;
        if parameter >= crate::routing::effect_parameters(slot.effect()).len() {
            return None;
        }
        let mut index = self.targets_before(track_index)
            + crate::routing::instrument_parameters(track.instrument()).len();
        for earlier in &track.inserts()[..chain_position] {
            index += crate::routing::effect_parameters(earlier.effect()).len();
        }
        Some(index + parameter)
    }

    // None where the track declares no effect at that chain position
    pub fn insert_target_index(&self, track_index: usize) -> Option<usize> {
        self.insert_target_index_at(track_index, 0)
    }

    // One effect's parameter position, by its place in the chain. The instrument's target comes
    // first, so a chain position is offset by one from the track's own base
    pub fn insert_target_index_at(&self, track_index: usize, position: usize) -> Option<usize> {
        // The effect's DEPTH, which is what "the insert's target" meant when a device
        // contributed one target in total rather than one per descriptor
        self.insert_parameter_index(track_index, position, spectre_dsp::GLOAM_DEPTH)
    }

    pub fn gain_target_index(&self, track_index: usize) -> usize {
        self.targets_before(track_index) + self.track_parameter_span(track_index)
            - GAIN_PARAMETERS.len()
    }

    pub fn master_target_index(&self) -> usize {
        self.targets_before(self.tracks.len())
    }

    // How many targets precede the given track: a real prefix sum over each earlier track's own
    // chain length, not a fixed stride. It was `index * 2 + inserts` while a track could hold at
    // most one effect; with a chain of any length that arithmetic silently addresses the wrong
    // parameter from the second track onward
    fn targets_before(&self, track_index: usize) -> usize {
        let upto = track_index.min(self.tracks.len());
        (0..upto)
            .map(|index| self.track_parameter_span(index))
            .sum()
    }

    // How many targets one track contributes: every parameter of its instrument, of each chained
    // effect, and of its gain. It was a fixed 2-plus-chain-length while a device contributed one
    // target; counting descriptors is what lets every control on a device be addressed
    fn track_parameter_span(&self, track_index: usize) -> usize {
        let Some(track) = self.tracks.get(track_index) else {
            return 0;
        };
        let instrument = crate::routing::instrument_parameters(track.instrument()).len();
        let chain: usize = track
            .inserts()
            .iter()
            .map(|slot| crate::routing::effect_parameters(slot.effect()).len())
            .sum();
        instrument + chain + GAIN_PARAMETERS.len()
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
