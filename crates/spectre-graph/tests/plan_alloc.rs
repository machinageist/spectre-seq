// Author: Jeff
// Date: 2026-07-17
// Description: R2 allocation gate — CompiledPlan::process allocates and frees nothing
// Notes: Thread-local counters keep parallel test threads from polluting the measurement

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use spectre_core::IdGen;
use spectre_dsp::{
    AudioProcessor, Gain, NoteEvent, NoteEventKind, PulseInstrument, Saturator, SumBus, Waveform,
};
use spectre_graph::{Connection, EditableGraph, NodeId, PlanNoteInput};

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
fn plan_process_is_allocation_free() {
    let mut ids = IdGen::new(0x0041_4c4c_4f43);
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
    let mut plan = graph
        .compile(saturator, 512, &mut |node| {
            if node == pulse {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
            } else if node == gain {
                Ok(Box::new(Gain::new(0.7)?))
            } else {
                Ok(Box::new(Saturator::new(2.5, 0.35)?))
            }
        })
        .unwrap();

    let events = [
        NoteEvent {
            frame_offset: 0,
            sequence: 0,
            kind: NoteEventKind::On {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.8,
            },
        },
        NoteEvent {
            frame_offset: 511,
            sequence: 1,
            kind: NoteEventKind::Off {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.0,
            },
        },
    ];
    let notes = [PlanNoteInput {
        node: pulse,
        events: &events,
    }];

    // Warm-up quantum, then measure steady-state quanta
    plan.process(48_000.0, 512, &notes).unwrap();
    let before = traffic();
    for _ in 0..8 {
        plan.process(48_000.0, 512, &notes).unwrap();
    }
    let after = traffic();
    assert_eq!(before, after, "plan process must not allocate or free");
    assert!(plan.last_output().is_some());
}

// R4-4 test 14 — the summing device runs under the same allocation guard as every other node.
// A SumBus node is an ordinary plan step, so putting one in the fixture puts the new device
// under the existing RT-001 guard rather than inventing a second one
#[test]
fn a_plan_containing_a_sum_bus_is_allocation_free() {
    let mut ids = IdGen::new(0x0053_554d_414c_4c00);
    let pulse = NodeId::new(ids.next_id());
    let gain = NodeId::new(ids.next_id());
    let sum = NodeId::new(ids.next_id());
    let master = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            pulse,
            PulseInstrument::new(Waveform::Saw, 0.3).unwrap().io(),
        )
        .unwrap();
    graph.add_node(gain, Gain::new(0.7).unwrap().io()).unwrap();
    graph.add_node(sum, SumBus::new(1).unwrap().io()).unwrap();
    graph
        .add_node(master, Gain::new(1.0).unwrap().io())
        .unwrap();
    for (from, to, to_bus) in [(pulse, gain, 0), (gain, sum, 0), (sum, master, 0)] {
        graph
            .connect(Connection {
                from,
                from_bus: 0,
                to,
                to_bus,
            })
            .unwrap();
    }
    let mut plan = graph
        .compile(master, 256, &mut |node| {
            if node == pulse {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
            } else if node == gain {
                Ok(Box::new(Gain::new(0.7)?))
            } else if node == sum {
                Ok(Box::new(SumBus::new(1)?))
            } else {
                Ok(Box::new(Gain::new(1.0)?))
            }
        })
        .unwrap();

    let events = [NoteEvent {
        frame_offset: 0,
        sequence: 0,
        kind: NoteEventKind::On {
            id: 1,
            channel: 0,
            note: 45,
            velocity: 0.8,
        },
    }];
    let inputs = [PlanNoteInput {
        node: pulse,
        events: &events,
    }];
    // Warm the path first, so the measurement covers steady state rather than first-call setup
    plan.process(48_000.0, 256, &inputs).unwrap();

    let before = traffic();
    for _ in 0..8 {
        plan.process(48_000.0, 256, &[]).unwrap();
    }
    let after = traffic();
    assert_eq!(after.0 - before.0, 0, "the sum path must not allocate");
    assert_eq!(after.1 - before.1, 0, "the sum path must not deallocate");
}
