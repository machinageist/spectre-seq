// Author: Jeff
// Date: 2026-07-12
// Description: Deterministic offline project-inspection harness
// Notes: R0 validates project input; R2 extends this seam with compiled-plan audio rendering

use geist_core::{IdGen, TempoMap, Transport};
use geist_dsp::{
    AudioProcessor, Gain, NoteEvent, NoteEventKind, ProcessContext, PulseInstrument, Saturator,
    Waveform,
};
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

// Render PulseInstrument -> Gain -> Saturator into an offline stereo fixture
pub fn render_vertical_slice(sample_rate: f64, frames: usize) -> Result<RenderReport, String> {
    if frames < 2 {
        return Err("render requires at least two frames".into());
    }
    let events = [
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
    ];
    let instrument_context =
        ProcessContext::new(sample_rate, frames, &events).map_err(|error| format!("{error:?}"))?;
    let effect_context =
        ProcessContext::new(sample_rate, frames, &[]).map_err(|error| format!("{error:?}"))?;

    let mut instrument = PulseInstrument::new(Waveform::Saw, 0.3).map_err(ToString::to_string)?;
    let mut gain = Gain::new(0.7).map_err(ToString::to_string)?;
    let mut saturator = Saturator::new(2.5, 0.35).map_err(ToString::to_string)?;
    let mut synth_left = vec![0.0; frames];
    let mut synth_right = vec![0.0; frames];
    let mut gain_left = vec![0.0; frames];
    let mut gain_right = vec![0.0; frames];
    let mut final_left = vec![0.0; frames];
    let mut final_right = vec![0.0; frames];

    instrument
        .process(
            &instrument_context,
            &[],
            &mut [&mut synth_left, &mut synth_right],
        )
        .map_err(|error| format!("{error:?}"))?;
    gain.process(
        &effect_context,
        &[&synth_left, &synth_right],
        &mut [&mut gain_left, &mut gain_right],
    )
    .map_err(|error| format!("{error:?}"))?;
    saturator
        .process(
            &effect_context,
            &[&gain_left, &gain_right],
            &mut [&mut final_left, &mut final_right],
        )
        .map_err(|error| format!("{error:?}"))?;

    let mut peak = 0.0_f32;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for sample in final_left.iter().chain(final_right.iter()) {
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
