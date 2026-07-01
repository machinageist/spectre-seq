// =============================================================================
// File: crates/geist-fx/src/eq/mod.rs
// Layer: internal effects devices
// Purpose: EQ effect: pure-DSP engine + graph node
// Status: Implemented; engine + daw_node. External plugin export is intentionally unsupported.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod daw_node;
pub mod engine;

pub use daw_node::{BandConfig, EqNode};
