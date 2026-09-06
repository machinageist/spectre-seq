// Author: Jeff
// Date: 2026-07-12
// Description: Atomic project commands, grouped transactions, and bounded undo/redo history
// Notes: App-thread project mutation seam; never callback-reachable

use crate::{ProjectDoc, Track, TrackError};
use spectre_core::ObjectId;
use std::collections::VecDeque;

// Command and history failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError {
    EmptyTransaction,
    InvalidProjectName,
    ZeroHistoryCapacity,
    // TrackList already refuses an absent id and an out-of-range index before it mutates
    // anything, so the failure vocabulary is wrapped rather than reinvented
    Track(TrackError),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyTransaction => "transaction must contain at least one command",
            Self::InvalidProjectName => "project name must contain a non-whitespace character",
            Self::ZeroHistoryCapacity => "history capacity must be greater than zero",
            Self::Track(error) => return write!(f, "{error}"),
        };
        f.write_str(message)
    }
}

impl std::error::Error for CommandError {}

// Eq is deliberately absent from here down. Track levels are f32, and a type that claimed Eq
// over them would be claiming an equality the values do not have
#[derive(Debug, Clone, PartialEq)]
enum CommandKind {
    SetProjectName { name: String, validate: bool },
    ReorderTrack { id: ObjectId, to_index: usize },
    // InsertTrack and RemoveTrack are exact mutual inverses because TrackList::remove returns
    // the whole Track and insert takes one back at an index -- nothing about the track has to
    // be reconstructed, so an undone remove restores clips, level, mute, solo, and identity
    InsertTrack { index: usize, track: Box<Track> },
    RemoveTrack { id: ObjectId },
    SetTrackName { id: ObjectId, name: String },
    SetTrackLevel { id: ObjectId, level: f32 },
    SetTrackInstrumentLevel { id: ObjectId, level: f32 },
    SetTrackMuted { id: ObjectId, muted: bool },
    SetTrackSoloed { id: ObjectId, soloed: bool },
    SetMasterLevel { level: f32 },
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

    pub fn set_master_level(level: f32) -> Self {
        Self {
            kind: CommandKind::SetMasterLevel { level },
        }
    }

    fn apply(&self, project: &mut ProjectDoc) -> Result<Self, CommandError> {
        match &self.kind {
            CommandKind::SetProjectName { name, validate } => {
                if *validate && name.trim().is_empty() {
                    return Err(CommandError::InvalidProjectName);
                }
                let previous = std::mem::replace(&mut project.name, name.clone());
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
                let from = project
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
                project
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
                let index = project
                    .tracks
                    .index_of(*id)
                    .ok_or(CommandError::Track(TrackError::UnknownTrack(*id)))?;
                let track = project.tracks.remove(*id).map_err(CommandError::Track)?;
                Ok(Self {
                    kind: CommandKind::InsertTrack {
                        index,
                        track: Box::new(track),
                    },
                })
            }
            CommandKind::SetTrackName { id, name } => {
                let track = project
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
                let track = project
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
                let track = project
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
                let track = project
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
                let track = project
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
            CommandKind::SetMasterLevel { level } => {
                let previous = project.tracks.master_level();
                project.tracks.set_master_level(*level);
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

    fn execute(&self, project: &mut ProjectDoc) -> Result<Self, CommandError> {
        let mut inverses = Vec::with_capacity(self.commands.len());
        for command in &self.commands {
            match command.apply(project) {
                Ok(inverse) => inverses.push(inverse),
                Err(error) => {
                    for inverse in inverses.iter().rev() {
                        inverse
                            .apply(project)
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
    pub fn apply(
        &mut self,
        project: &mut ProjectDoc,
        transaction: Transaction,
    ) -> Result<(), CommandError> {
        let inverse = transaction.execute(project)?;
        Self::push_bounded(&mut self.undo, inverse, self.capacity);
        self.redo.clear();
        Ok(())
    }

    // Reverse the latest edit; false means no edit was available
    pub fn undo(&mut self, project: &mut ProjectDoc) -> Result<bool, CommandError> {
        Self::transfer(&mut self.undo, &mut self.redo, project, self.capacity)
    }

    // Reapply the latest reversed edit; false means no edit was available
    pub fn redo(&mut self, project: &mut ProjectDoc) -> Result<bool, CommandError> {
        Self::transfer(&mut self.redo, &mut self.undo, project, self.capacity)
    }

    fn transfer(
        source: &mut VecDeque<Transaction>,
        destination: &mut VecDeque<Transaction>,
        project: &mut ProjectDoc,
        capacity: usize,
    ) -> Result<bool, CommandError> {
        let Some(transaction) = source.pop_back() else {
            return Ok(false);
        };
        match transaction.execute(project) {
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
