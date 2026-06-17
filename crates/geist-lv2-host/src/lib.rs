// =============================================================================
// File: crates/geist-lv2-host/src/lib.rs
// Layer: LV2 host
// Purpose: LV2 host entrypoint; presents the same AudioNode contract as CLAP
// Status: Implemented incrementally; scanner first. Manifest parsing and binary
//         loading land in a later slice once the lilv-vs-raw-ABI choice is made.
// Notes: LV2 is lower priority than CLAP and must not shape core graph APIs. The
//        scanner is pure safe Rust (bundle discovery only), so this crate has no
//        FFI yet; unsafe_op_in_unsafe_fn is pre-armed for when the world/instance/
//        node layers arrive.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Force explicit unsafe blocks even inside unsafe fns once the FFI layer lands
#![warn(unsafe_op_in_unsafe_fn)]

pub mod scanner;

// Stable public surface for the LV2 host
pub mod prelude {
    pub use crate::scanner::{discover_bundles, scan_standard, standard_lv2_paths};
}
