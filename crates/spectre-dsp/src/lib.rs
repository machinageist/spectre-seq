// =============================================================================
// File: crates/spectre-dsp/src/lib.rs
// Layer: DSP primitives
// Purpose: Pure DSP crate root; math, oscillators, filters, envelopes, effects
// Status: Implemented incrementally; math primitives land first.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Pure DSP: no FFI, no I/O, no allocation in the hot path; operates on f32 slices
#![deny(unsafe_code)]

// Shared math used by every other module; further modules land as implemented
pub mod env;
pub mod filter;
pub mod fx;
pub mod lfo;
pub mod math;
pub mod osc;

// Crate-internal shared PRNG for noise and modulation sources
mod rng;

// Stable surface for the most-used primitives
pub mod prelude {
    pub use crate::env::adsr::{Adsr, AdsrStage};
    pub use crate::env::ahdsr::{Ahdsr, AhdsrStage};
    pub use crate::env::follower::{EnvelopeFollower, FollowerMode};
    pub use crate::filter::biquad::Biquad;
    pub use crate::filter::comb::Comb;
    pub use crate::filter::ladder::Ladder;
    pub use crate::filter::svf::{Svf, SvfMode};
    pub use crate::fx::chorus::Chorus;
    pub use crate::fx::delay::StereoDelay;
    pub use crate::fx::eq::ParametricEq;
    pub use crate::fx::reverb::{Convolver, Reverb};
    pub use crate::fx::saturator::{SaturationCurve, Saturator};
    pub use crate::lfo::{Lfo, LfoWaveform, StepSequencer};
    pub use crate::math::{
        cents_to_ratio, db_to_linear, fast_tanh, lerp, linear_to_db, midi_to_hz, poly_blamp,
        poly_blep,
    };
    pub use crate::osc::noise::{Crackle, PinkNoise, WhiteNoise};
    pub use crate::osc::polyblep::{PolyBlepOsc, Waveform};
    pub use crate::osc::sine::SineOsc;
    pub use crate::osc::wavetable::{Wavetable, WavetableOsc};
    pub use crate::osc::Phasor;
}
