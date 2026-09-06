// Author: Jeff
// Date: 2026-08-24
// Description: App-thread construction of the track-to-master render graph
// Notes: Allocates freely; never callback-reachable. Node identities are allocated in a fixed
//   order so a rebuild from the same list and seed produces the same graph.

use crate::track::{TrackEffect, TrackInsert, TrackInstrument, TrackList, MAX_TRACKS};
use spectre_core::{IdGen, ObjectId};
use spectre_dsp::{
    AudioProcessor, Filament, Gain, Gloam, PulseInstrument, SumBus, Waveform, FILAMENT_PARAMETERS,
    GLOAM_DAMP_HZ, GLOAM_PARAMETERS, GLOAM_TRACK_MS, MAX_SUM_BUSES,
};

// Descriptor positions in FILAMENT_PARAMETERS. Named rather than inlined, so a reordering of the
// array cannot silently swap rise for fall
const FILAMENT_LEAN: usize = 0;
const FILAMENT_RISE: usize = 1;
const FILAMENT_FALL: usize = 2;
use spectre_graph::{Connection, EditableGraph, GraphError, NodeId};

// One compiled gain node and the instance ID of its single automatable parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GainNode {
    pub node: NodeId,
    pub gain_parameter: ObjectId,
}

// One compiled instrument node and the instance ID of its level parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstrumentNode {
    pub node: NodeId,
    pub level_parameter: ObjectId,
}

// One compiled insert node and the instance ID of its depth parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsertNode {
    pub node: NodeId,
    pub depth_parameter: ObjectId,
}

// One summing node and the number of stereo input buses it declares. The tree builds one of
// these per group per level, and the factory needs the bus count to construct the device: it is
// no longer `tracks.len()` once more than one summing node exists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SumNode {
    pub node: NodeId,
    pub buses: usize,
}

// Node identities for one built track graph; index i corresponds to TrackList::tracks()[i].
// `inserts[i]` is None for a track with no insert, which is every track written before the slot
// existed -- that track's graph is byte-identical to the R4-4 shape
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackPathNodes {
    pub instruments: Vec<InstrumentNode>,
    pub inserts: Vec<Option<InsertNode>>,
    pub track_gains: Vec<GainNode>,
    // The tree's root, which is what the master reads from. Kept as its own field because every
    // existing caller asks for "the sum" and means this one
    pub sum: NodeId,
    // Every summing node, root included, in the order they were built. One entry at or below
    // one node's fan-in, which is the shape this graph always had
    pub sums: Vec<SumNode>,
    pub master: GainNode,
}

// App-thread routing failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingError {
    TooManyTracks { count: usize, limit: usize },
    Graph(GraphError),
    Device(&'static str),
}

impl std::fmt::Display for RoutingError {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyTracks { count, limit } => {
                write!(formatter, "{count} tracks exceeds the limit of {limit}")
            }
            Self::Graph(error) => write!(formatter, "graph: {error}"),
            Self::Device(error) => write!(formatter, "device: {error}"),
        }
    }
}

impl std::error::Error for RoutingError {}

impl TrackPathNodes {
    // The instrument node for one track index, for RenderBridge's single note node
    pub fn note_node(&self, index: usize) -> Option<NodeId> {
        self.instruments.get(index).map(|entry| entry.node)
    }

    // (device ObjectId, parameter ObjectId) pairs, in track order then master. Returned as plain
    // IDs rather than a spectre-audio type so spectre-project never depends on the audio crate
    pub fn parameter_targets(&self) -> Vec<(ObjectId, ObjectId)> {
        let mut targets = Vec::with_capacity(self.instruments.len() * 3 + 1);
        for ((instrument, insert), gain) in self
            .instruments
            .iter()
            .zip(&self.inserts)
            .zip(&self.track_gains)
        {
            targets.push((instrument.node.object_id(), instrument.level_parameter));
            // Emitted between instrument and gain, and only where a track has one, so this stays
            // in lockstep with spectre-app's parameter_route_nodes. A track without an insert
            // emits exactly the pair it emitted before the slot existed
            if let Some(insert) = insert {
                targets.push((insert.node.object_id(), insert.depth_parameter));
            }
            targets.push((gain.node.object_id(), gain.gain_parameter));
        }
        targets.push((self.master.node.object_id(), self.master.gain_parameter));
        targets
    }
}

// Build the editable graph for a track list. Allocates; app thread only.
//
// Node identities are allocated from `ids` in a fixed order — for each track in list order:
// instrument node, instrument parameter, then the insert node and its depth parameter WHERE THE
// TRACK DECLARES ONE, then gain node, gain parameter — then the sum bus, then the master gain and
// its parameter. That order is the contract: it is what makes a rebuild from the same list with
// the same seed produce the same node IDs. A track with no insert allocates nothing extra, so it
// reproduces the identities the pre-insert order produced
pub fn build_track_graph(
    tracks: &TrackList,
    ids: &mut IdGen,
) -> Result<(EditableGraph, TrackPathNodes), RoutingError> {
    if tracks.len() > MAX_TRACKS {
        return Err(RoutingError::TooManyTracks {
            count: tracks.len(),
            limit: MAX_TRACKS,
        });
    }

    let mut graph = EditableGraph::new();
    let mut instruments = Vec::with_capacity(tracks.len());
    let mut inserts = Vec::with_capacity(tracks.len());
    let mut track_gains = Vec::with_capacity(tracks.len());

    for track in tracks.tracks() {
        let instrument_node = NodeId::new(ids.next_id());
        let level_parameter = ids.next_id();
        // Allocated between instrument and gain, and ONLY when the track declares an insert, so
        // a track without one produces exactly the identities the R4-4 order produced
        let insert_slot = track
            .insert()
            .map(|slot| (slot, NodeId::new(ids.next_id()), ids.next_id()));
        let gain_node = NodeId::new(ids.next_id());
        let gain_parameter = ids.next_id();

        let instrument = instrument_for(track.instrument(), track.instrument_level())?;
        graph
            .add_node(instrument_node, instrument.io())
            .map_err(RoutingError::Graph)?;

        // Mute and solo fold into the gain rather than taking their own node, so they are
        // addressable by the existing latest-wins parameter lane
        let effective = tracks
            .effective_gain(track.id())
            .expect("the track is in the list being iterated");
        let gain = Gain::new(effective).map_err(RoutingError::Device)?;
        graph
            .add_node(gain_node, gain.io())
            .map_err(RoutingError::Graph)?;
        // instrument -> [insert] -> gain. The insert is a stereo in/out device, so it drops into
        // the existing single connection rather than changing the path's shape
        let gain_source = match insert_slot {
            None => instrument_node,
            Some((slot, insert_node, _)) => {
                let effect = effect_for(slot)?;
                graph
                    .add_node(insert_node, effect.io())
                    .map_err(RoutingError::Graph)?;
                graph
                    .connect(Connection {
                        from: instrument_node,
                        from_bus: 0,
                        to: insert_node,
                        to_bus: 0,
                    })
                    .map_err(RoutingError::Graph)?;
                insert_node
            }
        };
        graph
            .connect(Connection {
                from: gain_source,
                from_bus: 0,
                to: gain_node,
                to_bus: 0,
            })
            .map_err(RoutingError::Graph)?;

        instruments.push(InstrumentNode {
            node: instrument_node,
            level_parameter,
        });
        inserts.push(insert_slot.map(|(_, node, depth_parameter)| InsertNode {
            node,
            depth_parameter,
        }));
        track_gains.push(GainNode {
            node: gain_node,
            gain_parameter,
        });
    }

    let sources: Vec<NodeId> = track_gains.iter().map(|gain| gain.node).collect();
    let (sum_node, sums) = build_sum_tree(&mut graph, ids, &sources)?;

    let master_node = NodeId::new(ids.next_id());
    let master_parameter = ids.next_id();
    let master = Gain::new(tracks.master_level()).map_err(RoutingError::Device)?;
    graph
        .add_node(master_node, master.io())
        .map_err(RoutingError::Graph)?;
    graph
        .connect(Connection {
            from: sum_node,
            from_bus: 0,
            to: master_node,
            to_bus: 0,
        })
        .map_err(RoutingError::Graph)?;

    Ok((
        graph,
        TrackPathNodes {
            instruments,
            inserts,
            track_gains,
            sum: sum_node,
            sums,
            master: GainNode {
                node: master_node,
                gain_parameter: master_parameter,
            },
        },
    ))
}

// Construct the processor for one node of a graph built by build_track_graph.
// Values are read from `tracks` at construction, so the returned plan carries the model's levels;
// live changes travel on the parameter lane instead
pub fn track_device_factory<'a>(
    tracks: &'a TrackList,
    nodes: &'a TrackPathNodes,
) -> impl FnMut(NodeId) -> Result<Box<dyn AudioProcessor>, &'static str> + 'a {
    move |node| {
        for (index, instrument) in nodes.instruments.iter().enumerate() {
            if instrument.node == node {
                let track = &tracks.tracks()[index];
                return instrument_for(track.instrument(), track.instrument_level())
                    .map_err(|_| "instrument construction failed");
            }
        }
        for (index, insert) in nodes.inserts.iter().enumerate() {
            let Some(insert) = insert else { continue };
            if insert.node == node {
                let slot = tracks.tracks()[index]
                    .insert()
                    .expect("a node in inserts means the track declares one");
                return effect_for(slot).map_err(|_| "effect construction failed");
            }
        }
        for (index, gain) in nodes.track_gains.iter().enumerate() {
            if gain.node == node {
                let id = tracks.tracks()[index].id();
                let effective = tracks.effective_gain(id).unwrap_or(0.0);
                return Ok(Box::new(Gain::new(effective)?));
            }
        }
        // Every summing node the tree built, each with its own declared bus count. Matching only
        // the root would fail to construct an intermediate node, and using tracks.len() as the
        // count would be wrong for every node in a tree of more than one
        for sum in nodes.sums.iter() {
            if sum.node == node {
                return Ok(Box::new(SumBus::new(sum.buses)?));
            }
        }
        if node == nodes.master.node {
            return Ok(Box::new(Gain::new(tracks.master_level())?));
        }
        Err("node is not part of this track graph")
    }
}

// Build the insert a track slot declares. Damp and track take their descriptor defaults; the
// stored depth is the one value the track model carries, for the reason TrackInsert states
fn effect_for(slot: TrackInsert) -> Result<Box<dyn AudioProcessor>, RoutingError> {
    match slot.effect() {
        TrackEffect::Gloam => Ok(Box::new(
            Gloam::new(
                GLOAM_PARAMETERS[GLOAM_DAMP_HZ].default(),
                slot.depth(),
                GLOAM_PARAMETERS[GLOAM_TRACK_MS].default(),
            )
            .map_err(RoutingError::Device)?,
        )),
    }
}

// Build the instrument a track slot declares
fn instrument_for(
    instrument: TrackInstrument,
    level: f32,
) -> Result<Box<dyn AudioProcessor>, RoutingError> {
    match instrument {
        TrackInstrument::Pulse => Ok(Box::new(
            PulseInstrument::new(Waveform::Saw, level).map_err(RoutingError::Device)?,
        )),
        // The three shaping controls take their own descriptor defaults; the track's own
        // instrument level is the one value a track slot carries, so it is the one passed in.
        // Per-device parameter state on a track slot is a later slice's, not this one's
        TrackInstrument::Filament => Ok(Box::new(
            Filament::new(
                FILAMENT_PARAMETERS[FILAMENT_LEAN].default(),
                FILAMENT_PARAMETERS[FILAMENT_RISE].default(),
                FILAMENT_PARAMETERS[FILAMENT_FALL].default(),
                level,
            )
            .map_err(RoutingError::Device)?,
        )),
    }
}

// Sum any number of stereo sources into one, without any node exceeding MAX_SUM_BUSES.
//
// A single summing node cannot take more inputs than the accepted flat-input bound, and that is
// where MAX_TRACKS came from. But the product requirement is an unbounded number of TRACKS, not
// an unbounded number of inputs on one node -- a tree satisfies the first while every node stays
// inside the second. Nothing on the render path changes: no fixed array grows, no allocation is
// added to the callback, and the accepted device contract is untouched.
//
// At or below MAX_SUM_BUSES this builds exactly the single node it always built, so every
// existing project compiles to a byte-identical graph and R4's render evidence stays valid
// unchanged. Above it, the tree rounds f64 to f32 once per level rather than once overall --
// stated because SumBus's own contract is "accumulates in f64 and rounds once at store", and a
// tree of them rounds once per level by construction.
//
// Group order is source order and bus index is position within the group, so the result is
// deterministic despite float addition not being associative. Nodes are allocated level by
// level, left to right, which keeps the ID sequence a function of the track list alone
fn build_sum_tree(
    graph: &mut EditableGraph,
    ids: &mut IdGen,
    sources: &[NodeId],
) -> Result<(NodeId, Vec<SumNode>), RoutingError> {
    // An empty project still needs a node to render silence from, which is what a zero-bus
    // SumBus is defined to do
    let mut level: Vec<NodeId> = sources.to_vec();
    let mut sums = Vec::new();
    loop {
        // The last level: one node takes everything that remains and is the tree's root
        if level.len() <= MAX_SUM_BUSES {
            let node = NodeId::new(ids.next_id());
            let sum = SumBus::new(level.len()).map_err(RoutingError::Device)?;
            sums.push(SumNode {
                node,
                buses: level.len(),
            });
            graph
                .add_node(node, sum.io())
                .map_err(RoutingError::Graph)?;
            for (bus, source) in level.iter().enumerate() {
                graph
                    .connect(Connection {
                        from: *source,
                        from_bus: 0,
                        to: node,
                        to_bus: bus,
                    })
                    .map_err(RoutingError::Graph)?;
            }
            return Ok((node, sums));
        }

        // One full level of fixed-width groups, so the shape is a function of the count alone.
        // An id is minted per node actually built, never speculatively
        let mut next = Vec::with_capacity(level.len().div_ceil(MAX_SUM_BUSES));
        for group in level.chunks(MAX_SUM_BUSES) {
            let node = NodeId::new(ids.next_id());
            let sum = SumBus::new(group.len()).map_err(RoutingError::Device)?;
            sums.push(SumNode {
                node,
                buses: group.len(),
            });
            graph
                .add_node(node, sum.io())
                .map_err(RoutingError::Graph)?;
            for (bus, source) in group.iter().enumerate() {
                graph
                    .connect(Connection {
                        from: *source,
                        from_bus: 0,
                        to: node,
                        to_bus: bus,
                    })
                    .map_err(RoutingError::Graph)?;
            }
            next.push(node);
        }
        level = next;
    }
}
