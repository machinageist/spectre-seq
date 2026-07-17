// Author: Jeff
// Date: 2026-07-17
// Description: Editable device graph, validated compilation, and immutable compiled render plan
// Notes: GRAPH-001 split — EditableGraph is app-thread-only; CompiledPlan::process is
//   allocation-free, lock-free, and the only execution surface. V1 buses are stereo,
//   each input bus accepts exactly one connection, and implicit cycles fail compilation.

use geist_core::ObjectId;
use geist_dsp::{AudioProcessor, DeviceIo, NoteEvent, ProcessContext, ProcessError};

// Channels per semantic bus in the v1 stereo-only graph
const CHANNELS_PER_BUS: usize = 2;

// V1 flattened-channel bounds derived from the accepted device layouts
const MAX_FLAT_INPUTS: usize = 4;
const FLAT_OUTPUTS: usize = 2;

// Graph-node identity wrapping the project-stable object ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(ObjectId);

impl NodeId {
    // Wrap a stable object ID as graph-node identity
    pub fn new(id: ObjectId) -> Self {
        Self(id)
    }

    // Recover the underlying project object ID
    pub fn object_id(self) -> ObjectId {
        self.0
    }
}

// One directed stereo-bus connection between two nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connection {
    pub from: NodeId,
    pub from_bus: usize,
    pub to: NodeId,
    pub to_bus: usize,
}

// App-thread editing and compilation failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphError {
    DuplicateNode(NodeId),
    UnknownNode(NodeId),
    // Odd channel counts, missing output, or channels beyond the v1 flat bounds
    InvalidLayout(NodeId),
    OutputBusOutOfRange { node: NodeId, bus: usize },
    InputBusOutOfRange { node: NodeId, bus: usize },
    SelfConnection(NodeId),
    // V1 admits exactly one connection per input bus; mixing is a later explicit decision
    InputBusOccupied { node: NodeId, bus: usize },
    MissingInput { node: NodeId, bus: usize },
    // Implicit cycle diagnostic names one node on the cycle (GRAPH-002 seed)
    Cycle { node: NodeId },
    DeviceBuild { node: NodeId, reason: &'static str },
    // Factory-built processor disagrees with the node's declared layout
    IoMismatch { node: NodeId },
    InvalidMaxFrames,
}

impl std::fmt::Display for GraphError {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateNode(node) => write!(f, "node {} already exists", node.0.raw()),
            Self::UnknownNode(node) => write!(f, "node {} does not exist", node.0.raw()),
            Self::InvalidLayout(node) => {
                write!(f, "node {} has an unsupported v1 layout", node.0.raw())
            }
            Self::OutputBusOutOfRange { node, bus } => {
                write!(f, "node {} has no output bus {bus}", node.0.raw())
            }
            Self::InputBusOutOfRange { node, bus } => {
                write!(f, "node {} has no input bus {bus}", node.0.raw())
            }
            Self::SelfConnection(node) => {
                write!(f, "node {} cannot feed itself directly", node.0.raw())
            }
            Self::InputBusOccupied { node, bus } => {
                write!(f, "input bus {bus} of node {} is already fed", node.0.raw())
            }
            Self::MissingInput { node, bus } => {
                write!(f, "input bus {bus} of node {} is unconnected", node.0.raw())
            }
            Self::Cycle { node } => write!(
                f,
                "implicit cycle through node {}; feedback requires an explicit priced edge",
                node.0.raw()
            ),
            Self::DeviceBuild { node, reason } => {
                write!(
                    f,
                    "device for node {} failed to build: {reason}",
                    node.0.raw()
                )
            }
            Self::IoMismatch { node } => write!(
                f,
                "built processor for node {} contradicts its declared layout",
                node.0.raw()
            ),
            Self::InvalidMaxFrames => write!(f, "max_frames must be at least one"),
        }
    }
}

impl std::error::Error for GraphError {}

// One declared node in the editable graph
#[derive(Debug, Clone, Copy)]
struct GraphNode {
    id: NodeId,
    io: DeviceIo,
}

// Mutable app-thread graph; never reachable from the audio callback
#[derive(Debug, Default)]
pub struct EditableGraph {
    // Insertion order is the deterministic tie-break for compilation
    nodes: Vec<GraphNode>,
    connections: Vec<Connection>,
}

impl EditableGraph {
    // Create an empty editable graph
    pub fn new() -> Self {
        Self::default()
    }

    // Declare a node with its immutable device layout
    pub fn add_node(&mut self, id: NodeId, io: DeviceIo) -> Result<(), GraphError> {
        if self.find(id).is_some() {
            return Err(GraphError::DuplicateNode(id));
        }
        if io.audio_outputs != FLAT_OUTPUTS
            || !io.audio_inputs.is_multiple_of(CHANNELS_PER_BUS)
            || io.audio_inputs > MAX_FLAT_INPUTS
        {
            return Err(GraphError::InvalidLayout(id));
        }
        self.nodes.push(GraphNode { id, io });
        Ok(())
    }

    // Connect one output bus to one unoccupied input bus
    pub fn connect(&mut self, connection: Connection) -> Result<(), GraphError> {
        let from = self
            .find(connection.from)
            .ok_or(GraphError::UnknownNode(connection.from))?;
        let to = self
            .find(connection.to)
            .ok_or(GraphError::UnknownNode(connection.to))?;
        if connection.from == connection.to {
            return Err(GraphError::SelfConnection(connection.from));
        }
        if connection.from_bus >= from.io.audio_outputs / CHANNELS_PER_BUS {
            return Err(GraphError::OutputBusOutOfRange {
                node: connection.from,
                bus: connection.from_bus,
            });
        }
        if connection.to_bus >= to.io.audio_inputs / CHANNELS_PER_BUS {
            return Err(GraphError::InputBusOutOfRange {
                node: connection.to,
                bus: connection.to_bus,
            });
        }
        if self
            .connections
            .iter()
            .any(|edge| edge.to == connection.to && edge.to_bus == connection.to_bus)
        {
            return Err(GraphError::InputBusOccupied {
                node: connection.to,
                bus: connection.to_bus,
            });
        }
        self.connections.push(connection);
        Ok(())
    }

    // Locate a declared node
    fn find(&self, id: NodeId) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    // Locate a node's insertion index
    fn index_of(&self, id: NodeId) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    // Validate, order, and freeze the ancestors of `output` into an executable plan
    pub fn compile(
        &self,
        output: NodeId,
        max_frames: usize,
        factory: &mut dyn FnMut(NodeId) -> Result<Box<dyn AudioProcessor>, &'static str>,
    ) -> Result<CompiledPlan, GraphError> {
        if max_frames == 0 {
            return Err(GraphError::InvalidMaxFrames);
        }
        let output_index = self
            .index_of(output)
            .ok_or(GraphError::UnknownNode(output))?;

        // Reverse reachability: the plan contains exactly the ancestors of the output node
        let mut included = vec![false; self.nodes.len()];
        let mut pending = vec![output_index];
        while let Some(index) = pending.pop() {
            if included[index] {
                continue;
            }
            included[index] = true;
            let id = self.nodes[index].id;
            for edge in self.connections.iter().filter(|edge| edge.to == id) {
                let from_index = self
                    .index_of(edge.from)
                    .expect("connect validated both endpoints");
                pending.push(from_index);
            }
        }

        // Every included input bus must be fed; ancestors of included nodes are included
        for (index, node) in self.nodes.iter().enumerate() {
            if !included[index] {
                continue;
            }
            for bus in 0..node.io.audio_inputs / CHANNELS_PER_BUS {
                if !self
                    .connections
                    .iter()
                    .any(|edge| edge.to == node.id && edge.to_bus == bus)
                {
                    return Err(GraphError::MissingInput { node: node.id, bus });
                }
            }
        }

        // Kahn ordering over the included subgraph; leftovers prove an implicit cycle
        let mut indegree = vec![0usize; self.nodes.len()];
        for edge in &self.connections {
            let to_index = self.index_of(edge.to).expect("validated");
            let from_index = self.index_of(edge.from).expect("validated");
            if included[to_index] && included[from_index] {
                indegree[to_index] += 1;
            }
        }
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut placed = vec![false; self.nodes.len()];
        loop {
            let next =
                (0..self.nodes.len()).find(|&i| included[i] && !placed[i] && indegree[i] == 0);
            let Some(index) = next else {
                break;
            };
            placed[index] = true;
            order.push(index);
            let id = self.nodes[index].id;
            for edge in self.connections.iter().filter(|edge| edge.from == id) {
                let to_index = self.index_of(edge.to).expect("validated");
                if included[to_index] {
                    indegree[to_index] -= 1;
                }
            }
        }
        if let Some(stuck) = (0..self.nodes.len()).find(|&i| included[i] && !placed[i]) {
            return Err(GraphError::Cycle {
                node: self.nodes[stuck].id,
            });
        }

        // Every included output bus owns dedicated channel buffers; no reuse in v1
        let mut channel_of_bus = vec![usize::MAX; self.nodes.len()];
        let mut channel_count = 0usize;
        for &index in &order {
            channel_of_bus[index] = channel_count;
            channel_count += self.nodes[index].io.audio_outputs;
        }

        let mut steps = Vec::with_capacity(order.len());
        let mut processors = Vec::with_capacity(order.len());
        for &index in &order {
            let node = self.nodes[index];
            let processor = factory(node.id).map_err(|reason| GraphError::DeviceBuild {
                node: node.id,
                reason,
            })?;
            if processor.io() != node.io {
                return Err(GraphError::IoMismatch { node: node.id });
            }

            // Flatten fed input buses into channel indices in bus order
            let mut inputs = [0usize; MAX_FLAT_INPUTS];
            let input_channels = node.io.audio_inputs;
            for bus in 0..input_channels / CHANNELS_PER_BUS {
                let edge = self
                    .connections
                    .iter()
                    .find(|edge| edge.to == node.id && edge.to_bus == bus)
                    .expect("missing inputs rejected above");
                let from_index = self.index_of(edge.from).expect("validated");
                let base = channel_of_bus[from_index] + edge.from_bus * CHANNELS_PER_BUS;
                inputs[bus * CHANNELS_PER_BUS] = base;
                inputs[bus * CHANNELS_PER_BUS + 1] = base + 1;
            }

            steps.push(PlanStep {
                node: node.id,
                inputs,
                input_channels,
                output_base: channel_of_bus[index],
                accepts_notes: node.io.accepts_notes,
            });
            processors.push(processor);
        }

        let output_base = channel_of_bus[output_index];
        Ok(CompiledPlan {
            steps,
            processors,
            buffers: vec![vec![0.0; max_frames]; channel_count],
            max_frames,
            output_channels: [output_base, output_base + 1],
            rendered_frames: 0,
        })
    }
}

// One frozen execution step; indices reference the plan's preallocated channel pool
#[derive(Debug, Clone, Copy)]
struct PlanStep {
    node: NodeId,
    inputs: [usize; MAX_FLAT_INPUTS],
    input_channels: usize,
    output_base: usize,
    accepts_notes: bool,
}

// Bounded note delivery to one plan node for one render quantum
#[derive(Debug, Clone, Copy)]
pub struct PlanNoteInput<'a> {
    pub node: NodeId,
    pub events: &'a [NoteEvent],
}

// Plan execution failure surfaced before or during a render quantum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    // Frames outside 1..=max_frames
    FrameCapacity,
    // Note input names a node that is not in the plan
    UnknownNoteNode(NodeId),
    // Note input names a node whose layout refuses events
    NoteRefused(NodeId),
    // Two note inputs name the same node
    DuplicateNoteNode(NodeId),
    Process { node: NodeId, error: ProcessError },
}

impl std::fmt::Display for PlanError {
    // Render an actionable app-thread diagnostic; never called on the render path
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FrameCapacity => write!(f, "frames must be within 1..=max_frames"),
            Self::UnknownNoteNode(node) => {
                write!(f, "note input names absent node {}", node.0.raw())
            }
            Self::NoteRefused(node) => {
                write!(f, "node {} does not accept note events", node.0.raw())
            }
            Self::DuplicateNoteNode(node) => {
                write!(f, "node {} received two note inputs", node.0.raw())
            }
            Self::Process { node, error } => {
                write!(f, "node {} failed to process: {error:?}", node.0.raw())
            }
        }
    }
}

impl std::error::Error for PlanError {}

// Immutable compiled render plan; the only structure the render path executes.
// It exposes no node or edge mutation API by design (GRAPH-001).
pub struct CompiledPlan {
    steps: Vec<PlanStep>,
    processors: Vec<Box<dyn AudioProcessor>>,
    // Planar channel pool preallocated at compile; never resized afterward
    buffers: Vec<Vec<f32>>,
    max_frames: usize,
    output_channels: [usize; 2],
    rendered_frames: usize,
}

impl std::fmt::Debug for CompiledPlan {
    // Summarize the frozen plan without exposing processor internals
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledPlan")
            .field("steps", &self.steps.len())
            .field("max_frames", &self.max_frames)
            .finish_non_exhaustive()
    }
}

impl CompiledPlan {
    // Report the preallocated frame capacity
    pub fn max_frames(&self) -> usize {
        self.max_frames
    }

    // Report how many plan steps execute per quantum
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    // Execute one render quantum; allocation-free and lock-free for valid input
    pub fn process(
        &mut self,
        sample_rate: f64,
        frames: usize,
        note_inputs: &[PlanNoteInput<'_>],
    ) -> Result<(), PlanError> {
        if frames == 0 || frames > self.max_frames {
            return Err(PlanError::FrameCapacity);
        }
        for (position, input) in note_inputs.iter().enumerate() {
            let step = self
                .steps
                .iter()
                .find(|step| step.node == input.node)
                .ok_or(PlanError::UnknownNoteNode(input.node))?;
            if !step.accepts_notes {
                return Err(PlanError::NoteRefused(input.node));
            }
            if note_inputs[..position]
                .iter()
                .any(|earlier| earlier.node == input.node)
            {
                return Err(PlanError::DuplicateNoteNode(input.node));
            }
        }

        for (step, processor) in self.steps.iter().zip(self.processors.iter_mut()) {
            let events = note_inputs
                .iter()
                .find(|input| input.node == step.node)
                .map_or(&[][..], |input| input.events);
            let context = ProcessContext::new(sample_rate, frames, events).map_err(|error| {
                PlanError::Process {
                    node: step.node,
                    error,
                }
            })?;

            // Output buffers leave the pool by pointer swap; inputs borrow the pool directly
            let mut left = std::mem::take(&mut self.buffers[step.output_base]);
            let mut right = std::mem::take(&mut self.buffers[step.output_base + 1]);
            let mut inputs: [&[f32]; MAX_FLAT_INPUTS] = [&[]; MAX_FLAT_INPUTS];
            for (slot, &channel) in inputs
                .iter_mut()
                .zip(step.inputs.iter())
                .take(step.input_channels)
            {
                *slot = &self.buffers[channel][..frames];
            }
            let result = processor.process(
                &context,
                &inputs[..step.input_channels],
                &mut [&mut left[..frames], &mut right[..frames]],
            );
            // Buffers return to the pool before any error propagates
            self.buffers[step.output_base] = left;
            self.buffers[step.output_base + 1] = right;
            result.map_err(|error| PlanError::Process {
                node: step.node,
                error,
            })?;
        }
        self.rendered_frames = frames;
        Ok(())
    }

    // Borrow the output node's stereo channels from the latest quantum
    pub fn last_output(&self) -> Option<[&[f32]; 2]> {
        if self.rendered_frames == 0 {
            return None;
        }
        Some([
            &self.buffers[self.output_channels[0]][..self.rendered_frames],
            &self.buffers[self.output_channels[1]][..self.rendered_frames],
        ])
    }
}
