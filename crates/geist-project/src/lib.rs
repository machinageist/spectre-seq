// =============================================================================
// File: crates/geist-project/src/lib.rs
// Layer: project persistence
// Purpose: Versioned DAW project format; CBOR project file + TOML settings
// Status: Implemented; schema, serialize, settings, asset map, migrate, autosave.
// Notes: A decoupled serde data model is the on-disk contract. Converting to and
//        from live engine types is the app layer's job, kept out of this crate
//        so the wire format stays stable across internal refactors. Disk I/O
//        only; nothing here runs on the audio thread.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Persistence is pure safe Rust; only dependencies carry unsafe
#![deny(unsafe_code)]

pub mod asset_map;
pub mod autosave;
pub mod migrate;
pub mod schema;
pub mod serialize;
pub mod settings;

// Stable surface for the project format
pub mod prelude {
    pub use crate::asset_map::{hash_bytes, AssetMap};
    pub use crate::autosave::{
        atomic_write_cbor, autosave_path, find_recovery_files, Autosaver, DEFAULT_AUTOSAVE_INTERVAL,
    };
    pub use crate::migrate::{load_and_migrate, migrate};
    pub use crate::schema::{
        AssetRef, AutomationLaneEntry, BreakpointEntry, ClipEntry, ClipKind, Connection, CurveKind,
        GraphTopology, NodeEntry, NoteEntry, ParamValue, ProjectFile, ProjectMeta, TrackEntry,
        SCHEMA_VERSION,
    };
    pub use crate::serialize::{from_cbor, load_from_path, save_to_path, to_cbor, ProjectError};
    pub use crate::settings::Settings;
}
