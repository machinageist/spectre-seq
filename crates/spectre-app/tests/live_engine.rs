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
    usize,
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
        primary_index,
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
    (stream, sender, telemetry, targets, has_clips, primary_index)
}

// Wrap an opened null stream in the engine without erasing its concrete type
fn engine_over_null(parts: EngineParts) -> LiveEngine<NullStream> {
    let config = parts.config;
    let (stream, sender, telemetry, targets, has_clips, primary_index) = open_null(parts);
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
    // Same reason as the flag above: an engine that forgot this addresses every published
    // schedule one destination out, so one track plays another's material
    engine.set_primary_index(primary_index);
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
    let (mut stream, mut sender, telemetry, _targets, _, _) = open_null(parts);

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
        let (mut stream, mut sender, _telemetry, _targets, _, _) = open_null(parts);
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

    // Through open_track_engine, the function main.rs calls. It used to go through open_default,
    // which no product path invoked
    let model_for_open = AppModel::prototype();
    let result = spectre_app::engine::open_track_engine(
        &FailingBackend,
        model_for_open.track_list(),
        model_for_open.tempo_map(),
        spectre_app::engine::APP_GRAPH_SEED,
        model_for_open.selected_track_id(),
    );
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
// main.rs calls open_track_engine, whose targets are ObjectIds the graph builder allocated from
// APP_GRAPH_SEED. Those two ID spaces are disjoint, and until 2026-08-28 every Shape edit in the
// product was refused as an unknown target -- R4-2's exit row was closed against a configuration
// the app does not use. apply_parameter_edit now resolves the destination by ROLE on the selected
// track, and this asserts the criterion where it actually matters.
#[test]
fn a_shape_edit_changes_live_audio_through_the_engine_the_app_opens() {
    let mut model = AppModel::prototype();
    let parts = build_track_engine_parts(
        model.track_list(),
        model.tempo_map(),
        spectre_app::engine::APP_GRAPH_SEED,
        model.selected_track_id(),
        config(),
    )
    .unwrap();
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
        "the edit did not change what the render thread produced"
    );
    assert_eq!(
        engine.health().parameters_pending,
        0,
        "R4-2: an edit the lane could not apply is still pending"
    );
}

// A device the track does not host must NOT be silently misrouted onto some other node. The flat
// list carries a saturator and no track hosts one, so its edit stays model-only and says so --
// which is the honest answer, and the control that proves the binding above is by role rather
// than by position in a list
#[test]
fn an_edit_for_a_device_no_track_hosts_stays_model_only() {
    let mut model = AppModel::prototype();
    let parts = build_track_engine_parts(
        model.track_list(),
        model.tempo_map(),
        spectre_app::engine::APP_GRAPH_SEED,
        model.selected_track_id(),
        config(),
    )
    .unwrap();
    let engine = engine_over_null(parts);

    let mut status = String::new();
    apply_parameter_edit(
        &mut model,
        Some(&engine),
        "saturator",
        "drive",
        3.0,
        &mut status,
    );
    assert!(
        status.contains("did not reach live audio"),
        "a device absent from the track path must not claim to have been published: {status:?}"
    );
    // The model still keeps the edit; it is the value of record for offline rendering
    assert!(
        model
            .devices()
            .iter()
            .find(|device| device.key == "saturator")
            .and_then(|device| {
                device
                    .parameters
                    .iter()
                    .find(|p| p.descriptor.key.as_str() == "drive")
            })
            .is_some_and(|p| p.value == 3.0),
        "the model must keep an edit the lane could not carry"
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
    let mut engine = open_track_engine(
        &backend,
        model.track_list(),
        model.tempo_map(),
        APP_GRAPH_SEED,
        model.selected_track_id(),
    )
    .expect("a real output device is required for this drill");

    assert!(matches!(engine.state(), EngineState::Opened { .. }));
    // Play, so the drill measures a render that carries signal rather than a clean silence. The
    // prototype has no clips, so this is the audition voice -- which is what R4-1's exit row
    // means by "produces sound" until a project is loaded
    engine
        .start_audition()
        .expect("the transport lane must accept Play");
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
        "backend={name} device={device} rate={sample_rate} frames={frames} blocks={} xruns={} worst_headroom={} plan_errors={} contaminated={} stream_errors={} params_pending={} session_peak={}",
        health.blocks_rendered,
        health.xruns,
        health.worst_headroom,
        health.plan_errors,
        health.contaminated_nodes,
        health.stream_errors,
        health.parameters_pending,
        health.session_peak,
    );
    assert!(
        health.blocks_rendered > 0,
        "the driver must have called back"
    );
    assert_eq!(health.plan_errors, 0, "no block may fail to render");
    assert_eq!(health.contaminated_nodes, 0, "output must stay finite");
    assert_eq!(health.stream_errors, 0, "the driver reported an error");
    // Without this the drill passes on a clean render of pure silence, which is the exact
    // failure r4-qa-protocol.md calls the highest-value one it exists to catch. It is the
    // objective half of "produces sound"; the operator still has to hear it
    assert!(
        health.session_peak > 0.0,
        "the render reached the driver but carried no signal"
    );
}

// ---- the edit-listen loop ----
//
// The RT machinery for swapping a clip schedule has existed since R4-5 -- ClipPlayer::install, a
// bounded lane, off-thread reclamation, telemetry -- with no product caller. Worse, the lane
// carried a bare schedule, so install_pending_schedule had nowhere to put one but the primary
// player: every clip voice was frozen for the life of the stream.
//
// Without this a musician writes a note, hears nothing change, and has to rebuild the engine --
// a stream restart with an audible gap.

const AUTHORING_BAR: i64 = 960 * 4;

fn note_at(start: i64, pitch: u8) -> spectre_project::ClipNote {
    spectre_project::ClipNote::new(
        spectre_core::BeatTicks(start),
        spectre_core::BeatTicks(480),
        0,
        pitch,
        0.8,
    )
    .expect("a valid note")
}

// One track with one clip, which is what a musician has the moment they create one
fn authored_session() -> (AppModel, spectre_core::ObjectId) {
    let mut model = AppModel::prototype();
    let track = model.add_track("Keys").expect("the track is added");
    let placement = model
        .create_clip(
            track,
            "Riff",
            spectre_core::BeatTicks(AUTHORING_BAR),
            spectre_core::BeatTicks(0),
        )
        .expect("the clip is created");
    let clip = model
        .track_list()
        .get(track)
        .expect("the track exists")
        .clips()
        .get(placement)
        .expect("the placement exists")
        .clip();
    (model, clip)
}

fn authored_engine(model: &AppModel) -> LiveEngine<NullStream> {
    let parts = build_track_engine_parts(
        model.track_list(),
        model.tempo_map(),
        spectre_app::engine::APP_GRAPH_SEED,
        model.selected_track_id(),
        StreamConfig::stereo(NULL_SAMPLE_RATE, 256).unwrap(),
    )
    .expect("the track parts build");
    engine_over_null(parts)
}

#[test]
fn an_edit_reaches_the_running_engine_without_a_restart() {
    let (mut model, clip) = authored_session();
    let mut engine = authored_engine(&model);
    let before = engine.health().schedules_installed;

    model
        .add_note(clip, note_at(0, 60))
        .expect("the note is accepted");
    let published = engine
        .publish_schedules(model.track_list(), model.tempo_map())
        .expect("the lane accepts the schedules");
    assert!(published > 0, "nothing was published");

    for _ in 0..published + 2 {
        engine.stream_mut().pump().unwrap();
    }
    assert!(
        engine.health().schedules_installed > before,
        "a published schedule never reached the render thread"
    );
    assert_eq!(
        engine.health().schedules_misaddressed,
        0,
        "a schedule was published to a destination with no player behind it"
    );
}

// Every instrument track gets a destination once the project has material, including one that is
// empty when the stream opens. A track with no player has no address, so an edit to it could
// never be heard
#[test]
fn a_track_that_was_empty_at_open_can_still_be_reached() {
    let (mut model, _) = authored_session();
    let second = model.add_track("Empty").expect("the track is added");
    let mut engine = authored_engine(&model);

    let placement = model
        .create_clip(
            second,
            "Later",
            spectre_core::BeatTicks(AUTHORING_BAR),
            spectre_core::BeatTicks(0),
        )
        .expect("the clip is created");
    let clip = model
        .track_list()
        .get(second)
        .expect("the track exists")
        .clips()
        .get(placement)
        .expect("the placement exists")
        .clip();
    model
        .add_note(clip, note_at(0, 67))
        .expect("the note is accepted");

    let published = engine
        .publish_schedules(model.track_list(), model.tempo_map())
        .expect("the lane accepts the schedules");
    assert_eq!(
        published,
        model.tracks().len(),
        "not every track received a schedule"
    );
    for _ in 0..published + 2 {
        engine.stream_mut().pump().unwrap();
    }
    assert_eq!(
        engine.health().schedules_misaddressed,
        0,
        "the track that was empty at open had no destination"
    );
}

// A project with no material must behave exactly as it did before R4-5 -- no player, no voices,
// Play auditions. That is what keeps R4-1's one-block Play/Stop refusal evidence valid
#[test]
fn a_project_with_no_clips_still_auditions() {
    let model = AppModel::prototype();
    let engine = authored_engine(&model);
    assert!(
        !engine.has_clips(),
        "an empty project attached a clip player and would no longer audition"
    );
}

#[test]
fn a_project_with_clips_does_not_audition() {
    let (model, _) = authored_session();
    let engine = authored_engine(&model);
    assert!(engine.has_clips());
}

// Publishing is app-thread work and must not wedge the render thread. The lane is bounded, so
// publishing past its depth is a counted refusal rather than a block or a silent drop
#[test]
fn overflowing_the_schedule_lane_is_a_counted_refusal() {
    let (model, _) = authored_session();
    let mut engine = authored_engine(&model);
    let mut refused = false;
    for _ in 0..32 {
        if engine
            .publish_schedules(model.track_list(), model.tempo_map())
            .is_err()
        {
            refused = true;
            break;
        }
    }
    assert!(
        refused,
        "the bounded schedule lane accepted an unbounded number of publications"
    );
    for _ in 0..8 {
        engine.stream_mut().pump().unwrap();
    }
    engine
        .publish_schedules(model.track_list(), model.tempo_map())
        .expect("the lane recovers once the render thread has drained it");
}

// Which player a published schedule lands on is a stated rule and needs its own falsifiable
// test. Counting installs does not cover it: ignoring the destination entirely and installing
// every schedule on the primary installs exactly as many, and passes every test above.
//
// Observed through the mixer, because a voice's schedule cannot be read from outside. The
// primary track is MUTED and a later track is not, and only the later track carries notes. With
// correct routing its material reaches an audible track; with the destination ignored, that
// material lands on the muted primary and the render is silent
#[test]
fn a_published_schedule_lands_on_the_track_it_was_addressed_to() {
    let mut model = AppModel::prototype();
    let quiet = model.add_track("Muted").expect("the track is added");
    let loud = model.add_track("Audible").expect("the track is added");
    // The primary is the selected track, and it is the one that must not be heard
    model.select_track(quiet);
    model
        .set_track_muted(quiet, true)
        .expect("the mute applies");

    // Both tracks carry an EMPTY placement, so both get a player and neither has material
    let mut clips = Vec::new();
    for track in [quiet, loud] {
        let placement = model
            .create_clip(
                track,
                "C",
                spectre_core::BeatTicks(AUTHORING_BAR),
                spectre_core::BeatTicks(0),
            )
            .expect("the clip is created");
        clips.push(
            model
                .track_list()
                .get(track)
                .expect("the track exists")
                .clips()
                .get(placement)
                .expect("the placement exists")
                .clip(),
        );
    }

    // The engine opens with no material anywhere. The note is written AFTERWARDS, so the only
    // way it can reach the audible track is the publish path -- if it were written first, the
    // open path would have installed it correctly and this test could not fail
    let mut engine = authored_engine(&model);
    model
        .add_note(clips[1], note_at(0, 60))
        .expect("the note is accepted");
    let published = engine
        .publish_schedules(model.track_list(), model.tempo_map())
        .expect("the lane accepts the schedules");
    for _ in 0..published + 2 {
        engine.stream_mut().pump().unwrap();
    }
    engine
        .send_transport(TransportCommand::Play)
        .expect("the transport lane accepts Play");
    for _ in 0..8 {
        engine.stream_mut().pump().unwrap();
    }

    assert_eq!(engine.health().schedules_misaddressed, 0);
    assert!(
        engine.health().session_peak > 0.0,
        "the render is silent, so the audible track's material was installed on the muted one"
    );
}

// ---- loop playback ----
//
// "Sketch a loop, branch variations, audition instantly" is the vision's FIRST core-loop item.
// Transport carried a loop region, Transport::advance wrapped inside it, and ClipPlayer honoured
// it -- and nothing in spectre-app ever set one, so a musician could not loop four bars while
// writing into them.

#[test]
fn a_published_loop_wraps_the_playhead() {
    let (model, _) = authored_session();
    let mut engine = authored_engine(&model);

    // Two bars at the project tempo
    let two_bars = spectre_core::BeatTicks(AUTHORING_BAR * 2);
    engine
        .publish_loop(
            Some((spectre_core::BeatTicks(0), two_bars)),
            model.tempo_map(),
        )
        .expect("the transport lane accepts the loop");
    engine
        .send_transport(TransportCommand::Play)
        .expect("the transport lane accepts Play");

    let rate = spectre_core::SampleRate::new(NULL_SAMPLE_RATE).expect("a valid rate");
    let loop_len = model.tempo_map().ticks_to_samples(two_bars, rate).0;
    // Render past the loop end; a wrapping playhead never reaches it
    let blocks = (loop_len / 256) as usize + 8;
    for _ in 0..blocks {
        engine.stream_mut().pump().unwrap();
    }

    let position = engine
        .health()
        .position_samples
        .expect("blocks rendered, so there is a position");
    assert!(
        position < loop_len,
        "the playhead ran past the loop end at {position}, so it did not wrap"
    );
}

// Without a loop the playhead runs on. Asserting only the wrap would pass against a transport
// that never advanced at all
#[test]
fn without_a_loop_the_playhead_runs_past_the_same_point() {
    let (model, _) = authored_session();
    let mut engine = authored_engine(&model);
    engine
        .send_transport(TransportCommand::Play)
        .expect("the transport lane accepts Play");

    let rate = spectre_core::SampleRate::new(NULL_SAMPLE_RATE).expect("a valid rate");
    let two_bars = model
        .tempo_map()
        .ticks_to_samples(spectre_core::BeatTicks(AUTHORING_BAR * 2), rate)
        .0;
    for _ in 0..(two_bars / 256) as usize + 8 {
        engine.stream_mut().pump().unwrap();
    }

    let position = engine
        .health()
        .position_samples
        .expect("blocks rendered, so there is a position");
    assert!(
        position > two_bars,
        "the playhead stopped at {position} with no loop set, so the wrap test proves nothing"
    );
}

// Clearing the loop must release the playhead rather than leaving it circling
#[test]
fn clearing_the_loop_lets_the_playhead_run_on() {
    let (model, _) = authored_session();
    let mut engine = authored_engine(&model);
    let two_bars = spectre_core::BeatTicks(AUTHORING_BAR * 2);
    let rate = spectre_core::SampleRate::new(NULL_SAMPLE_RATE).expect("a valid rate");
    let loop_len = model.tempo_map().ticks_to_samples(two_bars, rate).0;

    engine
        .publish_loop(
            Some((spectre_core::BeatTicks(0), two_bars)),
            model.tempo_map(),
        )
        .expect("the loop is accepted");
    engine
        .send_transport(TransportCommand::Play)
        .expect("Play is accepted");
    for _ in 0..(loop_len / 256) as usize + 4 {
        engine.stream_mut().pump().unwrap();
    }
    engine
        .publish_loop(None, model.tempo_map())
        .expect("clearing the loop is accepted");
    for _ in 0..(loop_len / 256) as usize + 8 {
        engine.stream_mut().pump().unwrap();
    }

    let position = engine
        .health()
        .position_samples
        .expect("blocks rendered, so there is a position");
    assert!(
        position > loop_len,
        "the playhead is still inside the cleared loop at {position}"
    );
}

// The loop is stored in ticks and sent in samples, so the same bar range must land somewhere
// different once the tempo changes -- which is why a tempo edit republishes it
#[test]
fn the_same_bar_range_maps_to_fewer_samples_at_a_faster_tempo() {
    let (mut model, _) = authored_session();
    let rate = spectre_core::SampleRate::new(NULL_SAMPLE_RATE).expect("a valid rate");
    let two_bars = spectre_core::BeatTicks(AUTHORING_BAR * 2);
    let at_120 = model.tempo_map().ticks_to_samples(two_bars, rate).0;

    model.set_tempo(240.0).expect("240 is a valid tempo");
    let at_240 = model.tempo_map().ticks_to_samples(two_bars, rate).0;

    assert!(
        at_240 < at_120,
        "doubling the tempo did not shorten the loop: {at_120} vs {at_240}"
    );
}

// An inverted or empty range is refused by the model, so the transport never sees one
#[test]
fn an_empty_loop_range_is_refused() {
    let (mut model, _) = authored_session();
    model
        .set_loop(Some((
            spectre_core::BeatTicks(AUTHORING_BAR),
            spectre_core::BeatTicks(AUTHORING_BAR),
        )))
        .expect_err("an empty range must be refused");
    model
        .set_loop(Some((
            spectre_core::BeatTicks(AUTHORING_BAR * 2),
            spectre_core::BeatTicks(AUTHORING_BAR),
        )))
        .expect_err("an inverted range must be refused");
    assert!(model.loop_ticks().is_none());
}
