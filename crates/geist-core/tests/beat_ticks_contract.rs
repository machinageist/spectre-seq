// Author: Jeff
// Date: 2026-07-16
// Description: Acceptance evidence for the R1 BeatTicks resolution and representation contract
// Notes: Pins the accepted 960 ticks/beat resolution and exact common musical grids

use geist_core::{BeatTicks, TICKS_PER_BEAT};

const MAX_SAFE_WHOLE_BEATS: i64 = 9_607_679_205_057_058;
const MIN_SAFE_WHOLE_BEATS: i64 = -9_607_679_205_057_058;

#[test]
fn beat_tick_resolution_supports_exact_common_grids() {
    assert_eq!(TICKS_PER_BEAT, 960);

    for divisor in [2i64, 3, 4, 5, 6, 8, 10, 12, 16, 24, 32, 48, 64] {
        assert_eq!(
            TICKS_PER_BEAT % divisor,
            0,
            "grid 1/{divisor} must be exact"
        );
    }
}

#[test]
fn checked_whole_beat_construction_accepts_safe_boundaries() {
    assert_eq!(
        BeatTicks::checked_from_beats(MAX_SAFE_WHOLE_BEATS),
        Some(BeatTicks(9_223_372_036_854_775_680))
    );
    assert_eq!(
        BeatTicks::checked_from_beats(MIN_SAFE_WHOLE_BEATS),
        Some(BeatTicks(-9_223_372_036_854_775_680))
    );
}

#[test]
fn checked_whole_beat_construction_rejects_overflow() {
    assert_eq!(
        BeatTicks::checked_from_beats(MAX_SAFE_WHOLE_BEATS + 1),
        None
    );
    assert_eq!(
        BeatTicks::checked_from_beats(MIN_SAFE_WHOLE_BEATS - 1),
        None
    );
}

#[test]
fn beat_ticks_json_is_a_signed_integer_at_contract_values() {
    let cases = [
        (BeatTicks(0), "0"),
        (BeatTicks(-960), "-960"),
        (BeatTicks(480), "480"),
        (BeatTicks(320), "320"),
        (BeatTicks(240), "240"),
        (BeatTicks(192), "192"),
        (BeatTicks(160), "160"),
        (BeatTicks(120), "120"),
        (BeatTicks(96), "96"),
        (BeatTicks(80), "80"),
        (BeatTicks(60), "60"),
        (BeatTicks(40), "40"),
        (BeatTicks(30), "30"),
        (BeatTicks(20), "20"),
        (BeatTicks(15), "15"),
        (BeatTicks(9_223_372_036_854_775_680), "9223372036854775680"),
        (
            BeatTicks(-9_223_372_036_854_775_680),
            "-9223372036854775680",
        ),
    ];

    for (ticks, expected_json) in cases {
        let json = serde_json::to_string(&ticks).expect("BeatTicks must serialize");
        assert_eq!(json, expected_json);
        assert_eq!(
            serde_json::from_str::<BeatTicks>(&json).expect("BeatTicks must deserialize"),
            ticks
        );
    }
}
