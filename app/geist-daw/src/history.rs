// =============================================================================
// File: app/geist-daw/src/history.rs
// Layer: application binary
// Purpose: Snapshot-based undo/redo for the studio's editable surface
// Status: Implemented; bounded undo/redo over a clone of the edit state.
// Notes: Studio edits mutate UI state and a per-frame diff resyncs the engine,
//        so undo restores a prior UI snapshot and lets the diff catch the engine
//        up. Snapshots coalesce a gesture (committed on pointer release), so one
//        drag is one undo step. No audio-thread involvement.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::HashMap;

use geist_ui::model::{Note, RackModel, SessionGrid, TimelineModel};

// Most undo steps retained; older steps drop off the bottom
const MAX_HISTORY: usize = 64;

// A clone of the editable studio surface at one settled point in time
#[derive(Clone, PartialEq)]
pub struct EditSnapshot {
    pub timeline: TimelineModel,
    pub clip_notes: HashMap<u64, Vec<Note>>,
    pub session_grid: SessionGrid,
    pub session_notes: HashMap<(u8, u8), Vec<Note>>,
    pub rack: RackModel,
    pub track_racks: Vec<RackModel>,
}

// Bounded undo/redo stacks plus the last committed state
#[derive(Default)]
pub struct EditHistory {
    last: Option<EditSnapshot>,
    undo: Vec<EditSnapshot>,
    redo: Vec<EditSnapshot>,
}

impl EditHistory {
    pub fn new() -> Self {
        Self::default()
    }

    // True before the first snapshot is recorded
    pub fn needs_seed(&self) -> bool {
        self.last.is_none()
    }

    // Record the initial state without pushing an undo step
    pub fn seed(&mut self, snapshot: EditSnapshot) {
        self.last = Some(snapshot);
    }

    // Commit a settled edit: push the prior state to undo and clear redo. A no-op
    // when nothing changed, so it is cheap to call on every gesture boundary.
    pub fn commit(&mut self, current: EditSnapshot) {
        if self.last.as_ref() == Some(&current) {
            return;
        }
        if let Some(prev) = self.last.replace(current) {
            self.undo.push(prev);
            if self.undo.len() > MAX_HISTORY {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
    }

    // Step back: returns the state to restore, given the current state for redo
    pub fn undo(&mut self, current: EditSnapshot) -> Option<EditSnapshot> {
        let snapshot = self.undo.pop()?;
        self.redo.push(current);
        self.last = Some(snapshot.clone());
        Some(snapshot)
    }

    // Step forward: returns the state to restore, given the current state for undo
    pub fn redo(&mut self, current: EditSnapshot) -> Option<EditSnapshot> {
        let snapshot = self.redo.pop()?;
        self.undo.push(current);
        self.last = Some(snapshot.clone());
        Some(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(len: f32) -> EditSnapshot {
        EditSnapshot {
            timeline: TimelineModel { length_beats: len, ..Default::default() },
            clip_notes: HashMap::new(),
            session_grid: SessionGrid::new(1, 1),
            session_notes: HashMap::new(),
            rack: RackModel::default(),
            track_racks: Vec::new(),
        }
    }

    #[test]
    fn undo_then_redo_round_trips() {
        let mut h = EditHistory::new();
        h.seed(snap(1.0));
        h.commit(snap(2.0));
        h.commit(snap(3.0));

        // Undo from 3 -> 2 -> 1
        let s = h.undo(snap(3.0)).unwrap();
        assert_eq!(s.timeline.length_beats, 2.0);
        let s = h.undo(snap(2.0)).unwrap();
        assert_eq!(s.timeline.length_beats, 1.0);
        assert!(h.undo(snap(1.0)).is_none(), "undo past the seed");

        // Redo back up to 3
        let s = h.redo(snap(1.0)).unwrap();
        assert_eq!(s.timeline.length_beats, 2.0);
        let s = h.redo(snap(2.0)).unwrap();
        assert_eq!(s.timeline.length_beats, 3.0);
    }

    #[test]
    fn commit_truncates_redo() {
        let mut h = EditHistory::new();
        h.seed(snap(1.0));
        h.commit(snap(2.0));
        let _ = h.undo(snap(2.0));
        // A new edit after undo drops the redo branch
        h.commit(snap(5.0));
        assert!(h.redo(snap(5.0)).is_none());
    }

    #[test]
    fn unchanged_commit_is_a_noop() {
        let mut h = EditHistory::new();
        h.seed(snap(1.0));
        h.commit(snap(1.0));
        assert!(h.undo(snap(1.0)).is_none(), "identical commit created a step");
    }
}
