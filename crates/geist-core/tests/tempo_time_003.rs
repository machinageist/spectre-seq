// Author: Jeff
// Date: 2026-07-16
// Description: Deterministic 24-hour and boundary evidence for TIME-003
// Notes: Exercises accepted signed pre-roll and absolute-position rounding semantics

use geist_core::{BeatTicks, SampleRate, SampleTime, TempoMap, TempoSegment, TICKS_PER_BEAT};

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;
const SAMPLE_RATES: [u32; 4] = [44_100, 48_000, 96_000, 192_000];

fn rate(hz: u32) -> SampleRate {
    SampleRate::new(hz).expect("test rates are nonzero")
}

fn day_map() -> TempoMap {
    // Each segment lasts exactly six hours. Integer BPM values make every
    // one-second probe land on an exact BeatTicks position at 960 PPQ.
    TempoMap::new(vec![
        TempoSegment {
            start: BeatTicks(0),
            bpm: 10.0,
        },
        TempoSegment {
            start: BeatTicks::from_beats(3_600),
            bpm: 1_200.0,
        },
        TempoSegment {
            start: BeatTicks::from_beats(435_600),
            bpm: 174.0,
        },
        TempoSegment {
            start: BeatTicks::from_beats(498_240),
            bpm: 300.0,
        },
    ])
    .expect("ordered segments use supported tempos")
}

fn fractional_day_map() -> TempoMap {
    TempoMap::new(vec![
        TempoSegment {
            start: BeatTicks(0),
            bpm: 137.25,
        },
        TempoSegment {
            start: BeatTicks(1_001),
            bpm: 251.5,
        },
        TempoSegment {
            start: BeatTicks(987_653),
            bpm: 83.125,
        },
    ])
    .expect("ordered segments use supported fractional tempos")
}

fn fractional_seconds_at(ticks: BeatTicks) -> f64 {
    let ticks_per_beat = TICKS_PER_BEAT as f64;
    1_001.0 * 60.0 / (137.25 * ticks_per_beat)
        + (987_653.0 - 1_001.0) * 60.0 / (251.5 * ticks_per_beat)
        + (ticks.0 as f64 - 987_653.0) * 60.0 / (83.125 * ticks_per_beat)
}

fn fractional_local_bpm(ticks: BeatTicks) -> f64 {
    match ticks.0 {
        ..1_001 => 137.25,
        1_001..987_653 => 251.5,
        _ => 83.125,
    }
}

fn ticks_at_second(second: i64) -> BeatTicks {
    const SIX_HOURS: i64 = 6 * 60 * 60;
    let (segment_second, start_beats, bpm) = match second / SIX_HOURS {
        0 => (second, 0, 10),
        1 => (second - SIX_HOURS, 3_600, 1_200),
        2 => (second - 2 * SIX_HOURS, 435_600, 174),
        _ => (second - 3 * SIX_HOURS, 498_240, 300),
    };
    BeatTicks::from_beats(start_beats) + BeatTicks(segment_second * bpm * TICKS_PER_BEAT / 60)
}

#[test]
fn supported_rates_and_tempo_extremes_are_exact_at_24_hours() {
    for rate_hz in SAMPLE_RATES {
        for bpm in [10_i64, 120, 174, 300, 1_200] {
            let map = TempoMap::constant(bpm as f64).expect("tempo is supported");
            let ticks = BeatTicks::from_beats(bpm * 24 * 60);
            let samples = SampleTime(i64::from(rate_hz) * SECONDS_PER_DAY);

            assert_eq!(map.ticks_to_samples(ticks, rate(rate_hz)), samples);
            assert_eq!(map.samples_to_ticks(samples, rate(rate_hz)), ticks);
        }
    }
}

#[test]
fn piecewise_map_is_monotone_and_exact_at_one_second_probes_for_24_hours() {
    let map = day_map();

    for rate_hz in SAMPLE_RATES {
        let rate = rate(rate_hz);
        let mut previous = SampleTime(-1);
        for second in 0..=SECONDS_PER_DAY {
            let ticks = ticks_at_second(second);
            let samples = map.ticks_to_samples(ticks, rate);

            assert_eq!(samples, SampleTime(i64::from(rate_hz) * second));
            assert_eq!(map.samples_to_ticks(samples, rate), ticks);
            if second > 0 {
                assert!(samples > previous);
            }
            previous = samples;
        }
    }
}

#[test]
fn fractional_tempo_anchors_round_once_and_boundaries_round_trip_exactly() {
    let map = TempoMap::new(vec![
        TempoSegment {
            start: BeatTicks(0),
            bpm: 137.0,
        },
        TempoSegment {
            start: BeatTicks(1_001),
            bpm: 251.5,
        },
        TempoSegment {
            start: BeatTicks(987_654),
            bpm: 83.0,
        },
    ])
    .expect("ordered segments use supported tempos");
    let expected_boundary_samples = [
        (44_100, [20_139, 10_833_110]),
        (48_000, [21_920, 11_791_140]),
        (96_000, [43_839, 23_582_281]),
        (192_000, [87_679, 47_164_562]),
    ];

    for (rate_hz, expected) in expected_boundary_samples {
        let rate = rate(rate_hz);
        for (boundary, sample) in [BeatTicks(1_001), BeatTicks(987_654)]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(map.ticks_to_samples(boundary, rate), SampleTime(sample));
            assert_eq!(map.samples_to_ticks(SampleTime(sample), rate), boundary);
            assert!(map.ticks_to_samples(boundary - BeatTicks(1), rate) < SampleTime(sample));
            assert!(map.ticks_to_samples(boundary + BeatTicks(1), rate) > SampleTime(sample));
        }
    }
}

#[test]
fn fractional_piecewise_map_stays_within_rounding_tolerance_at_24_hours() {
    let map = fractional_day_map();
    let target = BeatTicks(115_572_941);

    // Independent duration oracle. The odd tick boundaries produce
    // non-integral sample anchors at every supported rate; the selected final
    // tick is 86_399.999_557_385_4 seconds from zero.
    let segment_seconds = [
        1_001.0 * 60.0 / (137.25 * TICKS_PER_BEAT as f64),
        (987_653.0 - 1_001.0) * 60.0 / (251.5 * TICKS_PER_BEAT as f64),
        (target.0 as f64 - 987_653.0) * 60.0 / (83.125 * TICKS_PER_BEAT as f64),
    ];
    let expected_seconds = segment_seconds.into_iter().sum::<f64>();
    assert!((expected_seconds - SECONDS_PER_DAY as f64).abs() < 0.001);

    for rate_hz in SAMPLE_RATES {
        let rate = rate(rate_hz);
        let actual = map.ticks_to_samples(target, rate);
        let expected_samples = expected_seconds * f64::from(rate_hz);

        assert!(
            (actual.0 as f64 - expected_samples).abs() <= 0.500_1,
            "{rate_hz} Hz: {actual:?} differs from independent oracle {expected_samples}"
        );
        assert_eq!(map.samples_to_ticks(actual, rate), target);

        let independently_rounded_segments = segment_seconds
            .into_iter()
            .map(|seconds| (seconds * f64::from(rate_hz)).round() as i64)
            .sum::<i64>();
        if rate_hz == 44_100 {
            assert_eq!(actual, SampleTime(3_810_239_980));
            assert_eq!(independently_rounded_segments, 3_810_239_981);
            assert_ne!(actual.0, independently_rounded_segments);
        }
    }
}

#[test]
fn fractional_piecewise_arbitrary_samples_obey_local_bounds_at_long_horizon() {
    let map = fractional_day_map();

    for rate_hz in SAMPLE_RATES {
        let rate = rate(rate_hz);
        let rate_f64 = f64::from(rate_hz);

        // Probe on both sides of each fractional anchor. The farther probes
        // cross to the adjacent segment's tick grid; the +/-1 probes exercise
        // source samples close enough to select the boundary tick itself.
        for (boundary, seconds, before_bpm, after_bpm) in [
            (
                BeatTicks(1_001),
                1_001.0 * 60.0 / (137.25 * TICKS_PER_BEAT as f64),
                137.25,
                251.5,
            ),
            (
                BeatTicks(987_653),
                1_001.0 * 60.0 / (137.25 * TICKS_PER_BEAT as f64)
                    + (987_653.0 - 1_001.0) * 60.0 / (251.5 * TICKS_PER_BEAT as f64),
                251.5,
                83.125,
            ),
        ] {
            let anchor_sample = (seconds * rate_f64).round() as i64;
            let before_half_tick =
                (rate_f64 * 60.0 / (before_bpm * TICKS_PER_BEAT as f64) / 2.0).ceil() as i64 + 1;
            let after_half_tick =
                (rate_f64 * 60.0 / (after_bpm * TICKS_PER_BEAT as f64) / 2.0).ceil() as i64 + 1;

            for offset in [-before_half_tick, -1, 1, after_half_tick] {
                let original = SampleTime(anchor_sample + offset);
                let selected_tick = map.samples_to_ticks(original, rate);
                let quantized = map.ticks_to_samples(selected_tick, rate);
                let local_bpm = fractional_local_bpm(selected_tick);
                let local_samples_per_tick = rate_f64 * 60.0 / (local_bpm * TICKS_PER_BEAT as f64);
                let bound = local_samples_per_tick / 2.0 + 0.5;
                let error = (quantized - original).0.abs() as f64;

                assert_ne!(quantized, original, "probe must be off the tick grid");
                assert!(
                    error <= bound,
                    "{rate_hz} Hz boundary {boundary:?}, sample {original:?} selected \
                     {selected_tick:?} at {local_bpm} BPM: error {error} > {bound}"
                );
                if offset == -before_half_tick {
                    assert!(selected_tick < boundary);
                } else if offset == after_half_tick {
                    assert!(selected_tick > boundary);
                }
            }
        }

        // The final probes are centered on the independently calculated
        // 86_399.999_557_385_4-second anchor, not on a production conversion.
        let final_tick = BeatTicks(115_572_941);
        let final_sample = (fractional_seconds_at(final_tick) * rate_f64).round() as i64;
        let final_half_tick =
            (rate_f64 * 60.0 / (83.125 * TICKS_PER_BEAT as f64) / 2.0).ceil() as i64 + 1;
        for offset in [-final_half_tick, -1, 1, final_half_tick] {
            let original = SampleTime(final_sample + offset);
            let selected_tick = map.samples_to_ticks(original, rate);
            let quantized = map.ticks_to_samples(selected_tick, rate);
            let local_bpm = fractional_local_bpm(selected_tick);
            let local_samples_per_tick = rate_f64 * 60.0 / (local_bpm * TICKS_PER_BEAT as f64);
            let bound = local_samples_per_tick / 2.0 + 0.5;
            let error = (quantized - original).0.abs() as f64;

            assert_eq!(local_bpm, 83.125);
            assert_ne!(quantized, original, "probe must be off the tick grid");
            assert!(
                error <= bound,
                "{rate_hz} Hz near-day sample {original:?} selected {selected_tick:?}: \
                 error {error} > {bound}"
            );
        }
    }
}

#[test]
fn first_tempo_segment_extends_into_signed_pre_roll() {
    let map = TempoMap::new(vec![
        TempoSegment {
            start: BeatTicks(0),
            bpm: 120.0,
        },
        TempoSegment {
            start: BeatTicks::from_beats(8),
            bpm: 174.0,
        },
    ])
    .expect("ordered segments use supported tempos");
    let rate = rate(48_000);

    for (ticks, samples) in [
        (BeatTicks::from_beats(-4), SampleTime(-96_000)),
        (BeatTicks(-1), SampleTime(-25)),
        (BeatTicks(0), SampleTime(0)),
    ] {
        assert_eq!(map.ticks_to_samples(ticks, rate), samples);
        assert_eq!(map.samples_to_ticks(samples, rate), ticks);
    }
}

#[test]
fn arbitrary_samples_obey_the_nearest_tick_quantization_bound() {
    // At the coarsest supported conversion, one tick is 275.625 samples.
    // Sample 138 therefore selects tick 1 and maps to sample 276: an honest
    // 138-sample error, not the impossible generic one-sample bound.
    let slow_map = TempoMap::constant(10.0).expect("tempo is supported");
    let slow_rate = rate(44_100);
    let sample = SampleTime(138);
    let ticks = slow_map.samples_to_ticks(sample, slow_rate);
    let quantized = slow_map.ticks_to_samples(ticks, slow_rate);
    assert_eq!(ticks, BeatTicks(1));
    assert_eq!(quantized, SampleTime(276));
    assert_eq!((quantized - sample).0.abs(), 138);

    for rate_hz in SAMPLE_RATES {
        for bpm in [10.0, 83.125, 137.25, 251.5, 1_200.0] {
            let map = TempoMap::constant(bpm).expect("tempo is supported");
            let rate = rate(rate_hz);
            let samples_per_tick = f64::from(rate_hz) * 60.0 / (bpm * TICKS_PER_BEAT as f64);
            let bound = samples_per_tick / 2.0 + 0.5;

            for sample in [-1_234_567, -138, -1, 0, 1, 137, 138, 139, 1_234_567] {
                let original = SampleTime(sample);
                let ticks = map.samples_to_ticks(original, rate);
                let quantized = map.ticks_to_samples(ticks, rate);
                let error = (quantized - original).0.abs() as f64;

                assert!(
                    error <= bound,
                    "{rate_hz} Hz/{bpm} BPM sample {sample}: error {error} > {bound}"
                );
            }
        }
    }
}

#[test]
fn piecewise_absolute_positions_accumulate_independent_segment_durations() {
    let map = TempoMap::new(vec![
        TempoSegment {
            start: BeatTicks(0),
            bpm: 1_200.0,
        },
        TempoSegment {
            start: BeatTicks(3),
            bpm: 10.0,
        },
        TempoSegment {
            start: BeatTicks(6),
            bpm: 251.5,
        },
    ])
    .expect("ordered segments use supported tempos");
    let rate = rate(44_100);

    // Independent exact anchors before final rounding:
    // tick 3 = 3 * 2.296875 = 6.890625 samples;
    // tick 6 = 6.890625 + 3 * 275.625 = 833.765625 samples;
    // tick 8 = 833.765625 + 2 * 10.959... = 855.684... samples.
    let positions = [
        (BeatTicks(0), SampleTime(0)),
        (BeatTicks(3), SampleTime(7)),
        (BeatTicks(5), SampleTime(558)),
        (BeatTicks(6), SampleTime(834)),
        (BeatTicks(8), SampleTime(856)),
    ];
    for (ticks, expected) in positions {
        assert_eq!(map.ticks_to_samples(ticks, rate), expected);
    }

    let start = map.ticks_to_samples(BeatTicks(0), rate);
    let first_boundary = map.ticks_to_samples(BeatTicks(3), rate);
    let second_boundary = map.ticks_to_samples(BeatTicks(6), rate);
    let end = map.ticks_to_samples(BeatTicks(8), rate);
    assert_eq!(first_boundary - start, SampleTime(7));
    assert_eq!(second_boundary - first_boundary, SampleTime(827));
    assert_eq!(end - second_boundary, SampleTime(22));
    assert_eq!(
        (first_boundary - start) + (second_boundary - first_boundary) + (end - second_boundary),
        end - start
    );

    // In the 10 BPM segment, each standalone one-tick duration rounds from
    // 275.625 to 276. Two such rounded durations (552) differ from subtracting
    // rounded absolute positions at ticks 3 and 5 (558 - 7 = 551).
    let independently_rounded_parts = SampleTime(276) + SampleTime(276);
    assert_eq!(independently_rounded_parts, SampleTime(552));
    assert_ne!(
        independently_rounded_parts,
        map.ticks_to_samples(BeatTicks(5), rate) - first_boundary
    );
}
