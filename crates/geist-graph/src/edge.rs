// =============================================================================
// File: crates/geist-graph/src/edge.rs
// Layer: audio graph
// Purpose: Edge { src: PortId, dst: PortId }, port specifications
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::ids::PortId;
use geist_core::port::PortType;

// Declarative request for one port when adding a node
// Direction is implied by which list the spec appears in
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PortSpec {
    pub port_type: PortType,
    pub channels: u16,
    pub name: &'static str,
}

impl PortSpec {
    // Build a port specification
    pub const fn new(port_type: PortType, channels: u16, name: &'static str) -> Self {
        Self {
            port_type,
            channels,
            name,
        }
    }
}

// Directed connection from one output port to one input port
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Edge {
    pub src: PortId,
    pub dst: PortId,
}
