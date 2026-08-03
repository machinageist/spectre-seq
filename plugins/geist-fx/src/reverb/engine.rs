// =============================================================================
// File: plugins/geist-fx/src/reverb/engine.rs
// Layer: effects plugin
// Purpose: Pure-DSP reverb engine for the reverb effect
// Status: Implemented; re-exports the spectre-dsp Reverb (zero duplication).
// Notes: The FFT convolution reverb lives in spectre-dsp; this only adapts it.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub use spectre_dsp::prelude::Reverb;
