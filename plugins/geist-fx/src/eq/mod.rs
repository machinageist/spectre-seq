// =============================================================================
// File: plugins/geist-fx/src/eq/mod.rs
// Layer: effects plugin
// Purpose: EQ effect: pure-DSP engine + graph node
// Status: Implemented; engine + daw_node. CLAP wrapper lands with the host.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod daw_node;
pub mod engine;

pub use daw_node::{BandConfig, EqNode};
