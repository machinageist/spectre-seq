// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 8 evidence — timestamped MIDI ingress and equal-timestamp ordering
// Notes: The ordering assertions are checked twice: directly, and by feeding the result through
//   the real plan, which rejects unsorted events. A test that only inspected the array could
//   agree with itself while still violating the contract the DSP layer enforces.

use spectre_audio::midi::{MidiError, MidiIngress, MidiMessage, MAX_BLOCK_EVENTS};
use spectre_core::{IdGen, SampleTime};
use spectre_dsp::{AudioProcessor, Gain, NoteEventKind, PulseInstrument, Waveform};
use spectre_graph::{Connection, EditableGraph, NodeId, PlanNoteInput};

const FRAMES: usize = 512;
const BLOCK_START: SampleTime = SampleTime(48_000);

const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;
const CONTROL_CHANGE: u8 = 0xB0;
const ALL_NOTES_OFF: u8 = 123;

// Build a message at an offset relative to the standard block start
fn at(offset: i64, status: u8, data1: u8, data2: u8) -> MidiMessage {
    MidiMessage::new(SampleTime(BLOCK_START.0 + offset), status, data1, data2)
}

#[test]
fn timestamps_become_block_relative_frame_offsets() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    ingress
        .push(at(0, NOTE_ON, 60, 100), BLOCK_START, FRAMES)
        .unwrap();
    ingress
        .push(at(128, NOTE_ON, 64, 100), BLOCK_START, FRAMES)
        .unwrap();
    ingress
        .push(at(511, NOTE_ON, 67, 100), BLOCK_START, FRAMES)
        .unwrap();

    let events = ingress.block_events();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].frame_offset, 0);
    assert_eq!(events[1].frame_offset, 128);
    assert_eq!(events[2].frame_offset, 511);
}

#[test]
fn a_timestamp_beyond_the_block_is_refused_for_a_later_block() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    assert_eq!(
        ingress.push(at(FRAMES as i64, NOTE_ON, 60, 100), BLOCK_START, FRAMES),
        Err(MidiError::AfterBlock)
    );
    assert!(ingress.block_events().is_empty());
    // The same message lands once its block arrives
    let next_start = SampleTime(BLOCK_START.0 + FRAMES as i64);
    ingress.begin_block();
    ingress
        .push(at(FRAMES as i64, NOTE_ON, 60, 100), next_start, FRAMES)
        .unwrap();
    assert_eq!(ingress.block_events()[0].frame_offset, 0);
}

#[test]
fn a_late_timestamp_clamps_to_the_block_start_and_is_counted() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    ingress
        .push(at(-4_096, NOTE_ON, 60, 100), BLOCK_START, FRAMES)
        .unwrap();

    // Playing it immediately beats discarding a real performance gesture
    assert_eq!(ingress.block_events()[0].frame_offset, 0);
    assert_eq!(ingress.late_messages(), 1);
}

#[test]
fn equal_timestamps_place_releases_before_attacks() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    // Sound a note in an earlier block so the release has something to match
    ingress
        .push(at(0, NOTE_ON, 60, 100), BLOCK_START, FRAMES)
        .unwrap();
    ingress.block_events();

    ingress.begin_block();
    // Deliberately push the attack first; ordering must not depend on arrival order
    ingress
        .push(at(64, NOTE_ON, 67, 100), BLOCK_START, FRAMES)
        .unwrap();
    ingress
        .push(at(64, NOTE_OFF, 60, 0), BLOCK_START, FRAMES)
        .unwrap();

    let events = ingress.block_events();
    assert_eq!(events.len(), 2);
    assert!(
        matches!(events[0].kind, NoteEventKind::Off { .. }),
        "release must precede the attack at one frame offset"
    );
    assert!(matches!(events[1].kind, NoteEventKind::On { .. }));
}

#[test]
fn equal_timestamps_and_equal_rank_keep_arrival_order() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    for note in [72, 64, 67, 60] {
        ingress
            .push(at(32, NOTE_ON, note, 100), BLOCK_START, FRAMES)
            .unwrap();
    }

    let notes: Vec<u8> = ingress
        .block_events()
        .iter()
        .map(|event| match event.kind {
            NoteEventKind::On { note, .. } => note,
            _ => unreachable!("only attacks were pushed"),
        })
        .collect();
    // Sequence breaks the tie, so a chord keeps the order the driver delivered
    assert_eq!(notes, vec![72, 64, 67, 60]);
}

#[test]
fn out_of_order_timestamps_are_sorted_into_contract_order() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    for (offset, note) in [(400, 72), (16, 60), (256, 67), (0, 64)] {
        ingress
            .push(at(offset, NOTE_ON, note, 100), BLOCK_START, FRAMES)
            .unwrap();
    }

    let offsets: Vec<usize> = ingress
        .block_events()
        .iter()
        .map(|event| event.frame_offset)
        .collect();
    assert_eq!(offsets, vec![0, 16, 256, 400]);
}

#[test]
fn note_off_matches_the_id_its_note_on_allocated() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    ingress
        .push(at(0, NOTE_ON, 60, 100), BLOCK_START, FRAMES)
        .unwrap();
    ingress
        .push(at(100, NOTE_OFF, 60, 0), BLOCK_START, FRAMES)
        .unwrap();

    let events = ingress.block_events();
    let (on_id, off_id) = match (events[0].kind, events[1].kind) {
        (NoteEventKind::On { id: on, .. }, NoteEventKind::Off { id: off, .. }) => (on, off),
        _ => unreachable!("expected an attack then a release"),
    };
    assert_ne!(on_id, 0, "note IDs are nonzero");
    assert_eq!(on_id, off_id, "release must carry its attack's identity");
}

#[test]
fn note_on_with_zero_velocity_is_a_release() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    ingress
        .push(at(0, NOTE_ON, 60, 100), BLOCK_START, FRAMES)
        .unwrap();
    // Running-status convention: note-on velocity 0 means note-off
    ingress
        .push(at(64, NOTE_ON, 60, 0), BLOCK_START, FRAMES)
        .unwrap();

    let events = ingress.block_events();
    assert!(matches!(events[1].kind, NoteEventKind::Off { .. }));
}

#[test]
fn an_unmatched_release_is_refused() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    assert_eq!(
        ingress.push(at(0, NOTE_OFF, 60, 0), BLOCK_START, FRAMES),
        Err(MidiError::UnmatchedNoteOff)
    );
    assert!(ingress.block_events().is_empty());
}

#[test]
fn all_notes_off_releases_the_channel() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    ingress
        .push(at(0, NOTE_ON, 60, 100), BLOCK_START, FRAMES)
        .unwrap();
    ingress
        .push(
            at(64, CONTROL_CHANGE, ALL_NOTES_OFF, 0),
            BLOCK_START,
            FRAMES,
        )
        .unwrap();

    let events = ingress.block_events();
    assert!(matches!(
        events[1].kind,
        NoteEventKind::AllNotesOff { channel: Some(0) }
    ));

    // Every note on that channel is released, so a later note-off no longer matches
    ingress.begin_block();
    assert_eq!(
        ingress.push(at(0, NOTE_OFF, 60, 0), BLOCK_START, FRAMES),
        Err(MidiError::UnmatchedNoteOff)
    );
}

#[test]
fn unsupported_and_out_of_range_messages_are_refused() {
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    // Pitch bend is outside the v1 event vocabulary
    assert_eq!(
        ingress.push(at(0, 0xE0, 0, 64), BLOCK_START, FRAMES),
        Err(MidiError::Unsupported)
    );
    // A non-all-notes-off controller is also outside it
    assert_eq!(
        ingress.push(at(0, CONTROL_CHANGE, 7, 100), BLOCK_START, FRAMES),
        Err(MidiError::Unsupported)
    );
    assert_eq!(
        ingress.push(at(0, NOTE_ON, 200, 100), BLOCK_START, FRAMES),
        Err(MidiError::OutOfRange)
    );
    assert!(ingress.block_events().is_empty());
}

#[test]
fn a_full_block_buffer_refuses_rather_than_overwrites() {
    let mut ingress = MidiIngress::new(4);
    ingress.begin_block();
    for index in 0..4 {
        ingress
            .push(
                at(index, NOTE_ON, 60 + index as u8, 100),
                BLOCK_START,
                FRAMES,
            )
            .unwrap();
    }
    assert_eq!(
        ingress.push(at(5, NOTE_ON, 80, 100), BLOCK_START, FRAMES),
        Err(MidiError::BufferFull)
    );
    assert_eq!(ingress.block_events().len(), 4);
}

#[test]
fn capacity_is_clamped_to_the_dsp_contract_limit() {
    // An oversized request must not create a buffer the plan would reject as over capacity
    let mut ingress = MidiIngress::new(MAX_BLOCK_EVENTS * 4);
    ingress.begin_block();
    for index in 0..MAX_BLOCK_EVENTS {
        let note = (index % 128) as u8;
        ingress
            .push(
                at((index % FRAMES) as i64, NOTE_ON, note, 100),
                BLOCK_START,
                FRAMES,
            )
            .unwrap();
    }
    assert_eq!(
        ingress.push(at(0, NOTE_ON, 60, 100), BLOCK_START, FRAMES),
        Err(MidiError::BufferFull),
        "capacity must clamp to the contract's per-quantum limit"
    );
    assert_eq!(ingress.block_events().len(), MAX_BLOCK_EVENTS);
}

#[test]
fn ingress_output_is_accepted_by_the_real_plan() {
    // The plan validates event ordering, so this is the assertion that actually binds
    let mut ingress = MidiIngress::default();
    ingress.begin_block();
    ingress
        .push(at(0, NOTE_ON, 45, 100), BLOCK_START, FRAMES)
        .unwrap();
    ingress.block_events();

    ingress.begin_block();
    for (offset, status, note) in [
        (300, NOTE_ON, 52),
        (100, NOTE_ON, 48),
        (100, NOTE_OFF, 45),
        (300, NOTE_ON, 55),
    ] {
        ingress
            .push(at(offset, status, note, 100), BLOCK_START, FRAMES)
            .unwrap();
    }

    let mut ids = IdGen::new(0x0000_004d_4944_4900);
    let pulse = NodeId::new(ids.next_id());
    let gain = NodeId::new(ids.next_id());
    let mut graph = EditableGraph::new();
    graph
        .add_node(
            pulse,
            PulseInstrument::new(Waveform::Saw, 0.3).unwrap().io(),
        )
        .unwrap();
    graph.add_node(gain, Gain::new(0.7).unwrap().io()).unwrap();
    graph
        .connect(Connection {
            from: pulse,
            from_bus: 0,
            to: gain,
            to_bus: 0,
        })
        .unwrap();
    let mut plan = graph
        .compile(gain, FRAMES, &mut |node| {
            if node == pulse {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
            } else {
                Ok(Box::new(Gain::new(0.7)?))
            }
        })
        .unwrap();

    let events = ingress.block_events();
    plan.process(
        48_000.0,
        FRAMES,
        &[PlanNoteInput {
            node: pulse,
            events,
        }],
    )
    .expect("ingress must emit events the plan's ordering contract accepts");
}
