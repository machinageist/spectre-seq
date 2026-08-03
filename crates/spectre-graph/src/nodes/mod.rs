// =============================================================================
// File: crates/spectre-graph/src/nodes/mod.rs
// Layer: audio graph
// Purpose: Built-in graph nodes
// Status: Implemented; passthrough, mixer, delay. Monitor lands with metering channel.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub mod delay_node;
pub mod mixer;
pub mod monitor;
pub mod passthrough;

pub use delay_node::DelayNode;
pub use mixer::MixerNode;
pub use monitor::{MeterCell, MonitorNode};
pub use passthrough::PassthroughNode;
