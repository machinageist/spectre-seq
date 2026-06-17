// =============================================================================
// File: crates/geist-clap-host/src/ffi/mod.rs
// Layer: CLAP host
// Purpose: Host-side FFI surface the host exposes to plugins
// Status: Implemented incrementally; minimal stub host first.
// Notes: Groups the clap_host vtable implementation. Host extensions (log,
//        params rescan, thread-check, etc.) land with the params/host slice.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod events;
pub mod host_impl;
