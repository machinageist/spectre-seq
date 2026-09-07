// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 5 evidence — clip playback merged with live MIDI through the real bridge
// Notes: Carries its own guarding allocator and positive control, matching rt_guard.rs. Every
//   assertion drives the shipped RenderBridge rather than inspecting the player's array, because
//   the plan's own validator is the referee: a test that only reads the scheduler's output can
//   agree with itself while violating the event contract.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use spectre_audio::bridge::RenderBridge;
use spectre_audio::clip::{ClipPlayer, ClipSchedule, CLIP_EVENT_RESERVE, SCHEDULE_LANE_CAPACITY};
use spectre_audio::control::{control_channel, ControlError, ControlSender};
use spectre_audio::RenderBlock;
use spectre_core::{BeatTicks, SampleRate, SampleTime, TempoMap, TransportCommand, TICKS_PER_BEAT};
use spectre_dsp::{NoteEvent, NoteEventKind, MAX_NOTE_EVENTS_PER_BLOCK};
use spectre_graph::{CompiledPlan, NodeId};

const SAMPLE_RATE: f64 = 48_000.0;
const RATE_HZ: u32 = 48_000;
const FRAMES: usize = 256;
const CHANNELS: u16 = 2;
const BEAT: i64 = TICKS_PER_BEAT;

thread_local! {
    static IN_RT_SECTION: Cell<bool> = const { Cell::new(false) };
    static VIOLATIONS: Cell<u64> = const { Cell::new(0) };
}

struct GuardingAllocator;

unsafe impl GlobalAlloc for GuardingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if IN_RT_SECTION.with(Cell::get) {
            VIOLATIONS.with(|count| count.set(count.get() + 1));
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if IN_RT_SECTION.with(Cell::get) {
            VIOLATIONS.with(|count| count.set(count.get() + 1));
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: GuardingAllocator = GuardingAllocator;

fn rt_section<T>(body: impl FnOnce() -> T) -> (T, u64) {
    let start = VIOLATIONS.with(Cell::get);
    IN_RT_SECTION.with(|flag| flag.set(true));
    let value = body();
    IN_RT_SECTION.with(|flag| flag.set(false));
    (value, VIOLATIONS.with(Cell::get) - start)
}

// The fixture chain, from its single definition in spectre-offline. Not rebuilt here: two
// independently maintained specimens are drift, not verification, and drift between them would
// fire this file's live/offline mismatch for a reason that has nothing to do with the engine
fn fixture_plan(frames: usize) -> (CompiledPlan, NodeId) {
    spectre_offline::fixture::compile_fixture_plan(frames).unwrap()
}

// The FNV-1a walk the offline harness uses, over an interleaved buffer
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

fn schedule_of(notes: &[(i64, i64, u8)]) -> ClipSchedule {
    ClipSchedule::bake(
        notes
            .iter()
            .map(|(start, length, note)| (BeatTicks(*start), BeatTicks(*length), 0, *note, 1.0)),
        &TempoMap::constant(120.0).unwrap(),
        SampleRate::new(RATE_HZ).unwrap(),
        CLIP_EVENT_RESERVE,
    )
    .unwrap()
}

// A bridge with a clip player over the fixture plan, already installed and playing.
// The scratch is the accepted per-quantum ceiling, because the clip reserve and the ingress bound
// must both fit before either can push the other into notes_deferred on a legal block
fn playing_bridge(notes: &[(i64, i64, u8)]) -> (ControlSender, RenderBridge) {
    let (plan, note_node) = fixture_plan(FRAMES);
    let (mut sender, receiver) = control_channel(&[], 1_024, 64).unwrap();
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule_of(notes)));
    let bridge = RenderBridge::with_parameter_routes(
        plan,
        receiver,
        note_node,
        SAMPLE_RATE,
        MAX_NOTE_EVENTS_PER_BLOCK,
        spectre_audio::route::ParameterRoutes::empty(),
    )
    .with_clip_player(player);
    sender.send_transport(TransportCommand::Play).unwrap();
    (sender, bridge)
}

// Render one block and return its interleaved samples
fn render(bridge: &mut RenderBridge, buffer: &mut [f32]) {
    let mut block = RenderBlock::new(buffer, CHANNELS);
    bridge.render(&mut block);
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0_f32, |worst, s| worst.max(s.abs()))
}

// A live-MIDI note event, numbered from the low band the way MidiIngress does
fn live_note(sequence: u64, frame_offset: usize, note: u8) -> NoteEvent {
    NoteEvent {
        frame_offset,
        sequence,
        kind: NoteEventKind::On {
            id: sequence as u32 + 1,
            channel: 0,
            note,
            velocity: 0.5,
        },
    }
}

// I-1
#[test]
fn a_clip_renders_audible_output_through_the_plan() {
    let (_sender, mut bridge) = playing_bridge(&[(0, 16 * BEAT, 45)]);
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];

    let mut loudest = 0.0_f32;
    for _ in 0..8 {
        render(&mut bridge, &mut buffer);
        loudest = loudest.max(peak(&buffer));
    }

    // A wrong merge or a wrong emit makes the plan refuse and the output exactly silent, so the
    // nonzero-peak assertion is what makes this a result rather than two silent buffers agreeing
    assert!(loudest > 0.0, "the clip must sound");
    assert_eq!(bridge.telemetry().plan_errors(), 0);
    assert_eq!(bridge.telemetry().clip_events_refused(), 0);
    assert_eq!(bridge.telemetry().loop_segments_refused(), 0);
}

// I-2 — the whole feature. A missing sort, a cleared scratch, or a colliding sequence each
// produce UnsortedEvents and a nonzero plan_errors
#[test]
fn clip_and_live_notes_merge_into_one_sorted_block() {
    let dense: Vec<(i64, i64, u8)> = (0..96)
        .map(|index| (index * (BEAT / 4), BEAT / 8, 40 + (index % 30) as u8))
        .collect();
    let (mut sender, mut bridge) = playing_bridge(&dense);
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];

    let mut loudest = 0.0_f32;
    for block in 0..200_u64 {
        // Live events at offsets that interleave with the clip's, including frame 0 where a
        // clip event also lands
        for step in 0..4_u64 {
            let offset = (step as usize * 61) % FRAMES;
            sender
                .send_note(live_note(block * 4 + step, offset, 50))
                .unwrap();
        }
        render(&mut bridge, &mut buffer);
        loudest = loudest.max(peak(&buffer));
    }

    assert_eq!(
        bridge.telemetry().plan_errors(),
        0,
        "the merged block must satisfy the event contract on every block"
    );
    assert_eq!(bridge.telemetry().blocks_rendered(), 200);
    assert!(loudest > 0.0);
}

// I-3 — pins the smallest and most omittable edit in the feature: collect_notes appends
#[test]
fn collect_notes_appends_rather_than_clears() {
    // Three clip attacks inside the first block, at ticks 0, 4, 8 — samples 0, 100, 200
    let (mut sender, mut bridge) = playing_bridge(&[(0, 2, 60), (4, 2, 62), (8, 2, 64)]);
    for sequence in 0..3_u64 {
        sender
            .send_note(live_note(sequence, sequence as usize * 3, 70))
            .unwrap();
    }

    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];
    render(&mut bridge, &mut buffer);

    // Ten: one all-notes-off from the install, three clip attacks, three clip releases, and the
    // three live attacks. If collect_notes cleared the scratch instead of appending, the seven
    // clip events would be gone and the plan would still accept the block — so the count is what
    // catches it, not an error
    assert_eq!(bridge.telemetry().last_block_events(), 10);
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

// I-4
#[test]
fn repeated_renders_of_one_schedule_hash_identically() {
    let notes: Vec<(i64, i64, u8)> = (0..32)
        .map(|index| (index * (BEAT / 2), BEAT / 4, 45 + (index % 12) as u8))
        .collect();

    let mut hashes = Vec::new();
    let mut loudest = 0.0_f32;
    for _ in 0..2 {
        let (_sender, mut bridge) = playing_bridge(&notes);
        let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for _ in 0..200 {
            render(&mut bridge, &mut buffer);
            loudest = loudest.max(peak(&buffer));
            hash ^= hash_interleaved(&buffer, CHANNELS as usize, FRAMES);
        }
        assert_eq!(bridge.telemetry().plan_errors(), 0);
        hashes.push(hash);
    }

    assert_eq!(hashes[0], hashes[1]);
    assert!(loudest > 0.0, "determinism against silence is not a signal");
}

// I-5 — the live bridge and the offline harness are one computation over one event list
#[test]
fn the_live_bridge_hash_matches_the_offline_render() {
    // Three clip attacks inside the first block, at samples 0, 100, and 200
    let (_sender, mut bridge) = playing_bridge(&[(0, 2, 60), (4, 2, 62), (8, 2, 64)]);
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];
    render(&mut bridge, &mut buffer);

    // The events the player produced for that block, taken from a second player driven the same
    // way. Rendering them offline through the same fixture chain must reproduce the same samples
    let mut replay = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = replay.install(Box::new(schedule_of(&[(0, 2, 60), (4, 2, 62), (8, 2, 64)])));
    replay.release_all();
    let mut transport = spectre_core::Transport::new();
    transport.apply(TransportCommand::Play);
    let mut events = Vec::with_capacity(CLIP_EVENT_RESERVE);
    replay.emit_block(
        &transport,
        FRAMES,
        &mut events,
        spectre_audio::clip::CLIP_SEQUENCE_BAND,
    );

    let offline = spectre_offline::render_fixture_events(SAMPLE_RATE, FRAMES, &events).unwrap();
    let live = hash_interleaved(&buffer, CHANNELS as usize, FRAMES);

    assert_eq!(
        live, offline.hash,
        "the callback render must equal the offline render of identical input"
    );
    // Two silent buffers would agree, so the peak is part of the assertion
    assert!(offline.peak > 0.0);
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

// I-6
#[test]
fn a_non_finite_velocity_cannot_reach_a_device() {
    let (mut sender, mut bridge) = playing_bridge(&[]);
    sender
        .send_note(NoteEvent {
            frame_offset: 0,
            sequence: 0,
            kind: NoteEventKind::On {
                id: 1,
                channel: 0,
                note: 60,
                velocity: f32::NAN,
            },
        })
        .unwrap();

    let mut buffer = vec![1.0_f32; FRAMES * CHANNELS as usize];
    render(&mut bridge, &mut buffer);

    // NoteEvent::is_valid refuses it before any device sees it, and the block is exact silence
    assert_eq!(bridge.telemetry().plan_errors(), 1);
    assert!(buffer.iter().all(|sample| *sample == 0.0));
    assert_eq!(bridge.telemetry().blocks_rendered(), 0);
}

// I-7
#[test]
fn a_frame_capacity_refusal_does_not_advance_the_playhead() {
    let (_sender, mut bridge) = playing_bridge(&[(0, 16 * BEAT, 45)]);

    let mut oversized = vec![1.0_f32; FRAMES * 4 * CHANNELS as usize];
    render(&mut bridge, &mut oversized);
    assert_eq!(bridge.telemetry().frame_capacity_rejections(), 1);
    assert_eq!(
        bridge.telemetry().blocks_rendered(),
        0,
        "render returns before its increment on the refusal path"
    );
    assert!(oversized.iter().all(|sample| *sample == 0.0));
    // The refusal returns before the transport advances, so the playhead has not moved
    assert_eq!(bridge.transport().position, SampleTime(0));

    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];
    render(&mut bridge, &mut buffer);
    assert_eq!(bridge.telemetry().blocks_rendered(), 1);
    assert_eq!(bridge.transport().position, SampleTime(FRAMES as i64));
}

// I-8
#[test]
fn a_seek_releases_every_sounding_clip_note() {
    let (mut sender, mut bridge) = playing_bridge(&[(0, 64 * BEAT, 45)]);
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];

    render(&mut bridge, &mut buffer);
    render(&mut bridge, &mut buffer);
    assert!(peak(&buffer) > 0.0, "the note must be sounding first");

    // Seek far past the note's span, so nothing re-attacks
    sender
        .send_transport(TransportCommand::Seek(SampleTime(80 * 24_000)))
        .unwrap();
    render(&mut bridge, &mut buffer);
    render(&mut bridge, &mut buffer);

    assert_eq!(
        peak(&buffer),
        0.0,
        "a seek must release what the playhead left, not leave it hanging"
    );
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

// I-9
#[test]
fn a_schedule_swap_reclaims_on_the_app_thread() {
    let (mut sender, mut bridge) = playing_bridge(&[(0, BEAT, 45)]);
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];

    for index in 0..5_i64 {
        sender
            .send_schedule(0, Box::new(schedule_of(&[(index * BEAT, BEAT, 45)])))
            .unwrap();
        render(&mut bridge, &mut buffer);
        // Reclaim on the app thread every block, so the render thread never has to hold one
        assert_eq!(sender.reclaim(), 1, "block {index} retired one schedule");
    }

    assert_eq!(bridge.telemetry().schedules_installed(), 5);
    assert_eq!(
        bridge.telemetry().schedules_held(),
        0,
        "the reclaim lane had room on every block"
    );
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

// I-10
#[test]
fn a_full_schedule_lane_returns_the_schedule() {
    let (mut sender, mut bridge) = playing_bridge(&[]);

    // Publish until the lane refuses, with no arithmetic on the count
    let mut accepted = 0;
    let refused = loop {
        let schedule = Box::new(schedule_of(&[(0, BEAT, 45)]));
        match sender.send_schedule(0, schedule) {
            Ok(()) => accepted += 1,
            Err(rejected) => break rejected,
        }
        assert!(accepted <= 16, "the lane must refuse rather than grow");
    };

    // Pins the constant to the lane the primitive actually delivers
    assert_eq!(accepted, SCHEDULE_LANE_CAPACITY);
    assert_eq!(sender.schedule_lane_capacity(), SCHEDULE_LANE_CAPACITY);
    let (error, returned) = refused;
    assert_eq!(error, ControlError::ScheduleLaneFull);
    // Counted, not dropped: the box comes back intact and its notes are still readable
    assert_eq!(returned.len(), 1);
    assert_eq!(sender.telemetry().schedule_overflows(), 1);

    // Draining one makes room for exactly one more
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];
    render(&mut bridge, &mut buffer);
    assert_eq!(sender.reclaim(), 1);
    assert!(sender.send_schedule(0, returned).is_ok());
    assert!(sender
        .send_schedule(0, Box::new(schedule_of(&[(0, BEAT, 45)])))
        .is_err());
    assert_eq!(sender.telemetry().schedule_overflows(), 2);
}

// The positive control. Without it a broken guard would report success below
#[test]
fn the_guard_detects_a_deliberate_allocation() {
    let (_, violations) = rt_section(|| {
        let probe: Vec<u8> = Vec::with_capacity(64);
        std::hint::black_box(&probe);
    });
    assert!(
        violations > 0,
        "the guard did not fire on a real allocation"
    );
}

// I-11
#[test]
fn render_with_a_clip_player_allocates_nothing() {
    let dense: Vec<(i64, i64, u8)> = (0..64)
        .map(|index| (index * (BEAT / 4), BEAT / 8, 40 + (index % 30) as u8))
        .collect();
    let (mut sender, mut bridge) = playing_bridge(&dense);
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];

    // Two unguarded blocks: the first drains the install's release, the second settles the plan
    render(&mut bridge, &mut buffer);
    render(&mut bridge, &mut buffer);

    for block in 0..1_000_u64 {
        sender.send_note(live_note(block, 0, 55)).unwrap();
        let ((), violations) = rt_section(|| {
            let mut frame = RenderBlock::new(&mut buffer, CHANNELS);
            bridge.render(&mut frame);
        });
        assert_eq!(violations, 0, "render allocated on block {block}");
    }
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

// I-12
#[test]
fn a_deactivated_clip_produces_no_events() {
    // A deactivated placement contributes no notes to the bake, so the schedule it produces is
    // empty. The app-side filter is what makes that true; this pins the render-side consequence
    let (_sender, mut bridge) = playing_bridge(&[]);
    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];

    for _ in 0..100 {
        render(&mut bridge, &mut buffer);
        assert_eq!(peak(&buffer), 0.0);
    }
    assert_eq!(bridge.telemetry().last_block_events(), 0);
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

// The playhead the transport readout draws. Published every block off thread, so the app can
// report a position without reading render state -- and NOT published before a block renders,
// because a fabricated 1.1.1 is exactly what r4-qa-protocol.md row 4 exists to catch
#[test]
fn the_playhead_is_published_off_thread_and_advances_while_rolling() {
    let (_sender, mut bridge) = playing_bridge(&[(0, 960, 60)]);
    let telemetry = bridge.telemetry();

    assert_eq!(
        telemetry.position_samples(),
        None,
        "no block has rendered, so there is no position to report"
    );

    let mut buffer = vec![0.0_f32; FRAMES * CHANNELS as usize];
    render(&mut bridge, &mut buffer);
    let first = telemetry
        .position_samples()
        .expect("a rendered block publishes a position");
    assert!(telemetry.transport_rolling(), "the bridge was sent Play");

    render(&mut bridge, &mut buffer);
    let second = telemetry
        .position_samples()
        .expect("a rendered block publishes a position");
    assert_eq!(
        second - first,
        FRAMES as i64,
        "the playhead must advance by exactly one block"
    );
}
