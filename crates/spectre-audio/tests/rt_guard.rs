// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 5 evidence — RT-001 guards over every callback-reachable path
// Notes: Stronger than a before/after count. A thread-local flag marks an RT section and the
//   global allocator records any allocation or deallocation that happens inside one, so a
//   violation is attributed to the exact call rather than inferred from totals. The positive
//   control is load-bearing: without it a broken guard would report success everywhere.
//   Lock-freedom is enforced structurally instead, by scanning the RT modules for blocking
//   primitives; that is a weaker guarantee than the allocation guard and is labeled as such.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use spectre_audio::bridge::RenderBridge;
use spectre_audio::control::{control_channel, ControlSender, ParameterTarget};
use spectre_audio::null::{NullBackend, NULL_DEVICE_KEY};
use spectre_audio::{AudioStream, DeviceId, RenderBlock, StreamConfig};
use spectre_core::{IdGen, TransportCommand};
use spectre_dsp::{
    AudioProcessor, Filament, Gain, Gloam, NoteEvent, NoteEventKind, PulseInstrument, Saturator,
    Waveform, FILAMENT_PARAMETERS, GLOAM_PARAMETERS,
};
use spectre_graph::{CompiledPlan, Connection, EditableGraph, NodeId};

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;
const CHANNELS: u16 = 2;

// Compile-time proof that the render side can move to an audio thread at all. Without the
// Send bound on AudioProcessor this fails to build, which is how the missing bound surfaced.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    assert_send::<CompiledPlan>();
    assert_send::<RenderBridge>();
    assert_send::<spectre_audio::control::ControlReceiver>();
    assert_send::<ControlSender>();
};

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

// Build the fixture chain used by every guarded render
fn fixture(frames: usize) -> (CompiledPlan, NodeId) {
    let mut ids = IdGen::new(0x0000_5254_4755_4152);
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

// Build a valid note-on with a nonzero ID
fn note(sequence: u64, frame_offset: usize) -> NoteEvent {
    NoteEvent {
        frame_offset,
        sequence,
        kind: NoteEventKind::On {
            id: sequence as u32 + 1,
            channel: 0,
            note: 60,
            velocity: 0.5,
        },
    }
}

// Queue one block's worth of control traffic from the app thread
fn feed(sender: &mut ControlSender, block: u64) {
    sender.send_note(note(block, 0)).unwrap();
    sender.send_note(note(block + 1, 64)).unwrap();
    sender.send_transport(TransportCommand::Play).unwrap();
}

#[test]
fn the_guard_detects_a_deliberate_allocation() {
    // Positive control: proves a passing guard elsewhere is a real result, not a broken probe
    let (_, violations) = rt_section(|| {
        let leaked: Vec<u8> = Vec::with_capacity(4_096);
        std::hint::black_box(&leaked);
    });
    assert!(
        violations >= 2,
        "guard must catch both the allocation and its free, saw {violations}"
    );

    // And reports nothing for work that genuinely stays off the heap
    let (sum, clean) = rt_section(|| (0..64_u64).sum::<u64>());
    assert_eq!(sum, 2_016);
    assert_eq!(clean, 0, "arithmetic must not register a violation");
}

#[test]
fn bridge_render_is_rt_clean() {
    let (plan, note_node) = fixture(FRAMES);
    let mut ids = IdGen::new(0x0000_5254_5041_524d);
    let target = ParameterTarget {
        device: ids.next_id(),
        parameter: ids.next_id(),
    };
    let (mut sender, receiver) = control_channel(&[target], 256, 32).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);
    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS as usize];

    // Warm-up outside the guard so first-touch cost is not misread as a violation
    feed(&mut sender, 0);
    bridge.render(&mut RenderBlock::new(&mut interleaved, CHANNELS));

    for block in 1..8 {
        feed(&mut sender, block * 8);
        sender
            .parameters()
            .set_target(target, block as f32)
            .unwrap();
        let (_, violations) = rt_section(|| {
            bridge.render(&mut RenderBlock::new(&mut interleaved, CHANNELS));
        });
        assert_eq!(
            violations, 0,
            "bridge render violated RT-001 on block {block}"
        );
    }
    assert_eq!(bridge.telemetry().plan_errors(), 0);
}

#[test]
fn the_bridge_error_paths_are_rt_clean() {
    let (plan, note_node) = fixture(FRAMES);
    let (_sender, receiver) = control_channel(&[], 32, 8).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);

    // An oversized block takes the refusal path, which must also stay allocation-free
    let mut oversized = vec![0.0_f32; FRAMES * 4 * CHANNELS as usize];
    bridge.render(&mut RenderBlock::new(&mut oversized, CHANNELS));
    let (_, violations) = rt_section(|| {
        bridge.render(&mut RenderBlock::new(&mut oversized, CHANNELS));
    });
    assert_eq!(violations, 0, "frame-capacity refusal violated RT-001");

    // A zero-frame block is the other refusal path
    let mut empty: Vec<f32> = Vec::new();
    let (_, violations) = rt_section(|| {
        bridge.render(&mut RenderBlock::new(&mut empty, CHANNELS));
    });
    assert_eq!(violations, 0, "empty-block refusal violated RT-001");
    assert!(bridge.telemetry().frame_capacity_rejections() >= 2);
}

#[test]
fn control_receive_paths_are_rt_clean() {
    let mut ids = IdGen::new(0x0000_5254_434e_544c);
    let targets: Vec<ParameterTarget> = (0..8)
        .map(|_| ParameterTarget {
            device: ids.next_id(),
            parameter: ids.next_id(),
        })
        .collect();
    let (mut sender, mut receiver) = control_channel(&targets, 256, 32).unwrap();

    // Warm-up
    feed(&mut sender, 0);
    receiver.drain_parameters(|_, _| {});
    while receiver.next_note().is_some() {}
    while receiver.next_transport().is_some() {}

    for block in 1..8 {
        for (index, _) in targets.iter().enumerate() {
            sender.parameters().set(index, block as f32).unwrap();
        }
        feed(&mut sender, block * 8);

        let (_, violations) = rt_section(|| {
            receiver.drain_parameters(|_, _| {});
            while receiver.next_note().is_some() {}
            while receiver.next_transport().is_some() {}
        });
        assert_eq!(violations, 0, "control drain violated RT-001");
    }
}

#[test]
fn retiring_state_is_rt_clean_and_never_drops_on_the_render_thread() {
    let (mut sender, mut receiver) = control_channel(&[], 32, 8).unwrap();
    // The box is built on the app thread; retiring it must not free it on the render side
    let retired: Box<dyn Send> = Box::new(vec![0_u8; 8_192]);

    let (result, violations) = rt_section(|| receiver.retire(retired));
    assert!(result.is_ok());
    assert_eq!(violations, 0, "retire violated RT-001");

    // The app thread performs the actual free
    assert_eq!(sender.reclaim(), 1);
}

#[test]
fn plan_process_is_rt_clean_through_the_null_stream() {
    let backend = NullBackend::new();
    let config = StreamConfig::stereo(48_000, FRAMES).unwrap();
    let (plan, note_node) = fixture(FRAMES);
    let (mut sender, receiver) = control_channel(&[], 256, 32).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);

    // The backend's callback drives the bridge, matching the live wiring
    let callback: spectre_audio::RenderCallback = Box::new(move |mut block: RenderBlock| {
        bridge.render(&mut block);
    });
    let mut stream = backend
        .open_null_output(&DeviceId::new(NULL_DEVICE_KEY), config, callback)
        .unwrap();
    stream.start().unwrap();

    feed(&mut sender, 0);
    stream.pump().unwrap();

    for block in 1..8 {
        feed(&mut sender, block * 8);
        let (result, violations) = rt_section(|| stream.pump());
        result.unwrap();
        assert_eq!(
            violations, 0,
            "stream pump violated RT-001 on block {block}"
        );
    }
}

// Build the R4-6 voice chain at both devices' descriptor defaults
fn voice_chain(frames: usize) -> (CompiledPlan, NodeId) {
    let mut ids = IdGen::new(0x0000_5254_564f_4943);
    let filament = NodeId::new(ids.next_id());
    let gloam = NodeId::new(ids.next_id());

    let lean = FILAMENT_PARAMETERS[0].default();
    let rise_ms = FILAMENT_PARAMETERS[1].default();
    let fall_ms = FILAMENT_PARAMETERS[2].default();
    let level = FILAMENT_PARAMETERS[3].default();
    let damp_hz = GLOAM_PARAMETERS[0].default();
    let depth = GLOAM_PARAMETERS[1].default();
    let track_ms = GLOAM_PARAMETERS[2].default();

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            filament,
            Filament::new(lean, rise_ms, fall_ms, level).unwrap().io(),
        )
        .unwrap();
    graph
        .add_node(gloam, Gloam::new(damp_hz, depth, track_ms).unwrap().io())
        .unwrap();
    graph
        .connect(Connection {
            from: filament,
            from_bus: 0,
            to: gloam,
            to_bus: 0,
        })
        .unwrap();
    let plan = graph
        .compile(gloam, frames, &mut |node| {
            if node == filament {
                Ok(Box::new(Filament::new(lean, rise_ms, fall_ms, level)?)
                    as Box<dyn AudioProcessor>)
            } else {
                Ok(Box::new(Gloam::new(damp_hz, depth, track_ms)?))
            }
        })
        .unwrap();
    (plan, filament)
}

// R4-6's RT-001 evidence: the two new devices driven by the real callback path, not called
// directly. The positive control above is what makes a zero here a result rather than a
// broken probe reporting success
#[test]
fn voice_chain_process_is_rt_clean_through_the_null_stream() {
    let backend = NullBackend::new();
    let config = StreamConfig::stereo(48_000, FRAMES).unwrap();
    let (plan, note_node) = voice_chain(FRAMES);
    let (mut sender, receiver) = control_channel(&[], 256, 32).unwrap();
    let mut bridge = RenderBridge::new(plan, receiver, note_node, SAMPLE_RATE, 64);

    let callback: spectre_audio::RenderCallback = Box::new(move |mut block: RenderBlock| {
        bridge.render(&mut block);
    });
    let mut stream = backend
        .open_null_output(&DeviceId::new(NULL_DEVICE_KEY), config, callback)
        .unwrap();
    stream.start().unwrap();

    // One unguarded block first: the first callback may still be settling lazily initialized
    // state that belongs to the harness rather than to either device
    feed(&mut sender, 0);
    stream.pump().unwrap();

    for block in 1..8 {
        feed(&mut sender, block * 8);
        let (result, violations) = rt_section(|| stream.pump());
        result.unwrap();
        assert_eq!(
            violations, 0,
            "the voice chain violated RT-001 on block {block}"
        );
    }
}

#[test]
fn rt_modules_contain_no_blocking_primitives() {
    // Structural lock guard. Weaker than the allocation guard above: it proves these modules
    // never name a blocking primitive, not that some future call cannot reach one indirectly.
    const RT_MODULES: [&str; 5] = [
        "src/bridge.rs",
        "src/control.rs",
        "src/spsc.rs",
        "src/null.rs",
        // Added by R4-2: the parameter route table is resolved inside the render callback
        "src/route.rs",
    ];
    const FORBIDDEN: [&str; 7] = [
        "Mutex",
        "RwLock",
        "Condvar",
        "thread::sleep",
        "println!",
        "eprintln!",
        "dbg!",
    ];

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for module in RT_MODULES {
        let source = std::fs::read_to_string(root.join(module))
            .unwrap_or_else(|error| panic!("cannot read {module}: {error}"));
        for needle in FORBIDDEN {
            assert!(
                !source.contains(needle),
                "{module} names the blocking primitive {needle} on a callback-reachable path"
            );
        }
    }
}
