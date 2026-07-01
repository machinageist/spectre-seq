// =============================================================================
// File: crates/geist-fx/src/reverb/mod.rs
// Layer: internal effects devices
// Purpose: Reverb effect: pure-DSP engine + graph node
// Status: Implemented; engine + daw_node. External plugin export is intentionally unsupported.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod daw_node;
pub mod engine;

pub use daw_node::ReverbNode;
