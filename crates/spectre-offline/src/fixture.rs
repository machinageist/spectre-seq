// Author: Jeff
// Date: 2026-08-25
// Description: The single definition of the R4 fixture chain and its device values
// Notes: Extracted so the harness, the bounce, and the bridge's equivalence test build the same
//   specimen rather than three hand-maintained copies. Drift between copies would fire the
//   live/offline mismatch alarm for a reason that has nothing to do with the engine.

use crate::DeviceValues;
use spectre_core::IdGen;
use spectre_dsp::{AudioProcessor, Gain, PulseInstrument, Saturator, Waveform};
use spectre_graph::{CompiledPlan, Connection, EditableGraph, NodeId};

// The fixture's device values. Not new numbers: these are the literals the harness has rendered
// since R2, named here so the four call sites that used to repeat them cannot drift
pub const FIXTURE_PULSE_LEVEL: f32 = 0.3;
pub const FIXTURE_GAIN: f32 = 0.7;
pub const FIXTURE_SATURATOR_DRIVE: f32 = 2.5;
pub const FIXTURE_SATURATOR_MIX: f32 = 0.35;

// The ID seed the fixture has always used. Node identities are a function of it, so a test that
// rebuilds the chain from this seed gets the same nodes the harness compiled
pub const FIXTURE_SEED: u64 = 0x0000_5245_4e44_4552;

// Compile the fixture chain at its own device values
pub fn compile_fixture_plan(frames: usize) -> Result<(CompiledPlan, NodeId), String> {
    compile_fixture_plan_with(
        frames,
        DeviceValues {
            pulse_level: FIXTURE_PULSE_LEVEL,
            gain: FIXTURE_GAIN,
            saturator_drive: FIXTURE_SATURATOR_DRIVE,
            saturator_mix: FIXTURE_SATURATOR_MIX,
        },
    )
}

// Compile the fixture chain at caller-supplied device values, returning the plan and its one
// note node. The construction is the harness's own, moved rather than rewritten, so the
// extraction is bit-exact — the hand-wired equivalence tests are what prove that
pub(crate) fn compile_fixture_plan_with(
    frames: usize,
    values: DeviceValues,
) -> Result<(CompiledPlan, NodeId), String> {
    let mut ids = IdGen::new(FIXTURE_SEED);
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

    let plan = graph
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
    Ok((plan, pulse))
}
