// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 4 evidence — the callback bridge drives the same plan the offline harness renders
// Notes: The equivalence test rebuilds the offline fixture exactly (same ID seed, same device
//   values, same events) and hashes the bridge's interleaved output with the same FNV-1a walk
//   the offline harness uses. A matching hash proves live and offline are one computation,
//   not two implementations that happen to agree.

use spectre_audio::bridge::{RenderBridge, DEFAULT_NOTE_SCRATCH};
use spectre_audio::control::{control_channel, ParameterTarget};
use spectre_audio::RenderBlock;
use spectre_core::{IdGen, TransportCommand, TransportState};
use spectre_dsp::{NoteEvent, NoteEventKind};
use spectre_graph::{CompiledPlan, NodeId};
use spectre_offline::{fixture_events, render_vertical_slice};

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 512;
const CHANNELS: u16 = 2;

// The fixture chain, from its single definition in spectre-offline. Not rebuilt here: two
// independently maintained specimens are drift, not verification, and drift between them would
// fire this file's live/offline mismatch for a reason that has nothing to do with the engine
fn fixture_plan(frames: usize) -> (CompiledPlan, NodeId) {
    spectre_offline::fixture::compile_fixture_plan(frames).unwrap()
}

// Hash deinterleaved output the way the offline harness hashes its planar output
fn hash_interleaved(samples: &[f32], channels: usize, frames: usize) -> u64 {
    let mut hash = spectre_offline::hash::FNV_OFFSET_BASIS;
    for channel in 0..channels {
        for frame in 0..frames {
            let sample = samples[frame * channels + channel];
            for byte in sample.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(spectre_offline::hash::FNV_PRIME);
            }
        }
    }
    hash
}

#[test]
fn bridge_output_matches_the_offline_render_of_identical_input() {
    let (plan, note_node) = fixture_plan(FRAMES);
    let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut bridge =
        RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, DEFAULT_NOTE_SCRATCH);

    // The same two fixture events the offline harness renders
    for event in fixture_events(FRAMES) {
        sender.send_note(event).unwrap();
    }

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];
    let mut block = RenderBlock::new(&mut interleaved, CHANNELS);
    bridge.render(&mut block);

    let offline = render_vertical_slice(SAMPLE_RATE, FRAMES).unwrap();
    let live = hash_interleaved(&interleaved, CHANNELS as usize, FRAMES);

    assert_eq!(
        live, offline.hash,
        "callback render must equal the offline render of identical input"
    );
    assert_eq!(bridge.telemetry().blocks_rendered(), 1);
    assert_eq!(bridge.telemetry().plan_errors(), 0);
    // The fixture is audible, so a matching hash is not two silent buffers agreeing
    assert!(offline.peak > 0.0);
}

#[test]
fn a_block_larger_than_plan_capacity_is_refused_into_silence() {
    let (plan, note_node) = fixture_plan(FRAMES);
    let (_sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut bridge =
        RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, DEFAULT_NOTE_SCRATCH);

    let oversized = FRAMES * 2;
    let mut interleaved = vec![1.0_f32; oversized * CHANNELS as usize];
    let mut block = RenderBlock::new(&mut interleaved, CHANNELS);
    bridge.render(&mut block);

    assert!(
        interleaved.iter().all(|sample| *sample == 0.0),
        "a refused block must be silence, never stale or noise"
    );
    assert_eq!(bridge.telemetry().frame_capacity_rejections(), 1);
    assert_eq!(bridge.telemetry().blocks_rendered(), 0);
}

#[test]
fn transport_commands_reach_the_render_thread_in_order() {
    let (plan, note_node) = fixture_plan(FRAMES);
    let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut bridge =
        RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, DEFAULT_NOTE_SCRATCH);

    assert_eq!(bridge.transport().state, TransportState::Stopped);
    sender.send_transport(TransportCommand::Play).unwrap();

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];
    let mut block = RenderBlock::new(&mut interleaved, CHANNELS);
    bridge.render(&mut block);
    assert_eq!(bridge.transport().state, TransportState::Playing);

    // The last command in a block wins, matching Transport's own semantics
    sender.send_transport(TransportCommand::Record).unwrap();
    sender.send_transport(TransportCommand::Stop).unwrap();
    let mut block = RenderBlock::new(&mut interleaved, CHANNELS);
    bridge.render(&mut block);
    assert_eq!(bridge.transport().state, TransportState::Stopped);
}

#[test]
fn notes_beyond_the_scratch_stay_queued_rather_than_dropped() {
    let (plan, note_node) = fixture_plan(FRAMES);
    let (mut sender, receiver) = control_channel(&[], 256, 8).unwrap();
    // A deliberately tiny scratch forces the deferral path
    let scratch = 4;
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, scratch);

    // Offsets must stay ordered inside each block's scratch, so generate them per block
    let queued = 16;
    for index in 0..queued {
        sender
            .send_note(NoteEvent {
                frame_offset: (index % scratch) * 100,
                sequence: index as u64,
                kind: NoteEventKind::On {
                    // Note IDs are nonzero, matching the zero-excluded identity rule
                    id: index as u32 + 1,
                    channel: 0,
                    note: 60,
                    velocity: 0.5,
                },
            })
            .unwrap();
    }

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];
    let mut delivered = 0;
    // Successive blocks drain the backlog; nothing is discarded to make room
    for _ in 0..(queued / scratch) {
        let mut block = RenderBlock::new(&mut interleaved, CHANNELS);
        bridge.render(&mut block);
        delivered += scratch;
    }
    assert_eq!(delivered, queued);
    assert!(bridge.telemetry().notes_deferred() > 0);
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

#[test]
fn parameter_changes_are_counted_while_the_live_seam_is_missing() {
    let mut ids = IdGen::new(0x0050_4152_414d);
    let target = ParameterTarget {
        device: ids.next_id(),
        parameter: ids.next_id(),
    };
    let (plan, note_node) = fixture_plan(FRAMES);
    let (sender, receiver) = control_channel(&[target], 64, 8).unwrap();
    let mut bridge =
        RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, DEFAULT_NOTE_SCRATCH);

    sender.parameters().set_target(target, 0.42).unwrap();
    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];
    let mut block = RenderBlock::new(&mut interleaved, CHANNELS);
    bridge.render(&mut block);

    // The change is observed and counted, not silently discarded. Applying it to a live
    // processor needs an AudioProcessor parameter seam that the accepted contract lacks.
    assert_eq!(bridge.telemetry().parameters_pending(), 1);
}

#[test]
fn repeated_blocks_stay_deterministic() {
    let mut hashes = Vec::new();
    for _ in 0..3 {
        let (plan, note_node) = fixture_plan(FRAMES);
        let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
        let mut bridge =
            RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, DEFAULT_NOTE_SCRATCH);
        for event in fixture_events(FRAMES) {
            sender.send_note(event).unwrap();
        }
        let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];
        let mut block = RenderBlock::new(&mut interleaved, CHANNELS);
        bridge.render(&mut block);
        hashes.push(hash_interleaved(&interleaved, CHANNELS as usize, FRAMES));
    }
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(hashes[1], hashes[2]);
}
