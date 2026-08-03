// =============================================================================
// File: crates/spectre-dsp/src/lfo/mod.rs
// Layer: DSP primitives
// Purpose: LFO and step sequencer modulation sources
// Status: Implemented; LFO + step sequencer.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// File path follows PROPOSED_FILE_TREE (lfo/lfo.rs); re-exported as lfo::Lfo
#[allow(clippy::module_inception)]
pub mod lfo;
pub mod stepseq;

pub use lfo::{Lfo, LfoWaveform};
pub use stepseq::StepSequencer;
