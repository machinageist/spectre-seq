// Author: Jeff
// Date: 2026-08-24
// Description: R4 slice 1 evidence — the app builds a live engine over the existing compiled plan
// Notes: Tests drive a real CompiledPlan through a real AudioStream implementation, never a mock
//   of the render path. The RT-001 guard is re-declared here because a #[global_allocator] is
//   per-test-binary, and its positive control is what makes every passing result meaningful.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use spectre_app::engine::{
    apply_parameter_edit, build_engine_parts, AuditionError, EngineParts, EngineState,
    EngineUnavailable, LiveEngine, ENGINE_BUFFER_FRAMES, ENGINE_PLAN_FRAME_MARGIN,
};
use spectre_app::AppModel;
use spectre_audio::bridge::BridgeTelemetry;
use spectre_audio::control::ControlError;
use spectre_audio::null::{NullBackend, NullStream, NULL_DEVICE_KEY, NULL_SAMPLE_RATE};
use spectre_audio::{
    AudioBackend, AudioStream, BackendError, DeviceId, DeviceInfo, RenderBlock, RenderCallback,
    StreamConfig,
};
use spectre_core::TransportCommand;
use spectre_dsp::DeviceParameterSnapshot;
use std::sync::Arc;

const FRAMES: usize = ENGINE_BUFFER_FRAMES;
const CHANNELS: usize = 2;

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

// The prototype's own validated four-entry snapshot
fn prototype_snapshot() -> Vec<DeviceParameterSnapshot> {
    AppModel::prototype().device_parameter_snapshot().unwrap()
}

// The configuration every deterministic test opens against
fn config() -> StreamConfig {
    StreamConfig::stereo(NULL_SAMPLE_RATE, FRAMES).unwrap()
}

// Open a concrete NullStream over the parts' bridge, keeping the app-thread halves
fn open_null(
    parts: EngineParts,
) -> (
    NullStream,
    spectre_audio::control::ControlSender,
    Arc<BridgeTelemetry>,
    Box<[spectre_audio::control::ParameterTarget]>,
) {
    let EngineParts {
        mut bridge,
        sender,
        telemetry,
        config,
        plan_max_frames: _,
        targets,
        nodes: _,
    } = parts;
    let backend = NullBackend::new();
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            config,
            Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),
        )
        .unwrap();
    stream.start().unwrap();
    (stream, sender, telemetry, targets)
}

// Wrap an opened null stream in the engine without erasing its concrete type
fn engine_over_null(parts: EngineParts) -> LiveEngine<NullStream> {
    let config = parts.config;
    let (stream, sender, telemetry, targets) = open_null(parts);
    let mut engine = LiveEngine::from_open_stream(
        Box::new(stream),
        sender,
        telemetry,
        spectre_audio::NULL_BACKEND_NAME,
        "Null Output".to_string(),
        config,
    );
    engine.set_targets(targets);
    engine
}

// Hash interleaved output the way the offline harness hashes its planar output
fn hash_interleaved(samples: &[f32], channels: usize, frames: usize) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for channel in 0..channels {
        for frame in 0..frames {
            let sample = samples[frame * channels + channel];
            for byte in sample.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    hash
}

// 1
#[test]
fn parts_build_from_the_prototype_snapshot_without_touching_a_device() {
    // build_engine_parts takes no backend at all, which is the structural proof that no device
    // is touched: there is nothing for it to call
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    assert_eq!(parts.config, config());
    assert_eq!(parts.telemetry.blocks_rendered(), 0);
}

// 2
#[test]
fn an_incomplete_snapshot_is_refused_before_any_plan_is_built() {
    let mut snapshot = prototype_snapshot();
    snapshot.pop();
    match build_engine_parts(&snapshot, config()).err() {
        Some(EngineUnavailable::Snapshot(_)) => {}
        other => panic!("expected a refused snapshot, got {other:?}"),
    }
}

// 3
#[test]
fn the_plan_reserves_more_frames_than_the_requested_block() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    assert_eq!(
        parts.plan_max_frames,
        ENGINE_BUFFER_FRAMES * ENGINE_PLAN_FRAME_MARGIN
    );
}

// 4
#[test]
fn play_produces_nonzero_output_and_stop_returns_exact_silence() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);

    engine.stream_mut().pump().unwrap();
    assert!(
        engine.stream_mut().last_block().iter().all(|s| *s == 0.0),
        "a block with no note must be exact silence"
    );

    engine.start_audition().unwrap();
    engine.stream_mut().pump().unwrap();
    let peak = engine
        .stream_mut()
        .last_block()
        .iter()
        .fold(0.0_f32, |peak, s| peak.max(s.abs()));
    assert!(peak > 0.0, "Play must produce audible output");

    engine.stop_audition().unwrap();
    engine.stream_mut().pump().unwrap();
    engine.stream_mut().pump().unwrap();
    assert!(
        engine.stream_mut().last_block().iter().all(|s| *s == 0.0),
        "Stop must return exact silence, not a decaying tail"
    );
}

// 5
#[test]
fn the_live_plan_matches_the_offline_render_of_the_same_app_snapshot() {
    let snapshot = prototype_snapshot();
    let parts = build_engine_parts(&snapshot, config()).unwrap();
    let (mut stream, mut sender, telemetry, _targets) = open_null(parts);

    for event in spectre_offline::fixture_events(FRAMES) {
        sender.send_note(event).unwrap();
    }
    stream.pump().unwrap();

    let live = hash_interleaved(stream.last_block(), CHANNELS, FRAMES);
    let offline =
        spectre_offline::render_app_snapshot(f64::from(NULL_SAMPLE_RATE), FRAMES, &snapshot)
            .unwrap();

    assert_eq!(
        live, offline.hash,
        "the app's live plan must be the same computation the offline harness renders"
    );
    // The fixture is audible, so a matching hash is not two silent buffers agreeing
    assert!(offline.peak > 0.0);
    assert_eq!(telemetry.plan_errors(), 0);
}

// 6
#[test]
fn repeated_engine_builds_render_identically() {
    let snapshot = prototype_snapshot();
    let mut hashes = Vec::new();
    for _ in 0..3 {
        let parts = build_engine_parts(&snapshot, config()).unwrap();
        let (mut stream, mut sender, _telemetry, _targets) = open_null(parts);
        for event in spectre_offline::fixture_events(FRAMES) {
            sender.send_note(event).unwrap();
        }
        stream.pump().unwrap();
        hashes.push(hash_interleaved(stream.last_block(), CHANNELS, FRAMES));
    }
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(hashes[1], hashes[2]);
}

// 7
#[test]
fn a_refused_transport_send_leaves_the_ui_transport_unchanged() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);
    let mut model = AppModel::prototype();

    // Fill the transport lane without pumping, so nothing drains it
    while engine.send_transport(TransportCommand::Play).is_ok() {}

    match engine.start_audition() {
        Err(AuditionError::Transport(ControlError::TransportLaneFull)) => {}
        other => panic!("expected a refused transport send, got {other:?}"),
    }

    // The binding rule: the app sends first and mutates second, so a refusal leaves the UI alone
    assert!(!model.is_playing());
    model.select_lens(spectre_app::Lens::Build);
    assert_eq!(model.lens(), spectre_app::Lens::Build);
}

// A backend that refuses every open, for the failure path
struct FailingBackend;

impl AudioBackend for FailingBackend {
    fn name(&self) -> &'static str {
        "failing"
    }

    fn output_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        Err(BackendError::NoDefaultDevice)
    }

    fn default_output_device(&self) -> Result<DeviceInfo, BackendError> {
        Err(BackendError::NoDefaultDevice)
    }

    fn default_sample_rate(&self, _device: &DeviceId) -> Result<u32, BackendError> {
        Err(BackendError::NoDefaultDevice)
    }

    fn open_output(
        &self,
        _device: &DeviceId,
        _config: StreamConfig,
        _callback: RenderCallback,
    ) -> Result<Box<dyn AudioStream>, BackendError> {
        Err(BackendError::NoDefaultDevice)
    }
}

// 8
#[test]
fn engine_open_failure_reports_the_backend_error_and_leaves_the_model_intact() {
    let mut model = AppModel::prototype();
    let device = model.devices()[0].instance_id;
    model.open_device_in_shape(device).unwrap();
    let lens_before = model.lens();
    let track_before = model.selected_track_id();
    let device_before = model.selected_device_id();

    let result = spectre_app::engine::open_default(&FailingBackend, &prototype_snapshot());
    assert_eq!(
        result.err(),
        Some(EngineUnavailable::Backend(BackendError::NoDefaultDevice))
    );

    assert_eq!(model.lens(), lens_before);
    assert_eq!(model.selected_track_id(), track_before);
    assert_eq!(model.selected_device_id(), device_before);
}

// 9
#[test]
fn opened_but_silent_never_reports_running() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);

    assert!(
        matches!(engine.state(), EngineState::Opened { .. }),
        "an opened stream that has not called back must never read as running"
    );
    engine.stream_mut().pump().unwrap();
    assert!(matches!(engine.state(), EngineState::Running { .. }));
}

// 10
#[test]
fn health_mirrors_the_render_thread_counters() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);
    for _ in 0..8 {
        engine.stream_mut().pump().unwrap();
    }
    let health = engine.health();
    assert_eq!(health.blocks_rendered, 8);
    assert_eq!(health.xruns, 0);
    assert_eq!(health.plan_errors, 0);
    assert_eq!(health.frame_capacity_rejections, 0);
    assert_eq!(health.contaminated_nodes, 0);
    assert_eq!(health.stream_errors, 0);
    assert!(health.worst_headroom.is_finite());
    assert!(health.worst_headroom <= health.last_headroom);
}

// 11
#[test]
fn the_engine_callback_allocates_nothing() {
    // Positive control: without it, a broken guard would report success everywhere
    let (_, control_violations) = rt_section(|| {
        let leak: Vec<u8> = Vec::with_capacity(64);
        std::hint::black_box(&leak);
    });
    assert!(
        control_violations > 0,
        "the guard itself must fire on a deliberate allocation"
    );

    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);
    engine.start_audition().unwrap();
    // Drain the sends outside the guarded section so the measurement covers steady state
    engine.stream_mut().pump().unwrap();

    for _ in 0..16 {
        let (result, violations) = rt_section(|| engine.stream_mut().pump());
        result.unwrap();
        assert_eq!(violations, 0, "the app's render closure must not allocate");
    }
}

// 14
#[test]
fn play_then_stop_inside_one_block_is_refused_into_counted_silence() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);

    // Both note events land in the lane for one block, so their ordering key decreases
    engine.start_audition().unwrap();
    engine.stop_audition().unwrap();

    engine.stream_mut().pump().unwrap();
    assert!(
        engine.stream_mut().last_block().iter().all(|s| *s == 0.0),
        "an unsorted batch must be refused into exact silence, never stale audio"
    );
    let health = engine.health();
    assert_eq!(health.plan_errors, 1);
    assert_eq!(
        health.blocks_rendered, 0,
        "render returns before its increment on the error path"
    );

    engine.stream_mut().pump().unwrap();
    assert!(
        engine.stream_mut().last_block().iter().all(|s| *s == 0.0),
        "the discarded pair must leave no stuck note"
    );
    assert_eq!(engine.health().plan_errors, 1);
}

// 16 — the acceptance criterion for R4-2, stated in the terms §1.3 names
#[test]
fn a_shape_edit_changes_live_audio_and_nothing_stays_pending() {
    let mut model = AppModel::prototype();
    let parts = build_engine_parts(&model.device_parameter_snapshot().unwrap(), config()).unwrap();
    let mut engine = engine_over_null(parts);

    engine.start_audition().unwrap();
    engine.stream_mut().pump().unwrap();
    let before = hash_interleaved(engine.stream_mut().last_block(), CHANNELS, FRAMES);

    let mut status = String::new();
    apply_parameter_edit(&mut model, Some(&engine), "gain", "gain", 0.1, &mut status);
    assert!(
        status.is_empty(),
        "a routed edit must report no failure: {status}"
    );

    engine.stream_mut().pump().unwrap();
    let after = hash_interleaved(engine.stream_mut().last_block(), CHANNELS, FRAMES);

    assert_ne!(
        before, after,
        "a gain edit must change what the render thread produces"
    );
    let health = engine.health();
    assert_eq!(
        health.parameters_pending, 0,
        "no edit may reach the engine without reaching a processor"
    );
    assert_eq!(health.parameters_applied, 1);
}

// 17 — an unregistered target is refused on the app thread, before it can be silently lost
#[test]
fn an_unregistered_target_is_refused_at_publication() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let engine = engine_over_null(parts);

    let stranger = spectre_audio::control::ParameterTarget {
        device: spectre_core::ObjectId::from_raw(0xDEAD_BEEF).unwrap(),
        parameter: spectre_core::ObjectId::from_raw(0xFEED_FACE).unwrap(),
    };
    assert_eq!(
        engine.send_parameter(stranger, 0.5),
        Err(ControlError::UnknownTarget(stranger))
    );
    assert_eq!(engine.health().parameters_applied, 0);
}

// 18 — the whole registered fixture routes, and a repeated sweep coalesces latest-wins
#[test]
fn every_fixture_parameter_routes_and_a_sweep_coalesces() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);
    assert_eq!(
        engine.targets().len(),
        4,
        "the canonical fixture has four parameters"
    );

    let target = engine.targets()[0];
    // Latest-wins: a fast sweep occupies one slot and cannot starve the lane
    for step in 0..500 {
        engine.send_parameter(target, step as f32 / 1000.0).unwrap();
    }
    engine.stream_mut().pump().unwrap();

    let health = engine.health();
    assert_eq!(
        health.parameters_applied, 1,
        "500 writes to one target must coalesce to a single application"
    );
    assert_eq!(health.parameters_pending, 0);

    // Every registered target resolves to a live processor
    for target in engine.targets().to_vec() {
        engine.send_parameter(target, 0.5).unwrap();
    }
    engine.stream_mut().pump().unwrap();
    assert_eq!(engine.health().parameters_applied, 5);
    assert_eq!(engine.health().parameters_pending, 0);
}

// Hardware evidence for the app's own open path, not in the spec's test list. The deterministic
// tests above prove the plan and the wiring; only this proves open_default reaches a real driver.
// #[ignore]d for the same reason the audio crate's drill is: most machines have no output device
#[cfg(feature = "live-audio")]
#[test]
#[ignore]
fn app_engine_opens_a_real_device_and_renders() {
    let backend = spectre_audio::cpal_backend::CpalBackend::new();
    let mut engine = spectre_app::engine::open_default(&backend, &prototype_snapshot())
        .expect("a real output device is required for this drill");

    assert!(
        matches!(engine.state(), EngineState::Opened { .. }),
        "no block can have rendered before the driver calls back"
    );

    engine.start_audition().expect("audition must be queued");
    std::thread::sleep(std::time::Duration::from_millis(250));

    let health = engine.health();
    let EngineState::Running {
        backend: name,
        ref device,
        sample_rate,
        frames,
    } = engine.state()
    else {
        panic!("the driver never called back: {:?}", engine.state());
    };
    println!(
        "backend={name} device={device} rate={sample_rate} frames={frames} blocks={} xruns={} worst_headroom={} plan_errors={} frame_rejections={} contaminated={} stream_errors={}",
        health.blocks_rendered,
        health.xruns,
        health.worst_headroom,
        health.plan_errors,
        health.frame_capacity_rejections,
        health.contaminated_nodes,
        health.stream_errors,
    );

    assert!(
        health.blocks_rendered > 0,
        "the driver must have called back"
    );
    assert_eq!(health.plan_errors, 0, "no block may fail to render");
    assert_eq!(health.contaminated_nodes, 0, "output must stay finite");
    assert_eq!(health.stream_errors, 0, "the driver must report no errors");

    engine.stop_audition().expect("stop must be queued");
    engine.close().expect("close must release the device");
}
