// =============================================================================
// File: crates/spectre-graph/src/graph.rs
// Layer: audio graph
// Purpose: Graph struct: nodes, edges, port registry
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::BTreeMap;

use spectre_core::config::AudioConfig;
use spectre_core::errors::{SpectreError, SpectreResult};
use spectre_core::ids::{NodeId, PortId};
use spectre_core::port::{can_connect, PortDescriptor, PortDirection};

use crate::edge::{Edge, PortSpec};
use crate::node::AudioNode;

// Node instance plus the ports it owns
// Ports are split by direction for fast layout lookup during compilation
struct NodeEntry {
    node: Box<dyn AudioNode>,
    inputs: Vec<PortId>,
    outputs: Vec<PortId>,
}

// Mutable audio graph owned and edited only on the app thread
// Holds node instances, the port registry, and validated edges
// BTreeMap keeps iteration deterministic so compiled order is stable
#[derive(Default)]
pub struct Graph {
    nodes: BTreeMap<NodeId, NodeEntry>,
    ports: BTreeMap<PortId, PortDescriptor>,
    edges: Vec<Edge>,
    next_id: u64,
}

impl Graph {
    // Build an empty graph
    pub fn new() -> Self {
        Self::default()
    }

    // Hand out the next monotonic raw id for nodes and ports
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    // Register a node and its declared input/output ports
    // Returns the new node id; port ids are allocated internally
    pub fn add_node(
        &mut self,
        node: Box<dyn AudioNode>,
        inputs: &[PortSpec],
        outputs: &[PortSpec],
    ) -> NodeId {
        let node_id = NodeId::new(self.alloc_id());
        let input_ports = self.register_ports(node_id, PortDirection::Input, inputs);
        let output_ports = self.register_ports(node_id, PortDirection::Output, outputs);
        self.nodes.insert(
            node_id,
            NodeEntry {
                node,
                inputs: input_ports,
                outputs: output_ports,
            },
        );
        node_id
    }

    // Allocate descriptors for one direction's port specs
    fn register_ports(
        &mut self,
        node: NodeId,
        direction: PortDirection,
        specs: &[PortSpec],
    ) -> Vec<PortId> {
        let mut ids = Vec::with_capacity(specs.len());
        for spec in specs {
            let id = PortId::new(self.alloc_id());
            self.ports.insert(
                id,
                PortDescriptor {
                    id,
                    node,
                    direction,
                    port_type: spec.port_type,
                    channels: spec.channels,
                    name: spec.name,
                    param: None,
                },
            );
            ids.push(id);
        }
        ids
    }

    // Connect an output port to an input port after validating compatibility
    // Rejects unknown ports, illegal type/direction/channel pairings, and fan-in
    pub fn connect(&mut self, src: PortId, dst: PortId) -> SpectreResult<()> {
        let out = self.ports.get(&src).ok_or(SpectreError::InvalidPort(src))?;
        let inp = self.ports.get(&dst).ok_or(SpectreError::InvalidPort(dst))?;
        can_connect(out, inp)?;
        if self.edges.iter().any(|e| e.dst == dst) {
            return Err(SpectreError::Internal("input port already connected"));
        }
        self.edges.push(Edge { src, dst });
        Ok(())
    }

    // Prepare every node for a sample rate and block size before streaming
    // Runs on the app thread where node allocation is allowed
    pub fn prepare_all(&mut self, config: &AudioConfig) {
        for entry in self.nodes.values_mut() {
            entry.node.prepare(config);
        }
    }

    // Reset every node back to silence
    pub fn reset_all(&mut self) {
        for entry in self.nodes.values_mut() {
            entry.node.reset();
        }
    }

    // Remove a node and return its processor; used when building an executor
    pub fn take_node(&mut self, id: NodeId) -> Option<Box<dyn AudioNode>> {
        self.nodes.remove(&id).map(|entry| entry.node)
    }

    // Number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // Number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    // Whether a node id is present
    pub fn contains_node(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    // Borrow a port descriptor by id
    pub fn port(&self, id: PortId) -> Option<&PortDescriptor> {
        self.ports.get(&id)
    }

    // Owning node of a port
    pub fn port_node(&self, id: PortId) -> Option<NodeId> {
        self.ports.get(&id).map(|p| p.node)
    }

    // Input port ids of a node in declaration order
    pub fn input_ports(&self, node: NodeId) -> Option<&[PortId]> {
        self.nodes.get(&node).map(|e| e.inputs.as_slice())
    }

    // Output port ids of a node in declaration order
    pub fn output_ports(&self, node: NodeId) -> Option<&[PortId]> {
        self.nodes.get(&node).map(|e| e.outputs.as_slice())
    }

    // Iterate node ids in ascending order
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    // Borrow the validated edge list
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use spectre_core::context::ProcessContext;
    use spectre_core::port::PortType;

    // Inert node used to exercise topology without real DSP
    pub(crate) struct DummyNode;

    impl AudioNode for DummyNode {
        fn process(&mut self, _ctx: &mut ProcessContext) {}
    }

    // Add a node with one stereo audio input and one stereo audio output
    pub(crate) fn add_stereo_node(graph: &mut Graph) -> (NodeId, PortId, PortId) {
        let id = graph.add_node(
            Box::new(DummyNode),
            &[PortSpec::new(PortType::Audio, 2, "in")],
            &[PortSpec::new(PortType::Audio, 2, "out")],
        );
        let input = *graph.nodes[&id].inputs.first().unwrap();
        let output = *graph.nodes[&id].outputs.first().unwrap();
        (id, input, output)
    }

    #[test]
    fn add_node_allocates_unique_ids_and_ports() {
        let mut g = Graph::new();
        let (a, a_in, a_out) = add_stereo_node(&mut g);
        let (b, _b_in, _b_out) = add_stereo_node(&mut g);
        assert_ne!(a, b);
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.port(a_in).unwrap().direction, PortDirection::Input);
        assert_eq!(g.port(a_out).unwrap().direction, PortDirection::Output);
        assert_eq!(g.port_node(a_in), Some(a));
    }

    #[test]
    fn connect_accepts_matching_ports() {
        let mut g = Graph::new();
        let (_a, _a_in, a_out) = add_stereo_node(&mut g);
        let (_b, b_in, _b_out) = add_stereo_node(&mut g);
        assert!(g.connect(a_out, b_in).is_ok());
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn connect_rejects_unknown_port() {
        let mut g = Graph::new();
        let (_a, _a_in, a_out) = add_stereo_node(&mut g);
        let ghost = PortId::new(9_999);
        assert_eq!(
            g.connect(a_out, ghost),
            Err(SpectreError::InvalidPort(ghost))
        );
    }

    #[test]
    fn connect_rejects_second_source_into_one_input() {
        let mut g = Graph::new();
        let (_a, _a_in, a_out) = add_stereo_node(&mut g);
        let (_b, b_in, b_out) = add_stereo_node(&mut g);
        let (_c, _c_in, _c_out) = add_stereo_node(&mut g);
        assert!(g.connect(a_out, b_in).is_ok());
        // Second feed into the same input is rejected; use a mixer node instead
        assert!(g.connect(b_out, b_in).is_err());
    }

    #[test]
    fn connect_rejects_type_mismatch() {
        let mut g = Graph::new();
        let audio_out = g.add_node(
            Box::new(DummyNode),
            &[],
            &[PortSpec::new(PortType::Audio, 1, "out")],
        );
        let note_in = g.add_node(
            Box::new(DummyNode),
            &[PortSpec::new(PortType::Note, 1, "in")],
            &[],
        );
        let out = *g.nodes[&audio_out].outputs.first().unwrap();
        let inp = *g.nodes[&note_in].inputs.first().unwrap();
        assert!(g.connect(out, inp).is_err());
    }

    #[test]
    fn prepare_and_reset_dispatch_to_nodes() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        // Node that records each lifecycle call through a shared counter
        struct Counter {
            hits: Arc<AtomicU32>,
        }
        impl AudioNode for Counter {
            fn process(&mut self, _ctx: &mut ProcessContext) {}
            fn prepare(&mut self, _config: &AudioConfig) {
                self.hits.fetch_add(1, Ordering::Relaxed);
            }
            fn reset(&mut self) {
                self.hits.fetch_add(1, Ordering::Relaxed);
            }
        }

        let hits = Arc::new(AtomicU32::new(0));
        let mut g = Graph::new();
        g.add_node(Box::new(Counter { hits: hits.clone() }), &[], &[]);
        let cfg = AudioConfig::new(48_000, 128, 0, 2).unwrap();
        g.prepare_all(&cfg);
        g.reset_all();
        assert_eq!(hits.load(Ordering::Relaxed), 2);
    }
}
