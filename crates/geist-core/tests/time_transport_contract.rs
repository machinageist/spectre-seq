// Author: Jeff
// Date: 2026-07-16
// Description: R1 acceptance evidence for typed sample durations and deterministic event/transport time
// Notes: Covers TIME-001, TIME-002, and TIME-005 public contracts

use geist_core::{
    sort_events, EventKind, LoopRegion, ObjectId, SampleDuration, SampleRate, SampleTime, Seconds,
    TimedEvent, Transport, TransportCommand, TransportState,
};

fn duration(samples: u64) -> SampleDuration {
    SampleDuration::new(samples)
}

fn event(sequence: u64, kind: EventKind) -> TimedEvent {
    TimedEvent {
        time: SampleTime(512),
        sequence,
        kind,
    }
}

#[test]
fn sample_duration_has_explicit_checked_and_saturating_arithmetic() {
    assert_eq!(duration(7).samples(), 7);
    assert_eq!(duration(7).checked_add(duration(5)), Some(duration(12)));
    assert_eq!(duration(u64::MAX).checked_add(duration(1)), None);
    assert_eq!(
        duration(u64::MAX).saturating_add(duration(1)),
        duration(u64::MAX)
    );

    assert_eq!(
        SampleTime(i64::MAX - 1).checked_add_duration(duration(1)),
        Some(SampleTime(i64::MAX))
    );
    assert_eq!(SampleTime(i64::MAX).checked_add_duration(duration(1)), None);
    assert_eq!(
        SampleTime(i64::MAX).saturating_add_duration(duration(1)),
        SampleTime(i64::MAX)
    );
}

#[test]
fn wall_seconds_conversion_is_typed_and_rounds_to_nearest_sample() {
    let rate = SampleRate::new(2).unwrap();
    assert_eq!(
        SampleDuration::checked_from_seconds(Seconds(0.25), rate),
        Some(duration(1)),
        "half-sample ties round upward"
    );
    assert_eq!(
        SampleDuration::checked_from_seconds(Seconds(1.25), rate),
        Some(duration(3))
    );
    assert_eq!(duration(3).to_seconds(rate), Seconds(1.5));
}

#[test]
fn wall_seconds_conversion_rejects_invalid_or_out_of_range_values() {
    let rate = SampleRate::new(48_000).unwrap();
    for seconds in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.001, f64::MAX] {
        assert_eq!(
            SampleDuration::checked_from_seconds(Seconds(seconds), rate),
            None
        );
    }
}

#[test]
fn same_sample_order_covers_all_five_levels_and_sequence_ties() {
    let param = ObjectId::from_raw(1).unwrap();
    let kinds = [
        EventKind::TransportSeek { to: SampleTime(-1) },
        EventKind::NoteOff {
            channel: 1,
            key: 60,
        },
        EventKind::NoteOn {
            channel: 1,
            key: 60,
            velocity: 100,
        },
        EventKind::Control {
            channel: 1,
            cc: 74,
            value: 99,
        },
        EventKind::ParamChange { param, value: 0.5 },
    ];
    let mut events = vec![
        event(90, kinds[4]),
        event(80, kinds[3]),
        event(70, kinds[2]),
        event(60, kinds[1]),
        event(50, kinds[0]),
        event(9, kinds[3]),
        event(3, kinds[3]),
    ];

    sort_events(&mut events);

    assert_eq!(
        events.iter().map(|event| event.kind).collect::<Vec<_>>(),
        vec![kinds[0], kinds[1], kinds[2], kinds[3], kinds[3], kinds[3], kinds[4]]
    );
    assert_eq!(
        events[3..6]
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![3, 9, 80]
    );
}

#[test]
fn event_order_handles_signed_sample_boundaries_and_seek_payloads() {
    let mut events = vec![
        TimedEvent {
            time: SampleTime(i64::MAX),
            sequence: 0,
            kind: EventKind::TransportSeek {
                to: SampleTime(i64::MIN),
            },
        },
        TimedEvent {
            time: SampleTime(i64::MIN),
            sequence: 1,
            kind: EventKind::TransportSeek {
                to: SampleTime(i64::MAX),
            },
        },
        TimedEvent {
            time: SampleTime(0),
            sequence: 2,
            kind: EventKind::TransportSeek {
                to: SampleTime(-128),
            },
        },
    ];

    sort_events(&mut events);

    assert_eq!(
        events.iter().map(|event| event.time).collect::<Vec<_>>(),
        vec![SampleTime(i64::MIN), SampleTime(0), SampleTime(i64::MAX)]
    );
}

#[derive(Clone, Copy)]
enum Step {
    Command(TransportCommand),
    Advance(SampleDuration),
}

fn replay(steps: &[Step]) -> (Transport, Vec<u32>) {
    let mut transport = Transport::new();
    let mut wraps = Vec::new();
    for step in steps {
        match *step {
            Step::Command(command) => transport.apply(command),
            Step::Advance(samples) => wraps.push(transport.advance(samples)),
        }
    }
    (transport, wraps)
}

#[test]
fn transport_command_model_covers_states_seek_loop_toggle_and_replay() {
    let loop_region = LoopRegion::new(SampleTime(100), SampleTime(110)).unwrap();
    assert_eq!(loop_region.len(), duration(10));
    let steps = [
        Step::Command(TransportCommand::Seek(SampleTime(105))),
        Step::Advance(duration(3)), // stopped: no movement
        Step::Command(TransportCommand::Play),
        Step::Advance(duration(8)), // wrap to 113 without a loop
        Step::Command(TransportCommand::SetLoop(Some(loop_region))),
        Step::Command(TransportCommand::Seek(SampleTime(109))),
        Step::Advance(duration(1)), // half-open end wraps to start
        Step::Command(TransportCommand::Record),
        Step::Advance(duration(25)), // recording wraps twice
        Step::Command(TransportCommand::SetLoop(None)),
        Step::Advance(duration(7)), // disabled loop advances linearly
        Step::Command(TransportCommand::Stop),
        Step::Advance(duration(99)),
    ];

    let first = replay(&steps);
    let second = replay(&steps);

    assert_eq!(
        first, second,
        "replaying the same commands must be identical"
    );
    assert_eq!(first.1, vec![0, 0, 1, 2, 0, 0]);
    assert_eq!(first.0.state, TransportState::Stopped);
    assert_eq!(first.0.position, SampleTime(112));
    assert_eq!(first.0.loop_region, None);
    assert!(!first.0.loop_enabled);
}

#[test]
fn seeking_to_loop_end_is_outside_and_advances_without_wrapping() {
    let region = LoopRegion::new(SampleTime(-10), SampleTime(10)).unwrap();
    let mut transport = Transport::new();
    transport.apply(TransportCommand::SetLoop(Some(region)));
    transport.apply(TransportCommand::Seek(region.end));
    transport.apply(TransportCommand::Play);

    assert_eq!(transport.advance(duration(1)), 0);
    assert_eq!(transport.position, SampleTime(11));
}
