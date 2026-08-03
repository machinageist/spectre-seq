// =============================================================================
// File: crates/spectre-vst-host/src/lib.rs
// Layer: plugin host
// Purpose: VST3 host entrypoint; safe API over the raw vst3 COM bindings
// Status: Implemented incrementally; scanner first.
// Notes: This is the project's FFI boundary crate, so it does not deny unsafe.
//        Unsafe is confined to the ffi/instance layer behind narrow safe
//        wrappers; scanning and bundle resolution are pure safe Rust.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Force explicit unsafe blocks even inside unsafe fns in the FFI layer
#![warn(unsafe_op_in_unsafe_fn)]

pub mod bundle;
pub mod descriptor;
pub mod host_app;
pub mod instance;
pub mod module;
pub mod plugin_node;
pub mod scanner;

// Stable public surface for the VST3 host
pub mod prelude {
    pub use crate::bundle::Vst3Bundle;
    pub use crate::descriptor::{Vst3ClassInfo, Vst3FactoryInfo};
    pub use crate::host_app::HostApplication;
    pub use crate::instance::{InstanceError, VstInstance};
    pub use crate::module::{ModuleError, Vst3Module};
    pub use crate::plugin_node::VstPluginNode;
    pub use crate::scanner::{discover_bundles, scan_standard, standard_vst3_paths};
}
