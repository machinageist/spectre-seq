// Author: Jeff
// Date: 2026-08-24
// Description: R4 slice 1 evidence — the app builds a live engine over the existing compiled plan
// Notes: Tests drive a real CompiledPlan through a real AudioStream implementation, never a mock
//   of the render path. The RT-001 guard is re-declared here because a #[global_allocator] is
//   per-test-binary, and its positive control is what makes every passing result meaningful.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use spectre_app::engine::{
    apply_parameter_edit, build_engine_parts, build_track_engine_parts, engine_status_field,
    toggle_transport, AuditionError, EngineParts, EngineState, EngineUnavailable, LiveEngine,
    ENGINE_BUFFER_FRAMES, ENGINE_PLAN_FRAME_MARGIN,
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
    bool,
) {
    let EngineParts {
        mut bridge,
        sender,
        telemetry,
        config,
        plan_max_frames: _,
        targets,
        nodes: _,
        has_clips,
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
    (stream, sender, telemetry, targets, has_clips)
}

// Wrap an opened null stream in the engine without erasing its concrete type
fn engine_over_null(parts: EngineParts) -> LiveEngine<NullStream> {
    let config = parts.config;
    let (stream, sender, telemetry, targets, has_clips) = open_null(parts);
    let mut engine = LiveEngine::from_open_stream(
        Box::new(stream),
        sender,
        telemetry,
        spectre_audio::NULL_BACKEND_NAME,
        "Null Output".to_string(),
        config,
    );
    engine.set_targets(targets);
    // Set here for the same reason open_with_parts sets it: a LiveEngine that forgot the flag
    // would audition over its own clips, and the one-block Play/Stop test is what caught this
    // helper doing exactly that
    engine.set_has_clips(has_clips);
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
    let (mut stream, mut sender, telemetry, _targets, _) = open_null(parts);

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
        let (mut stream, mut sender, _telemetry, _targets, _) = open_null(parts);
        for event in spectre_offline::fixture_events(FRAMES) {
            sender.send_note(event).unwrap();
        }
        stream.pump().unwrap();
        hashes.push(hash_interleaved(stream.last_block(), CHANNELS, FRAMES));
    }
    assert_eq!(hashes[0], hashes[1]);
    assert_eq!(hashes[1], hashes[2]);
}

// 7 — REWRITTEN after R4-1's implementation review found the original could not fail: it built
// an AppModel, never passed it to the rule under test, and asserted a fresh prototype was not
// playing. This version drives the real rule and asserts the model it was actually given
#[test]
fn a_refused_transport_send_leaves_the_ui_transport_unchanged() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);
    let mut model = AppModel::prototype();
    let mut status = String::new();

    // Sanity: with a working lane the rule flips the UI, so the assertion below is about the
    // refusal and not about the rule never flipping anything
    toggle_transport(&mut model, Some(&mut engine), &mut status).unwrap();
    assert!(model.is_playing(), "an accepted send must flip the UI");
    toggle_transport(&mut model, Some(&mut engine), &mut status).unwrap();
    assert!(!model.is_playing());

    // Fill the transport lane without pumping, so nothing drains it
    while engine.send_transport(TransportCommand::Play).is_ok() {}

    let refused = toggle_transport(&mut model, Some(&mut engine), &mut status);
    assert_eq!(
        refused,
        Err(AuditionError::Transport(ControlError::TransportLaneFull))
    );
    assert!(
        !model.is_playing(),
        "a refused transport send must leave the UI transport unchanged"
    );
    assert!(
        status.contains("Nothing changed"),
        "the refusal must be reported to the user: {status}"
    );
}

// 7b — the other half of the binding rule: a refused NOTE after an accepted transport command
// still flips the UI, because a queued command cannot be recalled and a UI reading "stopped"
// while the render transport plays is the worse divergence
#[test]
fn a_refused_note_after_an_accepted_transport_command_still_flips_the_ui() {
    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);
    let mut model = AppModel::prototype();
    let mut status = String::new();

    // Fill the note lane only; the transport lane stays open
    while engine
        .send_note(spectre_dsp::NoteEventKind::AllNotesOff { channel: Some(0) })
        .is_ok()
    {}

    let result = toggle_transport(&mut model, Some(&mut engine), &mut status);
    assert!(
        matches!(result, Err(AuditionError::Note(_))),
        "expected a refused note, got {result:?}"
    );
    assert!(
        model.is_playing(),
        "the transport command was queued, so the UI must match what the render thread will see"
    );
}

// 7c — with no engine at all the prototype behaves exactly as it did before R4-1
#[test]
fn the_transport_rule_still_works_with_no_engine() {
    let mut model = AppModel::prototype();
    let mut status = String::new();
    let engine: Option<&mut LiveEngine<NullStream>> = None;

    toggle_transport(&mut model, engine, &mut status).unwrap();
    assert!(model.is_playing());
    assert!(status.is_empty(), "no engine is not an error condition");
}

// 7d — the smoke line's engine field is derived, not written. This is what makes the assertion
// in smoke_cli.rs falsifiable: it changes if the headless path ever opens a device
#[test]
fn the_smoke_engine_field_tracks_real_engine_state() {
    let none: Option<&LiveEngine<NullStream>> = None;
    assert_eq!(engine_status_field(none), "not-started");

    let parts = build_engine_parts(&prototype_snapshot(), config()).unwrap();
    let mut engine = engine_over_null(parts);
    assert_eq!(engine_status_field(Some(&engine)), "opened");

    engine.stream_mut().pump().unwrap();
    assert_eq!(engine_status_field(Some(&engine)), "running");
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

// R4 — the app's own engine plays the project's clips, not an audition note.
// This is the last half of "a MIDI clip plays through a track into master": the offline path and
// the bridge both compose, and this is what says the SHELL does
#[test]
fn a_project_with_clips_plays_its_own_material_rather_than_the_audition_note() {
    use spectre_app::engine::build_track_engine_parts;
    use spectre_core::{BeatTicks, IdGen, TempoMap};
    use spectre_project::{ClipNote, ClipPlacement, MidiClip, Track, TrackInstrument, TrackList};

    let mut ids = IdGen::new(0x0043_4c49_5041_5050);
    let mut tracks = TrackList::new();
    let track_id = ids.next_id();
    let clip_id = ids.next_id();
    let placement_id = ids.next_id();

    // 160 ticks at 120 BPM is 4,000 samples: material that sounds well inside the blocks below
    let mut clip = MidiClip::new(clip_id, "Take", BeatTicks(160)).unwrap();
    clip.insert_note(ClipNote::new(BeatTicks(0), BeatTicks(128), 0, 57, 0.8).unwrap())
        .unwrap();
    let mut track = Track::new(track_id, "Lead", TrackInstrument::Filament).unwrap();
    track
        .clips_mut()
        .insert(
            ClipPlacement::new(placement_id, clip_id, BeatTicks(0)).unwrap(),
            BeatTicks(160),
        )
        .unwrap();
    tracks.push(track).unwrap();
    tracks.add_clip(clip).unwrap();

    let tempo = TempoMap::constant(120.0).unwrap();
    let parts = build_track_engine_parts(
        &tracks,
        &tempo,
        0x0041_5050_4752_4148,
        Some(track_id),
        config(),
    )
    .unwrap();
    // The engine reports that it is playing project material, so Play will not audition
    assert!(parts.has_clips);

    let mut engine = engine_over_null(parts);
    assert!(engine.has_clips());
    engine.stream_mut().pump().unwrap();
    assert!(
        engine.stream_mut().last_block().iter().all(|s| *s == 0.0),
        "a stopped transport must render exact silence"
    );

    // Play sends the transport command alone. If it also sent the audition note, the merge would
    // put an all-notes-off before a note-on by rank on Stop and leave a note sounding
    engine.start_audition().unwrap();
    let mut peak = 0.0_f32;
    for _ in 0..4 {
        engine.stream_mut().pump().unwrap();
        peak = engine
            .stream_mut()
            .last_block()
            .iter()
            .fold(peak, |peak, s| peak.max(s.abs()));
    }
    assert!(peak > 0.0, "the project's own clip must sound on Play");
    assert_eq!(engine.health().plan_errors, 0);

    engine.stop_audition().unwrap();
    // Filament releases over its own fall contour rather than cutting to zero the way Pulse
    // does, so silence is asserted after DEV-013's own 64-quantum bound rather than after two
    // blocks. Anything still sounding at that point is a stuck note, not a release
    for _ in 0..64 {
        engine.stream_mut().pump().unwrap();
    }
    assert!(
        engine.stream_mut().last_block().iter().all(|s| *s == 0.0),
        "Stop must return exact silence, not a note the audition path left sounding"
    );
}

// A project with no clip material still auditions, so R4-1's evidence stays valid unchanged
#[test]
fn a_project_without_clips_still_auditions() {
    use spectre_app::engine::build_track_engine_parts;
    use spectre_core::{IdGen, TempoMap};
    use spectre_project::{Track, TrackInstrument, TrackList};

    let mut ids = IdGen::new(0x004e_4f43_4c49_5000);
    let mut tracks = TrackList::new();
    let track_id = ids.next_id();
    tracks
        .push(Track::new(track_id, "Lead", TrackInstrument::Pulse).unwrap())
        .unwrap();

    let tempo = TempoMap::constant(120.0).unwrap();
    let parts = build_track_engine_parts(
        &tracks,
        &tempo,
        0x0041_5050_4752_4148,
        Some(track_id),
        config(),
    )
    .unwrap();
    assert!(!parts.has_clips, "an empty project attaches no clip player");

    let mut engine = engine_over_null(parts);
    engine.start_audition().unwrap();
    engine.stream_mut().pump().unwrap();
    let peak = engine
        .stream_mut()
        .last_block()
        .iter()
        .fold(0.0_f32, |peak, s| peak.max(s.abs()));
    assert!(peak > 0.0, "with no clips, Play must still audition");
}

// The case the replacement rule exists for, and the one the test above cannot reach.
// R4-1's accepted evidence pins that a Play and a Stop landing in ONE block are refused into
// counted silence with no stuck note. With a clip player attached the bridge sorts the block by
// the contract key, and an audition note-on merged into that block would sort AFTER the
// all-notes-off by rank — leaving a note sounding after Stop. The project's material replacing
// the audition note is what keeps that refusal clean, and this is what says so
#[test]
fn play_then_stop_in_one_block_stays_silent_with_clips_attached() {
    use spectre_app::engine::build_track_engine_parts;
    use spectre_core::{BeatTicks, IdGen, TempoMap};
    use spectre_project::{ClipNote, ClipPlacement, MidiClip, Track, TrackInstrument, TrackList};

    let mut ids = IdGen::new(0x0053_5455_4b00_0001);
    let mut tracks = TrackList::new();
    let track_id = ids.next_id();
    let clip_id = ids.next_id();
    let placement_id = ids.next_id();

    let mut clip = MidiClip::new(clip_id, "Take", BeatTicks(160)).unwrap();
    clip.insert_note(ClipNote::new(BeatTicks(0), BeatTicks(128), 0, 57, 0.8).unwrap())
        .unwrap();
    let mut track = Track::new(track_id, "Lead", TrackInstrument::Filament).unwrap();
    track
        .clips_mut()
        .insert(
            ClipPlacement::new(placement_id, clip_id, BeatTicks(0)).unwrap(),
            BeatTicks(160),
        )
        .unwrap();
    tracks.push(track).unwrap();
    tracks.add_clip(clip).unwrap();

    let tempo = TempoMap::constant(120.0).unwrap();
    let parts = build_track_engine_parts(
        &tracks,
        &tempo,
        0x0041_5050_4752_4149,
        Some(track_id),
        config(),
    )
    .unwrap();
    let mut engine = engine_over_null(parts);
    // The engine, not only the parts: a helper that dropped the flag would audition over the
    // project's own clips, which is what this test caught the first time it ran
    assert!(engine.has_clips());

    // Both sends before any callback, so they land in one block
    engine.start_audition().unwrap();
    engine.stop_audition().unwrap();
    for _ in 0..64 {
        engine.stream_mut().pump().unwrap();
    }
    assert!(
        engine.stream_mut().last_block().iter().all(|s| *s == 0.0),
        "a Play and a Stop in one block must leave no note sounding"
    );
    assert_eq!(engine.health().plan_errors, 0);
}

// 26 — the acceptance criterion of test 16, re-asked against the engine ./spectre ACTUALLY opens.
//
// Test 16 builds its engine with build_engine_parts, whose lane targets are derived from the
// model's own device snapshot, so a Shape edit addresses a target that exists by construction.
// main.rs calls open_track_engine, whose targets come from TrackPathNodes::parameter_targets --
// ObjectIds allocated by the graph builder from APP_GRAPH_SEED. The model's device IDs and the
// graph's node IDs are disjoint ID spaces, so an edit routed through the app's real engine
// addresses a target that was never registered.
//
// If this test ever fails, the two ID spaces have been reconciled and R4-2's exit row can finally
// be claimed of the product rather than of a test-only configuration
#[test]
fn a_shape_edit_does_not_reach_live_audio_through_the_engine_the_app_opens() {
    let mut model = AppModel::prototype();
    let parts = build_track_engine_parts(
        model.track_list(),
        model.tempo_map(),
        0x0053_5045_4354_5245,
        model.selected_track_id(),
        config(),
    )
    .unwrap();
    let engine = engine_over_null(parts);

    let mut status = String::new();
    apply_parameter_edit(&mut model, Some(&engine), "gain", "gain", 0.1, &mut status);

    assert!(
        status.contains("did not reach live audio"),
        "expected the publish to be refused as an unknown target, got: {status:?}"
    );
}

// Hardware evidence for the path main.rs ACTUALLY takes.
//
// app_engine_opens_a_real_device_and_renders above drills `open_default`, which is called from
// this test file and nowhere else -- main.rs calls `open_track_engine`. So the drill that R4's
// exit row cites as "the row that establishes ./spectre reaches a real driver" exercises a
// function the binary never invokes. This one opens the engine exactly as main.rs does: the same
// track list, the same tempo map, the same APP_GRAPH_SEED, the same selected track.
#[cfg(feature = "live-audio")]
#[test]
#[ignore = "requires a real audio device; run on macOS and Linux beside the other drills"]
fn the_engine_main_actually_opens_reaches_a_real_device_and_renders() {
    use spectre_app::engine::{open_track_engine, APP_GRAPH_SEED};

    let model = AppModel::prototype();
    let backend = spectre_audio::cpal_backend::CpalBackend::new();
    let engine = open_track_engine(
        &backend,
        model.track_list(),
        model.tempo_map(),
        APP_GRAPH_SEED,
        model.selected_track_id(),
    )
    .expect("a real output device is required for this drill");

    assert!(matches!(engine.state(), EngineState::Opened { .. }));
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
        "backend={name} device={device} rate={sample_rate} frames={frames} blocks={} xruns={} worst_headroom={} plan_errors={} contaminated={} stream_errors={} params_pending={}",
        health.blocks_rendered,
        health.xruns,
        health.worst_headroom,
        health.plan_errors,
        health.contaminated_nodes,
        health.stream_errors,
        health.parameters_pending,
    );
    assert!(
        health.blocks_rendered > 0,
        "the driver must have called back"
    );
    assert_eq!(health.plan_errors, 0, "no block may fail to render");
    assert_eq!(health.contaminated_nodes, 0, "output must stay finite");
    assert_eq!(health.stream_errors, 0, "the driver reported an error");
}
