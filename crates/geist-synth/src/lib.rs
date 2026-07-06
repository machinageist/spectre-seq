// =============================================================================
// File: crates/geist-synth/src/lib.rs
// Layer: internal synth device
// Purpose: Flagship wavetable/subtractive synth crate root
// Status: Implemented incrementally; engine modules land first.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Pure-DSP synth engine; the AudioNode layer wraps it as an internal device
#![deny(unsafe_code)]

pub mod daw_node;
pub mod engine;

// Stable surface for the synth engine and its graph node
pub mod prelude {
    pub use crate::daw_node::SynthNode;
    pub use crate::engine::filter_stack::{FilterRouting, FilterStack};
    pub use crate::engine::mod_matrix::{ModMatrix, ModRoute, ModSource, ModTarget};
    pub use crate::engine::osc_stack::{OscStack, UnisonOsc};
    pub use crate::engine::voice::Voice;
    pub use crate::engine::voice_pool::{StealMode, VoicePool};
}
