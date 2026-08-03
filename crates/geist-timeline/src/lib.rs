// =============================================================================
// File: crates/geist-timeline/src/lib.rs
// Layer: timeline
// Purpose: Arrangement, clips, transport, and undo/redo timeline model
// Status: Implemented incrementally; arena foundation first.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// App-thread model; the audio thread reads published snapshots, never this state
#![deny(unsafe_code)]

pub mod arena;
pub mod clip;
pub mod commands;
pub mod pattern;
pub mod playhead;
pub mod tempo;
pub mod track;
pub mod transport;

// Stable surface for the timeline model
pub mod prelude {
    pub use crate::arena::{Arena, Index};
    pub use crate::clip::{AudioClip, AutomationClip, Breakpoint, Clip, MidiClip};
    pub use crate::commands::{
        AddTrack, Command, MoveClip, PlaceClip, RemoveClip, SetTrackMute, UndoStack,
    };
    pub use crate::pattern::{Note, Pattern};
    pub use crate::playhead::{LoopRegion, Playhead};
    pub use crate::tempo::TempoMap;
    pub use crate::track::{ClipPlacement, Timeline, Track};
    pub use crate::transport::Transport;

    // Compatibility window; musical time and canonical arrangement moved to core and document
    pub use geist_core::time::{MusicalTime, MAX_EXACT_MUSICAL_TIME_TICKS, TICKS_PER_QUARTER};
    pub use geist_document::arrangement::{
        Arrangement, ArrangementError, ArrangementTrack, ClipEntity, ClipLocation, RemovedClip,
    };
    pub use geist_document::identity::{ClipId, IdentityAllocator, TrackId};
}
