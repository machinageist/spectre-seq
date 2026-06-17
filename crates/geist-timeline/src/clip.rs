// =============================================================================
// File: crates/geist-timeline/src/clip.rs
// Layer: timeline
// Purpose: Clip enum: AudioClip, MidiClip, AutomationClip
// Status: Implemented; three clip kinds plus automation breakpoint evaluation.
// Notes: A MidiClip references a Pattern by arena Index, keeping note data out
//        of the clip. Audio resampling (rubato) is deferred; AudioClip carries a
//        source reference and range only.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::arena::Index;

// Reference to imported audio plus the source span the clip plays
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioClip {
    // Opaque id of the source asset resolved by the project layer
    pub source: u64,
    pub source_start: u64,
    pub source_len: u64,
    pub gain: f32,
}

// MIDI clip pointing at a pattern stored in the pattern arena
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiClip {
    pub pattern: Index,
}

// One automation breakpoint: a value at a sample position relative to the clip
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Breakpoint {
    pub pos: u64,
    pub value: f32,
}

// Automation clip: a breakpoint curve evaluated by linear interpolation
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutomationClip {
    // Sorted by pos
    points: Vec<Breakpoint>,
}

impl AutomationClip {
    // Build an empty automation curve
    pub fn new() -> Self {
        Self::default()
    }

    // Insert or replace a breakpoint, keeping the curve sorted by position
    pub fn set_point(&mut self, pos: u64, value: f32) {
        match self.points.iter().position(|p| p.pos >= pos) {
            Some(i) if self.points[i].pos == pos => self.points[i].value = value,
            Some(i) => self.points.insert(i, Breakpoint { pos, value }),
            None => self.points.push(Breakpoint { pos, value }),
        }
    }

    // Breakpoints in position order
    pub fn points(&self) -> &[Breakpoint] {
        &self.points
    }

    // Value at a sample position; flat before the first and after the last point
    pub fn value_at(&self, pos: u64) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if pos <= self.points[0].pos {
            return self.points[0].value;
        }
        let last = self.points.len() - 1;
        if pos >= self.points[last].pos {
            return self.points[last].value;
        }
        // Find the segment [a, b) containing pos and interpolate
        let b = self.points.iter().position(|p| p.pos > pos).unwrap();
        let a = b - 1;
        let (pa, pb) = (self.points[a], self.points[b]);
        let span = (pb.pos - pa.pos) as f32;
        let t = (pos - pa.pos) as f32 / span;
        pa.value + (pb.value - pa.value) * t
    }
}

// A timeline clip; data lives here, placement lives on the track
#[derive(Clone, Debug, PartialEq)]
pub enum Clip {
    Audio(AudioClip),
    Midi(MidiClip),
    Automation(AutomationClip),
}

impl Clip {
    // Whether this is an audio clip
    pub fn is_audio(&self) -> bool {
        matches!(self, Clip::Audio(_))
    }

    // Whether this is a MIDI clip
    pub fn is_midi(&self) -> bool {
        matches!(self, Clip::Midi(_))
    }

    // Whether this is an automation clip
    pub fn is_automation(&self) -> bool {
        matches!(self, Clip::Automation(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_kinds_are_distinguished() {
        let mut arena = crate::arena::Arena::new();
        let pat = arena.insert(crate::pattern::Pattern::new(1_000));

        let audio = Clip::Audio(AudioClip {
            source: 7,
            source_start: 0,
            source_len: 48_000,
            gain: 1.0,
        });
        let midi = Clip::Midi(MidiClip { pattern: pat });
        let auto = Clip::Automation(AutomationClip::new());

        assert!(audio.is_audio() && !audio.is_midi());
        assert!(midi.is_midi() && !midi.is_automation());
        assert!(auto.is_automation() && !auto.is_audio());
    }

    #[test]
    fn automation_interpolates_between_breakpoints() {
        let mut clip = AutomationClip::new();
        clip.set_point(0, 0.0);
        clip.set_point(100, 1.0);
        assert!((clip.value_at(0) - 0.0).abs() < 1e-6);
        assert!((clip.value_at(50) - 0.5).abs() < 1e-6);
        assert!((clip.value_at(100) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn automation_holds_flat_outside_range() {
        let mut clip = AutomationClip::new();
        clip.set_point(100, 0.3);
        clip.set_point(200, 0.9);
        assert!((clip.value_at(0) - 0.3).abs() < 1e-6); // before first
        assert!((clip.value_at(500) - 0.9).abs() < 1e-6); // after last
    }

    #[test]
    fn automation_empty_curve_is_zero() {
        let clip = AutomationClip::new();
        assert_eq!(clip.value_at(123), 0.0);
    }

    #[test]
    fn set_point_replaces_at_same_position() {
        let mut clip = AutomationClip::new();
        clip.set_point(50, 0.2);
        clip.set_point(50, 0.7);
        assert_eq!(clip.points().len(), 1);
        assert!((clip.value_at(50) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_point_keeps_curve_sorted() {
        let mut clip = AutomationClip::new();
        clip.set_point(300, 0.3);
        clip.set_point(100, 0.1);
        clip.set_point(200, 0.2);
        let positions: Vec<u64> = clip.points().iter().map(|p| p.pos).collect();
        assert_eq!(positions, vec![100, 200, 300]);
    }
}
