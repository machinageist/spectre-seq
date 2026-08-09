// Author: Jeff
// Date: 2026-08-09
// Description: R3 allocation gate — the render path allocates and frees nothing per block
// Notes: Mirrors the R2 plan-allocation gate. Thread-local counters keep parallel test
//   threads from polluting the measurement. This covers the seam's own render path;
//   RT-001's full callback-reachable guard arrives with the bridge in slice 5.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Arc;

use spectre_audio::control::{control_channel, ParameterTarget};
use spectre_audio::null::{NullBackend, NULL_DEVICE_KEY};
use spectre_audio::{AudioStream, DeviceId, RenderBlock, StreamConfig};
use spectre_core::{IdGen, TransportCommand};
use spectre_dsp::{NoteEvent, NoteEventKind};

thread_local! {
    // Const-initialized cells avoid lazy TLS setup allocating inside the allocator
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
    static DEALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

// System allocator wrapper counting this thread's traffic
struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|count| count.set(count.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.with(|count| count.set(count.get() + 1));
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

// Snapshot this thread's allocator traffic
fn traffic() -> (u64, u64) {
    (ALLOCATIONS.with(Cell::get), DEALLOCATIONS.with(Cell::get))
}

#[test]
fn pumping_a_running_stream_is_allocation_free() {
    let backend = NullBackend::new();
    let config = StreamConfig::stereo(48_000, 512).unwrap();
    let scratch: Arc<[f32]> = Arc::from(vec![0.25_f32; 1024]);
    let source = Arc::clone(&scratch);

    // The callback reads preallocated state and writes the driver buffer in place
    let callback: spectre_audio::RenderCallback = Box::new(move |mut block: RenderBlock| {
        let samples = block.samples_mut();
        for (index, sample) in samples.iter_mut().enumerate() {
            *sample = source[index % source.len()];
        }
    });

    let mut stream = backend
        .open_null_output(&DeviceId::new(NULL_DEVICE_KEY), config, callback)
        .unwrap();
    stream.start().unwrap();

    // Warm-up block, then measure steady-state blocks
    stream.pump().unwrap();
    let before = traffic();
    for _ in 0..8 {
        stream.pump().unwrap();
    }
    let after = traffic();

    assert_eq!(before, after, "stream pump must not allocate or free");
    assert_eq!(stream.blocks_rendered(), 9);
    assert!(stream.last_block().iter().all(|sample| *sample == 0.25));
}

#[test]
fn draining_the_control_channel_is_allocation_free() {
    let mut ids = IdGen::new(0x0043_5452_4c00);
    let targets: Vec<ParameterTarget> = (0..8)
        .map(|_| ParameterTarget {
            device: ids.next_id(),
            parameter: ids.next_id(),
        })
        .collect();
    let (mut sender, mut receiver) = control_channel(&targets, 256, 32).unwrap();

    // Warm-up block so any first-touch cost is outside the measurement
    sender.parameters().set(0, 0.1).unwrap();
    sender.send_note(note(0)).unwrap();
    sender.send_transport(TransportCommand::Play).unwrap();
    receiver.drain_parameters(|_, _| {});
    while receiver.next_note().is_some() {}
    while receiver.next_transport().is_some() {}

    let before = traffic();
    for block in 0..8 {
        // Producing and consuming on the same thread keeps the counters comparable
        for (index, _) in targets.iter().enumerate() {
            sender.parameters().set(index, block as f32).unwrap();
        }
        for sequence in 0..16 {
            sender.send_note(note(sequence)).unwrap();
        }
        sender.send_transport(TransportCommand::Stop).unwrap();

        let mut applied = 0;
        receiver.drain_parameters(|_, _| applied += 1);
        assert_eq!(applied, targets.len());
        let mut notes = 0;
        while receiver.next_note().is_some() {
            notes += 1;
        }
        assert_eq!(notes, 16);
        assert!(receiver.next_transport().is_some());
    }
    let after = traffic();

    assert_eq!(
        before, after,
        "control send and drain must not allocate or free"
    );
}

// Build a note-on carrying a distinguishing sequence number
fn note(sequence: u64) -> NoteEvent {
    NoteEvent {
        frame_offset: 0,
        sequence,
        kind: NoteEventKind::On {
            id: sequence as u32,
            channel: 0,
            note: 60,
            velocity: 0.5,
        },
    }
}
