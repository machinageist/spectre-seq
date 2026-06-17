// =============================================================================
// File: plugins/geist-fx/src/delay/engine.rs
// Layer: effects plugin
// Purpose: Pure-DSP delay engine for the delay effect
// Status: Implemented; re-exports the geist-dsp StereoDelay (zero duplication).
// Notes: The DSP lives in geist-dsp; this layer only adapts it to the graph.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub use geist_dsp::prelude::StereoDelay;
