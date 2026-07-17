// Author: Jeff
// Date: 2026-07-11
// Description: geist-core public surface: IDs, time, tempo, transport, events, parameters
// Notes: Requirements CORE-001..002, TIME-001..005; every type carries its ledger ID in docs

pub mod event;
pub mod id;
pub mod meter;
pub mod param;
pub mod tempo;
pub mod time;
pub mod transport;

pub use event::{sort_events, Event, EventKind, TimedEvent};
pub use id::{IdGen, ObjectId};
pub use meter::{MeterChange, MeterError, MeterMap, TimeSignature};
pub use param::{ParamDescriptor, ParamError, ParamSpec, ParamUnit, ParamValue};
pub use tempo::{TempoMap, TempoSegment};
pub use time::{BeatTicks, SampleDuration, SampleRate, SampleTime, Seconds, TICKS_PER_BEAT};
pub use transport::{LoopRegion, Transport, TransportCommand, TransportState};
