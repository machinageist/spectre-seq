// Author: Jeff
// Date: 2026-08-24
// Description: RT-001 and RT-003 evidence for the R4-6 devices at the device boundary
// Notes: spectre-audio's rt_guard drives a compiled plan and covers only the fixture chain it
//   builds; nothing there reaches spectre-dsp. This file guards Filament and Gloam directly with
//   the same thread-local flag and global allocator idiom, including the positive control that
//   makes a passing result mean something rather than being a broken probe

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use spectre_dsp::{
    AudioProcessor, Filament, Gloam, NoteEvent, NoteEventKind, ProcessContext, FILAMENT_PARAMETERS,
    GLOAM_PARAMETERS,
};

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;

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

// Build both devices at their declared defaults
fn devices() -> (Filament, Gloam) {
    (
        Filament::new(
            FILAMENT_PARAMETERS[0].default(),
            FILAMENT_PARAMETERS[1].default(),
            FILAMENT_PARAMETERS[2].default(),
            FILAMENT_PARAMETERS[3].default(),
        )
        .unwrap(),
        Gloam::new(
            GLOAM_PARAMETERS[0].default(),
            GLOAM_PARAMETERS[1].default(),
            GLOAM_PARAMETERS[2].default(),
        )
        .unwrap(),
    )
}

// Build one valid note-on at frame zero
fn note_on(id: u32, note: u8) -> NoteEvent {
    NoteEvent {
        frame_offset: 0,
        sequence: 0,
        kind: NoteEventKind::On {
            id,
            channel: 0,
            note,
            velocity: 0.8,
        },
    }
}

#[test]
fn the_guard_detects_a_deliberate_allocation() {
    // Positive control: proves a passing guard below is a real result, not a broken probe
    let (_, violations) = rt_section(|| {
        let leaked: Vec<u8> = Vec::with_capacity(4_096);
        std::hint::black_box(&leaked);
    });
    assert!(
        violations >= 2,
        "guard must catch both the allocation and its free, saw {violations}"
    );

    let (sum, clean) = rt_section(|| (0..64_u64).sum::<u64>());
    assert_eq!(sum, 2_016);
    assert_eq!(clean, 0, "arithmetic must not register a violation");
}

#[test]
fn voice_chain_process_and_setters_are_rt_clean() {
    let (mut instrument, mut effect) = devices();
    let events = [note_on(1, 69)];
    let mut voice_left = vec![0.0_f32; FRAMES];
    let mut voice_right = vec![0.0_f32; FRAMES];
    let mut out_left = vec![0.0_f32; FRAMES];
    let mut out_right = vec![0.0_f32; FRAMES];
    let voiced = ProcessContext::new(SAMPLE_RATE, FRAMES, &events).unwrap();
    let silent = ProcessContext::new(SAMPLE_RATE, FRAMES, &[]).unwrap();

    let (_, violations) = rt_section(|| {
        for block in 0..8 {
            // Parameter application is a separate step executed once before process
            let lean = FILAMENT_PARAMETERS[0];
            instrument
                .set_parameter(lean.key, lean.minimum() + (block as f32) * 0.1)
                .unwrap();
            let depth = GLOAM_PARAMETERS[1];
            effect.set_parameter(depth.key, depth.maximum()).unwrap();
            let _ = instrument.set_parameter(depth.key, 0.0);
            instrument
                .process(&voiced, &[], &mut [&mut voice_left, &mut voice_right])
                .unwrap();
            effect
                .process(
                    &silent,
                    &[&voice_left, &voice_right],
                    &mut [&mut out_left, &mut out_right],
                )
                .unwrap();
        }
    });

    assert_eq!(violations, 0, "device process path allocated {violations}");
    assert!(out_left.iter().all(|sample| sample.is_finite()));
    assert!(out_left.iter().any(|sample| *sample != 0.0));
}

#[test]
fn devices_emit_no_non_finite_sample_at_extreme_rates() {
    // Rates the process context admits but no device would ever see; the containment argument
    // has to hold at the edges of the contract, not only at 48 kHz
    for sample_rate in [f64::MIN_POSITIVE, 1e-30, 1.0, 8_000.0, 768_000.0, 1e30] {
        let (mut instrument, mut effect) = devices();
        let events = [note_on(1, 127)];
        let mut voice_left = vec![0.0_f32; FRAMES];
        let mut voice_right = vec![0.0_f32; FRAMES];
        let mut out_left = vec![0.0_f32; FRAMES];
        let mut out_right = vec![0.0_f32; FRAMES];
        let voiced = ProcessContext::new(sample_rate, FRAMES, &events).unwrap();
        let silent = ProcessContext::new(sample_rate, FRAMES, &[]).unwrap();

        for _ in 0..4 {
            instrument
                .process(&voiced, &[], &mut [&mut voice_left, &mut voice_right])
                .unwrap();
            effect
                .process(
                    &silent,
                    &[&voice_left, &voice_right],
                    &mut [&mut out_left, &mut out_right],
                )
                .unwrap();
            for sample in voice_left.iter().chain(out_left.iter()) {
                assert!(sample.is_finite(), "rate {sample_rate}: {sample}");
            }
        }
    }
}

// Structural lock guard for the device layer, the companion to spectre-audio's rt_guard.rs scan.
//
// That scan reads its own crate through CARGO_MANIFEST_DIR, so it covered bridge, control, spsc,
// null, and route -- and none of the DSP devices, whose `process` bodies are the most
// callback-reachable code in the workspace. Every module here executes inside the audio callback
// through CompiledPlan::process.
#[test]
fn device_modules_contain_no_blocking_primitives() {
    // Every module carrying an AudioProcessor impl or arithmetic those impls call
    const RENDER_MODULES: [&str; 6] = [
        "src/effect.rs",
        "src/source.rs",
        "src/filament.rs",
        "src/gloam.rs",
        "src/mix.rs",
        "src/io.rs",
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
    for module in RENDER_MODULES {
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
