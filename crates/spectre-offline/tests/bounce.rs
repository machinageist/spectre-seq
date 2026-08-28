// Author: Jeff
// Date: 2026-08-25
// Description: R4 slice 8 evidence — the shared fold, the block loop, and the WAV it writes
// Notes: Test 1's golden vector is the pin the whole refactor rests on. It involves no render and
//   no DSP deliberately: a test that hashed a render and compared it against the same function
//   would compare the implementation against itself and could not fail.

use spectre_core::{IdGen, ObjectId};
use spectre_dsp::{
    DeviceParameterSnapshot, NoteEvent, NoteEventKind, GAIN_PARAMETERS, PULSE_PARAMETERS,
    SATURATOR_PARAMETERS,
};
use spectre_graph::NodeId;
use spectre_offline::bounce::{
    bounce_into, bounce_report, fallback_config, first_divergence, max_frames, BounceConfig,
    BounceError, BounceProgress, BOUNCE_FALLBACK_BLOCK_FRAMES, BOUNCE_FALLBACK_SAMPLE_RATE,
};
use spectre_offline::fixture;
use spectre_offline::hash::{hash_block, hash_planar_quantum, SampleHasher};
use spectre_offline::{fixture_events, render_vertical_slice};
use std::sync::atomic::{AtomicBool, Ordering};

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;

// Eight f32 values, every one exactly representable in binary32, so the literals carry no
// rounding and the bit pattern is the whole input
const GOLDEN_INPUT: [f32; 8] = [0.0, -0.0, 1.0, -1.0, 0.5, -0.25, 3.75, -0.001953125];
// Bits in order: 0x00000000, 0x80000000, 0x3f800000, 0xbf800000,
//                0x3f000000, 0xbe800000, 0x40700000, 0xbb000000
const GOLDEN_HASH: u64 = 0xa49a_cc9c_e735_9a37;

// The complete four-parameter fixture snapshot at the fixture's own device values.
// Identities match the harness's own so a snapshot rejected there is rejected here for the same
// reason: DeviceValues::from_snapshot is the single gate both paths pass through
fn id(raw: u64) -> ObjectId {
    ObjectId::from_raw(raw).unwrap()
}

fn fixture_snapshot() -> Vec<DeviceParameterSnapshot> {
    vec![
        DeviceParameterSnapshot::new(
            id(10),
            "pulse",
            id(11),
            PULSE_PARAMETERS[0],
            fixture::FIXTURE_PULSE_LEVEL,
        ),
        DeviceParameterSnapshot::new(
            id(20),
            "gain",
            id(21),
            GAIN_PARAMETERS[0],
            fixture::FIXTURE_GAIN,
        ),
        DeviceParameterSnapshot::new(
            id(30),
            "saturator",
            id(31),
            SATURATOR_PARAMETERS[0],
            fixture::FIXTURE_SATURATOR_DRIVE,
        ),
        DeviceParameterSnapshot::new(
            id(30),
            "saturator",
            id(32),
            SATURATOR_PARAMETERS[1],
            fixture::FIXTURE_SATURATOR_MIX,
        ),
    ]
}

fn config(frames: usize, block_frames: usize) -> BounceConfig {
    BounceConfig {
        sample_rate: RATE,
        frames,
        block_frames,
        log_block_hashes: false,
    }
}

// A deterministic interleaved stream; a property of the hasher, so no render is involved
fn generated_stream(frames: usize) -> Vec<f32> {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut out = Vec::with_capacity(frames * 2);
    for _ in 0..frames * 2 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.push((state >> 40) as f32 / 8_388_608.0 - 1.0);
    }
    out
}

// 1
#[test]
fn the_shared_fold_matches_its_checked_in_golden_vector() {
    let mut hasher = SampleHasher::new();
    for sample in GOLDEN_INPUT {
        hasher.write(sample);
    }
    assert_eq!(hasher.finish(), GOLDEN_HASH);

    // The channel-major walk visits all of channel 0 then all of channel 1, which for this
    // split is exactly the array's own order
    assert_eq!(
        hash_planar_quantum([&GOLDEN_INPUT[..4], &GOLDEN_INPUT[4..]]),
        GOLDEN_HASH
    );

    // +0.0 and -0.0 sit next to each other because RT-003 flushes denormals to *signed* zero.
    // A fold that compared or normalized values instead of bytes would collapse them
    assert_eq!(GOLDEN_INPUT[0].to_bits(), 0x0000_0000);
    assert_eq!(GOLDEN_INPUT[1].to_bits(), 0x8000_0000);
    assert_ne!(
        {
            let mut one = SampleHasher::new();
            one.write(0.0);
            one.finish()
        },
        {
            let mut other = SampleHasher::new();
            other.write(-0.0);
            other.finish()
        }
    );
}

// 2
#[test]
fn the_streaming_hash_is_independent_of_how_the_stream_is_chopped() {
    let stream = generated_stream(4_096);

    let whole = {
        let mut hasher = SampleHasher::new();
        hash_block(&mut hasher, &stream, 2, 4_096);
        hasher.finish()
    };
    let even = {
        let mut hasher = SampleHasher::new();
        for chunk in stream.chunks(256 * 2) {
            hash_block(&mut hasher, chunk, 2, chunk.len() / 2);
        }
        hasher.finish()
    };
    let uneven = {
        let mut hasher = SampleHasher::new();
        let split = 100 * 2;
        hash_block(&mut hasher, &stream[..split], 2, 100);
        hash_block(&mut hasher, &stream[split..], 2, 3_996);
        hasher.finish()
    };

    // The composability property the whole bounce rests on. A channel-major hash_block
    // would fail here
    assert_eq!(whole, even);
    assert_eq!(whole, uneven);
}

// 3
#[test]
fn planar_and_interleaved_streaming_hashes_agree() {
    let (mut plan, note_node) = fixture::compile_fixture_plan(BLOCK).unwrap();
    let events = fixture_events(BLOCK);
    plan.process(
        RATE,
        BLOCK,
        &[spectre_graph::PlanNoteInput {
            node: note_node,
            events: &events,
        }],
    )
    .unwrap();
    let output = plan.last_output().unwrap();

    let planar = {
        let mut hasher = SampleHasher::new();
        spectre_offline::hash::hash_planar_block(&mut hasher, [output[0], output[1]], BLOCK);
        hasher.finish()
    };
    let mut interleaved = vec![0.0_f32; BLOCK * 2];
    for frame in 0..BLOCK {
        interleaved[frame * 2] = output[0][frame];
        interleaved[frame * 2 + 1] = output[1][frame];
    }
    let woven = {
        let mut hasher = SampleHasher::new();
        hash_block(&mut hasher, &interleaved, 2, BLOCK);
        hasher.finish()
    };

    // Two entry points into one traversal. If they drift, the live and offline sides of the
    // equivalence test disagree for a reason that has nothing to do with the audio
    assert_eq!(planar, woven);
}

// 4 — asserts an inequality, and that is the point
#[test]
fn the_streaming_hash_is_deliberately_not_the_quantum_hash() {
    let (mut plan, note_node) = fixture::compile_fixture_plan(512).unwrap();
    let events = fixture_events(512);
    plan.process(
        RATE,
        512,
        &[spectre_graph::PlanNoteInput {
            node: note_node,
            events: &events,
        }],
    )
    .unwrap();
    let output = plan.last_output().unwrap();

    let quantum = hash_planar_quantum([output[0], output[1]]);
    let streaming = {
        let mut hasher = SampleHasher::new();
        spectre_offline::hash::hash_planar_block(&mut hasher, [output[0], output[1]], 512);
        hasher.finish()
    };

    // Channel-major and frame-major are different walks of the same audio. If someone later
    // "unifies" them without thinking, this fires and sends them to the argument rather than
    // to a silently broken equivalence claim
    assert_ne!(quantum, streaming);
}

// 5 — the claim that makes a bounce comparable to a live render at all, made falsifiable
#[test]
fn a_clean_render_is_identical_at_every_block_size() {
    let snapshot = fixture_snapshot();
    let events = fixture_events(4_096);

    // 300 divides nothing here on purpose. Without it every block is full, the short-tail trim
    // never runs, and a bounce that ignored `produced` would pass this test unchanged
    let reports: Vec<_> = [64, 128, 256, 300, 512]
        .into_iter()
        .map(|block| bounce_report(config(4_096, block), &snapshot, &events).unwrap())
        .collect();

    for report in &reports[1..] {
        assert_eq!(report.hash, reports[0].hash);
        assert_eq!(report.peak, reports[0].peak);
        assert_eq!(report.denormals_flushed, reports[0].denormals_flushed);
    }
    for report in &reports {
        assert_eq!(report.contaminated_nodes, 0);
        assert_eq!(report.frames, 4_096);
    }
    // This is the test that fires the day a device's output depends on block size — an LFO
    // stepped once per block, an FFT frame, a lookahead delay. That is why the claim is scoped
    // to the R4 device set rather than asserted as an invariant
    assert!(reports[0].peak > 0.0, "a silent render would prove nothing");
}

// 7
#[test]
fn events_are_rebased_per_block_and_the_note_lands_on_the_right_frame() {
    let snapshot = fixture_snapshot();
    // Offsets are absolute in the caller's terms; the bounce rebases them per block
    let events = [
        NoteEvent {
            frame_offset: 700,
            sequence: 0,
            kind: NoteEventKind::On {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.8,
            },
        },
        NoteEvent {
            frame_offset: 900,
            sequence: 1,
            kind: NoteEventKind::Off {
                id: 1,
                channel: 0,
                note: 45,
                velocity: 0.0,
            },
        },
    ];

    let mut wav = Vec::new();
    let report = bounce_into(
        config(1_024, BLOCK),
        &snapshot,
        &events,
        &mut wav,
        &AtomicBool::new(false),
        &BounceProgress::default(),
    )
    .unwrap();

    // A plan error is distinguishable from silence: un-rebased offsets would be refused by
    // ProcessContext::new and surface as BounceError::Plan rather than as a quiet buffer
    assert_eq!(report.blocks, 4);
    assert!(report.peak > 0.0, "the note must sound somewhere");

    let samples = samples_of(&wav);
    assert!(
        samples[..700 * 2].iter().all(|s| *s == 0.0),
        "nothing may sound before the note-on"
    );
    assert!(
        samples[700 * 2..900 * 2].iter().any(|s| *s != 0.0),
        "the note must sound while it is held"
    );
    assert!(
        samples[900 * 2..].iter().all(|s| *s == 0.0),
        "nothing may sound after the note-off"
    );
}

// 8
#[test]
fn silence_stays_exactly_silent_across_every_block() {
    let report = bounce_report(config(4_096, BLOCK), &fixture_snapshot(), &[]).unwrap();
    assert_eq!(report.peak, 0.0);

    // The reference is computed here, not taken from the implementation
    let mut hasher = SampleHasher::new();
    for _ in 0..4_096 * 2 {
        hasher.write(0.0);
    }
    assert_eq!(report.hash, hasher.finish());
}

// 9
#[test]
fn repeated_bounces_of_the_same_input_are_identical() {
    let snapshot = fixture_snapshot();
    let events = fixture_events(2_048);
    // The log follows the comparison, and no comparison is running here, so the test asks for
    // it rather than inheriting it
    let mut settings = config(2_048, BLOCK);
    settings.log_block_hashes = true;

    let reports: Vec<_> = (0..3)
        .map(|_| bounce_report(settings, &snapshot, &events).unwrap())
        .collect();

    assert_eq!(reports[0].block_hashes.len(), 8);
    for report in &reports[1..] {
        assert_eq!(report.hash, reports[0].hash);
        assert_eq!(report.block_hashes, reports[0].block_hashes);
    }
    assert!(reports[0].peak > 0.0);
}

// 10
#[test]
fn a_length_beyond_the_ceiling_is_refused_before_anything_is_rendered() {
    let ceiling = max_frames(RATE);
    let result = bounce_report(config(ceiling + 1, BLOCK), &fixture_snapshot(), &[]);

    assert!(matches!(
        result,
        Err(BounceError::LengthOutOfRange { frames, max_frames })
            if frames == ceiling + 1 && max_frames == ceiling
    ));
    // 24 hours at 48 kHz, the accepted time horizon expressed in samples
    assert_eq!(ceiling, 24 * 60 * 60 * 48_000);
}

// 11
#[test]
fn a_zero_length_or_zero_block_config_is_refused() {
    let snapshot = fixture_snapshot();
    for broken in [config(0, BLOCK), config(1_024, 0)] {
        assert!(matches!(
            bounce_report(broken, &snapshot, &[]),
            Err(BounceError::InvalidConfig(_))
        ));
    }
    // A non-finite or non-positive rate is refused at the same boundary
    for rate in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut broken = config(1_024, BLOCK);
        broken.sample_rate = rate;
        assert!(matches!(
            bounce_report(broken, &snapshot, &[]),
            Err(BounceError::InvalidConfig(_))
        ));
    }
    // And an incomplete fixture snapshot cannot silently render at defaults
    assert!(matches!(
        bounce_report(config(1_024, BLOCK), &snapshot[..3], &[]),
        Err(BounceError::InvalidConfig(_))
    ));
}

// 12
#[test]
fn a_cancelled_bounce_stops_and_reports_the_blocks_it_wrote() {
    // A flag already set: cancellation is observed at the block boundary, which is what makes
    // deleting the partial file a bounded operation
    let cancel = AtomicBool::new(true);
    let mut wav = Vec::new();
    let result = bounce_into(
        config(4_096, BLOCK),
        &fixture_snapshot(),
        &[],
        &mut wav,
        &cancel,
        &BounceProgress::default(),
    );
    assert!(matches!(
        result,
        Err(BounceError::Cancelled { blocks_written: 0 })
    ));

    // Cancelled partway. The writer counts bytes, not calls: wav::write_block emits one call per
    // sample, so a call count would depend on the writer's internals rather than on the audio
    const HEADER_BYTES: usize = 44;
    const BLOCK_BYTES: usize = BLOCK * 2 * 4;
    struct StopAfter<'a> {
        written: usize,
        limit: usize,
        cancel: &'a AtomicBool,
    }
    impl std::io::Write for StopAfter<'_> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.written += buf.len();
            if self.written >= self.limit {
                self.cancel.store(true, Ordering::Relaxed);
            }
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let cancel = AtomicBool::new(false);
    let mut sink = StopAfter {
        written: 0,
        limit: HEADER_BYTES + 2 * BLOCK_BYTES,
        cancel: &cancel,
    };
    let progress = BounceProgress::default();
    let result = bounce_into(
        config(4_096, BLOCK),
        &fixture_snapshot(),
        &[],
        &mut sink,
        &cancel,
        &progress,
    );
    assert!(matches!(
        result,
        Err(BounceError::Cancelled { blocks_written: 2 })
    ));
    assert_eq!(progress.blocks_done(), 2);
    assert_eq!(progress.blocks_total(), 16);
}

// 13
#[test]
fn the_wav_header_describes_the_bytes_that_follow() {
    let mut wav = Vec::new();
    bounce_into(
        config(128, BLOCK),
        &fixture_snapshot(),
        &fixture_events(128),
        &mut wav,
        &AtomicBool::new(false),
        &BounceProgress::default(),
    )
    .unwrap();

    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(&wav[12..16], b"fmt ");
    assert_eq!(u32::from_le_bytes(wav[16..20].try_into().unwrap()), 16);
    // Format tag 3 is IEEE float; 1 would be integer PCM and would need a conversion
    assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 3);
    assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 48_000);
    assert_eq!(
        u32::from_le_bytes(wav[28..32].try_into().unwrap()),
        48_000 * 2 * 4
    );
    assert_eq!(u16::from_le_bytes(wav[32..34].try_into().unwrap()), 8);
    assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 32);
    assert_eq!(&wav[36..40], b"data");

    // A header that disagrees with its payload produces a file that opens and plays wrong,
    // which is worse than one that fails to open
    let declared = u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize;
    assert_eq!(declared, 128 * 2 * 4);
    assert_eq!(wav.len() - 44, declared);
    assert_eq!(
        u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize,
        36 + declared
    );
}

// 14
#[test]
fn first_divergence_reports_the_first_differing_sample_by_bits() {
    let live = vec![0.0_f32; 1_024 * 2];
    let mut offline = live.clone();
    offline[613 * 2 + 1] = -0.0;

    let found = first_divergence(&live, &offline, 2, BLOCK).unwrap();
    assert_eq!(found.absolute_frame, 613);
    assert_eq!(found.channel, 1);
    assert_eq!(found.block, 613 / BLOCK);
    assert_eq!(found.frame_in_block, 613 % BLOCK);
    // This whole test fails if the comparison uses == on f32: +0.0 == -0.0 is true
    assert_eq!(found.live_bits, 0x0000_0000);
    assert_eq!(found.offline_bits, 0x8000_0000);

    assert_eq!(first_divergence(&live, &live, 2, BLOCK), None);

    // And NaN != NaN would report a false match on equal payloads
    let mut left = vec![f32::NAN; 8];
    let right = left.clone();
    assert_eq!(first_divergence(&left, &right, 2, BLOCK), None);
    left[3] = f32::from_bits(f32::NAN.to_bits() ^ 1);
    assert!(first_divergence(&left, &right, 2, BLOCK).is_some());
}

// 21 — the one property of the shared builder nothing else pins
#[test]
fn the_shared_fixture_builder_still_names_the_nodes_the_seed_names() {
    // Recomputed here rather than imported, so this is not a self-comparison
    let mut ids = IdGen::new(0x0000_5245_4e44_4552);
    let pulse = NodeId::new(ids.next_id());
    let _gain = NodeId::new(ids.next_id());
    let _saturator = NodeId::new(ids.next_id());

    let (_, note_node) = fixture::compile_fixture_plan(BLOCK).unwrap();
    assert_eq!(note_node, pulse);

    // Reordering the three next_id() calls changes every node ID, changes nothing about the
    // audio, and is therefore invisible to every hash gate in the workspace
    assert_eq!(fixture::FIXTURE_SEED, 0x0000_5245_4e44_4552);
    assert_eq!(fixture::FIXTURE_PULSE_LEVEL, 0.3);
    assert_eq!(fixture::FIXTURE_GAIN, 0.7);
    assert_eq!(fixture::FIXTURE_SATURATOR_DRIVE, 2.5);
    assert_eq!(fixture::FIXTURE_SATURATOR_MIX, 0.35);

    // The extraction must also leave the one-quantum harness path untouched
    assert!(render_vertical_slice(RATE, BLOCK).unwrap().peak > 0.0);
}

// The named fallback exists so a no-engine bounce is visible at its call site rather than
// hidden behind a Default that three of the four fields could not justify
#[test]
fn the_fallback_config_carries_the_corpus_values_and_no_log() {
    let settings = fallback_config(1_024);
    assert_eq!(settings.sample_rate, BOUNCE_FALLBACK_SAMPLE_RATE);
    assert_eq!(settings.block_frames, BOUNCE_FALLBACK_BLOCK_FRAMES);
    assert_eq!(settings.frames, 1_024);
    assert!(!settings.log_block_hashes);

    let report = bounce_report(settings, &fixture_snapshot(), &fixture_events(1_024)).unwrap();
    assert!(report.block_hashes.is_empty());
    assert_eq!(report.blocks, 4);
    assert_eq!(report.channels, 2);
}

// Read the f32 samples back out of a written WAV
fn samples_of(wav: &[u8]) -> Vec<f32> {
    wav[44..]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_bits(u32::from_le_bytes(*bytes)))
        .collect()
}
