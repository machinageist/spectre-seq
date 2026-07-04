// =============================================================================
// File: crates/geist-synth/src/engine/mod.rs
// Layer: internal synth device
// Purpose: Pure-DSP synth engine; oscillators, filters, voices, modulation
// Status: Implemented incrementally; osc/filter, voice, mod matrix, voice pool.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod filter_stack;
pub mod mod_matrix;
pub mod osc_stack;
pub mod params;
pub mod voice;
pub mod voice_pool;
