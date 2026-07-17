// Author: Jeff
// Date: 2026-07-11
// Description: Property tests for tempo round-trips, loop wrap, event ordering, and ID stability
// Notes: Acceptance evidence for TIME-002, TIME-003, TIME-005, CORE-001

use geist_core::{
    sort_events, BeatTicks, EventKind, IdGen, LoopRegion, MeterChange, MeterMap, SampleDuration,
    SampleRate, SampleTime, TempoMap, TempoSegment, TimeSignature, TimedEvent, Transport,
    TransportCommand, TICKS_PER_BEAT,
};
use proptest::prelude::*;

// Strategy for a valid time signature
fn time_signature_strategy() -> impl Strategy<Value = TimeSignature> {
    (1u8..=99, prop::sample::select(vec![1u8, 2, 4, 8, 16]))
        .prop_map(|(n, d)| TimeSignature::new(n, d).expect("strategy builds valid signatures"))
}

// Strategy for a valid strictly ordered meter map
fn meter_map_strategy() -> impl Strategy<Value = MeterMap> {
    (
        prop::collection::vec((1i64..512, time_signature_strategy()), 0..6),
        time_signature_strategy(),
    )
        .prop_map(|(steps, first)| {
            let mut changes = vec![MeterChange {
                start: BeatTicks(0),
                signature: first,
            }];
            let mut at = 0i64;
            for (beats, signature) in steps {
                at += beats;
                changes.push(MeterChange {
                    start: BeatTicks::from_beats(at),
                    signature,
                });
            }
            MeterMap::new(changes).expect("strategy builds valid maps")
        })
}

// Strategy for a valid multi-segment tempo map
fn tempo_map_strategy() -> impl Strategy<Value = TempoMap> {
    (
        prop::collection::vec((1i64..512, 20.0f64..300.0), 0..6),
        20.0f64..300.0,
    )
        .prop_map(|(steps, first_bpm)| {
            let mut segments = vec![TempoSegment {
                start: BeatTicks(0),
                bpm: first_bpm,
            }];
            let mut at = 0i64;
            for (beats, bpm) in steps {
                at += beats;
                segments.push(TempoSegment {
                    start: BeatTicks::from_beats(at),
                    bpm,
                });
            }
            TempoMap::new(segments).expect("strategy builds valid maps")
        })
}

proptest! {
    // TIME-003: ticks -> samples -> ticks round-trips within one sample of tick error
    #[test]
    fn tempo_round_trip_within_one_sample(
        map in tempo_map_strategy(),
        beats in 0i64..4096,
        rate_hz in prop::sample::select(vec![44_100u32, 48_000, 96_000, 192_000]),
    ) {
        let rate = SampleRate::new(rate_hz).unwrap();
        let ticks = BeatTicks::from_beats(beats);
        let samples = map.ticks_to_samples(ticks, rate);
        let back = map.samples_to_ticks(samples, rate);
        let err = (back - ticks).0.abs() as f64;
        // One sample of tick error at the slowest permitted tempo in the strategy
        let worst_ticks_per_sample = 300.0 * TICKS_PER_BEAT as f64 / (rate_hz as f64 * 60.0);
        prop_assert!(err <= worst_ticks_per_sample.ceil() + 1.0,
            "round-trip error {err} ticks at {beats} beats, {rate_hz} Hz");
    }

    // TIME-003: conversion is monotone in musical position
    #[test]
    fn tempo_conversion_is_monotone(
        map in tempo_map_strategy(),
        a in 0i64..4096,
        b in 0i64..4096,
    ) {
        let rate = SampleRate::new(48_000).unwrap();
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let sa = map.ticks_to_samples(BeatTicks::from_beats(lo), rate);
        let sb = map.ticks_to_samples(BeatTicks::from_beats(hi), rate);
        prop_assert!(sa <= sb);
    }

    // TIME-002/TIME-005: loop wrap always lands inside the region and counts wraps exactly
    #[test]
    fn loop_wrap_stays_in_region(
        start in 0i64..10_000,
        len in 1i64..10_000,
        offset in 0i64..10_000,
        block in 1i64..100_000,
    ) {
        let region = LoopRegion::new(SampleTime(start), SampleTime(start + len)).unwrap();
        let mut t = Transport::new();
        t.apply(TransportCommand::SetLoop(Some(region)));
        t.apply(TransportCommand::Seek(SampleTime(start + (offset % len))));
        t.apply(TransportCommand::Play);
        let before = t.position;
        let wraps = t.advance(SampleDuration::new(block as u64));
        prop_assert!(region.contains(t.position));
        let travelled = (t.position.0 - before.0)
            + wraps as i64 * region.len().samples() as i64;
        prop_assert_eq!(travelled, block, "distance must be conserved across wraps");
    }

    // TIME-002: event sorting is a total deterministic order (idempotent, permutation-invariant)
    #[test]
    fn event_order_is_permutation_invariant(
        times in prop::collection::vec(0i64..64, 2..40),
        seed in 0u64..1000,
    ) {
        let kinds = [
            EventKind::TransportSeek { to: SampleTime(0) },
            EventKind::NoteOff { channel: 0, key: 60 },
            EventKind::NoteOn { channel: 0, key: 60, velocity: 100 },
            EventKind::Control { channel: 0, cc: 1, value: 64 },
        ];
        let mut events: Vec<TimedEvent> = times
            .iter()
            .enumerate()
            .map(|(i, t)| TimedEvent {
                time: SampleTime(*t),
                sequence: i as u64,
                kind: kinds[(i + seed as usize) % kinds.len()],
            })
            .collect();
        let mut shuffled = events.clone();
        shuffled.reverse();
        sort_events(&mut events);
        sort_events(&mut shuffled);
        prop_assert_eq!(events, shuffled);
    }

    // CORE-001: generated IDs are unique and reproducible per seed
    #[test]
    fn ids_unique_and_seed_stable(seed in 0u64..10_000, count in 1usize..2000) {
        let mut g1 = IdGen::new(seed);
        let mut g2 = IdGen::new(seed);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..count {
            let id = g1.next_id();
            prop_assert_eq!(id, g2.next_id());
            prop_assert!(seen.insert(id));
        }
    }

    // TIME-004: serde round trip preserves any valid meter map exactly
    #[test]
    fn meter_map_serde_round_trip(map in meter_map_strategy()) {
        let json = serde_json::to_string(&map).expect("serialize");
        let back: MeterMap = serde_json::from_str(&json).expect("deserialize valid map");
        prop_assert_eq!(back, map);
    }

    // TIME-004: lookup matches a linear scan for any position, including before zero
    #[test]
    fn meter_lookup_matches_linear_scan(
        map in meter_map_strategy(),
        beats in -8i64..4096,
    ) {
        let pos = BeatTicks::from_beats(beats);
        let expected = map
            .changes()
            .iter()
            .rev()
            .find(|c| c.start <= pos)
            .unwrap_or(&map.changes()[0])
            .signature;
        prop_assert_eq!(map.signature_at(pos), expected);
    }

    // TIME-004: shuffling a multi-change map breaks strict ordering and is rejected
    #[test]
    fn meter_map_rejects_reordered_changes(map in meter_map_strategy()) {
        let mut changes = map.changes().to_vec();
        if changes.len() >= 2 {
            changes.reverse();
            prop_assert!(MeterMap::new(changes).is_err());
        }
    }

    // TIME-004: bar length in ticks is always positive and exactly divisible by the numerator
    #[test]
    fn meter_bar_length_is_exact(sig in time_signature_strategy()) {
        let bar = sig.ticks_per_bar();
        prop_assert!(bar.0 > 0);
        prop_assert_eq!(bar.0 % sig.numerator() as i64, 0);
        prop_assert_eq!(bar.0 / sig.numerator() as i64, TICKS_PER_BEAT * 4 / sig.denominator() as i64);
    }
}
