// =============================================================================
// File: crates/geist-timeline/src/commands.rs
// Layer: timeline
// Purpose: command objects for undo/redo
// Status: Implemented; reversible arrangement commands + an undo/redo stack.
// Notes: Each command captures enough state at apply time to reverse itself.
//        New edits truncate the redo stack, the standard editor contract.
//        Tempo-edit commands wait on unifying TempoMap ownership into the
//        arrangement; today the TempoMap lives on Transport (playback).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::arena::Index;
use crate::track::Timeline;

// A reversible edit on the timeline
pub trait Command {
    // Apply the edit
    fn apply(&mut self, timeline: &mut Timeline);
    // Reverse the edit applied earlier
    fn undo(&mut self, timeline: &mut Timeline);
    // Short human-readable label for history UIs
    fn label(&self) -> &'static str;
}

// Place a clip on a track at a position
pub struct PlaceClip {
    pub track: usize,
    pub clip: Index,
    pub start: u64,
}

impl Command for PlaceClip {
    fn apply(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            track.place_clip(self.clip, self.start);
        }
    }
    fn undo(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            track.remove_placement(self.clip, self.start);
        }
    }
    fn label(&self) -> &'static str {
        "Place clip"
    }
}

// Remove a clip placement; undo restores it at the same position
pub struct RemoveClip {
    pub track: usize,
    pub clip: Index,
    pub start: u64,
}

impl Command for RemoveClip {
    fn apply(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            track.remove_placement(self.clip, self.start);
        }
    }
    fn undo(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            track.place_clip(self.clip, self.start);
        }
    }
    fn label(&self) -> &'static str {
        "Remove clip"
    }
}

// Move a clip placement to a new position
pub struct MoveClip {
    pub track: usize,
    pub clip: Index,
    pub from: u64,
    pub to: u64,
}

impl Command for MoveClip {
    fn apply(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            track.move_placement(self.clip, self.from, self.to);
        }
    }
    fn undo(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            track.move_placement(self.clip, self.to, self.from);
        }
    }
    fn label(&self) -> &'static str {
        "Move clip"
    }
}

// Set a track's mute flag; undo restores the previous value
pub struct SetTrackMute {
    pub track: usize,
    pub muted: bool,
    // Captured at apply time
    prev: bool,
}

impl SetTrackMute {
    // Build a mute-set command
    pub fn new(track: usize, muted: bool) -> Self {
        Self {
            track,
            muted,
            prev: false,
        }
    }
}

impl Command for SetTrackMute {
    fn apply(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            self.prev = track.muted;
            track.muted = self.muted;
        }
    }
    fn undo(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.track_mut(self.track) {
            track.muted = self.prev;
        }
    }
    fn label(&self) -> &'static str {
        "Set track mute"
    }
}

// Append a track; undo removes the last track
#[derive(Default)]
pub struct AddTrack;

impl Command for AddTrack {
    fn apply(&mut self, timeline: &mut Timeline) {
        timeline.add_track();
    }
    fn undo(&mut self, timeline: &mut Timeline) {
        timeline.remove_last_track();
    }
    fn label(&self) -> &'static str {
        "Add track"
    }
}

// Undo/redo history; executing a new command clears the redo stack
#[derive(Default)]
pub struct UndoStack {
    done: Vec<Box<dyn Command>>,
    undone: Vec<Box<dyn Command>>,
}

impl UndoStack {
    // Build an empty history
    pub fn new() -> Self {
        Self::default()
    }

    // Apply a command, record it, and invalidate any redo history
    pub fn execute(&mut self, mut command: Box<dyn Command>, timeline: &mut Timeline) {
        command.apply(timeline);
        self.done.push(command);
        self.undone.clear();
    }

    // Whether there is an applied command to reverse
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    // Whether there is a reversed command to reapply
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    // Reverse the most recent command; returns false if none
    pub fn undo(&mut self, timeline: &mut Timeline) -> bool {
        if let Some(mut command) = self.done.pop() {
            command.undo(timeline);
            self.undone.push(command);
            true
        } else {
            false
        }
    }

    // Reapply the most recently undone command; returns false if none
    pub fn redo(&mut self, timeline: &mut Timeline) -> bool {
        if let Some(mut command) = self.undone.pop() {
            command.apply(timeline);
            self.done.push(command);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::{Clip, MidiClip};
    use crate::pattern::{Note, Pattern};
    use spectre_core::events::NoteEventKind;

    // Timeline with a one-note MIDI clip and a single track, returning clip handle
    fn fixture() -> (Timeline, Index) {
        let mut tl = Timeline::new();
        let mut pat = Pattern::new(1_000);
        pat.add_note(Note::new(100, 50, 60, 1.0));
        let pidx = tl.add_pattern(pat);
        let clip = tl.add_clip(Clip::Midi(MidiClip { pattern: pidx }));
        tl.add_track();
        (tl, clip)
    }

    fn note_count(tl: &Timeline) -> usize {
        let mut out = Vec::new();
        tl.emit_notes(0, 256, &mut out);
        out.iter().filter(|e| e.kind == NoteEventKind::On).count()
    }

    #[test]
    fn place_clip_undo_redo_round_trips() {
        let (mut tl, clip) = fixture();
        let mut stack = UndoStack::new();

        stack.execute(
            Box::new(PlaceClip {
                track: 0,
                clip,
                start: 0,
            }),
            &mut tl,
        );
        assert_eq!(note_count(&tl), 1);

        assert!(stack.undo(&mut tl));
        assert_eq!(note_count(&tl), 0);

        assert!(stack.redo(&mut tl));
        assert_eq!(note_count(&tl), 1);
    }

    #[test]
    fn move_clip_is_reversible() {
        let (mut tl, clip) = fixture();
        let mut stack = UndoStack::new();
        stack.execute(
            Box::new(PlaceClip {
                track: 0,
                clip,
                start: 0,
            }),
            &mut tl,
        );
        // Move the clip out past the query window, then back
        stack.execute(
            Box::new(MoveClip {
                track: 0,
                clip,
                from: 0,
                to: 10_000,
            }),
            &mut tl,
        );
        assert_eq!(note_count(&tl), 0); // clip moved away
        stack.undo(&mut tl);
        assert_eq!(note_count(&tl), 1); // back at 0
    }

    #[test]
    fn remove_clip_is_reversible() {
        let (mut tl, clip) = fixture();
        tl.track_mut(0).unwrap().place_clip(clip, 0);
        let mut stack = UndoStack::new();
        stack.execute(
            Box::new(RemoveClip {
                track: 0,
                clip,
                start: 0,
            }),
            &mut tl,
        );
        assert_eq!(note_count(&tl), 0);
        stack.undo(&mut tl);
        assert_eq!(note_count(&tl), 1);
    }

    #[test]
    fn set_track_mute_restores_previous() {
        let (mut tl, clip) = fixture();
        tl.track_mut(0).unwrap().place_clip(clip, 0);
        let mut stack = UndoStack::new();
        assert_eq!(note_count(&tl), 1);

        stack.execute(Box::new(SetTrackMute::new(0, true)), &mut tl);
        assert_eq!(note_count(&tl), 0); // muted
        stack.undo(&mut tl);
        assert_eq!(note_count(&tl), 1); // unmuted again
    }

    #[test]
    fn new_command_truncates_redo() {
        let (mut tl, clip) = fixture();
        let mut stack = UndoStack::new();
        stack.execute(Box::new(AddTrack), &mut tl);
        stack.undo(&mut tl);
        assert!(stack.can_redo());

        // A fresh edit discards the redo branch
        stack.execute(
            Box::new(PlaceClip {
                track: 0,
                clip,
                start: 0,
            }),
            &mut tl,
        );
        assert!(!stack.can_redo());
    }

    #[test]
    fn undo_redo_flags_track_availability() {
        let (mut tl, _) = fixture();
        let mut stack = UndoStack::new();
        assert!(!stack.can_undo() && !stack.can_redo());
        stack.execute(Box::new(AddTrack), &mut tl);
        assert!(stack.can_undo() && !stack.can_redo());
        stack.undo(&mut tl);
        assert!(!stack.can_undo() && stack.can_redo());
    }

    #[test]
    fn add_track_undo_removes_it() {
        let (mut tl, _) = fixture();
        let before = tl.track_count();
        let mut stack = UndoStack::new();
        stack.execute(Box::new(AddTrack), &mut tl);
        assert_eq!(tl.track_count(), before + 1);
        stack.undo(&mut tl);
        assert_eq!(tl.track_count(), before);
    }
}
