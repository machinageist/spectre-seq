// =============================================================================
// File: crates/geist-automation/src/tests/mod_sum_tests.rs
// Layer: automation
// Purpose: End-to-end checks of base + modulation summation and clamping
// Status: Implemented; exercises the unified evaluator across all layers.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::curve::CurveShape;
use crate::evaluator::{ParamSpec, ParameterEvaluator};
use crate::lane::AutomationLane;
use crate::route::ModRoute;
use geist_core::ids::{ParamId, PortId};

const CUTOFF: ParamId = ParamId::new(1);
const ENV: PortId = PortId::new(10);
const LFO: PortId = PortId::new(11);

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

// A timeline-automated base with two modulation sources summed on top, clamped
#[test]
fn automation_plus_two_mod_sources_sum_and_clamp() {
    let mut ev = ParameterEvaluator::new();
    ev.set_spec(CUTOFF, ParamSpec::new(0.0, 1.0, 0.0));

    // Automation ramps the base from 0.0 to 0.5 over the first 100 samples
    let mut lane = AutomationLane::new(CUTOFF);
    lane.set_point(0, 0.0, CurveShape::Linear);
    lane.set_point(100, 0.5, CurveShape::Linear);
    ev.set_lane(lane);

    // Two modulation routes into the same target
    ev.add_route(ModRoute::bipolar(ENV, CUTOFF, 0.5));
    ev.add_route(ModRoute::bipolar(LFO, CUTOFF, 0.25));

    let sources = |port: PortId| match port {
        ENV => 0.4, // 0.4 * 0.5 = 0.2
        LFO => 0.8, // 0.8 * 0.25 = 0.2
        _ => 0.0,
    };

    // At pos 50 the automated base is 0.25; modulation adds 0.4 -> 0.65
    assert!(close(ev.evaluate(CUTOFF, 50, sources), 0.65));
}

#[test]
fn modulation_can_be_driven_past_the_ceiling_then_clamped() {
    let mut ev = ParameterEvaluator::new();
    ev.set_spec(CUTOFF, ParamSpec::new(0.0, 1.0, 0.9));
    ev.add_route(ModRoute::bipolar(ENV, CUTOFF, 1.0));
    // 0.9 + 1.0 -> 1.9 clamped to the 1.0 ceiling
    assert!(close(ev.evaluate(CUTOFF, 0, |_| 1.0), 1.0));
    // 0.9 - 1.0 -> -0.1 clamped to the 0.0 floor
    assert!(close(ev.evaluate(CUTOFF, 0, |_| -1.0), 0.0));
}

#[test]
fn zero_modulation_leaves_the_automated_base_untouched() {
    let mut ev = ParameterEvaluator::new();
    ev.set_spec(CUTOFF, ParamSpec::new(0.0, 1.0, 0.0));
    let mut lane = AutomationLane::new(CUTOFF);
    lane.set_point(0, 0.3, CurveShape::Step);
    lane.set_point(100, 0.7, CurveShape::Linear);
    ev.set_lane(lane);
    // Step segment holds 0.3; no routes means no modulation
    assert!(close(ev.evaluate(CUTOFF, 50, |_| 0.0), 0.3));
}

#[test]
fn negative_bipolar_modulation_subtracts() {
    let mut ev = ParameterEvaluator::new();
    ev.set_spec(CUTOFF, ParamSpec::new(0.0, 2.0, 1.0));
    ev.add_route(ModRoute::bipolar(LFO, CUTOFF, 1.0));
    // base 1.0 + (-0.5 * 1.0) = 0.5
    assert!(close(ev.evaluate(CUTOFF, 0, |_| -0.5), 0.5));
}
