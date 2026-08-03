// =============================================================================
// File: plugins/spectre-fx/src/delay/engine.rs
// Layer: effects plugin
// Purpose: Pure-DSP delay engine for the delay effect
// Status: Implemented; re-exports the spectre-dsp StereoDelay (zero duplication).
// Notes: The DSP lives in spectre-dsp; this layer only adapts it to the graph.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub use spectre_dsp::prelude::StereoDelay;
