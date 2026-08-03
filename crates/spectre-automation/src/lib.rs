// =============================================================================
// File: crates/spectre-automation/src/lib.rs
// Layer: automation
// Purpose: Automation lanes + modulation matrix unified into parameter values
// Status: Implemented incrementally; curve shapes first.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// App/control-rate math; no audio-thread allocation in the hot evaluation path
#![deny(unsafe_code)]

pub mod curve;
pub mod evaluator;
pub mod lane;
pub mod matrix;
pub mod route;

// Cross-module integration tests live in their own directory
#[cfg(test)]
mod tests {
    mod mod_sum_tests;
}

// Stable surface for the automation system
pub mod prelude {
    pub use crate::curve::CurveShape;
    pub use crate::evaluator::{ParamSpec, ParameterEvaluator};
    pub use crate::lane::{AutomationLane, Breakpoint};
    pub use crate::matrix::ModMatrix;
    pub use crate::route::ModRoute;
}
