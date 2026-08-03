// =============================================================================
// File: crates/spectre-graph/src/process_list.rs
// Layer: audio graph
// Purpose: routing-plan compilation and the realtime block executor
// Status: Implemented; compilation, feedback routing, and the zero-alloc executor.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::BTreeMap;

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use spectre_core::errors::{SpectreError, SpectreResult};
use spectre_core::events::{NoteEvent, ParameterChange};
use spectre_core::ids::{NodeId, PortId};
use spectre_core::transport::TransportSnapshot;

use crate::graph::Graph;
use crate::node::AudioNode;
use crate::topology::schedule;

// Where one input channel of a node reads its samples
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ChannelSource {
    // Read from a pool buffer produced earlier this block
    Buffer(usize),
    // Read from the shared silent buffer; input is unconnected
    Silent,
}

// One node's place in compiled order with resolved buffer routing
// Channels are flattened across the node's ports in declaration order
#[derive(Clone, Debug)]
pub struct NodeStep {
    pub node: NodeId,
    pub inputs: Vec<ChannelSource>,
    pub outputs: Vec<usize>,
}

// Immutable routing plan compiled on the app thread
// Pure data: pairs with node instances and a preallocated buffer pool to run
#[derive(Clone, Debug)]
pub struct ProcessPlan {
    pub steps: Vec<NodeStep>,
    // Count of pool channel buffers the executor must preallocate
    pub buffer_count: usize,
    pub frames: usize,
}

// Compile a graph into a buffer-routing plan in scheduled order
// Feedback edges become a one-block delay: the consumer is ordered before its
// producer and reads the producer's buffer from the previous block
pub fn compile(graph: &Graph, config: &AudioConfig) -> SpectreResult<ProcessPlan> {
    let order = schedule(graph).order;

    // Assign a contiguous buffer range to each output port's channels
    let mut output_base: BTreeMap<PortId, usize> = BTreeMap::new();
    let mut buffer_count = 0;
    for &node in &order {
        for &port in graph.output_ports(node).unwrap_or(&[]) {
            let channels = graph.port(port).expect("output port exists").channels as usize;
            output_base.insert(port, buffer_count);
            buffer_count += channels;
        }
    }

    // Resolve each input port to the output port feeding it
    let mut feed: BTreeMap<PortId, PortId> = BTreeMap::new();
    for edge in graph.edges() {
        feed.insert(edge.dst, edge.src);
    }

    let mut steps = Vec::with_capacity(order.len());
    for &node in &order {
        let inputs = resolve_inputs(graph, node, &feed, &output_base);
        let outputs = resolve_outputs(graph, node, &output_base);
        steps.push(NodeStep {
            node,
            inputs,
            outputs,
        });
    }

    Ok(ProcessPlan {
        steps,
        buffer_count,
        frames: config.frames_per_block(),
    })
}

// Map a node's input channels to buffer sources, channel by channel
fn resolve_inputs(
    graph: &Graph,
    node: NodeId,
    feed: &BTreeMap<PortId, PortId>,
    output_base: &BTreeMap<PortId, usize>,
) -> Vec<ChannelSource> {
    let mut inputs = Vec::new();
    for &port in graph.input_ports(node).unwrap_or(&[]) {
        let channels = graph.port(port).expect("input port exists").channels as usize;
        match feed.get(&port) {
            Some(source) => {
                let base = output_base[source];
                for channel in 0..channels {
                    inputs.push(ChannelSource::Buffer(base + channel));
                }
            }
            None => {
                for _ in 0..channels {
                    inputs.push(ChannelSource::Silent);
                }
            }
        }
    }
    inputs
}

// Map a node's output channels to their assigned pool buffers
fn resolve_outputs(
    graph: &Graph,
    node: NodeId,
    output_base: &BTreeMap<PortId, usize>,
) -> Vec<usize> {
    let mut outputs = Vec::new();
    for &port in graph.output_ports(node).unwrap_or(&[]) {
        let channels = graph.port(port).expect("output port exists").channels as usize;
        let base = output_base[&port];
        for channel in 0..channels {
            outputs.push(base + channel);
        }
    }
    outputs
}

// Runs a compiled plan against a preallocated buffer pool on the audio thread
// All storage is allocated on construction; process_block never allocates or locks
pub struct Executor {
    steps: Vec<NodeStep>,
    nodes: Vec<Box<dyn AudioNode>>,
    // Flat channel-major pool, one frames-sized region per compiled buffer
    pool: Vec<f32>,
    // Reusable gather area for a node's input channels
    input_scratch: Vec<f32>,
    frames: usize,
    sample_rate_hz: u32,
}

impl Executor {
    // Build an executor by taking prepared nodes out of the graph in plan order
    pub fn new(mut graph: Graph, plan: ProcessPlan, config: &AudioConfig) -> SpectreResult<Self> {
        graph.prepare_all(config);
        let mut nodes = Vec::with_capacity(plan.steps.len());
        for step in &plan.steps {
            let node = graph
                .take_node(step.node)
                .ok_or(SpectreError::Internal("plan node missing"))?;
            nodes.push(node);
        }
        let frames = plan.frames;
        let max_inputs = plan.steps.iter().map(|s| s.inputs.len()).max().unwrap_or(0);
        Ok(Self {
            pool: vec![0.0; plan.buffer_count * frames],
            input_scratch: vec![0.0; max_inputs * frames],
            steps: plan.steps,
            nodes,
            frames,
            sample_rate_hz: config.sample_rate_hz,
        })
    }

    // Process exactly one full block; events are global for now and per-node later
    pub fn process_block(
        &mut self,
        transport: TransportSnapshot,
        notes: &[NoteEvent],
        params: &[ParameterChange],
    ) {
        let stride = self.frames;
        let sample_rate_hz = self.sample_rate_hz;
        // Empty backing slice for nodes that produce no output
        let mut sink: [f32; 0] = [];
        for (step, node) in self.steps.iter().zip(self.nodes.iter_mut()) {
            // Gather this node's inputs contiguously into the scratch area
            for (channel, source) in step.inputs.iter().enumerate() {
                let start = channel * stride;
                let dst = &mut self.input_scratch[start..start + stride];
                match *source {
                    ChannelSource::Buffer(buffer) => {
                        let base = buffer * stride;
                        dst.copy_from_slice(&self.pool[base..base + stride]);
                    }
                    ChannelSource::Silent => dst.fill(0.0),
                }
            }

            let input = &self.input_scratch[..step.inputs.len() * stride];
            // A node's output buffers are a contiguous ascending run in the pool
            let output: &mut [f32] = if step.outputs.is_empty() {
                &mut sink
            } else {
                let base = step.outputs[0] * stride;
                &mut self.pool[base..base + step.outputs.len() * stride]
            };

            let mut ctx = ProcessContext::new(
                stride,
                sample_rate_hz,
                input,
                output,
                notes,
                params,
                transport,
            );
            node.process(&mut ctx);
        }
    }

    // Borrow one compiled buffer after a block; used by sinks and tests
    pub fn buffer(&self, index: usize) -> &[f32] {
        let base = index * self.frames;
        &self.pool[base..base + self.frames]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::PortSpec;
    use crate::graph::tests::{add_stereo_node, DummyNode};
    use spectre_core::port::PortType;

    fn config() -> AudioConfig {
        AudioConfig::new(48_000, 128, 2, 2).unwrap()
    }

    #[test]
    fn chain_routes_output_buffer_into_next_input() {
        let mut g = Graph::new();
        let (_a, _a_in, a_out) = add_stereo_node(&mut g);
        let (_b, b_in, _b_out) = add_stereo_node(&mut g);
        g.connect(a_out, b_in).unwrap();
        let plan = compile(&g, &config()).unwrap();

        // a runs first, claims buffers 0,1; b reads exactly those for its input
        assert_eq!(plan.steps[0].outputs, vec![0, 1]);
        assert_eq!(
            plan.steps[1].inputs,
            vec![ChannelSource::Buffer(0), ChannelSource::Buffer(1)]
        );
        assert_eq!(plan.buffer_count, 4);
        assert_eq!(plan.frames, 128);
    }

    #[test]
    fn unconnected_input_reads_silence() {
        let mut g = Graph::new();
        add_stereo_node(&mut g);
        let plan = compile(&g, &config()).unwrap();
        assert_eq!(
            plan.steps[0].inputs,
            vec![ChannelSource::Silent, ChannelSource::Silent]
        );
    }

    #[test]
    fn buffer_count_sums_output_channels() {
        let mut g = Graph::new();
        // Mono source plus a stereo node
        g.add_node(
            Box::new(DummyNode),
            &[],
            &[PortSpec::new(PortType::Audio, 1, "out")],
        );
        add_stereo_node(&mut g);
        let plan = compile(&g, &config()).unwrap();
        assert_eq!(plan.buffer_count, 1 + 2);
    }

    #[test]
    fn feedback_cycle_compiles_with_one_block_delay() {
        let mut g = Graph::new();
        let (_a, a_in, a_out) = add_stereo_node(&mut g);
        let (_b, b_in, b_out) = add_stereo_node(&mut g);
        g.connect(a_out, b_in).unwrap();
        g.connect(b_out, a_in).unwrap();
        // The cycle no longer fails; the back-edge is scheduled as a one-block delay
        let plan = compile(&g, &config()).unwrap();
        assert_eq!(plan.steps.len(), 2);
    }

    // Test node that adds one to each input sample
    struct AddOneNode;
    impl crate::node::AudioNode for AddOneNode {
        fn process(&mut self, ctx: &mut spectre_core::context::ProcessContext) {
            let frames = ctx.frames();
            let (input, output) = ctx.io();
            for (channel, out) in output.chunks_mut(frames).enumerate() {
                let start = channel * frames;
                for (sample, &fed) in out.iter_mut().zip(&input[start..start + frames]) {
                    *sample = fed + 1.0;
                }
            }
        }
    }

    #[test]
    fn self_feedback_accumulates_one_block_at_a_time() {
        use spectre_core::transport::TransportSnapshot;

        // A node that feeds its own input becomes an accumulator with one-block delay
        let mut g = Graph::new();
        let node = g.add_node(
            Box::new(AddOneNode),
            &[PortSpec::new(PortType::Audio, 1, "in")],
            &[PortSpec::new(PortType::Audio, 1, "out")],
        );
        let out = g.output_ports(node).unwrap()[0];
        let input = g.input_ports(node).unwrap()[0];
        g.connect(out, input).unwrap();

        let cfg = AudioConfig::new(48_000, 2, 0, 1).unwrap();
        let plan = compile(&g, &cfg).unwrap();
        let buffer = plan.steps.iter().find(|s| s.node == node).unwrap().outputs[0];
        let mut exec = Executor::new(g, plan, &cfg).unwrap();

        for expected in [1.0_f32, 2.0, 3.0] {
            exec.process_block(TransportSnapshot::stopped(48_000), &[], &[]);
            assert!(exec
                .buffer(buffer)
                .iter()
                .all(|&s| (s - expected).abs() < 1e-6));
        }
    }

    // Test source that ignores input and writes a constant to every output channel
    struct ConstNode {
        value: f32,
    }
    impl crate::node::AudioNode for ConstNode {
        fn process(&mut self, ctx: &mut spectre_core::context::ProcessContext) {
            let (_input, output) = ctx.io();
            output.fill(self.value);
        }
    }

    #[test]
    fn executor_runs_a_compiled_chain_end_to_end() {
        use crate::nodes::PassthroughNode;
        use spectre_core::transport::TransportSnapshot;

        // Constant source feeds a passthrough; the passthrough output must carry the value
        let mut g = Graph::new();
        let src = g.add_node(
            Box::new(ConstNode { value: 0.5 }),
            &[],
            &[PortSpec::new(PortType::Audio, 1, "out")],
        );
        let pass = g.add_node(
            Box::new(PassthroughNode::new()),
            &[PortSpec::new(PortType::Audio, 1, "in")],
            &[PortSpec::new(PortType::Audio, 1, "out")],
        );
        let src_out = g.output_ports(src).unwrap()[0];
        let pass_in = g.input_ports(pass).unwrap()[0];
        g.connect(src_out, pass_in).unwrap();

        let cfg = AudioConfig::new(48_000, 8, 0, 2).unwrap();
        let plan = compile(&g, &cfg).unwrap();
        let pass_buffer = plan.steps.iter().find(|s| s.node == pass).unwrap().outputs[0];

        let mut exec = Executor::new(g, plan, &cfg).unwrap();
        exec.process_block(TransportSnapshot::stopped(48_000), &[], &[]);
        assert!(exec
            .buffer(pass_buffer)
            .iter()
            .all(|&s| (s - 0.5).abs() < 1e-6));
    }

    #[test]
    fn executor_silences_unconnected_inputs() {
        use crate::nodes::PassthroughNode;
        use spectre_core::transport::TransportSnapshot;

        // A lone passthrough with no source must emit silence, not garbage
        let mut g = Graph::new();
        let pass = g.add_node(
            Box::new(PassthroughNode::new()),
            &[PortSpec::new(PortType::Audio, 1, "in")],
            &[PortSpec::new(PortType::Audio, 1, "out")],
        );
        let cfg = AudioConfig::new(48_000, 8, 0, 2).unwrap();
        let plan = compile(&g, &cfg).unwrap();
        let buffer = plan.steps.iter().find(|s| s.node == pass).unwrap().outputs[0];

        let mut exec = Executor::new(g, plan, &cfg).unwrap();
        exec.process_block(TransportSnapshot::stopped(48_000), &[], &[]);
        assert!(exec.buffer(buffer).iter().all(|&s| s == 0.0));
    }
}
