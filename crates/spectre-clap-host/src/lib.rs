// =============================================================================
// File: crates/spectre-clap-host/src/lib.rs
// Layer: CLAP host
// Purpose: CLAP host entrypoint; safe API over the raw clap-sys FFI bindings.
// Status: Implemented incrementally; scanner first.
// Notes: This is an FFI boundary crate, so it does not deny unsafe. Unsafe is
//        confined to the bundle/instance/ffi layers behind narrow safe wrappers;
//        scanning is pure safe Rust.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Force explicit unsafe blocks even inside unsafe fns in the FFI layer
#![warn(unsafe_op_in_unsafe_fn)]

pub mod bundle;
pub mod db;
pub mod gui;
pub mod instance;
pub mod params;
pub mod plugin_node;
pub mod scanner;
pub mod state;

// Host-side FFI surface; internal to the crate for now
mod ffi;

// Stable public surface for the CLAP host
pub mod prelude {
    pub use crate::bundle::{BundleError, ClapBundle, ClapPluginDescriptor};
    pub use crate::db::{
        default_cache_path, load_bundle_descriptors, CacheEntry, Fingerprint, PluginCache,
    };
    pub use crate::gui::{ClapGui, ClapWindow, ResizeHints, WindowApi};
    pub use crate::instance::{ClapInstance, InstanceError};
    pub use crate::params::{ClapParams, ParamInfo};
    pub use crate::plugin_node::ClapPluginNode;
    pub use crate::scanner::{discover_bundles, scan_standard, standard_clap_paths};
    pub use crate::state::ClapState;
}
