// Author: Jeff
// Date: 2026-07-12
// Description: Deterministic offline project-inspection and compiled-plan render harness
// Notes: R0 validates project input; R2 renders the native fixture through geist-graph

use geist_core::{IdGen, TempoMap, Transport};
use geist_dsp::{
    AudioProcessor, Gain, NoteEvent, NoteEventKind, PulseInstrument, Saturator, Waveform,
};
use geist_graph::{Connection, EditableGraph, NodeId, PlanNoteInput};
use geist_project::{from_bytes, ProjectDoc, ProjectEnvelope, SCHEMA_VERSION};
use serde::Serialize;
use serde_json::Map;

// Stable machine-readable report emitted by the offline harness
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OfflineReport {
    pub schema_version: u32,
    pub project_id: u64,
    pub project_name: String,
    pub tempo_segment_count: usize,
    pub transport_position_samples: i64,
}

// Deterministic summary of the initial native-device render chain
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RenderReport {
    pub frames: usize,
    pub channels: usize,
    pub peak: f32,
    pub hash: u64,
}

// Build the deterministic empty project used by smoke tests and future render fixtures
pub fn default_project() -> ProjectEnvelope {
    let mut ids = IdGen::new(0x0047_4549_5354);
    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: ids.next_id(),
            name: "Untitled".into(),
            tempo_map: TempoMap::constant(120.0).expect("constant default tempo is valid"),
            transport: Transport::new(),
            unknown: Map::new(),
        },
        unknown: Map::new(),
    }
}

// Validate project bytes and summarize the deterministic inputs to future rendering
pub fn inspect_project(bytes: &[u8]) -> Result<OfflineReport, String> {
    let envelope = from_bytes(bytes).map_err(|error| error.to_string())?;
    Ok(OfflineReport {
        schema_version: envelope.schema_version,
        project_id: envelope.project.id.raw(),
        project_name: envelope.project.name,
        tempo_segment_count: envelope.project.tempo_map.segments().len(),
        transport_position_samples: envelope.project.transport.position.0,
    })
}

// Fixture note events: one held note released on the final frame
pub fn fixture_events(frames: usize) -> [NoteEvent; 2] {
    [
        NoteEvent {
            frame_offset: 0,
            sequence: 0,
            kind: NoteEventKind::On {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.8,
            },
        },
        NoteEvent {
            frame_offset: frames - 1,
            sequence: 1,
            kind: NoteEventKind::Off {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.0,
            },
        },
    ]
}

// Compile the fixture graph and render `events` through the plan
fn render_plan(
    sample_rate: f64,
    frames: usize,
    events: &[NoteEvent],
) -> Result<RenderReport, String> {
    let mut ids = IdGen::new(0x0000_5245_4e44_4552);
    let pulse = NodeId::new(ids.next_id());
    let gain = NodeId::new(ids.next_id());
    let saturator = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(pulse, PulseInstrument::new(Waveform::Saw, 0.3)?.io())
        .map_err(|error| error.to_string())?;
    graph
        .add_node(gain, Gain::new(0.7)?.io())
        .map_err(|error| error.to_string())?;
    graph
        .add_node(saturator, Saturator::new(2.5, 0.35)?.io())
        .map_err(|error| error.to_string())?;
    for (from, to) in [(pulse, gain), (gain, saturator)] {
        graph
            .connect(Connection {
                from,
                from_bus: 0,
                to,
                to_bus: 0,
            })
            .map_err(|error| error.to_string())?;
    }

    let mut plan = graph
        .compile(saturator, frames, &mut |node| {
            if node == pulse {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
            } else if node == gain {
                Ok(Box::new(Gain::new(0.7)?))
            } else {
                Ok(Box::new(Saturator::new(2.5, 0.35)?))
            }
        })
        .map_err(|error| error.to_string())?;
    plan.process(
        sample_rate,
        frames,
        &[PlanNoteInput {
            node: pulse,
            events,
        }],
    )
    .map_err(|error| error.to_string())?;

    let output = plan.last_output().expect("quantum just rendered");
    let mut peak = 0.0_f32;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for sample in output[0].iter().chain(output[1].iter()) {
        peak = peak.max(sample.abs());
        for byte in sample.to_bits().to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    Ok(RenderReport {
        frames,
        channels: 2,
        peak,
        hash,
    })
}

// Render PulseInstrument -> Gain -> Saturator through the compiled graph plan
pub fn render_vertical_slice(sample_rate: f64, frames: usize) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    render_plan(sample_rate, frames, &fixture_events(frames))
}

// Render the same fixture chain with no notes; silence must stay exact silence
pub fn render_silence(sample_rate: f64, frames: usize) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    render_plan(sample_rate, frames, &[])
}
