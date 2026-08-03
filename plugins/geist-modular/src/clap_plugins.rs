// =============================================================================
// File: plugins/geist-modular/src/clap_plugins.rs
// Layer: modular utilities
// Purpose: Register every utility node as a CLAP plugin
// Status: Deferred to Phase 4 (CLAP host). Intentionally NOT in the module tree.
// Notes: CLAP export needs unsafe FFI and the host's entry/factory machinery,
//        which land with spectre-clap-host. lib.rs denies unsafe, so this file is
//        excluded from compilation until that wrapper exists.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================
