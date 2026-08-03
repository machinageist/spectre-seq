// =============================================================================
// File: crates/spectre-dsp/src/env/mod.rs
// Layer: DSP primitives
// Purpose: Envelope generators and followers; shared segment-curve math
// Status: Implemented; ADSR + AHDSR + envelope follower.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod adsr;
pub mod ahdsr;
pub mod follower;

// Curvature of each segment; the attack is gentler than the analog decay/release
pub(crate) const ATTACK_CURVE_EXP: f32 = -1.5;
pub(crate) const ANALOG_CURVE_EXP: f32 = -4.95;

// Shortest stage, in samples, so coefficients stay finite
pub(crate) const MIN_STAGE_SAMPLES: f32 = 1.0;

// Multiplicative coefficient for an exponential stage of `time_samples` with curvature `tco`
// Paired with a base term, `value = base + value * coef` lands on the target in the set time
pub(crate) fn stage_coef(time_samples: f32, tco: f32) -> f32 {
    let n = time_samples.max(MIN_STAGE_SAMPLES);
    (-((1.0 + tco) / tco).ln() / n).exp()
}
