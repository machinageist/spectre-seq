// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 5 evidence — bake arithmetic, emission ordering, and the player's refusals
// Notes: Carries its own guarding allocator and positive control, matching rt_guard.rs. The guard
//   is load-bearing: without the positive control a broken probe would report success everywhere.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use spectre_audio::clip::{
    ClipBlockOutcome, ClipPlayer, ClipSchedule, ScheduleError, CLIP_EVENT_RESERVE,
    CLIP_NOTE_ID_BASE, CLIP_SEQUENCE_BAND,
};
use spectre_core::{
    BeatTicks, LoopRegion, SampleDuration, SampleRate, SampleTime, TempoMap, Transport,
    TransportCommand, TICKS_PER_BEAT,
};
use spectre_dsp::{NoteEvent, NoteEventKind, ProcessContext};

const RATE_HZ: u32 = 48_000;
const RATE: f64 = 48_000.0;
const FRAMES: usize = 256;
const BEAT: i64 = TICKS_PER_BEAT;

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

fn rate() -> SampleRate {
    SampleRate::new(RATE_HZ).unwrap()
}

fn tempo() -> TempoMap {
    TempoMap::constant(120.0).unwrap()
}

// A schedule from (start_tick, length_tick, note) triples on channel 0 at full velocity
fn schedule_of(notes: &[(i64, i64, u8)]) -> ClipSchedule {
    ClipSchedule::bake(
        notes
            .iter()
            .map(|(start, length, note)| (BeatTicks(*start), BeatTicks(*length), 0, *note, 1.0)),
        &tempo(),
        rate(),
        CLIP_EVENT_RESERVE,
    )
    .unwrap()
}

// A playing transport at the origin
fn playing() -> Transport {
    let mut transport = Transport::new();
    transport.apply(TransportCommand::Play);
    transport
}

// Render one block and return the events the player emitted
fn block(player: &mut ClipPlayer, transport: &mut Transport, out: &mut Vec<NoteEvent>) {
    out.clear();
    player.emit_block(transport, FRAMES, out, CLIP_SEQUENCE_BAND);
    transport.advance(SampleDuration::new(FRAMES as u64));
}

// S-1
#[test]
fn bake_places_a_beat_at_the_tempo_maps_sample() {
    let schedule = schedule_of(&[(BEAT, BEAT, 60)]);
    // 120 BPM is two beats a second, so one beat is 24,000 samples at 48 kHz. The value is the
    // tempo map's own; the bake adds no second conversion
    assert_eq!(schedule.notes()[0].start, SampleTime(24_000));
    assert_eq!(schedule.notes()[0].end, SampleTime(48_000));
    assert_eq!(
        schedule.notes()[0].start,
        tempo().ticks_to_samples(BeatTicks(BEAT), rate())
    );
}

// S-2
#[test]
fn bake_ids_are_disjoint_from_ingress_ids() {
    let notes: Vec<(i64, i64, u8)> = (0..100).map(|i| (i * BEAT, BEAT, 60)).collect();
    let schedule = schedule_of(&notes);

    assert_eq!(schedule.len(), 100);
    for note in schedule.notes() {
        // MidiIngress allocates from 1 upward, so anything below the base could collide with it
        assert!(note.id >= CLIP_NOTE_ID_BASE);
        assert_ne!(note.id, 0);
    }
}

// S-3
#[test]
fn emit_block_produces_contract_ordered_events() {
    // At 120 BPM and 48 kHz one tick is exactly 25 samples, so tick 4 is sample 100 — inside the
    // first block. Two notes end there and two begin there
    let schedule = ClipSchedule::bake(
        [
            (BeatTicks(0), BeatTicks(4), 0, 60_u8, 1.0_f32),
            (BeatTicks(0), BeatTicks(4), 0, 64, 1.0),
            (BeatTicks(4), BeatTicks(4), 0, 67, 1.0),
            (BeatTicks(4), BeatTicks(4), 0, 71, 1.0),
        ]
        .into_iter(),
        &tempo(),
        rate(),
        CLIP_EVENT_RESERVE,
    )
    .unwrap();
    assert_eq!(schedule.notes()[0].end, SampleTime(100));
    assert_eq!(schedule.notes()[2].start, SampleTime(100));

    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule));
    player.release_all();
    let mut transport = playing();
    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);

    // Drain the release the install queued, then walk the block holding the boundary
    out.clear();
    player.emit_block(&transport, FRAMES, &mut out, CLIP_SEQUENCE_BAND);

    // The plan's own validator is the referee; a hand-check of the array could agree with itself
    assert!(ProcessContext::new(RATE, FRAMES, &out).is_ok());

    let boundary: Vec<&NoteEvent> = out.iter().filter(|e| e.frame_offset == 100).collect();
    assert_eq!(
        boundary.len(),
        4,
        "two releases and two attacks share frame 100"
    );
    assert_eq!(boundary[0].kind.rank(), 0);
    assert_eq!(boundary[1].kind.rank(), 0);
    assert_eq!(boundary[2].kind.rank(), 1);
    assert_eq!(boundary[3].kind.rank(), 1);
    let _ = transport.advance(SampleDuration::new(FRAMES as u64));
}

// S-4
#[test]
fn clip_sequences_never_collide_with_ingress_sequences() {
    let schedule = schedule_of(&[(0, BEAT, 60)]);
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule));
    let mut transport = playing();

    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);
    player.emit_block(&transport, FRAMES, &mut out, CLIP_SEQUENCE_BAND);
    let clip_on = *out
        .iter()
        .find(|event| matches!(event.kind, NoteEventKind::On { .. }))
        .unwrap();

    // An ingress event at the identical offset and identical rank. MidiIngress numbers from 0
    let ingress = NoteEvent {
        frame_offset: clip_on.frame_offset,
        sequence: 0,
        kind: NoteEventKind::On {
            id: 1,
            channel: 0,
            note: 62,
            velocity: 0.5,
        },
    };
    assert_ne!(ingress.sequence, clip_on.sequence);

    let mut merged = vec![ingress, clip_on];
    merged.sort_unstable_by(spectre_audio::clip::contract_order);
    // Equality is a rejection in ProcessContext::new, not a tie, so a shared key would fail here
    assert!(ProcessContext::new(RATE, FRAMES, &merged).is_ok());
    // The deliberate tie-break: the thing the player just did wins
    assert_eq!(merged[0].sequence, 0);
    let _ = transport.advance(SampleDuration::new(FRAMES as u64));
}

// S-5
#[test]
fn a_note_spanning_blocks_emits_on_then_off_once_each() {
    // 32 ticks at 25 samples a tick is 800 samples: it sounds on frames 0..799 and is silent from
    // 800, which lands in block 3. Half-open per TIME-002 puts the release on the first silent
    // frame, not the last sounding one — placing it at end-1 would shorten every note by a sample,
    // so the block index here is the contract's and not this slice's choice
    let schedule = schedule_of(&[(0, 32, 60)]);
    let id = schedule.notes()[0].id;
    assert_eq!(schedule.notes()[0].end, SampleTime(800));

    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule));
    player.release_all();
    let mut transport = playing();
    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);

    let mut attacks = Vec::new();
    let mut releases = Vec::new();
    for index in 0..5 {
        block(&mut player, &mut transport, &mut out);
        for event in &out {
            match event.kind {
                NoteEventKind::On { id: event_id, .. } => attacks.push((index, event_id)),
                NoteEventKind::Off { id: event_id, .. } => releases.push((index, event_id)),
                NoteEventKind::AllNotesOff { .. } => {}
            }
        }
    }

    assert_eq!(attacks, vec![(0, id)], "one attack, in the first block");
    assert_eq!(
        releases,
        vec![(3, id)],
        "one release, on the first silent frame"
    );
}

// S-6
#[test]
fn a_loop_wrap_releases_then_reattacks_at_the_same_frame() {
    // Loop of one block; a note starting at the loop's origin and lasting the whole region
    let region = LoopRegion::new(SampleTime(0), SampleTime(FRAMES as i64)).unwrap();
    let ticks = |s: SampleTime| tempo().samples_to_ticks(s, rate());
    let schedule = ClipSchedule::bake(
        [(
            ticks(SampleTime(0)),
            BeatTicks(ticks(SampleTime(FRAMES as i64 / 2)).0),
            0_u8,
            60_u8,
            1.0_f32,
        )]
        .into_iter(),
        &tempo(),
        rate(),
        CLIP_EVENT_RESERVE,
    )
    .unwrap();

    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule));
    player.release_all();
    let mut transport = playing();
    transport.apply(TransportCommand::SetLoop(Some(region)));
    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);

    // First block covers the whole region exactly, so it wraps to its own start
    block(&mut player, &mut transport, &mut out);
    block(&mut player, &mut transport, &mut out);

    assert!(
        ProcessContext::new(RATE, FRAMES, &out).is_ok(),
        "rank alone must order the wrap"
    );
    assert!(
        out.iter()
            .any(|e| matches!(e.kind, NoteEventKind::On { .. })),
        "the loop must re-attack its material"
    );
}

// S-7
#[test]
fn a_block_over_a_sub_block_loop_refuses_and_releases() {
    // A one-sample loop region is legal; a 256-frame block over it would need 256 segments
    let region = LoopRegion::new(SampleTime(0), SampleTime(1)).unwrap();
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule_of(&[(0, BEAT, 60)])));
    let mut transport = playing();
    transport.apply(TransportCommand::SetLoop(Some(region)));

    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);
    let outcome = player.emit_block(&transport, FRAMES, &mut out, CLIP_SEQUENCE_BAND);

    assert_eq!(outcome, ClipBlockOutcome::SegmentLimitExceeded);
    assert_eq!(out.len(), 1, "fail closed, never a partial event set");
    assert!(matches!(
        out[0].kind,
        NoteEventKind::AllNotesOff { channel: None }
    ));
    assert_eq!(out[0].frame_offset, 0);
}

// S-8
#[test]
fn exceeding_the_event_reserve_refuses_the_block() {
    // Notes stacked at the origin: each emits an attack, and the reserve binds before the block
    // ends. Representable material, which is exactly why the refusal has to exist
    let stacked: Vec<(BeatTicks, BeatTicks, u8, u8, f32)> = (0..CLIP_EVENT_RESERVE + 2)
        .map(|index| {
            (
                BeatTicks(0),
                BeatTicks(8 * BEAT),
                0_u8,
                (index % 128) as u8,
                1.0_f32,
            )
        })
        .collect();
    let schedule = ClipSchedule::bake(
        stacked.into_iter(),
        &tempo(),
        rate(),
        CLIP_EVENT_RESERVE + 2,
    )
    .unwrap();

    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule));
    player.release_all();
    let transport = playing();

    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE + 8);
    // Drain the pending release first so the refusal is attributable to the reserve
    player.emit_block(&transport, FRAMES, &mut out, CLIP_SEQUENCE_BAND);
    out.clear();
    let outcome = player.emit_block(&transport, FRAMES, &mut out, CLIP_SEQUENCE_BAND);

    assert_eq!(outcome, ClipBlockOutcome::EventReserveExceeded);
    assert_eq!(out.len(), 1, "no partial set, so no hung note");
    assert!(matches!(
        out[0].kind,
        NoteEventKind::AllNotesOff { channel: None }
    ));
    assert_eq!(player.sounding_count(), 0);
}

// S-9
#[test]
fn release_all_emits_all_notes_off_at_frame_zero() {
    let schedule = schedule_of(&[(0, 8 * BEAT, 60), (0, 8 * BEAT, 64)]);
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule));
    player.release_all();
    let mut transport = playing();
    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);

    block(&mut player, &mut transport, &mut out);
    assert_eq!(player.sounding_count(), 2, "both notes must be sounding");

    player.release_all();
    out.clear();
    player.emit_block(&transport, FRAMES, &mut out, CLIP_SEQUENCE_BAND);

    assert_eq!(out[0].frame_offset, 0);
    assert!(matches!(
        out[0].kind,
        NoteEventKind::AllNotesOff { channel: None }
    ));
    assert_eq!(player.sounding_count(), 0);
}

// S-10
#[test]
fn install_returns_the_previous_schedule() {
    let first = Box::new(schedule_of(&[(0, BEAT, 60)]));
    let second = Box::new(schedule_of(&[(0, BEAT, 62), (BEAT, BEAT, 64)]));
    let first_id = first.notes()[0].id;

    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let discarded = player.install(first);
    assert!(discarded.is_empty(), "the initial schedule is empty");

    let retired = player.install(second);
    // The box comes back so the caller can hand it to the reclaim lane; nothing drops here
    assert_eq!(retired.len(), 1);
    assert_eq!(retired.notes()[0].id, first_id);
    assert_eq!(player.schedule().len(), 2);
}

// S-11
#[test]
fn an_empty_schedule_emits_nothing() {
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(ClipSchedule::empty()));
    let mut transport = playing();
    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);

    // The first block carries the release the install queued; every one after is silent
    block(&mut player, &mut transport, &mut out);
    for _ in 0..10 {
        block(&mut player, &mut transport, &mut out);
        assert!(out.is_empty(), "zero clips is a legal, silent state");
        assert!(ProcessContext::new(RATE, FRAMES, &out).is_ok());
    }
}

// S-12
#[test]
fn bake_refuses_a_non_finite_velocity() {
    for bad in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        let result = ClipSchedule::bake(
            [(BeatTicks(0), BeatTicks(BEAT), 0_u8, 60_u8, bad)].into_iter(),
            &tempo(),
            rate(),
            CLIP_EVENT_RESERVE,
        );
        assert!(
            matches!(result, Err(ScheduleError::InvalidVelocity(_))),
            "velocity {bad} must be refused at the app-thread boundary"
        );
    }
    // The other bounds NoteEvent::is_valid enforces are refused at the same boundary
    for (channel, note) in [(16_u8, 60_u8), (0, 128)] {
        assert!(ClipSchedule::bake(
            [(BeatTicks(0), BeatTicks(BEAT), channel, note, 1.0)].into_iter(),
            &tempo(),
            rate(),
            CLIP_EVENT_RESERVE,
        )
        .is_err());
    }
    // And a bake past its own capacity refuses rather than truncating
    assert!(matches!(
        ClipSchedule::bake(
            (0..3).map(|i| (BeatTicks(i * BEAT), BeatTicks(BEAT), 0, 60, 1.0)),
            &tempo(),
            rate(),
            2,
        ),
        Err(ScheduleError::Capacity {
            requested: 3,
            limit: 2
        })
    ));
}

// The positive control. Without it a broken guard would report success in the test below
#[test]
fn the_guard_detects_a_deliberate_allocation() {
    let (_, violations) = rt_section(|| {
        let probe: Vec<u8> = Vec::with_capacity(64);
        probe.len()
    });
    assert!(
        violations > 0,
        "the allocation guard did not fire on a deliberate allocation"
    );
}

// S-13
#[test]
fn emit_block_allocates_nothing() {
    // Dense enough to exercise the sounding table and the sort, but inside the reserve
    let notes: Vec<(i64, i64, u8)> = (0..64)
        .map(|index| (index * (BEAT / 8), BEAT / 4, 48 + (index % 24) as u8))
        .collect();
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule_of(&notes)));
    let mut transport = playing();
    let mut out = Vec::with_capacity(CLIP_EVENT_RESERVE);

    // One unguarded block first: it drains the release the install queued
    block(&mut player, &mut transport, &mut out);

    let ((), violations) = rt_section(|| {
        for _ in 0..1_000 {
            out.clear();
            player.emit_block(&transport, FRAMES, &mut out, CLIP_SEQUENCE_BAND);
            transport.advance(SampleDuration::new(FRAMES as u64));
        }
    });
    assert_eq!(violations, 0, "emit_block violated RT-001");
}
