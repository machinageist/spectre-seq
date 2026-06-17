// =============================================================================
// File: crates/geist-timeline/src/track.rs
// Layer: timeline
// Purpose: Track { id, clip_ids, armed, muted, soloed } and Timeline assembly
// Status: Implemented; tracks place clips; Timeline emits notes per block.
// Notes: Tracks hold clip placements (handle + timeline position); clip and
//        pattern data live in arenas. Note emission honors solo over mute and
//        translates pattern-local offsets back to the query window.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::events::NoteEvent;

use crate::arena::{Arena, Index};
use crate::clip::Clip;
use crate::pattern::Pattern;

// A clip handle positioned at a timeline sample offset
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipPlacement {
    pub clip: Index,
    pub start: u64,
}

// A track: an ordered set of clip placements plus mix flags
#[derive(Clone, Debug, Default)]
pub struct Track {
    clips: Vec<ClipPlacement>,
    pub muted: bool,
    pub soloed: bool,
    pub armed: bool,
}

impl Track {
    // Build an empty, unmuted track
    pub fn new() -> Self {
        Self::default()
    }

    // Place a clip at a timeline sample position
    pub fn place_clip(&mut self, clip: Index, start: u64) {
        self.clips.push(ClipPlacement { clip, start });
    }

    // All clip placements on this track
    pub fn placements(&self) -> &[ClipPlacement] {
        &self.clips
    }

    // Remove the first placement of a clip handle, returning whether one was found
    pub fn remove_clip(&mut self, clip: Index) -> bool {
        if let Some(i) = self.clips.iter().position(|p| p.clip == clip) {
            self.clips.remove(i);
            true
        } else {
            false
        }
    }

    // Remove the exact (clip, start) placement; precise enough for undo/redo
    pub fn remove_placement(&mut self, clip: Index, start: u64) -> bool {
        if let Some(i) = self
            .clips
            .iter()
            .position(|p| p.clip == clip && p.start == start)
        {
            self.clips.remove(i);
            true
        } else {
            false
        }
    }

    // Move a placement from one start position to another
    pub fn move_placement(&mut self, clip: Index, from: u64, to: u64) -> bool {
        if let Some(p) = self
            .clips
            .iter_mut()
            .find(|p| p.clip == clip && p.start == from)
        {
            p.start = to;
            true
        } else {
            false
        }
    }
}

// The arrangement: tracks plus the clip and pattern arenas they reference
#[derive(Debug, Default)]
pub struct Timeline {
    clips: Arena<Clip>,
    patterns: Arena<Pattern>,
    tracks: Vec<Track>,
}

impl Timeline {
    // Build an empty timeline
    pub fn new() -> Self {
        Self::default()
    }

    // Store a clip and return its handle
    pub fn add_clip(&mut self, clip: Clip) -> Index {
        self.clips.insert(clip)
    }

    // Store a pattern and return its handle
    pub fn add_pattern(&mut self, pattern: Pattern) -> Index {
        self.patterns.insert(pattern)
    }

    // Borrow a clip by handle
    pub fn clip(&self, index: Index) -> Option<&Clip> {
        self.clips.get(index)
    }

    // Borrow a pattern by handle
    pub fn pattern(&self, index: Index) -> Option<&Pattern> {
        self.patterns.get(index)
    }

    // Borrow a pattern mutably for editing
    pub fn pattern_mut(&mut self, index: Index) -> Option<&mut Pattern> {
        self.patterns.get_mut(index)
    }

    // Append a track, returning its index
    pub fn add_track(&mut self) -> usize {
        self.tracks.push(Track::new());
        self.tracks.len() - 1
    }

    // Remove the last track; used to undo an append
    pub fn remove_last_track(&mut self) -> bool {
        self.tracks.pop().is_some()
    }

    // Number of tracks
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    // Borrow a track by index
    pub fn track(&self, index: usize) -> Option<&Track> {
        self.tracks.get(index)
    }

    // Borrow a track mutably by index
    pub fn track_mut(&mut self, index: usize) -> Option<&mut Track> {
        self.tracks.get_mut(index)
    }

    // Whether any track is soloed (solo overrides mute on other tracks)
    pub fn any_soloed(&self) -> bool {
        self.tracks.iter().any(|t| t.soloed)
    }

    // Emit note events for [window_start, window_start+window_len), honoring
    // solo/mute. Offsets are relative to window_start.
    pub fn emit_notes(&self, window_start: u64, window_len: u64, out: &mut Vec<NoteEvent>) {
        let solo = self.any_soloed();
        let window_end = window_start + window_len;
        let mut scratch = Vec::new();

        for track in &self.tracks {
            let audible = if solo { track.soloed } else { !track.muted };
            if !audible {
                continue;
            }
            for placement in &track.clips {
                let Some(Clip::Midi(midi)) = self.clips.get(placement.clip) else {
                    continue;
                };
                let Some(pattern) = self.patterns.get(midi.pattern) else {
                    continue;
                };
                let clip_end = placement.start + pattern.length();
                let eff_start = window_start.max(placement.start);
                let eff_end = window_end.min(clip_end);
                if eff_start >= eff_end {
                    continue;
                }
                // Query the pattern in its own coordinates, then shift offsets
                // from clip-relative back to window-relative
                let local_start = eff_start - placement.start;
                let local_len = eff_end - eff_start;
                scratch.clear();
                pattern.emit_events(local_start, local_len, &mut scratch);
                let delta = (eff_start - window_start) as u32;
                for mut ev in scratch.drain(..) {
                    ev.sample_offset += delta;
                    out.push(ev);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::MidiClip;
    use crate::pattern::Note;
    use geist_core::events::NoteEventKind;

    // Timeline with one MIDI clip (a single note at pattern-local 100) placed at `start`
    fn timeline_with_clip(start: u64) -> (Timeline, usize) {
        let mut tl = Timeline::new();
        let mut pat = Pattern::new(1_000);
        pat.add_note(Note::new(100, 50, 60, 1.0)); // on@100 off@150
        let pat_idx = tl.add_pattern(pat);
        let clip_idx = tl.add_clip(Clip::Midi(MidiClip { pattern: pat_idx }));
        let track = tl.add_track();
        tl.track_mut(track).unwrap().place_clip(clip_idx, start);
        (tl, track)
    }

    #[test]
    fn emits_clip_note_at_zero_offset_placement() {
        let (tl, _) = timeline_with_clip(0);
        let mut out = Vec::new();
        tl.emit_notes(0, 256, &mut out);
        // on@100 and off@150 both inside [0,256)
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, NoteEventKind::On);
        assert_eq!(out[0].sample_offset, 100);
    }

    #[test]
    fn placement_offset_shifts_note_timing() {
        let (tl, _) = timeline_with_clip(1_000); // clip starts at sample 1000
        let mut out = Vec::new();
        // Note on lands at timeline 1100; window [1024,1280) covers it
        tl.emit_notes(1_024, 256, &mut out);
        let on = out.iter().find(|e| e.kind == NoteEventKind::On).unwrap();
        assert_eq!(on.sample_offset, 1_100 - 1_024);
    }

    #[test]
    fn window_before_clip_emits_nothing() {
        let (tl, _) = timeline_with_clip(10_000);
        let mut out = Vec::new();
        tl.emit_notes(0, 256, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn muted_track_is_silent() {
        let (mut tl, track) = timeline_with_clip(0);
        tl.track_mut(track).unwrap().muted = true;
        let mut out = Vec::new();
        tl.emit_notes(0, 256, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn solo_silences_other_tracks() {
        let (mut tl, first) = timeline_with_clip(0);
        // Second track with its own clip
        let mut pat = Pattern::new(1_000);
        pat.add_note(Note::new(50, 50, 72, 1.0));
        let pidx = tl.add_pattern(pat);
        let cidx = tl.add_clip(Clip::Midi(MidiClip { pattern: pidx }));
        let second = tl.add_track();
        tl.track_mut(second).unwrap().place_clip(cidx, 0);

        // Solo the second track
        tl.track_mut(second).unwrap().soloed = true;
        let mut out = Vec::new();
        tl.emit_notes(0, 256, &mut out);
        // Only the soloed track's note (key 72) should appear
        assert!(out.iter().all(|e| e.key == 72));
        assert!(out.iter().any(|e| e.key == 72));
        let _ = first;
    }

    #[test]
    fn two_audible_tracks_sum_events() {
        let (mut tl, _) = timeline_with_clip(0);
        let mut pat = Pattern::new(1_000);
        pat.add_note(Note::new(80, 40, 67, 1.0));
        let pidx = tl.add_pattern(pat);
        let cidx = tl.add_clip(Clip::Midi(MidiClip { pattern: pidx }));
        let second = tl.add_track();
        tl.track_mut(second).unwrap().place_clip(cidx, 0);

        let mut out = Vec::new();
        tl.emit_notes(0, 256, &mut out);
        assert!(out.iter().any(|e| e.key == 60));
        assert!(out.iter().any(|e| e.key == 67));
    }

    #[test]
    fn remove_clip_drops_placement() {
        let (tl, _) = timeline_with_clip(0);
        let placement = tl.track(0).unwrap().placements()[0];
        let mut tl = tl;
        assert!(tl.track_mut(0).unwrap().remove_clip(placement.clip));
        let mut out = Vec::new();
        tl.emit_notes(0, 256, &mut out);
        assert!(out.is_empty());
    }
}
