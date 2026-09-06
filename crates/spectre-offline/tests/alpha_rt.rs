// Author: Jeff
// Date: 2026-08-29
// Description: RT-001 allocation evidence for the composed alpha, not for a three-node chain
// Notes: rt_guard.rs guards Pulse -> Gain -> Saturator and a Filament -> Gloam pair; clip_bridge.rs
//   guards the same three-node fixture. Neither is the plan ./spectre runs. This guards the alpha
//   as R4-9 specifies it: three tracks of instrument -> Gloam insert -> track gain, a sum bus, a
//   master, driven through RenderBridge with a clip player on the primary note node and a clip
//   voice for every other instrument track.
//   RT-001 is 35% of the AAA grading weight and its evidence covered a workload the product does
//   not have -- the same gap the headroom record carried until it was measured on the real plan.

use spectre_audio::bridge::{RenderBridge, DEFAULT_NOTE_SCRATCH};
use spectre_audio::clip::{ClipPlayer, ClipSchedule, CLIP_EVENT_RESERVE};
use spectre_audio::control::{control_channel, ParameterTarget};
use spectre_audio::route::{ParameterRoute, ParameterRoutes};
use spectre_audio::RenderBlock;
use spectre_core::{IdGen, SampleRate, TempoMap};
use spectre_dsp::GAIN_PARAMETERS;
use spectre_project::{
    build_track_graph, from_bytes, migrate_to_current, track_device_factory, ProjectEnvelope,
    TrackList,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

const FIXTURE: &[u8] = include_bytes!("fixtures/r4-alpha.json");
const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;
const CHANNELS: usize = 2;
const GRAPH_SEED: u64 = 0x0045_3245_414c_5048;

thread_local! {
    // Const-initialized so touching the flag inside the allocator cannot itself allocate
    static IN_RT_SECTION: Cell<bool> = const { Cell::new(false) };
    static VIOLATIONS: Cell<u64> = const { Cell::new(0) };
}

// Allocator that attributes any traffic inside an RT section as an RT-001 violation
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

// Run one closure as a callback-reachable section and report violations recorded inside it
fn rt_section<T>(body: impl FnOnce() -> T) -> (T, u64) {
    let start = VIOLATIONS.with(Cell::get);
    IN_RT_SECTION.with(|flag| flag.set(true));
    let value = body();
    IN_RT_SECTION.with(|flag| flag.set(false));
    (value, VIOLATIONS.with(Cell::get) - start)
}

fn fixture() -> ProjectEnvelope {
    // The fixture is a checked-in SCHEMA 2 document and is deliberately left that way: it is
    // the project's only regression evidence that a file written before the effect chain
    // existed still loads with its effects intact. Migration is what the shell's own adopt
    // path runs, so running it here tests the product's route rather than a shortcut
    migrate_to_current(from_bytes(FIXTURE).expect("the checked-in alpha fixture must decode"))
        .expect("the schema-2 fixture migrates")
        .envelope
}

// Bake one track's clip material through the one tick-to-sample conversion the workspace has
fn schedule_for(tracks: &TrackList, index: usize, tempo: &TempoMap) -> ClipSchedule {
    let track = &tracks.tracks()[index];
    let mut notes = Vec::new();
    for placement in track.clips().placements() {
        if !placement.is_active() {
            continue;
        }
        let clip = tracks
            .clip(placement.clip())
            .expect("a placement names a clip in the project");
        for note in clip.notes() {
            notes.push((
                spectre_core::BeatTicks(placement.start().0 + note.start().0),
                note.length(),
                note.channel(),
                note.note(),
                note.velocity(),
            ));
        }
    }
    ClipSchedule::bake(
        notes.into_iter(),
        tempo,
        SampleRate::new(SAMPLE_RATE as u32).unwrap(),
        CLIP_EVENT_RESERVE,
    )
    .expect("the fixture's clip material bakes")
}

// The composed alpha with every track's clips attached, the way ./spectre builds it
fn alpha_bridge(envelope: &ProjectEnvelope) -> RenderBridge {
    let tracks = &envelope.project.tracks;
    let mut ids = IdGen::new(GRAPH_SEED);
    let (graph, nodes) = build_track_graph(tracks, &mut ids).expect("the fixture graph compiles");
    let mut factory = track_device_factory(tracks, &nodes);
    let plan = graph
        .compile(nodes.master.node, FRAMES * 2, &mut factory)
        .expect("the fixture plan compiles");
    let instruments: Vec<_> = nodes.instruments.iter().map(|entry| entry.node).collect();

    let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut primary = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = primary.install(Box::new(schedule_for(
        tracks,
        0,
        &envelope.project.tempo_map,
    )));
    let voices: Vec<_> = (1..tracks.len())
        .map(|index| {
            let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
            let _ = player.install(Box::new(schedule_for(
                tracks,
                index,
                &envelope.project.tempo_map,
            )));
            (instruments[index], player)
        })
        .collect();

    let bridge = RenderBridge::new(
        plan,
        receiver,
        instruments[0],
        SAMPLE_RATE,
        DEFAULT_NOTE_SCRATCH,
    )
    .with_clip_player(primary)
    .with_clip_voices(voices, DEFAULT_NOTE_SCRATCH);
    sender
        .send_transport(spectre_core::TransportCommand::Play)
        .unwrap();
    bridge
}

// Positive control: without it a guard that never fires is indistinguishable from a clean render
#[test]
fn the_guard_detects_a_deliberate_allocation() {
    let (_, violations) = rt_section(|| {
        let leak = vec![0.0_f32; 64];
        leak.len()
    });
    assert!(
        violations > 0,
        "the guard did not observe a real allocation"
    );
}

// The alpha, rendered under the guard, across enough blocks to cover its material AND its tail --
// so voice draining, note emission, the sum bus, four Gain instances and three Gloam inserts all
// run inside the guarded section
#[test]
fn rendering_the_composed_alpha_allocates_nothing_on_the_callback_path() {
    let envelope = fixture();
    let mut bridge = alpha_bridge(&envelope);
    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS];

    // One block outside the section: the first render may still touch lazily-built state, and
    // RT-001 is a claim about the steady state a driver actually sees
    let mut warm = RenderBlock::new(&mut interleaved, CHANNELS as u16);
    bridge.render(&mut warm);

    let ((), violations) = rt_section(|| {
        for _ in 0..64 {
            let mut block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
            bridge.render(&mut block);
        }
    });
    assert_eq!(
        violations, 0,
        "the composed alpha allocated on the callback path"
    );

    // A silent render would satisfy the count above without exercising the voices
    assert!(
        bridge.telemetry().session_peak() > 0.0,
        "the guarded render carried no signal, so it proved nothing"
    );
    assert_eq!(bridge.telemetry().plan_errors(), 0);
    assert_eq!(bridge.telemetry().clip_events_refused(), 0);
}

// A gain edit is the one thing that makes Gain take its ramp path rather than its flat path, and
// the ramp is new as of D-R3's closure. Applying a drained parameter and ramping through it must
// not allocate either.
//
// This registers real lane targets and sends a real edit. An earlier version of this test built
// the bridge with no targets, so no parameter could ever arrive, Gain never left its flat path,
// and the test was a second copy of the one above wearing a different name
#[test]
fn a_gain_ramp_allocates_nothing_on_the_callback_path() {
    let envelope = fixture();
    let tracks = &envelope.project.tracks;
    let mut ids = IdGen::new(GRAPH_SEED);
    let (graph, nodes) = build_track_graph(tracks, &mut ids).expect("the fixture graph compiles");
    let mut factory = track_device_factory(tracks, &nodes);
    let plan = graph
        .compile(nodes.master.node, FRAMES * 2, &mut factory)
        .expect("the fixture plan compiles");

    // The master gain: one target, addressed the way the app addresses it
    let pairs = nodes.parameter_targets();
    let index = tracks.master_target_index();
    let (device, parameter) = pairs[index];
    let target = ParameterTarget { device, parameter };
    let routes = ParameterRoutes::new(vec![ParameterRoute {
        target,
        node: nodes.master.node,
        key: GAIN_PARAMETERS[0].key,
    }])
    .expect("one route is valid");

    let (mut sender, receiver) = control_channel(&[target], 64, 8).unwrap();
    let mut bridge = RenderBridge::with_parameter_routes(
        plan,
        receiver,
        nodes.instruments[0].node,
        SAMPLE_RATE,
        DEFAULT_NOTE_SCRATCH,
        routes,
    );
    sender
        .send_transport(spectre_core::TransportCommand::Play)
        .unwrap();

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS];
    let mut warm = RenderBlock::new(&mut interleaved, CHANNELS as u16);
    bridge.render(&mut warm);

    // Queued before the guarded section; the bridge drains and applies it on the next block, so
    // the ramp runs inside the guard
    // Slot 0: the target set registered above holds exactly one
    // Slot 0: the target set registered above holds exactly one
    sender
        .parameters()
        .set(0, 0.25)
        .expect("the lane accepts a registered target");

    let ((), violations) = rt_section(|| {
        for _ in 0..8 {
            let mut block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
            bridge.render(&mut block);
        }
    });
    assert_eq!(violations, 0, "the ramping path allocated");
    assert_eq!(
        bridge.telemetry().parameters_applied(),
        1,
        "the edit never reached a processor, so no ramp was exercised"
    );
    assert_eq!(bridge.telemetry().parameters_pending(), 0);
}
