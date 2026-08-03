// =============================================================================
// File: crates/spectre-graph/src/lib.rs
// Layer: audio graph
// Purpose: Public entrypoint for the audio process graph
// Status: Implemented; graph model, topology, compilation, executor, swap, channels.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Graph editing is pure safe Rust; no FFI lives in this crate
#![deny(unsafe_code)]

// Graph model, topology, compilation, nodes, channels, and lock-free swap
pub mod channel;
pub mod edge;
pub mod graph;
pub mod node;
pub mod nodes;
pub mod process_list;
pub mod swap;
pub mod topology;

// Stable public surface for graph construction and compilation
pub mod prelude {
    pub use crate::channel::{param_channel, ParamConsumer, ParamProducer};
    pub use crate::edge::{Edge, PortSpec};
    pub use crate::graph::Graph;
    pub use crate::node::AudioNode;
    pub use crate::nodes::{DelayNode, MeterCell, MixerNode, MonitorNode, PassthroughNode};
    pub use crate::process_list::{compile, ChannelSource, Executor, NodeStep, ProcessPlan};
    pub use crate::swap::{graph_swap, ActiveGraph, GraphPublisher};
    pub use crate::topology::{schedule, topological_order, Schedule};
}
