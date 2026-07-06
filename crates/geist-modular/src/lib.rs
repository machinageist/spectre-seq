// =============================================================================
// File: crates/geist-modular/src/lib.rs
// Layer: modular utilities
// Purpose: Utility/glue nodes plus rack generator/processor adapters
// Status: Implemented; math, logic, signal, timing, sample/hold, rack nodes.
// Notes: Pure safe Rust over channel-major CV buffers; per-node state only.
//        Rack nodes adapt geist-dsp primitives; no DSP is reimplemented here.
//        External plugin export is intentionally unsupported.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Glue nodes are pure safe routing math; nothing here needs unsafe
#![deny(unsafe_code)]

pub mod bridge;
pub mod catalog;
pub mod logic;
pub mod math;
pub mod rack_nodes;
pub mod sample_hold;
pub mod signal;
pub mod standards;
pub mod timing;

mod util;

// Stable surface for the utility nodes
pub mod prelude {
    pub use crate::bridge::{cv_to_hz, MidiCvNode, RackOutNode, TransportClockNode};
    pub use crate::catalog::{rack_catalog, rack_node, RackNodeSpec};
    pub use crate::logic::{AndNode, ComparatorNode, FlipFlopNode, NotNode, OrNode};
    pub use crate::math::{AbsNode, AddNode, ClipNode, MultiplyNode, RescaleNode};
    pub use crate::rack_nodes::{EnvNode, LfoRackNode, VcaNode, VcfNode, VcoNode};
    pub use crate::sample_hold::{SampleAndHoldNode, TrackAndHoldNode};
    pub use crate::signal::{AttenuverterNode, DcOffsetNode, DemuxNode, MuxNode};
    pub use crate::standards::{
        flush_non_finite, flush_non_finite_slice, hz_to_volts, volts_to_hz, Pulse, Schmitt,
        AUDIO_ZERO_V_HZ, GATE_V, LFO_ZERO_V_HZ, POLY_MAX, SCHMITT_HIGH_V, SCHMITT_LOW_V,
        TRIGGER_SECONDS,
    };
    pub use crate::timing::{ClockDividerNode, GateDelayNode, SlewLimiterNode};
    pub use crate::util::{get_poly_voltage, mono_audio_sum, mono_cv_first};
}
