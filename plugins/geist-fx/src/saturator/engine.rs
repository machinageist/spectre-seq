// =============================================================================
// File: plugins/geist-fx/src/saturator/engine.rs
// Layer: effects plugin
// Purpose: Pure-DSP saturator engine for the saturator effect
// Status: Implemented; re-exports the geist-dsp Saturator (zero duplication).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub use geist_dsp::prelude::{SaturationCurve, Saturator};
