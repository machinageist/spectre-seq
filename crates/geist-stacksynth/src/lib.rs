// =============================================================================
// File: crates/geist-stacksynth/src/lib.rs
// Layer: internal synth device (generator-stack synth)
// Purpose: Generator-stack modular synth; S0 = patch schema + validation
// Status: S0 implemented. Spec: docs/specs/geist-modular-synth-spec.md,
//         plan: docs/specs/geist-modular-synth-plan.md.
// Notes: Data-only at this phase; the voice-graph compiler and DSP land in S1+.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod compile;
pub mod osc;
pub mod schema;
pub mod source;
pub mod validate;

pub mod prelude {
    pub use crate::compile::{compile, ModBinding, OutputBinding, RenderPlan, Step};
    pub use crate::osc::AnalogOsc;
    pub use crate::schema::*;
    pub use crate::source::{
        base_hz, instantaneous_hz, phase_increment, start_phase, wrap_phase, FreqMod,
    };
    pub use crate::validate::{validate, ValidateError, ValidateWarning, Validation};
}
