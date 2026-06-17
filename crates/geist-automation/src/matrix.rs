// =============================================================================
// File: crates/geist-automation/src/matrix.rs
// Layer: automation
// Purpose: ModMatrix: Vec<ModRoute>
// Status: Implemented; route list summing source contributions per target.
// Notes: Source values are looked up through a caller-supplied closure so the
//        matrix stays decoupled from how CV is produced. Summation is per block.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::ids::{ParamId, PortId};

use crate::route::ModRoute;

// A set of modulation routes resolved into per-parameter modulation amounts
#[derive(Clone, Debug, Default)]
pub struct ModMatrix {
    routes: Vec<ModRoute>,
}

impl ModMatrix {
    // Build an empty matrix
    pub fn new() -> Self {
        Self::default()
    }

    // Append a route
    pub fn add_route(&mut self, route: ModRoute) {
        self.routes.push(route);
    }

    // All routes
    pub fn routes(&self) -> &[ModRoute] {
        &self.routes
    }

    // Number of routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    // Remove every route
    pub fn clear(&mut self) {
        self.routes.clear();
    }

    // Drop all routes feeding a target, returning how many were removed
    pub fn remove_target(&mut self, target: ParamId) -> usize {
        let before = self.routes.len();
        self.routes.retain(|r| r.target != target);
        before - self.routes.len()
    }

    // Whether any route feeds a target
    pub fn has_target(&self, target: ParamId) -> bool {
        self.routes.iter().any(|r| r.target == target)
    }

    // Total modulation for a target, summing each route's contribution
    // `source` resolves a CV value for a PortId
    pub fn modulation_for(&self, target: ParamId, source: impl Fn(PortId) -> f32) -> f32 {
        self.routes
            .iter()
            .filter(|r| r.target == target)
            .map(|r| r.contribution(source(r.source)))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENV: PortId = PortId::new(1);
    const LFO: PortId = PortId::new(2);
    const CUTOFF: ParamId = ParamId::new(100);
    const PITCH: ParamId = ParamId::new(101);

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    // Fixed source bank for tests
    fn sources(port: PortId) -> f32 {
        match port {
            ENV => 1.0,
            LFO => -0.5,
            _ => 0.0,
        }
    }

    #[test]
    fn empty_matrix_contributes_nothing() {
        let m = ModMatrix::new();
        assert!(close(m.modulation_for(CUTOFF, sources), 0.0));
    }

    #[test]
    fn single_route_scales_its_source() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::bipolar(ENV, CUTOFF, 0.5));
        assert!(close(m.modulation_for(CUTOFF, sources), 0.5)); // 1.0 * 0.5
    }

    #[test]
    fn routes_to_same_target_sum() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::bipolar(ENV, CUTOFF, 1.0)); // +1.0
        m.add_route(ModRoute::bipolar(LFO, CUTOFF, 1.0)); // -0.5
        assert!(close(m.modulation_for(CUTOFF, sources), 0.5));
    }

    #[test]
    fn only_matching_target_contributes() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::bipolar(ENV, CUTOFF, 1.0));
        m.add_route(ModRoute::bipolar(ENV, PITCH, 1.0));
        // PITCH only sees its own route
        assert!(close(m.modulation_for(PITCH, sources), 1.0));
        assert!(close(m.modulation_for(CUTOFF, sources), 1.0));
    }

    #[test]
    fn unipolar_route_rectifies_in_sum() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::unipolar(LFO, CUTOFF, 1.0)); // LFO=-0.5 -> 0
        assert!(close(m.modulation_for(CUTOFF, sources), 0.0));
    }

    #[test]
    fn remove_target_drops_its_routes() {
        let mut m = ModMatrix::new();
        m.add_route(ModRoute::bipolar(ENV, CUTOFF, 1.0));
        m.add_route(ModRoute::bipolar(LFO, CUTOFF, 1.0));
        m.add_route(ModRoute::bipolar(ENV, PITCH, 1.0));
        assert_eq!(m.remove_target(CUTOFF), 2);
        assert!(!m.has_target(CUTOFF));
        assert!(m.has_target(PITCH));
        assert_eq!(m.route_count(), 1);
    }
}
