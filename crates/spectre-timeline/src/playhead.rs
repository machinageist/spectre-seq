// =============================================================================
// File: crates/spectre-timeline/src/playhead.rs
// Layer: timeline
// Purpose: sample-accurate playhead position
// Status: Implemented; sample position with loop-aware advance.
// Notes: Position is an absolute sample count. Advancing folds the result into
//        the active loop region so the playhead never escapes it.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// A loop region over absolute sample positions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopRegion {
    pub enabled: bool,
    pub start: u64,
    pub end: u64,
}

impl LoopRegion {
    // A region that performs no wrapping
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            start: 0,
            end: 0,
        }
    }

    // An enabled region; start/end are ordered defensively
    pub fn new(start: u64, end: u64) -> Self {
        Self {
            enabled: true,
            start: start.min(end),
            end: start.max(end),
        }
    }

    // Whether the region actually wraps (enabled and non-empty)
    #[inline]
    pub fn is_active(&self) -> bool {
        self.enabled && self.end > self.start
    }

    // Fold a position into the region, leaving it alone when inactive
    #[inline]
    pub fn wrap(&self, pos: u64) -> u64 {
        if self.is_active() && pos >= self.end {
            let len = self.end - self.start;
            self.start + (pos - self.start) % len
        } else {
            pos
        }
    }
}

impl Default for LoopRegion {
    fn default() -> Self {
        Self::disabled()
    }
}

// Sample-accurate playhead position
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Playhead {
    sample_pos: u64,
}

impl Playhead {
    // Build a playhead at the origin
    pub fn new() -> Self {
        Self::default()
    }

    // Current absolute sample position
    #[inline]
    pub fn position(&self) -> u64 {
        self.sample_pos
    }

    // Jump to an absolute sample position
    pub fn seek(&mut self, pos: u64) {
        self.sample_pos = pos;
    }

    // Return to the origin
    pub fn reset(&mut self) {
        self.sample_pos = 0;
    }

    // Advance by `frames`, folding into the loop region if it is active
    pub fn advance(&mut self, frames: u64, loop_region: &LoopRegion) {
        self.sample_pos = loop_region.wrap(self.sample_pos + frames);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_without_loop_accumulates() {
        let mut head = Playhead::new();
        let region = LoopRegion::disabled();
        head.advance(512, &region);
        head.advance(512, &region);
        assert_eq!(head.position(), 1024);
    }

    #[test]
    fn advance_wraps_inside_active_loop() {
        let mut head = Playhead::new();
        head.seek(900);
        let region = LoopRegion::new(800, 1000); // length 200
        head.advance(150, &region); // 1050 -> wrap to 850
        assert_eq!(head.position(), 850);
    }

    #[test]
    fn loop_region_orders_bounds() {
        let region = LoopRegion::new(1000, 200);
        assert_eq!(region.start, 200);
        assert_eq!(region.end, 1000);
    }

    #[test]
    fn empty_loop_does_not_wrap() {
        let region = LoopRegion::new(500, 500);
        assert!(!region.is_active());
        assert_eq!(region.wrap(9_999), 9_999);
    }

    #[test]
    fn seek_and_reset() {
        let mut head = Playhead::new();
        head.seek(4242);
        assert_eq!(head.position(), 4242);
        head.reset();
        assert_eq!(head.position(), 0);
    }
}
