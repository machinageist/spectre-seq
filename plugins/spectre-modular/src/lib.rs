// =============================================================================
// File: plugins/spectre-modular/src/lib.rs
// Layer: modular utilities
// Purpose: Utility/glue nodes; the routing math for "any signal to any input"
// Status: Implemented; math, logic, signal, timing, sample/hold node families.
// Notes: Pure safe Rust over channel-major CV buffers; per-node state only.
//        clap_plugins.rs is excluded until the Phase 4 CLAP host lands.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Glue nodes are pure safe routing math; nothing here needs unsafe
#![deny(unsafe_code)]

pub mod logic;
pub mod math;
pub mod sample_hold;
pub mod signal;
pub mod timing;

mod util;

// Stable surface for the utility nodes
pub mod prelude {
    pub use crate::logic::{AndNode, ComparatorNode, FlipFlopNode, NotNode, OrNode};
    pub use crate::math::{AbsNode, AddNode, ClipNode, MultiplyNode, RescaleNode};
    pub use crate::sample_hold::{SampleAndHoldNode, TrackAndHoldNode};
    pub use crate::signal::{AttenuverterNode, DcOffsetNode, DemuxNode, MuxNode};
    pub use crate::timing::{ClockDividerNode, GateDelayNode, SlewLimiterNode};
}
