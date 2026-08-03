// Author: Jeff
// Date: 2026-05-27
// Description: Public entrypoint for shared Geist DAW primitives.
// Notes: Keep this crate dependency-free inside the workspace.

// Core primitives are pure and never need raw pointers
#![deny(unsafe_code)]

// Shared core primitives; Phase 0 is complete with all modules implemented
pub mod config;
pub mod context;
pub mod errors;
pub mod events;
pub mod ids;
pub mod params;
pub mod port;
pub mod signal;
pub mod time;
pub mod transport;

// Narrow prelude of stable public types for downstream crates
pub mod prelude {
    pub use crate::config::AudioConfig;
    pub use crate::context::ProcessContext;
    pub use crate::errors::{SpectreError, SpectreResult};
    pub use crate::events::{MidiEvent, NoteEvent, NoteEventKind, ParameterChange, TransportEvent};
    pub use crate::ids::{
        AssetId, AutomationTargetId, ClipId, DeviceId, MappingId, NodeId, NoteId, ParamId,
        ParamKey, PortId, ProjectId, RouteId, SceneId, TrackId,
    };
    pub use crate::params::{ParamInfo, ParamRange};
    pub use crate::port::{can_connect, PortDescriptor, PortDirection, PortType};
    pub use crate::signal::SignalRate;
    pub use crate::time::{MusicalTime, MAX_EXACT_MUSICAL_TIME_TICKS, TICKS_PER_QUARTER};
    pub use crate::transport::{AtomicTransport, TransportSnapshot, TransportState};
}
