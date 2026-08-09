// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 3 evidence — RT-002 split-lane control transport
// Notes: Proves the decision-21 policy directly: parameters coalesce and can never starve the
//   queue, notes and transport keep strict FIFO order and report overflow instead of dropping,
//   and retired state is dropped on the app thread rather than the render thread.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use spectre_audio::control::{
    control_channel, ControlError, ParameterTarget, DEFAULT_NOTE_CAPACITY,
    DEFAULT_TRANSPORT_CAPACITY,
};
use spectre_core::{IdGen, ObjectId, SampleTime, TransportCommand};
use spectre_dsp::{NoteEvent, NoteEventKind};

// Build a deterministic set of distinct parameter targets
fn targets(count: usize) -> Vec<ParameterTarget> {
    let mut ids = IdGen::new(0x0052_4543);
    (0..count)
        .map(|_| ParameterTarget {
            device: ids.next_id(),
            parameter: ids.next_id(),
        })
        .collect()
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

#[test]
fn parameter_sweep_coalesces_to_one_latest_value() {
    let targets = targets(1);
    let (sender, mut receiver) =
        control_channel(&targets, DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY).unwrap();

    // A sweep far larger than any queue depth still occupies exactly one slot
    for step in 0..5_000 {
        sender.parameters().set(0, step as f32).unwrap();
    }

    let mut applied = Vec::new();
    let count = receiver.drain_parameters(|target, value| applied.push((target, value)));
    assert_eq!(count, 1, "a sweep must coalesce to a single application");
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].0, targets[0]);
    assert_eq!(applied[0].1, 4_999.0, "latest value wins");

    // A second drain with no writes applies nothing
    assert_eq!(receiver.drain_parameters(|_, _| panic!("no change")), 0);
}

#[test]
fn drain_applies_only_changed_targets() {
    let targets = targets(4);
    let (sender, mut receiver) =
        control_channel(&targets, DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY).unwrap();

    sender.parameters().set(1, 0.25).unwrap();
    sender.parameters().set(3, 0.75).unwrap();

    let mut applied = Vec::new();
    let count = receiver.drain_parameters(|target, value| applied.push((target, value)));
    assert_eq!(count, 2);
    assert_eq!(applied, vec![(targets[1], 0.25), (targets[3], 0.75)]);

    // Re-writing the same value still counts as a change; versions, not values, drive delivery
    sender.parameters().set(1, 0.25).unwrap();
    assert_eq!(receiver.drain_parameters(|_, _| {}), 1);
}

#[test]
fn parameter_values_survive_bit_exactly() {
    let targets = targets(4);
    let (sender, mut receiver) =
        control_channel(&targets, DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY).unwrap();

    let exact = [-0.0_f32, f32::MIN_POSITIVE / 2.0, f32::MAX, f32::NAN];
    for (index, value) in exact.iter().enumerate() {
        sender.parameters().set(index, *value).unwrap();
    }

    let mut seen = Vec::new();
    receiver.drain_parameters(|_, value| seen.push(value));
    assert_eq!(seen.len(), 4);
    // Signed zero and subnormals must survive as bits, matching the snapshot contract
    assert_eq!(seen[0].to_bits(), (-0.0_f32).to_bits());
    assert_eq!(seen[1].to_bits(), (f32::MIN_POSITIVE / 2.0).to_bits());
    assert_eq!(seen[2], f32::MAX);
    assert!(seen[3].is_nan());
}

#[test]
fn parameter_targets_are_validated() {
    let targets = targets(2);
    let (sender, _receiver) =
        control_channel(&targets, DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY).unwrap();

    assert_eq!(
        sender.parameters().set(2, 1.0).unwrap_err(),
        ControlError::TargetIndexOutOfRange(2)
    );

    let unknown = ParameterTarget {
        device: ObjectId::from_raw(0xDEAD).unwrap(),
        parameter: ObjectId::from_raw(0xBEEF).unwrap(),
    };
    assert_eq!(
        sender.parameters().set_target(unknown, 1.0).unwrap_err(),
        ControlError::UnknownTarget(unknown)
    );

    // Targeting by identity resolves to the same slot as targeting by index
    sender.parameters().set_target(targets[1], 0.5).unwrap();
    assert_eq!(sender.parameters().index_of(targets[1]), Some(1));
}

#[test]
fn duplicate_targets_are_refused_at_construction() {
    let base = targets(1);
    let duplicated = vec![base[0], base[0]];
    assert_eq!(
        control_channel(
            &duplicated,
            DEFAULT_NOTE_CAPACITY,
            DEFAULT_TRANSPORT_CAPACITY
        )
        .err()
        .unwrap(),
        ControlError::DuplicateTarget(base[0])
    );
}

#[test]
fn note_lane_keeps_strict_fifo_order() {
    let (mut sender, mut receiver) = control_channel(&targets(1), 64, 8).unwrap();
    for sequence in 0..32 {
        sender.send_note(note(sequence)).unwrap();
    }
    for expected in 0..32 {
        assert_eq!(receiver.next_note().unwrap().sequence, expected);
    }
    assert!(receiver.next_note().is_none());
}

#[test]
fn note_lane_overflow_is_reported_and_counted_never_silent() {
    let (mut sender, mut receiver) = control_channel(&targets(1), 4, 8).unwrap();

    // Fill until the lane refuses, which is the documented defect path
    let mut accepted = 0;
    loop {
        match sender.send_note(note(accepted)) {
            Ok(()) => accepted += 1,
            Err(error) => {
                assert_eq!(error, ControlError::NoteLaneFull);
                break;
            }
        }
    }
    assert!(accepted >= 4, "requested depth must be honored");
    assert_eq!(sender.telemetry().note_overflows(), 1);

    // Everything accepted is still delivered in order; nothing was dropped to make room
    for expected in 0..accepted {
        assert_eq!(receiver.next_note().unwrap().sequence, expected);
    }
    assert!(receiver.next_note().is_none());
}

#[test]
fn transport_lane_keeps_order_and_reports_overflow() {
    let (mut sender, mut receiver) = control_channel(&targets(1), 8, 2).unwrap();
    let commands = [
        TransportCommand::Play,
        TransportCommand::Seek(SampleTime(4_800)),
        TransportCommand::Stop,
    ];

    let mut accepted = Vec::new();
    for command in commands {
        match sender.send_transport(command) {
            Ok(()) => accepted.push(command),
            Err(error) => assert_eq!(error, ControlError::TransportLaneFull),
        }
    }
    for expected in &accepted {
        assert_eq!(receiver.next_transport().unwrap(), *expected);
    }
    assert!(receiver.next_transport().is_none());
}

// Records the thread that dropped it, proving reclamation happens off the render thread
struct DropWitness {
    log: Arc<Mutex<Vec<ThreadId>>>,
}

impl Drop for DropWitness {
    // Record the dropping thread
    fn drop(&mut self) {
        self.log.lock().unwrap().push(std::thread::current().id());
    }
}

#[test]
fn retired_state_is_dropped_on_the_app_thread() {
    let (mut sender, mut receiver) = control_channel(&targets(1), 8, 8).unwrap();
    let log: Arc<Mutex<Vec<ThreadId>>> = Arc::default();
    let app_thread = std::thread::current().id();

    let witness = Box::new(DropWitness {
        log: Arc::clone(&log),
    });

    // The render thread retires the value and must not drop it
    let render_thread = std::thread::spawn(move || {
        let id = std::thread::current().id();
        receiver.retire(witness).map_err(|_| ()).unwrap();
        (receiver, id)
    });
    let (_receiver, render_id) = render_thread.join().unwrap();

    assert!(log.lock().unwrap().is_empty(), "not dropped while retiring");
    assert_eq!(sender.reclaim(), 1);

    let dropped_on = log.lock().unwrap().clone();
    assert_eq!(dropped_on.len(), 1);
    assert_eq!(dropped_on[0], app_thread, "must drop on the app thread");
    assert_ne!(
        dropped_on[0], render_id,
        "must not drop on the render thread"
    );
}

#[test]
fn reclaim_overflow_hands_the_value_back_instead_of_dropping_it() {
    let (mut sender, mut receiver) = control_channel(&targets(1), 8, 8).unwrap();
    let log: Arc<Mutex<Vec<ThreadId>>> = Arc::default();

    // Fill the reclaim lane without draining it
    let mut rejected = None;
    for _ in 0..256 {
        let witness = Box::new(DropWitness {
            log: Arc::clone(&log),
        });
        if let Err(returned) = receiver.retire(witness) {
            rejected = Some(returned);
            break;
        }
    }

    let returned = rejected.expect("reclaim lane must eventually refuse");
    assert!(receiver.telemetry().reclaim_overflows() >= 1);
    // Nothing has been dropped yet: the refused value came back intact
    assert!(log.lock().unwrap().is_empty());

    drop(returned);
    assert_eq!(log.lock().unwrap().len(), 1);
    sender.reclaim();
}

#[test]
fn parameters_and_notes_cross_threads_without_loss() {
    let targets = targets(2);
    let (mut sender, mut receiver) = control_channel(&targets, 4_096, 64).unwrap();
    let total = 2_000_u64;

    let producer = std::thread::spawn(move || {
        for sequence in 0..total {
            sender.parameters().set(0, sequence as f32).unwrap();
            sender.send_note(note(sequence)).unwrap();
        }
        sender
    });

    let received = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&received);
    let consumer = std::thread::spawn(move || {
        let mut last = None;
        while counter.load(Ordering::Relaxed) < total as usize {
            while let Some(event) = receiver.next_note() {
                let expected = last.map_or(0, |value| value + 1);
                assert_eq!(event.sequence, expected, "FIFO order must hold");
                last = Some(event.sequence);
                counter.fetch_add(1, Ordering::Relaxed);
            }
            receiver.drain_parameters(|_, _| {});
        }
        receiver
    });

    let _sender = producer.join().unwrap();
    let _receiver = consumer.join().unwrap();
    assert_eq!(received.load(Ordering::Relaxed), total as usize);
}
