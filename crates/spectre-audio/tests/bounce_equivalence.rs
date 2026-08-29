// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 8 evidence — a multi-block offline bounce equals the live callback path
// Notes: Neither side rebuilds the fixture. Both call spectre_offline::fixture, the same builder
//   render_plan and bounce.rs call, so "same plan, same values, same events" is a fact about the
//   code rather than a claim about three files agreeing. What stays independent is the traversal:
//   the live side folds interleaved driver memory, the bounce folds planar planes, and the two
//   are only equal if they describe the same audio.
//   No hash literal is pinned here. One was, and it was produced on macOS: device math reaches
//   the platform's libm through tanh and powf, so the number is per-platform and the assertion
//   failed the first time the suite ran on Linux while live and offline still agreed exactly.
//   QUAL-006 already refused a golden hash in e2e_alpha for this reason. The equality that
//   carries the evidence is live == offline, asserted below on whatever host runs it.

use spectre_audio::bridge::{RenderBridge, DEFAULT_NOTE_SCRATCH};
use spectre_audio::control::control_channel;
use spectre_audio::RenderBlock;
use spectre_core::ObjectId;
use spectre_dsp::{
    DeviceParameterSnapshot, NoteEvent, NoteEventKind, GAIN_PARAMETERS, PULSE_PARAMETERS,
    SATURATOR_PARAMETERS,
};
use spectre_offline::bounce::{bounce_report, first_divergence, BounceConfig, Divergence};
use spectre_offline::fixture;
use spectre_offline::fixture_events;
use spectre_offline::hash::{hash_block, SampleHasher};

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 4_096;
const BLOCK: usize = 256;
const BLOCKS: usize = FRAMES / BLOCK;
const CHANNELS: usize = 2;

fn id(raw: u64) -> ObjectId {
    ObjectId::from_raw(raw).unwrap()
}

// The fixture's four parameters at the fixture's own values, in the identity shape
// DeviceValues::from_snapshot accepts
fn fixture_snapshot() -> Vec<DeviceParameterSnapshot> {
    vec![
        DeviceParameterSnapshot::new(
            id(10),
            "pulse",
            id(11),
            PULSE_PARAMETERS[0],
            fixture::FIXTURE_PULSE_LEVEL,
        ),
        DeviceParameterSnapshot::new(
            id(20),
            "gain",
            id(21),
            GAIN_PARAMETERS[0],
            fixture::FIXTURE_GAIN,
        ),
        DeviceParameterSnapshot::new(
            id(30),
            "saturator",
            id(31),
            SATURATOR_PARAMETERS[0],
            fixture::FIXTURE_SATURATOR_DRIVE,
        ),
        DeviceParameterSnapshot::new(
            id(30),
            "saturator",
            id(32),
            SATURATOR_PARAMETERS[1],
            fixture::FIXTURE_SATURATOR_MIX,
        ),
    ]
}

// What one live run produced: every interleaved sample in driver order, the per-block folds, and
// the telemetry that says the blocks were rendered rather than refused
struct LiveRun {
    samples: Vec<f32>,
    block_hashes: Vec<u64>,
    hash: u64,
    blocks_rendered: u64,
    plan_errors: u64,
    frame_capacity_rejections: u64,
}

// Drive the bridge for BLOCKS callbacks, re-basing each block's events the way a driver would.
// `events` carries absolute offsets, exactly as the bounce receives them
fn live_run(events: &[NoteEvent]) -> LiveRun {
    let (plan, note_node) = fixture::compile_fixture_plan(BLOCK).unwrap();
    // The lane widths five of the six bridge_plan tests use; fixture_events returns two events,
    // so at most one crosses the note lane before any one block
    let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut bridge =
        RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, DEFAULT_NOTE_SCRATCH);

    let mut samples = Vec::with_capacity(FRAMES * CHANNELS);
    let mut block_hashes = Vec::with_capacity(BLOCKS);
    let mut hasher = SampleHasher::new();
    let mut interleaved = vec![0.0_f32; BLOCK * CHANNELS];

    for block in 0..BLOCKS {
        let start = block * BLOCK;
        for event in events
            .iter()
            .filter(|event| event.frame_offset >= start && event.frame_offset < start + BLOCK)
        {
            sender
                .send_note(NoteEvent {
                    frame_offset: event.frame_offset - start,
                    ..*event
                })
                .unwrap();
        }

        interleaved.fill(0.0);
        let mut render_block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
        bridge.render(&mut render_block);

        hash_block(&mut hasher, &interleaved, CHANNELS, BLOCK);
        let mut per_block = SampleHasher::new();
        hash_block(&mut per_block, &interleaved, CHANNELS, BLOCK);
        block_hashes.push(per_block.finish());
        samples.extend_from_slice(&interleaved);
    }

    let telemetry = bridge.telemetry();
    LiveRun {
        samples,
        block_hashes,
        hash: hasher.finish(),
        blocks_rendered: telemetry.blocks_rendered(),
        plan_errors: telemetry.plan_errors(),
        frame_capacity_rejections: telemetry.frame_capacity_rejections(),
    }
}

fn bounce_config() -> BounceConfig {
    BounceConfig {
        sample_rate: SAMPLE_RATE,
        frames: FRAMES,
        block_frames: BLOCK,
        // This test *is* a live/offline comparison, which is the condition the per-block log
        // exists for, so the flag is set rather than assumed
        log_block_hashes: true,
    }
}

// 15
#[test]
fn a_multi_block_bounce_matches_the_live_path_block_for_block() {
    let live = live_run(&fixture_events(FRAMES));
    let offline = bounce_report(
        bounce_config(),
        &fixture_snapshot(),
        &fixture_events(FRAMES),
    )
    .unwrap();

    assert_eq!(
        live.hash, offline.hash,
        "a sixteen-block bounce must equal sixteen callbacks of the same plan"
    );
    // Without this two silent buffers would satisfy the line above
    assert!(offline.peak > 0.0);
    assert_eq!(live.blocks_rendered, BLOCKS as u64);
    assert_eq!(live.plan_errors, 0);
    assert_eq!(live.frame_capacity_rejections, 0);
    assert_eq!(offline.block_hashes.len(), BLOCKS);
    assert_eq!(live.block_hashes, offline.block_hashes);
    assert_eq!(offline.contaminated_nodes, 0);
}

// 16
#[test]
fn a_divergence_is_localized_to_a_block_and_then_to_a_sample() {
    const BAD_FRAME: usize = 2_113;
    const BAD_BLOCK: usize = BAD_FRAME / BLOCK;
    const BAD_IN_BLOCK: usize = BAD_FRAME % BLOCK;

    let live = live_run(&fixture_events(FRAMES));
    let offline = bounce_report(
        bounce_config(),
        &fixture_snapshot(),
        &fixture_events(FRAMES),
    )
    .unwrap();

    // Corrupt one sample of the captured live stream. The defect is manufactured because the
    // engine has none: a localizer that is never exercised is a localizer that does not work
    let mut corrupted = live.samples.clone();
    let index = BAD_FRAME * CHANNELS;
    corrupted[index] = -corrupted[index] - 1.0;

    let mut corrupted_block_hashes = Vec::with_capacity(BLOCKS);
    for block in 0..BLOCKS {
        let start = block * BLOCK * CHANNELS;
        let mut hasher = SampleHasher::new();
        hash_block(
            &mut hasher,
            &corrupted[start..start + BLOCK * CHANNELS],
            CHANNELS,
            BLOCK,
        );
        corrupted_block_hashes.push(hasher.finish());
    }

    // First the block, from the logged per-block folds
    let first_bad_block = corrupted_block_hashes
        .iter()
        .zip(&offline.block_hashes)
        .position(|(left, right)| left != right);
    assert_eq!(first_bad_block, Some(BAD_BLOCK));

    // Then the sample, from the streams themselves
    let divergence = first_divergence(&corrupted, &live.samples, CHANNELS, BLOCK);
    assert_eq!(
        divergence,
        Some(Divergence {
            block: BAD_BLOCK,
            frame_in_block: BAD_IN_BLOCK,
            absolute_frame: BAD_FRAME,
            channel: 0,
            live_bits: corrupted[index].to_bits(),
            offline_bits: live.samples[index].to_bits(),
        })
    );

    // The uncorrupted pair is what the localizer says about a healthy engine
    assert_eq!(
        first_divergence(&live.samples, &live.samples, CHANNELS, BLOCK),
        None
    );
}

// 15b — the rebase gate test 15 cannot be
#[test]
fn a_mid_timeline_note_lands_on_the_same_frame_in_both_paths() {
    // fixture_events puts its note-on at frame 0 and its note-off on the final frame, where a
    // rebase is either the identity or one sample from the end. Test 15 therefore passes even
    // when per-block re-basing is wrong, which was verified by breaking it. These offsets fall
    // in the interior of blocks 2 and 9, where a mis-rebased event is audibly in the wrong place
    let events = [
        NoteEvent {
            frame_offset: 700,
            sequence: 0,
            kind: NoteEventKind::On {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.8,
            },
        },
        NoteEvent {
            frame_offset: 2_500,
            sequence: 1,
            kind: NoteEventKind::Off {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.0,
            },
        },
    ];

    let live = live_run(&events);
    let offline = bounce_report(bounce_config(), &fixture_snapshot(), &events).unwrap();

    assert_eq!(live.hash, offline.hash);
    assert_eq!(live.block_hashes, offline.block_hashes);
    assert!(offline.peak > 0.0);
    assert_eq!(live.plan_errors, 0);

    // The note is where it was put, not merely somewhere: silent before 700, sounding inside,
    // silent after 2,500. This is what makes the hash match a statement about placement
    assert!(live.samples[..700 * CHANNELS].iter().all(|s| *s == 0.0));
    assert!(live.samples[700 * CHANNELS..2_500 * CHANNELS]
        .iter()
        .any(|s| *s != 0.0));
    assert!(live.samples[2_500 * CHANNELS..].iter().all(|s| *s == 0.0));
}
