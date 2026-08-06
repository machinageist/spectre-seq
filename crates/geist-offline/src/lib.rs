// Author: Jeff
// Date: 2026-07-12
// Description: Deterministic offline project-inspection and compiled-plan render harness
// Notes: R0 validates project input; R2 renders the native fixture through geist-graph

use geist_core::{IdGen, TempoMap, Transport};
use geist_dsp::{
    AudioProcessor, DeviceParameterSnapshot, DspParameter, Gain, NoteEvent, NoteEventKind,
    PulseInstrument, Saturator, Waveform, GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS,
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

#[derive(Debug, Clone, Copy)]
struct DeviceValues {
    pulse_level: f32,
    gain: f32,
    saturator_drive: f32,
    saturator_mix: f32,
}

impl DeviceValues {
    fn from_snapshot(snapshot: &[DeviceParameterSnapshot]) -> Result<Self, String> {
        if snapshot.len() != 4 {
            return Err("fixture snapshot must contain exactly four parameters".into());
        }

        let mut pulse_level = None;
        let mut gain = None;
        let mut saturator_drive = None;
        let mut saturator_mix = None;

        for entry in snapshot {
            let (descriptor, target): (DspParameter, &mut Option<f32>) =
                match (entry.device_key(), entry.parameter_key()) {
                    ("pulse", key) if key == PULSE_PARAMETERS[0].key => {
                        (PULSE_PARAMETERS[0], &mut pulse_level)
                    }
                    ("gain", key) if key == GAIN_PARAMETERS[0].key => {
                        (GAIN_PARAMETERS[0], &mut gain)
                    }
                    ("saturator", key) if key == SATURATOR_PARAMETERS[0].key => {
                        (SATURATOR_PARAMETERS[0], &mut saturator_drive)
                    }
                    ("saturator", key) if key == SATURATOR_PARAMETERS[1].key => {
                        (SATURATOR_PARAMETERS[1], &mut saturator_mix)
                    }
                    _ => {
                        return Err(format!(
                            "unknown device parameter identity: {}.{}",
                            entry.device_key(),
                            entry.parameter_key().as_str()
                        ));
                    }
                };

            let value = entry.value();
            if !value.is_finite()
                || !(descriptor.minimum()..=descriptor.maximum()).contains(&value)
                || value != descriptor.clamp(value)
            {
                return Err(format!(
                    "non-canonical value for {}.{}",
                    entry.device_key(),
                    entry.parameter_key().as_str()
                ));
            }
            if target.replace(value).is_some() {
                return Err(format!(
                    "duplicate device parameter identity: {}.{}",
                    entry.device_key(),
                    entry.parameter_key().as_str()
                ));
            }
        }

        Ok(Self {
            pulse_level: pulse_level
                .ok_or("fixture snapshot is missing pulse.level despite complete length")?,
            gain: gain.ok_or("fixture snapshot is missing gain.gain despite complete length")?,
            saturator_drive: saturator_drive
                .ok_or("fixture snapshot is missing saturator.drive despite complete length")?,
            saturator_mix: saturator_mix
                .ok_or("fixture snapshot is missing saturator.mix despite complete length")?,
        })
    }
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
    values: DeviceValues,
) -> Result<RenderReport, String> {
    let mut ids = IdGen::new(0x0000_5245_4e44_4552);
    let pulse = NodeId::new(ids.next_id());
    let gain = NodeId::new(ids.next_id());
    let saturator = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            pulse,
            PulseInstrument::new(Waveform::Saw, values.pulse_level)?.io(),
        )
        .map_err(|error| error.to_string())?;
    graph
        .add_node(gain, Gain::new(values.gain)?.io())
        .map_err(|error| error.to_string())?;
    graph
        .add_node(
            saturator,
            Saturator::new(values.saturator_drive, values.saturator_mix)?.io(),
        )
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
                Ok(Box::new(PulseInstrument::new(
                    Waveform::Saw,
                    values.pulse_level,
                )?))
            } else if node == gain {
                Ok(Box::new(Gain::new(values.gain)?))
            } else {
                Ok(Box::new(Saturator::new(
                    values.saturator_drive,
                    values.saturator_mix,
                )?))
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
    render_plan(
        sample_rate,
        frames,
        &fixture_events(frames),
        DeviceValues {
            pulse_level: 0.3,
            gain: 0.7,
            saturator_drive: 2.5,
            saturator_mix: 0.35,
        },
    )
}

// Apply an immutable app-thread snapshot while constructing the compiled-plan processors
pub fn render_app_snapshot(
    sample_rate: f64,
    frames: usize,
    snapshot: &[DeviceParameterSnapshot],
) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    let values = DeviceValues::from_snapshot(snapshot)?;
    render_plan(sample_rate, frames, &fixture_events(frames), values)
}

// Render the same fixture chain with no notes; silence must stay exact silence
pub fn render_silence(sample_rate: f64, frames: usize) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    render_plan(
        sample_rate,
        frames,
        &[],
        DeviceValues {
            pulse_level: 0.3,
            gain: 0.7,
            saturator_drive: 2.5,
            saturator_mix: 0.35,
        },
    )
}
