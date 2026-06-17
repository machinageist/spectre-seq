// =============================================================================
// File: plugins/geist-fx/src/saturator/mod.rs
// Layer: effects plugin
// Purpose: Saturator effect: pure-DSP engine + graph node
// Status: Implemented; engine + daw_node. CLAP wrapper lands with the host.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod daw_node;
pub mod engine;

pub use daw_node::SaturatorNode;
pub use engine::SaturationCurve;
