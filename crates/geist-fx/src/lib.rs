// =============================================================================
// File: crates/geist-fx/src/lib.rs
// Layer: internal effects devices
// Purpose: Effects bundle; geist-dsp engines wrapped as graph AudioNodes
// Status: Implemented; delay, reverb, chorus, saturator, eq nodes.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Node wrappers are pure safe Rust; the DSP lives in geist-dsp
#![deny(unsafe_code)]

pub mod chorus;
pub mod delay;
pub mod eq;
pub mod io;
pub mod reverb;
pub mod saturator;

// Stable surface for the effect nodes
pub mod prelude {
    pub use crate::chorus::ChorusNode;
    pub use crate::delay::DelayNode;
    pub use crate::eq::{BandConfig, EqNode};
    pub use crate::reverb::ReverbNode;
    pub use crate::saturator::{SaturationCurve, SaturatorNode};
}
