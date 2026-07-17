// Author: Jeff
// Date: 2026-07-17
// Description: Behavioral tests for graph editing, compile validation, and plan execution
// Notes: GRAPH-001 evidence — the editable graph never processes; only the plan renders

use geist_core::IdGen;
use geist_dsp::{
    AudioProcessor, DeviceClass, DeviceIo, Gain, NoteEvent, NoteEventKind, ProcessContext,
    ProcessError, PulseInstrument, ToneSource, Waveform,
};
use geist_graph::{CompiledPlan, Connection, EditableGraph, GraphError, NodeId, PlanNoteInput};

// Deterministic node IDs for one test graph
fn ids(count: usize) -> Vec<NodeId> {
    let mut generator = IdGen::new(0x47_52_41_50_48);
    (0..count)
        .map(|_| NodeId::new(generator.next_id()))
        .collect()
}

// Stereo connection on bus zero
fn wire(from: NodeId, to: NodeId) -> Connection {
    Connection {
        from,
        from_bus: 0,
        to,
        to_bus: 0,
    }
}

// Compile a tone -> gain chain with real devices
fn tone_gain_plan(tone: NodeId, gain: NodeId, max_frames: usize) -> CompiledPlan {
    let mut graph = EditableGraph::new();
    graph
        .add_node(tone, ToneSource::new(220.0, 0.5).unwrap().io())
        .unwrap();
    graph.add_node(gain, Gain::new(0.5).unwrap().io()).unwrap();
    graph.connect(wire(tone, gain)).unwrap();
    graph
        .compile(gain, max_frames, &mut |node| {
            if node == tone {
                Ok(Box::new(ToneSource::new(220.0, 0.5)?))
            } else {
                Ok(Box::new(Gain::new(0.5)?))
            }
        })
        .unwrap()
}

#[test]
fn tone_gain_chain_renders_deterministically() {
    let nodes = ids(2);
    let mut first = tone_gain_plan(nodes[0], nodes[1], 256);
    let mut second = tone_gain_plan(nodes[0], nodes[1], 256);
    first.process(48_000.0, 256, &[]).unwrap();
    second.process(48_000.0, 256, &[]).unwrap();

    let a = first.last_output().unwrap();
    let b = second.last_output().unwrap();
    assert_eq!(a[0], b[0]);
    assert_eq!(a[1], b[1]);
    assert!(a[0].iter().all(|sample| sample.is_finite()));
    assert!(a[0].iter().any(|sample| *sample != 0.0));
    assert_eq!(a[0].len(), 256);
}

#[test]
fn implicit_cycle_fails_validation_with_diagnostic() {
    let nodes = ids(2);
    let mut graph = EditableGraph::new();
    let effect_io = Gain::new(1.0).unwrap().io();
    graph.add_node(nodes[0], effect_io).unwrap();
    graph.add_node(nodes[1], effect_io).unwrap();
    graph.connect(wire(nodes[0], nodes[1])).unwrap();
    graph.connect(wire(nodes[1], nodes[0])).unwrap();

    let error = graph
        .compile(nodes[1], 64, &mut |_| Ok(Box::new(Gain::new(1.0)?)))
        .unwrap_err();
    assert!(matches!(error, GraphError::Cycle { .. }));
    assert!(error.to_string().contains("implicit cycle"));
}

#[test]
fn editing_rejects_unknown_nodes_bad_buses_and_double_feed() {
    let nodes = ids(3);
    let mut graph = EditableGraph::new();
    let source_io = ToneSource::new(220.0, 0.5).unwrap().io();
    let effect_io = Gain::new(1.0).unwrap().io();
    graph.add_node(nodes[0], source_io).unwrap();
    graph.add_node(nodes[1], effect_io).unwrap();

    assert!(matches!(
        graph.add_node(nodes[0], source_io),
        Err(GraphError::DuplicateNode(_))
    ));
    assert!(matches!(
        graph.connect(wire(nodes[2], nodes[1])),
        Err(GraphError::UnknownNode(_))
    ));
    assert!(matches!(
        graph.connect(wire(nodes[0], nodes[0])),
        Err(GraphError::SelfConnection(_))
    ));
    assert!(matches!(
        graph.connect(Connection {
            from: nodes[0],
            from_bus: 1,
            to: nodes[1],
            to_bus: 0,
        }),
        Err(GraphError::OutputBusOutOfRange { .. })
    ));
    assert!(matches!(
        graph.connect(Connection {
            from: nodes[0],
            from_bus: 0,
            to: nodes[1],
            to_bus: 1,
        }),
        Err(GraphError::InputBusOutOfRange { .. })
    ));

    graph.connect(wire(nodes[0], nodes[1])).unwrap();
    graph.add_node(nodes[2], source_io).unwrap();
    assert!(matches!(
        graph.connect(wire(nodes[2], nodes[1])),
        Err(GraphError::InputBusOccupied { .. })
    ));
}

#[test]
fn compile_rejects_missing_input_and_io_mismatch() {
    let nodes = ids(2);
    let mut graph = EditableGraph::new();
    let effect_io = Gain::new(1.0).unwrap().io();
    graph.add_node(nodes[0], effect_io).unwrap();
    assert!(matches!(
        graph.compile(nodes[0], 64, &mut |_| Ok(Box::new(Gain::new(1.0)?))),
        Err(GraphError::MissingInput { .. })
    ));

    let mut graph = EditableGraph::new();
    graph
        .add_node(nodes[1], ToneSource::new(220.0, 0.5).unwrap().io())
        .unwrap();
    // Factory returns an instrument for a node declared as a plain source
    assert!(matches!(
        graph.compile(nodes[1], 64, &mut |_| {
            Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
        }),
        Err(GraphError::IoMismatch { .. })
    ));
}

#[test]
fn plan_routes_events_only_to_note_nodes() {
    let nodes = ids(2);
    let mut graph = EditableGraph::new();
    let instrument_io = PulseInstrument::new(Waveform::Saw, 0.3).unwrap().io();
    graph.add_node(nodes[0], instrument_io).unwrap();
    graph
        .add_node(nodes[1], Gain::new(0.8).unwrap().io())
        .unwrap();
    graph.connect(wire(nodes[0], nodes[1])).unwrap();
    let mut plan = graph
        .compile(nodes[1], 128, &mut |node| {
            if node == nodes[0] {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
            } else {
                Ok(Box::new(Gain::new(0.8)?))
            }
        })
        .unwrap();

    let on = [NoteEvent {
        frame_offset: 0,
        sequence: 0,
        kind: NoteEventKind::On {
            id: 1,
            channel: 0,
            note: 60,
            velocity: 0.9,
        },
    }];

    // Silent without notes, audible with them
    plan.process(48_000.0, 128, &[]).unwrap();
    assert!(plan.last_output().unwrap()[0]
        .iter()
        .all(|sample| *sample == 0.0));
    plan.process(
        48_000.0,
        128,
        &[PlanNoteInput {
            node: nodes[0],
            events: &on,
        }],
    )
    .unwrap();
    assert!(plan.last_output().unwrap()[0]
        .iter()
        .any(|sample| *sample != 0.0));

    // Events refuse non-note nodes, unknown nodes, and duplicate delivery
    assert!(plan
        .process(
            48_000.0,
            128,
            &[PlanNoteInput {
                node: nodes[1],
                events: &on,
            }],
        )
        .is_err());
    let stranger = ids(3)[2];
    assert!(plan
        .process(
            48_000.0,
            128,
            &[PlanNoteInput {
                node: stranger,
                events: &on,
            }],
        )
        .is_err());
    assert!(plan
        .process(
            48_000.0,
            128,
            &[
                PlanNoteInput {
                    node: nodes[0],
                    events: &on,
                },
                PlanNoteInput {
                    node: nodes[0],
                    events: &[],
                },
            ],
        )
        .is_err());
}

#[test]
fn frame_capacity_is_enforced() {
    let nodes = ids(2);
    let mut plan = tone_gain_plan(nodes[0], nodes[1], 64);
    assert!(plan.process(48_000.0, 0, &[]).is_err());
    assert!(plan.process(48_000.0, 65, &[]).is_err());
    plan.process(48_000.0, 64, &[]).unwrap();
    plan.process(48_000.0, 16, &[]).unwrap();
    assert_eq!(plan.last_output().unwrap()[0].len(), 16);
}

// Test-only source: unit impulse on frame zero, silence elsewhere
struct ImpulseSource;

impl AudioProcessor for ImpulseSource {
    fn io(&self) -> DeviceIo {
        DeviceIo {
            class: DeviceClass::Source,
            audio_inputs: 0,
            audio_outputs: 2,
            accepts_notes: false,
        }
    }

    fn process(
        &mut self,
        _context: &ProcessContext<'_>,
        _inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        for channel in outputs.iter_mut() {
            channel.fill(0.0);
            channel[0] = 1.0;
        }
        Ok(())
    }
}

// R2 impulse gate: a unit impulse crosses the plan wiring sample-exactly
#[test]
fn impulse_crosses_the_plan_sample_exactly() {
    let nodes = ids(2);
    let mut graph = EditableGraph::new();
    graph.add_node(nodes[0], ImpulseSource.io()).unwrap();
    graph
        .add_node(nodes[1], Gain::new(0.5).unwrap().io())
        .unwrap();
    graph.connect(wire(nodes[0], nodes[1])).unwrap();
    let mut plan = graph
        .compile(nodes[1], 64, &mut |node| {
            if node == nodes[0] {
                Ok(Box::new(ImpulseSource))
            } else {
                Ok(Box::new(Gain::new(0.5)?))
            }
        })
        .unwrap();

    plan.process(48_000.0, 64, &[]).unwrap();
    let output = plan.last_output().unwrap();
    for channel in output {
        assert!((channel[0] - 0.5).abs() < 1e-6);
        assert!(channel[1..].iter().all(|sample| *sample == 0.0));
        assert!(channel.iter().all(|sample| sample.is_finite()));
    }
}

#[test]
fn unreachable_nodes_are_excluded_from_the_plan() {
    let nodes = ids(3);
    let mut graph = EditableGraph::new();
    let source_io = ToneSource::new(220.0, 0.5).unwrap().io();
    graph.add_node(nodes[0], source_io).unwrap();
    graph
        .add_node(nodes[1], Gain::new(0.5).unwrap().io())
        .unwrap();
    // A disconnected second source must not be built or executed
    graph.add_node(nodes[2], source_io).unwrap();
    graph.connect(wire(nodes[0], nodes[1])).unwrap();

    let mut built = Vec::new();
    let plan = graph
        .compile(nodes[1], 64, &mut |node| {
            built.push(node);
            if node == nodes[0] {
                Ok(Box::new(ToneSource::new(220.0, 0.5)?))
            } else {
                Ok(Box::new(Gain::new(0.5)?))
            }
        })
        .unwrap();
    assert_eq!(plan.step_count(), 2);
    assert!(!built.contains(&nodes[2]));
}
