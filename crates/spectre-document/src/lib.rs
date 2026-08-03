// =============================================================================
// Author: Jeff
// Date: 2026-08-01
// Description: Public entrypoint for the canonical app-thread project document.
// Notes: Durable project truth; the audio thread reads published snapshots, never this state.
//
// File: crates/spectre-document/src/lib.rs
// Layer: document
// Purpose: Durable identity allocation and canonical arrangement authority
// Status: Implemented incrementally; identity and arrangement relocated from timeline.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Document state is pure app-thread data and never needs raw pointers
#![deny(unsafe_code)]

pub mod arrangement;
pub mod identity;
pub mod revision;

// Stable surface for the canonical document model
pub mod prelude {
    pub use crate::arrangement::{
        Arrangement, ArrangementError, ArrangementTrack, ClipEntity, ClipLocation, RemovedClip,
    };
    pub use crate::identity::{ClipId, IdentityAllocator, TrackId};
    pub use crate::revision::{Aggregate, DocumentRevision, EffectSet};
}
