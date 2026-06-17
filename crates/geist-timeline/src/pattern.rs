// =============================================================================
// File: crates/geist-timeline/src/pattern.rs
// Layer: timeline
// Purpose: Pattern: note grid for piano roll / step seq
// Status: Implemented; note list emitting NoteEvents for a sample window.
// Notes: Note positions are samples relative to the pattern origin. Emission
//        pushes On/Off events whose offset is relative to the query window.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::events::NoteEvent;

// A single note in a pattern; positions are samples from the pattern origin
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    pub start: u64,
    pub length: u64,
    pub key: u8,
    pub velocity: f32,
    pub channel: u8,
}

impl Note {
    // Build a note on channel 0
    pub fn new(start: u64, length: u64, key: u8, velocity: f32) -> Self {
        Self {
            start,
            length,
            key,
            velocity,
            channel: 0,
        }
    }

    // Sample position one past the note's last sounding sample
    #[inline]
    pub fn end(&self) -> u64 {
        self.start + self.length
    }
}

// A grid of notes over a fixed-length region; the piano-roll / step-seq model
#[derive(Clone, Debug, Default)]
pub struct Pattern {
    notes: Vec<Note>,
    length: u64,
}

impl Pattern {
    // Build an empty pattern of the given length in samples
    pub fn new(length: u64) -> Self {
        Self {
            notes: Vec::new(),
            length,
        }
    }

    // Pattern length in samples
    pub fn length(&self) -> u64 {
        self.length
    }

    // Set the pattern length in samples
    pub fn set_length(&mut self, length: u64) {
        self.length = length;
    }

    // Add a note to the grid
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    // All notes in insertion order
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    // Remove every note
    pub fn clear(&mut self) {
        self.notes.clear();
    }

    // Push On/Off events that fall inside [window_start, window_start+window_len)
    // Event offsets are relative to window_start; never allocates beyond `out`
    pub fn emit_events(&self, window_start: u64, window_len: u64, out: &mut Vec<NoteEvent>) {
        let window_end = window_start + window_len;
        for note in &self.notes {
            let on = note.start;
            if on >= window_start && on < window_end {
                let offset = (on - window_start) as u32;
                out.push(NoteEvent::on(offset, note.channel, note.key, note.velocity));
            }
            let off = note.end();
            if off >= window_start && off < window_end {
                let offset = (off - window_start) as u32;
                out.push(NoteEvent::off(offset, note.channel, note.key));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::events::NoteEventKind;

    fn pattern() -> Pattern {
        let mut p = Pattern::new(1_000);
        p.add_note(Note::new(100, 200, 60, 1.0)); // on@100 off@300
        p.add_note(Note::new(500, 100, 64, 0.8)); // on@500 off@600
        p
    }

    #[test]
    fn emits_note_on_at_relative_offset() {
        let p = pattern();
        let mut out = Vec::new();
        p.emit_events(0, 256, &mut out); // window [0,256): only note 60's on@100
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, NoteEventKind::On);
        assert_eq!(out[0].key, 60);
        assert_eq!(out[0].sample_offset, 100);
        assert_eq!(out[0].velocity, 1.0);
    }

    #[test]
    fn emits_note_off_when_end_falls_in_window() {
        let p = pattern();
        let mut out = Vec::new();
        p.emit_events(256, 256, &mut out); // window [256,512): off@300, on@500
        let kinds: Vec<_> = out
            .iter()
            .map(|e| (e.kind, e.key, e.sample_offset))
            .collect();
        assert!(kinds.contains(&(NoteEventKind::Off, 60, 300 - 256)));
        assert!(kinds.contains(&(NoteEventKind::On, 64, 500 - 256)));
    }

    #[test]
    fn notes_outside_window_emit_nothing() {
        let p = pattern();
        let mut out = Vec::new();
        p.emit_events(700, 100, &mut out); // window [700,800): nothing
        assert!(out.is_empty());
    }

    #[test]
    fn sustained_note_spanning_window_emits_neither_edge() {
        let mut p = Pattern::new(10_000);
        p.add_note(Note::new(0, 5_000, 48, 1.0)); // on@0 off@5000
        let mut out = Vec::new();
        p.emit_events(1_000, 256, &mut out); // fully inside the sustain
        assert!(out.is_empty());
    }

    #[test]
    fn window_boundary_is_half_open() {
        let mut p = Pattern::new(10_000);
        // Long note so only the on-edge falls near the boundary under test
        p.add_note(Note::new(256, 5_000, 60, 1.0)); // on@256 off@5256
        let mut out = Vec::new();
        p.emit_events(0, 256, &mut out); // [0,256) excludes 256
        assert!(out.is_empty());
        p.emit_events(256, 256, &mut out); // [256,512) includes 256
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, NoteEventKind::On);
        assert_eq!(out[0].sample_offset, 0);
    }
}
