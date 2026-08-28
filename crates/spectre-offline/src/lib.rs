// Author: Jeff
// Date: 2026-07-12
// Description: Deterministic offline project-inspection and compiled-plan render harness
// Notes: R0 validates project input; R2 renders the native fixture through spectre-graph

use serde::Serialize;
use serde_json::Map;
use spectre_core::{IdGen, ObjectId, TempoMap, Transport};
use spectre_dsp::{
    AudioProcessor, DeviceParameterSnapshot, DspParameter, Filament, Gloam, NoteEvent,
    NoteEventKind, FILAMENT_PARAMETERS, GAIN_PARAMETERS, GLOAM_PARAMETERS, PULSE_PARAMETERS,
    SATURATOR_PARAMETERS,
};
use spectre_graph::{Connection, EditableGraph, NodeId, PlanNoteInput};
use spectre_project::{
    build_track_graph, from_bytes, track_device_factory, ProjectDoc, ProjectEnvelope, TrackList,
    SCHEMA_VERSION,
};
pub mod bounce;
pub mod fixture;
pub mod hash;
pub mod wav;

use std::collections::{HashMap, HashSet};

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
pub(crate) struct DeviceValues {
    pub(crate) pulse_level: f32,
    pub(crate) gain: f32,
    pub(crate) saturator_drive: f32,
    pub(crate) saturator_mix: f32,
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
        let mut device_ids_by_key = HashMap::new();
        let mut device_keys_by_id = HashMap::new();
        let mut parameter_ids = HashSet::new();

        for entry in snapshot {
            if let Some(previous) =
                device_ids_by_key.insert(entry.device_key(), entry.device_instance_id())
            {
                if previous != entry.device_instance_id() {
                    return Err(format!(
                        "inconsistent device instance identity for {}",
                        entry.device_key()
                    ));
                }
            }
            if let Some(previous) =
                device_keys_by_id.insert(entry.device_instance_id(), entry.device_key())
            {
                if previous != entry.device_key() {
                    return Err(format!(
                        "inconsistent device instance identity aliases {previous} and {}",
                        entry.device_key()
                    ));
                }
            }
            if !parameter_ids.insert(entry.parameter_instance_id()) {
                return Err(format!(
                    "duplicate parameter instance ID for {}.{}",
                    entry.device_key(),
                    entry.parameter_key().as_str()
                ));
            }

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

        if device_keys_by_id
            .keys()
            .any(|device_id| parameter_ids.contains(device_id))
        {
            return Err("device and parameter instance IDs alias".into());
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
    let (mut plan, pulse) = fixture::compile_fixture_plan_with(frames, values)?;
    plan.process(
        sample_rate,
        frames,
        &[PlanNoteInput {
            node: pulse,
            events,
        }],
    )
    .map_err(|error| error.to_string())?;

    report_from(&plan, frames)
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
            pulse_level: fixture::FIXTURE_PULSE_LEVEL,
            gain: fixture::FIXTURE_GAIN,
            saturator_drive: fixture::FIXTURE_SATURATOR_DRIVE,
            saturator_mix: fixture::FIXTURE_SATURATOR_MIX,
        },
    )
}

// Render the fixture chain driven by caller-supplied note events.
// Exists so a live render of one event list can be compared against the offline render of the
// same list without either side owning a second render path or a second hash walk
pub fn render_fixture_events(
    sample_rate: f64,
    frames: usize,
    events: &[NoteEvent],
) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    render_plan(
        sample_rate,
        frames,
        events,
        DeviceValues {
            pulse_level: fixture::FIXTURE_PULSE_LEVEL,
            gain: fixture::FIXTURE_GAIN,
            saturator_drive: fixture::FIXTURE_SATURATOR_DRIVE,
            saturator_mix: fixture::FIXTURE_SATURATOR_MIX,
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
            pulse_level: fixture::FIXTURE_PULSE_LEVEL,
            gain: fixture::FIXTURE_GAIN,
            saturator_drive: fixture::FIXTURE_SATURATOR_DRIVE,
            saturator_mix: fixture::FIXTURE_SATURATOR_MIX,
        },
    )
}

// Hash and peak one rendered quantum. The single definition of the equivalence fold: every
// render entry point in this crate returns through here, so live/offline hash comparisons cannot
// drift by one path acquiring its own walk
fn report_from(plan: &spectre_graph::CompiledPlan, frames: usize) -> Result<RenderReport, String> {
    let output = plan.last_output().ok_or("no quantum has been rendered")?;
    let mut peak = 0.0_f32;
    for sample in output[0].iter().chain(output[1].iter()) {
        peak = peak.max(sample.abs());
    }
    Ok(RenderReport {
        frames,
        channels: 2,
        peak,
        hash: hash::hash_planar_quantum([output[0], output[1]]),
    })
}

// Render a track list through the compiled plan and report the same FNV-1a hash shape
// render_plan already produces. There is no second render path: this builds the same kind of
// CompiledPlan and drives it through the same process call
pub fn render_track_list(
    sample_rate: f64,
    frames: usize,
    tracks: &TrackList,
    seed: u64,
    note_track: Option<ObjectId>,
    events: &[NoteEvent],
) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    let mut ids = IdGen::new(seed);
    let (graph, nodes) = build_track_graph(tracks, &mut ids).map_err(|error| error.to_string())?;
    let mut factory = track_device_factory(tracks, &nodes);
    let mut plan = graph
        .compile(nodes.master.node, frames, &mut factory)
        .map_err(|error| error.to_string())?;

    // Notes address the instrument node of the named track; an unknown track renders silence
    // rather than failing, because an empty or unaddressed project is not an error state
    let note_node = note_track
        .and_then(|id| tracks.index_of(id))
        .and_then(|index| nodes.note_node(index));
    match note_node {
        Some(node) => plan.process(sample_rate, frames, &[PlanNoteInput { node, events }]),
        None => plan.process(sample_rate, frames, &[]),
    }
    .map_err(|error| error.to_string())?;

    report_from(&plan, frames)
}

// Render Filament -> Gloam through the compiled graph plan at both devices' descriptor defaults.
// Its own chain rather than an extension of the fixture: the fixture's four values are the
// offline contract spectre-app publishes, and appending to it would change that contract
pub fn render_voice_chain(sample_rate: f64, frames: usize) -> Result<RenderReport, String> {
    render_voice_chain_with(sample_rate, frames, &fixture_events(frames))
}

// The same chain driven by a caller-supplied event list, so silence is the same code path
fn render_voice_chain_with(
    sample_rate: f64,
    frames: usize,
    events: &[NoteEvent],
) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    let mut ids = IdGen::new(0x0056_4f49_4345_0000);
    let filament = NodeId::new(ids.next_id());
    let gloam = NodeId::new(ids.next_id());

    // Defaults come from the devices' own descriptors, so this harness declares no value of its own
    let lean = FILAMENT_PARAMETERS[0].default();
    let rise_ms = FILAMENT_PARAMETERS[1].default();
    let fall_ms = FILAMENT_PARAMETERS[2].default();
    let level = FILAMENT_PARAMETERS[3].default();
    let damp_hz = GLOAM_PARAMETERS[0].default();
    let depth = GLOAM_PARAMETERS[1].default();
    let track_ms = GLOAM_PARAMETERS[2].default();

    let mut graph = EditableGraph::new();
    graph
        .add_node(filament, Filament::new(lean, rise_ms, fall_ms, level)?.io())
        .map_err(|error| error.to_string())?;
    graph
        .add_node(gloam, Gloam::new(damp_hz, depth, track_ms)?.io())
        .map_err(|error| error.to_string())?;
    graph
        .connect(Connection {
            from: filament,
            from_bus: 0,
            to: gloam,
            to_bus: 0,
        })
        .map_err(|error| error.to_string())?;

    let mut plan = graph
        .compile(gloam, frames, &mut |node| {
            if node == filament {
                Ok(Box::new(Filament::new(lean, rise_ms, fall_ms, level)?)
                    as Box<dyn AudioProcessor>)
            } else {
                Ok(Box::new(Gloam::new(damp_hz, depth, track_ms)?))
            }
        })
        .map_err(|error| error.to_string())?;
    plan.process(
        sample_rate,
        frames,
        &[PlanNoteInput {
            node: filament,
            events,
        }],
    )
    .map_err(|error| error.to_string())?;

    report_from(&plan, frames)
}

// The same chain with no notes; silence must stay exact silence
pub fn render_voice_chain_silence(sample_rate: f64, frames: usize) -> Result<RenderReport, String> {
    render_voice_chain_with(sample_rate, frames, &[])
}
