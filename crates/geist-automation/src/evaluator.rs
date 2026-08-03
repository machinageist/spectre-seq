// =============================================================================
// File: crates/geist-automation/src/evaluator.rs
// Layer: automation
// Purpose: per-block curve evaluation + mod sum resolution
// Status: Implemented; unifies automation lanes, modulation, and clamping.
// Notes: Final value = clamp(base + sum(modulation), spec.min, spec.max), where
//        base is the automation lane value at the timeline position or the
//        parameter default. Lookups are by ParamId; evaluation never allocates.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::HashMap;

use spectre_core::ids::{ParamId, PortId};

use crate::lane::AutomationLane;
use crate::matrix::ModMatrix;
use crate::route::ModRoute;

// Declared range and default for one automatable parameter
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamSpec {
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

impl ParamSpec {
    // Build a spec, pinning the default inside the range
    pub fn new(min: f32, max: f32, default: f32) -> Self {
        Self {
            min,
            max,
            default: default.clamp(min, max),
        }
    }

    // Constrain a value to the declared range
    #[inline]
    pub fn clamp(&self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }
}

// Resolves final parameter values from automation plus modulation
#[derive(Clone, Debug, Default)]
pub struct ParameterEvaluator {
    specs: HashMap<ParamId, ParamSpec>,
    lanes: HashMap<ParamId, AutomationLane>,
    matrix: ModMatrix,
}

impl ParameterEvaluator {
    // Build an evaluator with no parameters registered
    pub fn new() -> Self {
        Self::default()
    }

    // Register or replace a parameter's range and default
    pub fn set_spec(&mut self, param: ParamId, spec: ParamSpec) {
        self.specs.insert(param, spec);
    }

    // Look up a parameter's spec
    pub fn spec(&self, param: ParamId) -> Option<&ParamSpec> {
        self.specs.get(&param)
    }

    // Attach an automation lane, keyed by the lane's target parameter
    pub fn set_lane(&mut self, lane: AutomationLane) {
        self.lanes.insert(lane.target(), lane);
    }

    // Borrow a parameter's automation lane mutably for editing
    pub fn lane_mut(&mut self, param: ParamId) -> Option<&mut AutomationLane> {
        self.lanes.get_mut(&param)
    }

    // Borrow the modulation matrix
    pub fn matrix(&self) -> &ModMatrix {
        &self.matrix
    }

    // Borrow the modulation matrix mutably
    pub fn matrix_mut(&mut self) -> &mut ModMatrix {
        &mut self.matrix
    }

    // Convenience: add one modulation route
    pub fn add_route(&mut self, route: ModRoute) {
        self.matrix.add_route(route);
    }

    // Base value before modulation: the lane value, else the parameter default
    pub fn base_value(&self, param: ParamId, pos: u64) -> f32 {
        if let Some(value) = self.lanes.get(&param).and_then(|l| l.value_at(pos)) {
            return value;
        }
        self.specs.get(&param).map(|s| s.default).unwrap_or(0.0)
    }

    // Final value: clamp(base + modulation) at a timeline position
    // `source` resolves a CV value for each route's source port
    pub fn evaluate(&self, param: ParamId, pos: u64, source: impl Fn(PortId) -> f32) -> f32 {
        let base = self.base_value(param, pos);
        let modulation = self.matrix.modulation_for(param, source);
        let raw = base + modulation;
        match self.specs.get(&param) {
            Some(spec) => spec.clamp(raw),
            None => raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::CurveShape;

    const CUTOFF: ParamId = ParamId::new(1);
    const ENV: PortId = PortId::new(10);

    fn no_sources(_: PortId) -> f32 {
        0.0
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn param_spec_pins_default_in_range() {
        let spec = ParamSpec::new(0.0, 1.0, 5.0);
        assert!(close(spec.default, 1.0));
        assert!(close(spec.clamp(-3.0), 0.0));
        assert!(close(spec.clamp(2.0), 1.0));
    }

    #[test]
    fn base_falls_back_to_default_without_a_lane() {
        let mut ev = ParameterEvaluator::new();
        ev.set_spec(CUTOFF, ParamSpec::new(0.0, 1.0, 0.4));
        assert!(close(ev.base_value(CUTOFF, 0), 0.4));
        assert!(close(ev.evaluate(CUTOFF, 0, no_sources), 0.4));
    }

    #[test]
    fn automation_lane_drives_the_base() {
        let mut ev = ParameterEvaluator::new();
        ev.set_spec(CUTOFF, ParamSpec::new(0.0, 1.0, 0.0));
        let mut lane = AutomationLane::new(CUTOFF);
        lane.set_point(0, 0.0, CurveShape::Linear);
        lane.set_point(100, 1.0, CurveShape::Linear);
        ev.set_lane(lane);
        assert!(close(ev.evaluate(CUTOFF, 50, no_sources), 0.5));
    }

    #[test]
    fn modulation_adds_to_base_and_clamps() {
        let mut ev = ParameterEvaluator::new();
        ev.set_spec(CUTOFF, ParamSpec::new(0.0, 1.0, 0.5));
        ev.add_route(ModRoute::bipolar(ENV, CUTOFF, 1.0));
        // base 0.5 + 1.0 * 0.8 = 1.3 -> clamped to 1.0
        assert!(close(ev.evaluate(CUTOFF, 0, |_| 0.8), 1.0));
        // base 0.5 + 1.0 * -0.2 = 0.3
        assert!(close(ev.evaluate(CUTOFF, 0, |_| -0.2), 0.3));
    }

    #[test]
    fn unregistered_param_is_unclamped_zero_base() {
        let ev = ParameterEvaluator::new();
        assert!(close(ev.evaluate(ParamId::new(99), 0, |_| 0.0), 0.0));
    }
}
