// =============================================================================
// File: crates/spectre-automation/src/lane.rs
// Layer: automation
// Purpose: AutomationLane: breakpoint curve over timeline
// Status: Implemented; sorted breakpoints with per-segment curve evaluation.
// Notes: Each breakpoint's CurveShape governs the segment to its right. The lane
//        holds flat before the first point and after the last. Positions are
//        timeline samples; one lane targets one ParamId.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::ids::ParamId;

use crate::curve::CurveShape;

// One automation point: a value at a timeline sample position
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Breakpoint {
    pub pos: u64,
    pub value: f32,
    // Shape of the segment from this point to the next
    pub curve: CurveShape,
}

// A breakpoint curve driving one parameter over the timeline
#[derive(Clone, Debug)]
pub struct AutomationLane {
    target: ParamId,
    // Sorted ascending by pos
    points: Vec<Breakpoint>,
}

impl AutomationLane {
    // Build an empty lane targeting a parameter
    pub fn new(target: ParamId) -> Self {
        Self {
            target,
            points: Vec::new(),
        }
    }

    // Parameter this lane drives
    pub fn target(&self) -> ParamId {
        self.target
    }

    // Whether the lane has no breakpoints
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    // Breakpoints in position order
    pub fn points(&self) -> &[Breakpoint] {
        &self.points
    }

    // Insert or replace the breakpoint at a position, keeping the lane sorted
    pub fn set_point(&mut self, pos: u64, value: f32, curve: CurveShape) {
        let point = Breakpoint { pos, value, curve };
        match self.points.iter().position(|p| p.pos >= pos) {
            Some(i) if self.points[i].pos == pos => self.points[i] = point,
            Some(i) => self.points.insert(i, point),
            None => self.points.push(point),
        }
    }

    // Remove the breakpoint at a position, returning whether one existed
    pub fn remove_point(&mut self, pos: u64) -> bool {
        if let Some(i) = self.points.iter().position(|p| p.pos == pos) {
            self.points.remove(i);
            true
        } else {
            false
        }
    }

    // Value at a timeline position; None when the lane has no points
    pub fn value_at(&self, pos: u64) -> Option<f32> {
        if self.points.is_empty() {
            return None;
        }
        let first = &self.points[0];
        if pos <= first.pos {
            return Some(first.value);
        }
        let last = &self.points[self.points.len() - 1];
        if pos >= last.pos {
            return Some(last.value);
        }
        // Locate the segment [a, b) and interpolate with a's curve
        let b_idx = self.points.iter().position(|p| p.pos > pos).unwrap();
        let a = &self.points[b_idx - 1];
        let b = &self.points[b_idx];
        let span = (b.pos - a.pos) as f32;
        let t = (pos - a.pos) as f32 / span;
        Some(a.curve.interpolate(a.value, b.value, t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lane() -> AutomationLane {
        let mut l = AutomationLane::new(ParamId::new(1));
        l.set_point(0, 0.0, CurveShape::Linear);
        l.set_point(100, 1.0, CurveShape::Linear);
        l
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn empty_lane_has_no_value() {
        let l = AutomationLane::new(ParamId::new(7));
        assert_eq!(l.value_at(0), None);
        assert!(l.is_empty());
        assert_eq!(l.target(), ParamId::new(7));
    }

    #[test]
    fn interpolates_linearly_between_points() {
        let l = lane();
        assert!(close(l.value_at(0).unwrap(), 0.0));
        assert!(close(l.value_at(50).unwrap(), 0.5));
        assert!(close(l.value_at(100).unwrap(), 1.0));
    }

    #[test]
    fn holds_flat_outside_the_range() {
        let mut l = AutomationLane::new(ParamId::new(1));
        l.set_point(100, 0.3, CurveShape::Linear);
        l.set_point(200, 0.9, CurveShape::Linear);
        assert!(close(l.value_at(0).unwrap(), 0.3)); // before first
        assert!(close(l.value_at(500).unwrap(), 0.9)); // after last
    }

    #[test]
    fn step_segment_holds_left_value() {
        let mut l = AutomationLane::new(ParamId::new(1));
        l.set_point(0, 0.2, CurveShape::Step);
        l.set_point(100, 0.8, CurveShape::Linear);
        assert!(close(l.value_at(0).unwrap(), 0.2));
        assert!(close(l.value_at(99).unwrap(), 0.2)); // held across the segment
        assert!(close(l.value_at(100).unwrap(), 0.8)); // jumps at the next point
    }

    #[test]
    fn per_segment_curve_is_used() {
        // The left point's curve governs its segment
        let mut l = AutomationLane::new(ParamId::new(1));
        l.set_point(0, 0.0, CurveShape::Exponential);
        l.set_point(100, 1.0, CurveShape::Linear);
        // Exponential ease-in sits below the linear midpoint
        assert!(l.value_at(50).unwrap() < 0.5);
    }

    #[test]
    fn set_point_replaces_at_same_position() {
        let mut l = lane();
        l.set_point(50, 0.25, CurveShape::Linear);
        l.set_point(50, 0.75, CurveShape::Linear); // replace
        assert_eq!(l.points().len(), 3);
        assert!(close(l.value_at(50).unwrap(), 0.75));
    }

    #[test]
    fn remove_point_drops_it() {
        let mut l = lane();
        assert!(l.remove_point(100));
        assert!(!l.remove_point(100)); // already gone
                                       // Only the origin point remains, so the lane is flat
        assert!(close(l.value_at(999).unwrap(), 0.0));
    }

    #[test]
    fn out_of_order_inserts_stay_sorted() {
        let mut l = AutomationLane::new(ParamId::new(1));
        l.set_point(300, 0.3, CurveShape::Linear);
        l.set_point(100, 0.1, CurveShape::Linear);
        l.set_point(200, 0.2, CurveShape::Linear);
        let positions: Vec<u64> = l.points().iter().map(|p| p.pos).collect();
        assert_eq!(positions, vec![100, 200, 300]);
    }
}
