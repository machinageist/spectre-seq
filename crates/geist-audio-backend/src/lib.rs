// =============================================================================
// File: crates/geist-audio-backend/src/lib.rs
// Layer: audio I/O
// Purpose: Platform audio I/O abstraction entrypoint
// Status: Implemented; backend abstraction. cpal/JACK/PipeWire impls land next.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// The abstraction layer is pure safe Rust; FFI lives in the platform impls
#![deny(unsafe_code)]

// Backend abstraction; platform implementations land in their own modules
pub mod backend;
pub mod bridge;
pub mod cpal_backend;
pub mod device;
pub mod stream;

// Stable public surface for selecting and driving an audio backend
pub mod prelude {
    pub use crate::backend::{AudioBackend, RenderCallback, Stream};
    pub use crate::bridge::{BlockBridge, BlockProcessor};
    pub use crate::cpal_backend::CpalBackend;
    pub use crate::device::DeviceInfo;
    pub use crate::stream::{
        capture_ring, CaptureConsumer, CaptureProducer, StreamConfig, XrunCounter,
    };
}
