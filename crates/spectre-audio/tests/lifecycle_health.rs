// Author: Jeff
// Date: 2026-08-09
// Description: R3 slices 7 and 9 evidence — device lifecycle drill and off-thread health telemetry
// Notes: The drill runs against the null backend here so it is deterministic and hardware-free.
//   That covers the state machine, not a real driver. The hardware half of the slice 7 exit row
//   is the ignored `hardware_lifecycle_drill` below, which must be run explicitly on macOS and
//   on Linux; until it has, the exit row stays open and is documented as open.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use spectre_audio::bridge::RenderBridge;
use spectre_audio::control::control_channel;
use spectre_audio::null::{NullBackend, NULL_DEVICE_KEY};
use spectre_audio::{
    AudioBackend, AudioStream, BackendError, DeviceId, RenderBlock, StreamConfig, StreamState,
};
use spectre_core::IdGen;
use spectre_dsp::{AudioProcessor, Gain, PulseInstrument, Saturator, Waveform};
use spectre_graph::{CompiledPlan, Connection, EditableGraph, NodeId};

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;
const CHANNELS: u16 = 2;

// Build the fixture chain the drill renders
fn fixture(frames: usize) -> (CompiledPlan, NodeId) {
    let mut ids = IdGen::new(0x0000_4c49_4645_4359);
    let pulse = NodeId::new(ids.next_id());
    let gain = NodeId::new(ids.next_id());
    let saturator = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            pulse,
            PulseInstrument::new(Waveform::Saw, 0.3).unwrap().io(),
        )
        .unwrap();
    graph.add_node(gain, Gain::new(0.7).unwrap().io()).unwrap();
    graph
        .add_node(saturator, Saturator::new(2.5, 0.35).unwrap().io())
        .unwrap();
    for (from, to) in [(pulse, gain), (gain, saturator)] {
        graph
            .connect(Connection {
                from,
                from_bus: 0,
                to,
                to_bus: 0,
            })
            .unwrap();
    }
    let plan = graph
        .compile(saturator, frames, &mut |node| {
            if node == pulse {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
            } else if node == gain {
                Ok(Box::new(Gain::new(0.7)?))
            } else {
                Ok(Box::new(Saturator::new(2.5, 0.35)?))
            }
        })
        .unwrap();
    (plan, pulse)
}

#[test]
fn start_stop_restart_cycles_do_not_disturb_rendering() {
    let backend = NullBackend::new();
    let (plan, note_node) = fixture(FRAMES);
    let (_sender, receiver) = control_channel(&[], 64, 16).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);
    let telemetry = bridge.telemetry();

    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let callback: spectre_audio::RenderCallback = Box::new(move |mut block: RenderBlock| {
        bridge.render(&mut block);
        counter.fetch_add(1, Ordering::Relaxed);
    });
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            StreamConfig::stereo(48_000, FRAMES).unwrap(),
            callback,
        )
        .unwrap();

    // Three full start/stop cycles, rendering in each
    for cycle in 0..3 {
        stream.start().unwrap();
        for _ in 0..4 {
            stream.pump().unwrap();
        }
        stream.stop().unwrap();
        // A stopped stream renders nothing, so the count must hold across the gap
        assert_eq!(stream.pump().unwrap_err(), BackendError::NotRunning);
        assert_eq!(calls.load(Ordering::Relaxed), (cycle + 1) * 4);
    }

    stream.close().unwrap();
    assert_eq!(stream.state(), StreamState::Closed);
    assert_eq!(telemetry.plan_errors(), 0);
    assert_eq!(telemetry.blocks_rendered(), 12);
}

#[test]
fn a_sample_rate_change_reopens_the_stream_without_losing_the_device() {
    let backend = NullBackend::new();
    let device = DeviceId::new(NULL_DEVICE_KEY);

    for rate in [44_100, 48_000, 96_000] {
        let config = StreamConfig::stereo(rate, FRAMES).unwrap();
        let mut stream = backend
            .open_null_output(
                &device,
                config,
                Box::new(|mut block: RenderBlock| block.fill_silence()),
            )
            .unwrap();
        assert_eq!(stream.config().sample_rate, rate);
        stream.start().unwrap();
        stream.pump().unwrap();
        stream.stop().unwrap();
        stream.close().unwrap();
    }

    // The device survives every reopen
    assert_eq!(backend.default_output_device().unwrap().id, device);
}

#[test]
fn device_loss_is_reported_rather_than_panicking() {
    let backend = NullBackend::new();
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            StreamConfig::stereo(48_000, FRAMES).unwrap(),
            Box::new(|mut block: RenderBlock| block.fill_silence()),
        )
        .unwrap();
    stream.start().unwrap();
    stream.pump().unwrap();

    // Closing mid-run models the device disappearing underneath us
    stream.close().unwrap();
    assert_eq!(stream.pump().unwrap_err(), BackendError::Closed);
    assert_eq!(stream.start().unwrap_err(), BackendError::Closed);
    assert_eq!(stream.stop().unwrap_err(), BackendError::Closed);

    // A fresh stream on the same device recovers
    let mut recovered = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            StreamConfig::stereo(48_000, FRAMES).unwrap(),
            Box::new(|mut block: RenderBlock| block.fill_silence()),
        )
        .unwrap();
    recovered.start().unwrap();
    recovered.pump().unwrap();
    assert_eq!(recovered.state(), StreamState::Running);
}

#[test]
fn headroom_is_published_off_thread_after_every_block() {
    let backend = NullBackend::new();
    let (plan, note_node) = fixture(FRAMES);
    let (_sender, receiver) = control_channel(&[], 64, 16).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);
    let telemetry = bridge.telemetry();

    // Nothing has rendered, so no headroom has been observed yet
    assert!(telemetry.last_headroom().is_infinite());
    assert_eq!(telemetry.xruns(), 0);

    let callback: spectre_audio::RenderCallback = Box::new(move |mut block: RenderBlock| {
        bridge.render(&mut block);
    });
    let mut stream = backend
        .open_null_output(
            &DeviceId::new(NULL_DEVICE_KEY),
            StreamConfig::stereo(48_000, FRAMES).unwrap(),
            callback,
        )
        .unwrap();
    stream.start().unwrap();
    for _ in 0..16 {
        stream.pump().unwrap();
    }

    let last = telemetry.last_headroom();
    let worst = telemetry.worst_headroom();
    assert!(last.is_finite(), "a rendered block must publish headroom");
    assert!(
        worst <= last,
        "worst headroom must not exceed the most recent block"
    );
    // No wall-clock assertion here, deliberately. This test's subject is that headroom is
    // PUBLISHED off thread after every block, and the two assertions above are the whole of that
    // claim. `worst > 0.0` and `xruns == 0` were also asserted until 2026-08-28 and made this a
    // flaky gate: the null backend's pump has no realtime scheduling, so a busy machine preempts
    // a block and publishes negative headroom truthfully. It failed inside a full `cargo test
    // --workspace` run, which executes test binaries in parallel, and passed five times in
    // isolation on the same commit.
    //
    // The performance claim still has homes, both of which measure under conditions that make it
    // meaningful: `an_overrunning_block_is_counted_as_an_xrun` below drives the xrun path
    // deliberately rather than hoping for it, and the #[ignore]d hardware drills report real
    // driver headroom on a quiet machine into the milestone's qualification record
    let _ = telemetry.xruns();
}

#[test]
fn an_overrunning_block_is_counted_as_an_xrun() {
    let (plan, note_node) = fixture(FRAMES);
    let (_sender, receiver) = control_channel(&[], 64, 16).unwrap();
    // A sample rate this absurd makes the block budget vanishingly small, so any real work
    // overruns it. This exercises the xrun path without depending on machine speed.
    let mut bridge = RenderBridge::new(plan, receiver, note_node, 1.0e12, 64);
    let telemetry = bridge.telemetry();

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];
    bridge.render(&mut RenderBlock::new(&mut interleaved, CHANNELS));

    assert_eq!(telemetry.xruns(), 1, "an overrun must be counted");
    assert!(
        telemetry.last_headroom() <= 0.0,
        "an overrun must publish non-positive headroom"
    );
}

#[test]
fn containment_counters_reach_the_app_thread_through_telemetry() {
    let (plan, note_node) = fixture(FRAMES);
    let (_sender, receiver) = control_channel(&[], 64, 16).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);
    let telemetry = bridge.telemetry();

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];
    bridge.render(&mut RenderBlock::new(&mut interleaved, CHANNELS));

    // The shipping fixture is clean, so containment must report nothing
    assert_eq!(telemetry.contaminated_nodes(), 0);
    assert_eq!(telemetry.denormals_flushed(), 0);
    assert_eq!(telemetry.last_contaminated_node(), None);
}

// Hardware qualification for the slice 7 exit row. Ignored by default because CI and most
// dev machines have no usable output device; run explicitly on macOS and on Linux with:
//   cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture
#[test]
#[ignore = "requires a real audio device; run on macOS and Linux to close the slice 7 exit row"]
#[cfg(feature = "cpal-backend")]
fn hardware_lifecycle_drill() {
    use spectre_audio::cpal_backend::CpalBackend;

    let backend = CpalBackend::new();
    let devices = backend.output_devices().expect("enumeration must succeed");
    assert!(!devices.is_empty(), "no output device to qualify against");
    let default = backend
        .default_output_device()
        .expect("a default device is required");
    println!("backend={} device={}", backend.name(), default.name);

    let (plan, note_node) = fixture(FRAMES);
    let (_sender, receiver) = control_channel(&[], 64, 16).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);
    let telemetry = bridge.telemetry();

    let config = StreamConfig::stereo(48_000, FRAMES).unwrap();
    let mut stream = backend
        .open_output(
            &default.id,
            config,
            Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),
        )
        .expect("opening the default device must succeed");

    // Start, run, stop, restart: the lifecycle the drill exists to prove
    for _ in 0..2 {
        stream.start().expect("start must succeed");
        std::thread::sleep(std::time::Duration::from_millis(250));
        stream.stop().expect("stop must succeed");
    }
    stream.close().expect("close must succeed");

    println!(
        "blocks={} xruns={} worst_headroom={} plan_errors={} contaminated={} frame_capacity_rejections={}",
        telemetry.blocks_rendered(),
        telemetry.xruns(),
        telemetry.worst_headroom(),
        telemetry.plan_errors(),
        telemetry.contaminated_nodes(),
        telemetry.frame_capacity_rejections()
    );
    assert!(
        telemetry.blocks_rendered() > 0,
        "the driver must have called back"
    );
    assert_eq!(telemetry.plan_errors(), 0, "no block may fail to render");
    assert_eq!(telemetry.contaminated_nodes(), 0, "output must stay finite");
    // Without this the record cannot distinguish "the host honored the requested block size"
    // from "it did not and every oversized block was refused into counted silence"
    assert_eq!(
        telemetry.frame_capacity_rejections(),
        0,
        "the host must honor the requested block size"
    );
}
