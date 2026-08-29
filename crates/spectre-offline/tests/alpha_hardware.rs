// Author: Jeff
// Date: 2026-08-28
// Description: Callback headroom measured on the composed alpha plan, not the three-node fixture
// Notes: Every headroom number this project owns -- macOS 2026-08-09 and 2026-08-24, Linux
//   2026-08-28 -- was taken on lifecycle_health.rs's Pulse -> Gain -> Saturator chain. R4-4 and
//   R4-6 made the plan the product actually runs several times that, so those numbers describe a
//   workload the alpha does not have. STATUS records this as a known gap on both platforms; this
//   file closes it.
//   The deterministic half runs everywhere and quantifies the gap as a ratio, so the claim is a
//   number in CI rather than a hardware-only observation. The hardware half is #[ignore]d for the
//   same reason lifecycle_health.rs's is: most machines and all of CI have no usable device.

use spectre_audio::bridge::{RenderBridge, DEFAULT_NOTE_SCRATCH};
use spectre_audio::clip::{ClipPlayer, ClipSchedule, CLIP_EVENT_RESERVE};
use spectre_audio::control::control_channel;
use spectre_audio::RenderBlock;
use spectre_core::{IdGen, SampleRate, TempoMap};
use spectre_dsp::{GLOAM_DEPTH, GLOAM_PARAMETERS};
use spectre_graph::CompiledPlan;
use spectre_offline::hash::{hash_block, SampleHasher};
use spectre_project::{
    build_track_graph, from_bytes, track_device_factory, ProjectEnvelope, TrackList,
};

const FIXTURE: &[u8] = include_bytes!("fixtures/r4-alpha.json");

// Held identical to lifecycle_health.rs so the two qualification records compare directly
const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;
const CHANNELS: usize = 2;

// The plan reserves twice the requested block, the same margin crates/spectre-app/src/engine.rs
// opens with, so a host that grants a larger block is absorbed rather than refused into silence
const PLAN_RESERVE: usize = FRAMES * 2;

// Matches e2e_alpha.rs's seed: a rebuild from the same list must produce the same nodes
const GRAPH_SEED: u64 = 0x0045_3245_414c_5048;

fn fixture() -> ProjectEnvelope {
    from_bytes(FIXTURE).expect("the checked-in alpha fixture must decode")
}

// Compile the fixture's own track graph, the way the app's engine does
fn compile_alpha(tracks: &TrackList) -> (CompiledPlan, Vec<spectre_graph::NodeId>) {
    let mut ids = IdGen::new(GRAPH_SEED);
    let (graph, nodes) = build_track_graph(tracks, &mut ids).expect("the fixture graph compiles");
    let mut factory = track_device_factory(tracks, &nodes);
    let plan = graph
        .compile(nodes.master.node, PLAN_RESERVE, &mut factory)
        .expect("the fixture plan compiles");
    let instruments = nodes.instruments.iter().map(|entry| entry.node).collect();
    (plan, instruments)
}

// Bake one track's clip material into absolute sample positions, through the one tick-to-sample
// conversion the workspace has
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

// Build the alpha bridge with every track's clips attached, the way ./spectre does
fn alpha_bridge(envelope: &ProjectEnvelope) -> (RenderBridge, usize) {
    let tracks = &envelope.project.tracks;
    let (plan, instruments) = compile_alpha(tracks);
    let steps = plan.step_count();

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

    // Play before the bridge moves to the audio thread; the first callback drains it. The sender
    // is dropped here on purpose -- the drill sends no further commands, and a dropped sender
    // must not disturb a bridge that has already taken its transport
    sender
        .send_transport(spectre_core::TransportCommand::Play)
        .unwrap();
    (bridge, steps)
}

// The gap this file exists to close, as a number, on every platform and with no hardware.
// Without this the "headroom is under-measured" claim would rest on the hardware half, which
// most machines skip
#[test]
fn the_alpha_plan_is_materially_larger_than_the_chain_headroom_was_measured_on() {
    let envelope = fixture();
    let (alpha, _) = compile_alpha(&envelope.project.tracks);
    let (chain, _) = spectre_offline::fixture::compile_fixture_plan(PLAN_RESERVE)
        .expect("the three-node fixture compiles");

    let alpha_steps = alpha.step_count();
    let chain_steps = chain.step_count();
    println!("plan_size alpha={alpha_steps} qualification_chain={chain_steps}");

    assert_eq!(
        chain_steps, 3,
        "lifecycle_health.rs measures Pulse -> Gain -> Saturator; if that changed, this file's \
         premise changed with it"
    );
    // Eleven: three tracks of instrument -> Gloam insert -> track gain, one sum bus, one master.
    // It was eight until 2026-08-28, when the insert slot R4-9's spec requires was implemented
    assert_eq!(
        alpha_steps, 11,
        "the composed alpha's node count moved; the qualification record quotes this number"
    );
    assert!(
        alpha_steps > chain_steps * 2,
        "the alpha must be more than twice the measured chain for the gap to be real"
    );
}

// Render the composed alpha through the bridge and fold the result
fn render_hash(envelope: &ProjectEnvelope, blocks: usize) -> u64 {
    let (mut bridge, _) = alpha_bridge(envelope);
    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS];
    let mut hasher = SampleHasher::new();
    for _ in 0..blocks {
        interleaved.fill(0.0);
        let mut block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
        bridge.render(&mut block);
        hash_block(&mut hasher, &interleaved, CHANNELS, FRAMES);
    }
    hasher.finish()
}

// The insert R4-9's spec requires, proved to be in the audible path rather than merely stored.
// Until 2026-08-28 build_track_graph wired instrument -> track gain and constructed no effect at
// all, so R4-6's Gloam reached no render and the R4-6 exit row overstated what shipped
#[test]
fn the_tracks_gloam_insert_is_in_the_signal_path() {
    let mut envelope = fixture();
    let baseline = render_hash(&envelope, 16);

    let tracks = &mut envelope.project.tracks;
    let id = tracks.tracks()[0].id();
    let stored = tracks.tracks()[0]
        .insert()
        .expect("R4-9's fixture declares a Gloam insert on every track")
        .depth();
    tracks
        .get_mut(id)
        .expect("the track is in the list")
        .set_insert_depth(GLOAM_PARAMETERS[GLOAM_DEPTH].default());
    assert_ne!(
        stored,
        GLOAM_PARAMETERS[GLOAM_DEPTH].default(),
        "the fixture must store a non-default depth for this test to move anything"
    );

    let moved = render_hash(&envelope, 16);
    assert_ne!(
        baseline, moved,
        "the track's insert depth changed nothing, so Gloam is not in the render"
    );
}

// The project's DeviceDoc list is the app's Build/Shape surface, NOT the render path -- the
// render reads the track model. Moving a value here changes no audio, and that is the current
// design rather than a defect: crates/spectre-app/src/project.rs loads this list into the model's
// device cards, and R4-2's parameter lane is what carries an edit to a live node.
// It is pinned because the two representations are easy to mistake for one
#[test]
fn the_project_device_doc_list_is_a_surface_and_not_the_render_path() {
    let mut envelope = fixture();
    let baseline = render_hash(&envelope, 16);

    let gloam = envelope
        .project
        .devices
        .iter_mut()
        .find(|device| device.key == "gloam")
        .expect("the alpha fixture stores a gloam device");
    let stored = gloam.parameters[0].value;
    // The descriptor default is a legal value that e2e_alpha.rs already asserts differs from the
    // stored one, so this moves the parameter without inventing a bound
    gloam.parameters[0].value = GLOAM_PARAMETERS[0].default();
    assert_ne!(
        stored, gloam.parameters[0].value,
        "the fixture must store a non-default depth for this test to move anything"
    );

    let moved = render_hash(&envelope, 16);
    assert_eq!(
        baseline, moved,
        "the DeviceDoc list reached the render; if that is now intended, this test and the \
         two-representation note above must be rewritten together"
    );
}

// Hardware qualification on the plan the product runs. Ignored by default because CI and most
// dev machines have no usable output device; run explicitly on macOS and on Linux with:
//   cargo test -p spectre-offline --test alpha_hardware -- --ignored --nocapture
#[test]
#[ignore = "requires a real audio device; run on macOS and Linux beside the lifecycle drill"]
fn alpha_plan_headroom_drill() {
    use spectre_audio::cpal_backend::CpalBackend;
    use spectre_audio::{AudioBackend, StreamConfig};

    let backend = CpalBackend::new();
    let devices = backend.output_devices().expect("enumeration must succeed");
    assert!(!devices.is_empty(), "no output device to qualify against");
    let default = backend
        .default_output_device()
        .expect("a default device is required");

    let envelope = fixture();
    let (mut bridge, steps) = alpha_bridge(&envelope);
    let telemetry = bridge.telemetry();
    println!(
        "backend={} device={} plan_steps={steps}",
        backend.name(),
        default.name
    );

    let config = StreamConfig::stereo(48_000, FRAMES).unwrap();
    let mut stream = backend
        .open_output(
            &default.id,
            config,
            Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),
        )
        .expect("opening the default device must succeed");

    // Start, run, stop, restart: the same lifecycle shape the three-node drill proves, so the
    // only variable between the two records is the plan
    for _ in 0..2 {
        stream.start().expect("start must succeed");
        std::thread::sleep(std::time::Duration::from_millis(250));
        stream.stop().expect("stop must succeed");
    }
    stream.close().expect("close must succeed");

    println!(
        "blocks={} xruns={} worst_headroom={} plan_errors={} contaminated={} frame_capacity_rejections={} clip_events_refused={} session_peak={}",
        telemetry.blocks_rendered(),
        telemetry.xruns(),
        telemetry.worst_headroom(),
        telemetry.plan_errors(),
        telemetry.contaminated_nodes(),
        telemetry.frame_capacity_rejections(),
        telemetry.clip_events_refused(),
        telemetry.session_peak()
    );
    assert!(
        telemetry.blocks_rendered() > 0,
        "the driver must have called back"
    );
    assert_eq!(telemetry.plan_errors(), 0, "no block may fail to render");
    assert_eq!(telemetry.contaminated_nodes(), 0, "output must stay finite");
    assert_eq!(
        telemetry.frame_capacity_rejections(),
        0,
        "a refused block is silence, and silence must not be recorded as headroom"
    );
    assert_eq!(
        telemetry.clip_events_refused(),
        0,
        "a dropped clip event would make this a measurement of a quieter project"
    );
    // A headroom figure for a silent render measures nothing anyone will hear
    assert!(
        telemetry.session_peak() > 0.0,
        "the alpha rendered to the driver but carried no signal"
    );
}

// The control for the test above. Without it, a render_hash that returned a constant -- or a
// bridge that silently rendered silence -- would make "the effect is inert" indistinguishable
// from "nothing here measures anything". master_level IS on the signal path, so moving it MUST
// change the hash that gloam's depth did not
#[test]
fn a_parameter_that_is_in_the_signal_path_does_change_the_render() {
    let mut envelope = fixture();
    let baseline = render_hash(&envelope, 16);

    let tracks = &mut envelope.project.tracks;
    let original = tracks.master_level();
    tracks.set_master_level(original * 0.5);
    assert_ne!(original, tracks.master_level());

    let moved = render_hash(&envelope, 16);
    assert_ne!(
        baseline, moved,
        "halving the master level changed nothing, so this file measures nothing"
    );
}

// Isolates the peak publication from the driver: if this passes and the hardware drill's peak is
// zero, the difference is the device, not the fold
#[test]
fn a_rendered_alpha_block_publishes_a_nonzero_peak() {
    let envelope = fixture();
    let (mut bridge, _) = alpha_bridge(&envelope);
    let telemetry = bridge.telemetry();
    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS];
    let mut best = 0.0_f32;
    for _ in 0..16 {
        interleaved.fill(0.0);
        let mut block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
        bridge.render(&mut block);
        best = best.max(telemetry.last_peak());
    }
    assert!(best > 0.0, "the alpha rendered silence off the driver too");
}
