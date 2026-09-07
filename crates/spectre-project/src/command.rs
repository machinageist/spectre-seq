// Author: Jeff
// Date: 2026-07-12
// Description: Atomic project commands, grouped transactions, and bounded undo/redo history
// Notes: App-thread project mutation seam; never callback-reachable

use crate::clip::{ClipError, ClipNote, ClipPlacement, MidiClip};
use crate::{ProjectDoc, Track, TrackError, TrackInsert, TrackInstrument, TrackList};
use spectre_core::ObjectId;
use spectre_core::{BeatTicks, TempoMap};
use std::collections::VecDeque;

// Command and history failures
//
// Not Eq: ClipError carries a velocity, and a type claiming Eq over an f32 would be claiming an
// equality the value does not have. Same reason CommandKind dropped it when levels arrived
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandError {
    EmptyTransaction,
    InvalidProjectName,
    ZeroHistoryCapacity,
    // The app derives its project name from the file path and holds none to edit, so a rename
    // aimed at a target that has no name is refused rather than silently dropped
    NameNotEditable,
    // The target carries no tempo. Refused rather than ignored, for the same reason a rename is
    TempoNotEditable,
    // TrackList already refuses an absent id and an out-of-range index before it mutates
    // anything, so the failure vocabulary is wrapped rather than reinvented
    Track(TrackError),
    // Clip authoring refuses through ClipError for the same reason track edits refuse through
    // TrackError: the model already has a failure vocabulary and a second one would drift
    Clip(ClipError),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyTransaction => "transaction must contain at least one command",
            Self::InvalidProjectName => "project name must contain a non-whitespace character",
            Self::ZeroHistoryCapacity => "history capacity must be greater than zero",
            Self::NameNotEditable => "this edit target has no project name to change",
            Self::TempoNotEditable => "this edit target has no tempo to change",
            Self::Track(error) => return write!(f, "{error}"),
            Self::Clip(error) => return write!(f, "{error}"),
        };
        f.write_str(message)
    }
}

impl std::error::Error for CommandError {}

// The state a command may mutate, borrowed from whatever holds it.
//
// This exists because the app does NOT hold a ProjectDoc and should not be made to. Its device
// list carries resolved &'static str keys and DspParameter descriptors, while the document's
// carries wire-format strings -- a deliberate difference that project.rs states outright -- so
// forcing one type on both would either strip the app of its descriptors or push them into the
// file format. Commands only ever touch the name and the track list, so that is what they take
pub struct EditScope<'a> {
    // None where the target has no editable name. Stated as an option rather than an empty
    // string so a rename is refused instead of appearing to succeed against nothing
    name: Option<&'a mut String>,
    tracks: &'a mut TrackList,
    // None where the target carries no tempo, for the same reason. A bare TrackList has none,
    // so a tempo edit against one is refused rather than silently doing nothing
    tempo: Option<&'a mut TempoMap>,
}

impl<'a> EditScope<'a> {
    // Build a scope from borrowed pieces. Public so a crate outside this one can supply a target
    // assembled from its own fields; the fields stay private so the set a command may touch is
    // still declared here rather than by whoever constructs one
    pub fn new(
        name: Option<&'a mut String>,
        tracks: &'a mut TrackList,
        tempo: Option<&'a mut TempoMap>,
    ) -> Self {
        Self {
            name,
            tracks,
            tempo,
        }
    }
}

// Anything a transaction can be applied to
pub trait Editable {
    fn edit_scope(&mut self) -> EditScope<'_>;
}

impl Editable for ProjectDoc {
    fn edit_scope(&mut self) -> EditScope<'_> {
        EditScope {
            name: Some(&mut self.name),
            tracks: &mut self.tracks,
            tempo: Some(&mut self.tempo_map),
        }
    }
}

// The app's own target. Its project name comes from the file path, so renames are refused here
impl Editable for TrackList {
    fn edit_scope(&mut self) -> EditScope<'_> {
        EditScope {
            name: None,
            tracks: self,
            tempo: None,
        }
    }
}

// Eq is deliberately absent from here down. Track levels are f32, and a type that claimed Eq
// over them would be claiming an equality the values do not have
#[derive(Debug, Clone, PartialEq)]
enum CommandKind {
    SetProjectName {
        name: String,
        validate: bool,
    },
    ReorderTrack {
        id: ObjectId,
        to_index: usize,
    },
    // InsertTrack and RemoveTrack are exact mutual inverses because TrackList::remove returns
    // the whole Track and insert takes one back at an index -- nothing about the track has to
    // be reconstructed, so an undone remove restores clips, level, mute, solo, and identity
    InsertTrack {
        index: usize,
        track: Box<Track>,
    },
    RemoveTrack {
        id: ObjectId,
    },
    SetTrackName {
        id: ObjectId,
        name: String,
    },
    SetTrackLevel {
        id: ObjectId,
        level: f32,
    },
    SetTrackInstrumentLevel {
        id: ObjectId,
        level: f32,
    },
    SetTrackMuted {
        id: ObjectId,
        muted: bool,
    },
    SetTrackSoloed {
        id: ObjectId,
        soloed: bool,
    },
    SetMasterLevel {
        level: f32,
    },
    // The WHOLE map, not a bpm. A tempo map may hold segments, so an inverse carrying only a
    // number could not restore one that did -- it would silently flatten a tempo curve into a
    // constant and call that an undo
    SetTempoMap {
        map: TempoMap,
    },
    // Changing an instrument changes the graph's shape, so it advances structure_revision and
    // the shell reports PLAN STALE until the engine is rebuilt -- the same rule adding an effect
    // follows. The inverse carries the previous instrument, read before the mutation
    SetTrackInstrument {
        id: ObjectId,
        instrument: TrackInstrument,
    },
    // InsertEffect and RemoveEffect are exact mutual inverses for the same reason the track pair
    // is: TrackList::remove_effect returns the effect whole, so an undone delete restores the
    // effect that was there rather than a rebuild that resembles it
    InsertEffect {
        track: ObjectId,
        position: usize,
        insert: TrackInsert,
    },
    RemoveEffect {
        track: ObjectId,
        position: usize,
    },
    MoveEffect {
        track: ObjectId,
        from: usize,
        to: usize,
    },
    SetEffectDepth {
        track: ObjectId,
        position: usize,
        depth: f32,
    },
    // Clip authoring. AddClip/RemoveClip are mutual inverses because remove_clip returns the
    // clip whole -- with every note in it -- so an undone delete restores the material rather
    // than an empty clip of the same name
    AddClip {
        clip: Box<MidiClip>,
    },
    RemoveClip {
        id: ObjectId,
    },
    InsertPlacement {
        track: ObjectId,
        placement: ClipPlacement,
    },
    RemovePlacement {
        track: ObjectId,
        id: ObjectId,
    },
    MovePlacement {
        track: ObjectId,
        id: ObjectId,
        start: BeatTicks,
    },
    // Note authoring. Editing one note -- a piano roll's drag, resize, or velocity change -- is
    // a REMOVE and an INSERT grouped in one Transaction rather than a command of its own:
    // notes are kept sorted by (start, note), so any edit that moves a note moves its index,
    // and Transaction already applies and reverses a group atomically
    InsertNote {
        clip: ObjectId,
        note: ClipNote,
    },
    RemoveNote {
        clip: ObjectId,
        index: usize,
    },
    // Resizing touches the clip AND every placement's cached length, so it is one command rather
    // than a caller remembering to do both
    SetClipLength {
        clip: ObjectId,
        length: BeatTicks,
    },
}

// One reversible project-model mutation
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectCommand {
    kind: CommandKind,
}

impl ProjectCommand {
    // Create a validated project rename
    pub fn rename(name: impl Into<String>) -> Self {
        Self {
            kind: CommandKind::SetProjectName {
                name: name.into(),
                validate: true,
            },
        }
    }

    // Move one track to an absolute index. Addressed by ObjectId, never by a from-index: a
    // position can be stale, an identity cannot, and identity is what CORE-001 is about
    pub fn reorder_tracks(id: ObjectId, to_index: usize) -> Self {
        Self {
            kind: CommandKind::ReorderTrack { id, to_index },
        }
    }

    // Add a track at an index. The Track arrives fully formed, with its ObjectId already
    // minted, so the command is deterministic and its inverse addresses a stable identity
    pub fn insert_track(index: usize, track: Track) -> Self {
        Self {
            kind: CommandKind::InsertTrack {
                index,
                track: Box::new(track),
            },
        }
    }

    // Remove a track by identity. The inverse carries the removed track whole
    pub fn remove_track(id: ObjectId) -> Self {
        Self {
            kind: CommandKind::RemoveTrack { id },
        }
    }

    pub fn set_track_name(id: ObjectId, name: impl Into<String>) -> Self {
        Self {
            kind: CommandKind::SetTrackName {
                id,
                name: name.into(),
            },
        }
    }

    pub fn set_track_level(id: ObjectId, level: f32) -> Self {
        Self {
            kind: CommandKind::SetTrackLevel { id, level },
        }
    }

    pub fn set_track_instrument_level(id: ObjectId, level: f32) -> Self {
        Self {
            kind: CommandKind::SetTrackInstrumentLevel { id, level },
        }
    }

    pub fn set_track_muted(id: ObjectId, muted: bool) -> Self {
        Self {
            kind: CommandKind::SetTrackMuted { id, muted },
        }
    }

    pub fn set_track_soloed(id: ObjectId, soloed: bool) -> Self {
        Self {
            kind: CommandKind::SetTrackSoloed { id, soloed },
        }
    }

    // Add one effect at a position in a track's chain. Order is the signal path, so the
    // position is part of the edit rather than an implementation detail
    pub fn insert_effect(track: ObjectId, position: usize, insert: TrackInsert) -> Self {
        Self {
            kind: CommandKind::InsertEffect {
                track,
                position,
                insert,
            },
        }
    }

    pub fn remove_effect(track: ObjectId, position: usize) -> Self {
        Self {
            kind: CommandKind::RemoveEffect { track, position },
        }
    }

    pub fn move_effect(track: ObjectId, from: usize, to: usize) -> Self {
        Self {
            kind: CommandKind::MoveEffect { track, from, to },
        }
    }

    pub fn set_effect_depth(track: ObjectId, position: usize, depth: f32) -> Self {
        Self {
            kind: CommandKind::SetEffectDepth {
                track,
                position,
                depth,
            },
        }
    }

    pub fn add_clip(clip: MidiClip) -> Self {
        Self {
            kind: CommandKind::AddClip {
                clip: Box::new(clip),
            },
        }
    }

    pub fn remove_clip(id: ObjectId) -> Self {
        Self {
            kind: CommandKind::RemoveClip { id },
        }
    }

    pub fn insert_placement(track: ObjectId, placement: ClipPlacement) -> Self {
        Self {
            kind: CommandKind::InsertPlacement { track, placement },
        }
    }

    pub fn remove_placement(track: ObjectId, id: ObjectId) -> Self {
        Self {
            kind: CommandKind::RemovePlacement { track, id },
        }
    }

    pub fn move_placement(track: ObjectId, id: ObjectId, start: BeatTicks) -> Self {
        Self {
            kind: CommandKind::MovePlacement { track, id, start },
        }
    }

    pub fn set_clip_length(clip: ObjectId, length: BeatTicks) -> Self {
        Self {
            kind: CommandKind::SetClipLength { clip, length },
        }
    }

    pub fn insert_note(clip: ObjectId, note: ClipNote) -> Self {
        Self {
            kind: CommandKind::InsertNote { clip, note },
        }
    }

    pub fn remove_note(clip: ObjectId, index: usize) -> Self {
        Self {
            kind: CommandKind::RemoveNote { clip, index },
        }
    }

    // Replace the project's tempo map. Takes the whole map so a caller can set a constant or a
    // curve through one command, and so the inverse can restore either exactly
    pub fn set_track_instrument(id: ObjectId, instrument: TrackInstrument) -> Self {
        Self {
            kind: CommandKind::SetTrackInstrument { id, instrument },
        }
    }

    pub fn set_tempo_map(map: TempoMap) -> Self {
        Self {
            kind: CommandKind::SetTempoMap { map },
        }
    }

    pub fn set_master_level(level: f32) -> Self {
        Self {
            kind: CommandKind::SetMasterLevel { level },
        }
    }

    fn apply(&self, scope: &mut EditScope<'_>) -> Result<Self, CommandError> {
        match &self.kind {
            CommandKind::SetProjectName { name, validate } => {
                if *validate && name.trim().is_empty() {
                    return Err(CommandError::InvalidProjectName);
                }
                let Some(target) = scope.name.as_deref_mut() else {
                    return Err(CommandError::NameNotEditable);
                };
                let previous = std::mem::replace(target, name.clone());
                Ok(Self {
                    kind: CommandKind::SetProjectName {
                        name: previous,
                        validate: false,
                    },
                })
            }
            // TrackList::reorder returns the index the track came from, so the inverse is exact
            // by construction rather than by an argument about remove/insert symmetry. The
            // refusal happens before any mutation, which is what preserves Transaction's
            // all-or-nothing property
            CommandKind::ReorderTrack { id, to_index } => {
                let from = scope
                    .tracks
                    .reorder(*id, *to_index)
                    .map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::ReorderTrack {
                        id: *id,
                        to_index: from,
                    },
                })
            }
            // TrackList::insert refuses a duplicate id, an out-of-range index, and a full list
            // before mutating, so a refusal here leaves the transaction's rollback exact
            CommandKind::InsertTrack { index, track } => {
                let id = track.id();
                scope
                    .tracks
                    .insert(*index, (**track).clone())
                    .map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::RemoveTrack { id },
                })
            }
            // The removed track travels into the inverse whole, so undoing a delete restores
            // clips, mixer state, and identity rather than a track that merely resembles it
            CommandKind::RemoveTrack { id } => {
                let index = scope
                    .tracks
                    .index_of(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?;
                let track = scope.tracks.remove(*id).map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::InsertTrack {
                        index,
                        track: Box::new(track),
                    },
                })
            }
            CommandKind::SetTrackName { id, name } => {
                let track = scope
                    .tracks
                    .get_mut(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?;
                let previous = track.name().to_string();
                track.set_name(name).map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::SetTrackName {
                        id: *id,
                        name: previous,
                    },
                })
            }
            // Every level inverse captures the STORED value, read before the mutation and
            // therefore already clamped. Capturing the requested value instead would make undo
            // restore a number the track never held
            CommandKind::SetTrackLevel { id, level } => {
                let track = scope
                    .tracks
                    .get_mut(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?;
                let previous = track.level();
                track.set_level(*level);
                Ok(Self {
                    kind: CommandKind::SetTrackLevel {
                        id: *id,
                        level: previous,
                    },
                })
            }
            CommandKind::SetTrackInstrumentLevel { id, level } => {
                let track = scope
                    .tracks
                    .get_mut(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?;
                let previous = track.instrument_level();
                track.set_instrument_level(*level);
                Ok(Self {
                    kind: CommandKind::SetTrackInstrumentLevel {
                        id: *id,
                        level: previous,
                    },
                })
            }
            CommandKind::SetTrackMuted { id, muted } => {
                let track = scope
                    .tracks
                    .get_mut(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?;
                let previous = track.is_muted();
                track.set_muted(*muted);
                Ok(Self {
                    kind: CommandKind::SetTrackMuted {
                        id: *id,
                        muted: previous,
                    },
                })
            }
            CommandKind::SetTrackSoloed { id, soloed } => {
                let track = scope
                    .tracks
                    .get_mut(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?;
                let previous = track.is_soloed();
                track.set_soloed(*soloed);
                Ok(Self {
                    kind: CommandKind::SetTrackSoloed {
                        id: *id,
                        soloed: previous,
                    },
                })
            }
            CommandKind::InsertEffect {
                track,
                position,
                insert,
            } => {
                scope
                    .tracks
                    .insert_effect(*track, *position, *insert)
                    .map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::RemoveEffect {
                        track: *track,
                        position: *position,
                    },
                })
            }
            CommandKind::RemoveEffect { track, position } => {
                let removed = scope
                    .tracks
                    .remove_effect(*track, *position)
                    .map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::InsertEffect {
                        track: *track,
                        position: *position,
                        insert: removed,
                    },
                })
            }
            // Its own inverse with the ends swapped. remove-then-insert is not symmetric for an
            // arbitrary pair, so the inverse is stated rather than assumed: moving from `to` back
            // to `from` restores the original order for any pair of positions
            CommandKind::MoveEffect { track, from, to } => {
                scope
                    .tracks
                    .move_effect(*track, *from, *to)
                    .map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::MoveEffect {
                        track: *track,
                        from: *to,
                        to: *from,
                    },
                })
            }
            // The stored, already-clamped depth, read before the mutation
            CommandKind::SetEffectDepth {
                track,
                position,
                depth,
            } => {
                let previous = scope
                    .tracks
                    .set_effect_depth(*track, *position, *depth)
                    .map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::SetEffectDepth {
                        track: *track,
                        position: *position,
                        depth: previous,
                    },
                })
            }
            CommandKind::AddClip { clip } => {
                let id = clip.id();
                scope
                    .tracks
                    .add_clip((**clip).clone())
                    .map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::RemoveClip { id },
                })
            }
            CommandKind::RemoveClip { id } => {
                let removed = scope.tracks.remove_clip(*id).map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::AddClip {
                        clip: Box::new(removed),
                    },
                })
            }
            CommandKind::InsertPlacement { track, placement } => {
                scope
                    .tracks
                    .insert_placement(*track, *placement)
                    .map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::RemovePlacement {
                        track: *track,
                        id: placement.id(),
                    },
                })
            }
            CommandKind::RemovePlacement { track, id } => {
                let removed = scope
                    .tracks
                    .remove_placement(*track, *id)
                    .map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::InsertPlacement {
                        track: *track,
                        placement: removed,
                    },
                })
            }
            // The inverse carries the start the placement CAME FROM, which move_placement
            // returns, rather than a start recomputed from a delta
            CommandKind::MovePlacement { track, id, start } => {
                let previous = scope
                    .tracks
                    .move_placement(*track, *id, *start)
                    .map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::MovePlacement {
                        track: *track,
                        id: *id,
                        start: previous,
                    },
                })
            }
            // The inverse addresses the index the note actually landed at, which insert_note
            // returns. Notes are kept sorted, so that is not the index the caller expected
            CommandKind::InsertNote { clip, note } => {
                let index = scope
                    .tracks
                    .insert_note(*clip, *note)
                    .map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::RemoveNote { clip: *clip, index },
                })
            }
            CommandKind::RemoveNote { clip, index } => {
                let removed = scope
                    .tracks
                    .remove_note(*clip, *index)
                    .map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::InsertNote {
                        clip: *clip,
                        note: removed,
                    },
                })
            }
            CommandKind::SetTrackInstrument { id, instrument } => {
                let previous = scope
                    .tracks
                    .get(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?
                    .instrument();
                scope
                    .tracks
                    .set_instrument(*id, *instrument)
                    .map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::SetTrackInstrument {
                        id: *id,
                        instrument: previous,
                    },
                })
            }
            CommandKind::SetTempoMap { map } => {
                let Some(target) = scope.tempo.as_deref_mut() else {
                    return Err(CommandError::TempoNotEditable);
                };
                let previous = std::mem::replace(target, map.clone());
                Ok(Self {
                    kind: CommandKind::SetTempoMap { map: previous },
                })
            }
            CommandKind::SetClipLength { clip, length } => {
                let previous = scope
                    .tracks
                    .set_clip_length(*clip, *length)
                    .map_err(CommandError::Clip)?;
                Ok(Self {
                    kind: CommandKind::SetClipLength {
                        clip: *clip,
                        length: previous,
                    },
                })
            }
            CommandKind::SetMasterLevel { level } => {
                let previous = scope.tracks.master_level();
                scope.tracks.set_master_level(*level);
                Ok(Self {
                    kind: CommandKind::SetMasterLevel { level: previous },
                })
            }
        }
    }
}

// Ordered command group that applies and reverses atomically
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    commands: Vec<ProjectCommand>,
}

impl Transaction {
    // Reject empty edits that would pollute history
    pub fn new(commands: Vec<ProjectCommand>) -> Result<Self, CommandError> {
        if commands.is_empty() {
            return Err(CommandError::EmptyTransaction);
        }
        Ok(Self { commands })
    }

    // Build a one-command transaction
    pub fn single(command: ProjectCommand) -> Self {
        Self {
            commands: vec![command],
        }
    }

    fn execute(&self, scope: &mut EditScope<'_>) -> Result<Self, CommandError> {
        let mut inverses = Vec::with_capacity(self.commands.len());
        for command in &self.commands {
            match command.apply(scope) {
                Ok(inverse) => inverses.push(inverse),
                Err(error) => {
                    for inverse in inverses.iter().rev() {
                        inverse
                            .apply(scope)
                            .expect("generated inverse commands are infallible");
                    }
                    return Err(error);
                }
            }
        }
        inverses.reverse();
        Ok(Self { commands: inverses })
    }
}

// Bounded undo/redo stacks with oldest-edit eviction
#[derive(Debug)]
pub struct EditHistory {
    capacity: usize,
    undo: VecDeque<Transaction>,
    redo: VecDeque<Transaction>,
}

impl EditHistory {
    // Create history with an explicit nonzero bound
    pub fn new(capacity: usize) -> Result<Self, CommandError> {
        if capacity == 0 {
            return Err(CommandError::ZeroHistoryCapacity);
        }
        Ok(Self {
            capacity,
            undo: VecDeque::with_capacity(capacity),
            redo: VecDeque::with_capacity(capacity),
        })
    }

    // Apply one atomic edit and clear the abandoned redo branch
    pub fn apply<E: Editable + ?Sized>(
        &mut self,
        target: &mut E,
        transaction: Transaction,
    ) -> Result<(), CommandError> {
        let inverse = transaction.execute(&mut target.edit_scope())?;
        Self::push_bounded(&mut self.undo, inverse, self.capacity);
        self.redo.clear();
        Ok(())
    }

    // Reverse the latest edit; false means no edit was available
    pub fn undo<E: Editable + ?Sized>(&mut self, target: &mut E) -> Result<bool, CommandError> {
        Self::transfer(&mut self.undo, &mut self.redo, target, self.capacity)
    }

    // Reapply the latest reversed edit; false means no edit was available
    pub fn redo<E: Editable + ?Sized>(&mut self, target: &mut E) -> Result<bool, CommandError> {
        Self::transfer(&mut self.redo, &mut self.undo, target, self.capacity)
    }

    // Whether an edit is available in each direction, so a shell can disable rather than offer
    // a control that would do nothing
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn transfer<E: Editable + ?Sized>(
        source: &mut VecDeque<Transaction>,
        destination: &mut VecDeque<Transaction>,
        target: &mut E,
        capacity: usize,
    ) -> Result<bool, CommandError> {
        let Some(transaction) = source.pop_back() else {
            return Ok(false);
        };
        match transaction.execute(&mut target.edit_scope()) {
            Ok(inverse) => {
                Self::push_bounded(destination, inverse, capacity);
                Ok(true)
            }
            Err(error) => {
                source.push_back(transaction);
                Err(error)
            }
        }
    }

    fn push_bounded(stack: &mut VecDeque<Transaction>, transaction: Transaction, capacity: usize) {
        if stack.len() == capacity {
            stack.pop_front();
        }
        stack.push_back(transaction);
    }
}
