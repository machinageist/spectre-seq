// Author: Jeff
// Date: 2026-08-24
// Description: R4 slice 4 evidence — the track graph compiles, is deterministic, and sums in order
// Notes: No new render path: every assertion here drives the existing CompiledPlan::process

use spectre_core::{IdGen, ObjectId};
use spectre_dsp::{NoteEvent, NoteEventKind};
use spectre_graph::PlanNoteInput;
use spectre_project::{build_track_graph, track_device_factory, Track, TrackInstrument, TrackList};

const SEED: u64 = 0x0052_4f55_5449_4e47;
const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 64;

// Build a list whose tracks carry the given instrument levels
fn list_with(levels: &[f32]) -> (TrackList, Vec<ObjectId>) {
    let mut ids = IdGen::new(SEED);
    let mut list = TrackList::new();
    let mut created = Vec::new();
    for (index, level) in levels.iter().enumerate() {
        let id = ids.next_id();
        let mut track = Track::new(id, &format!("t{index}"), TrackInstrument::Pulse).unwrap();
        track.set_instrument_level(*level);
        list.push(track).unwrap();
        created.push(id);
    }
    (list, created)
}

// One held note across the whole block
fn note_events() -> [NoteEvent; 1] {
    [NoteEvent {
        frame_offset: 0,
        sequence: 0,
        kind: NoteEventKind::On {
            id: 1,
            channel: 0,
            note: 45,
            velocity: 0.8,
        },
    }]
}

// Build, compile, and render one quantum with notes addressed to one track's instrument.
// A free function rather than a closure so the immutable borrow of `list` ends at the return,
// leaving the caller free to reorder between renders
fn render_track(list: &TrackList, note_index: usize, events: &[NoteEvent]) -> Vec<f32> {
    let mut gen = IdGen::new(SEED);
    let (graph, nodes) = build_track_graph(list, &mut gen).unwrap();
    let mut factory = track_device_factory(list, &nodes);
    let mut plan = graph
        .compile(nodes.master.node, FRAMES, &mut factory)
        .unwrap();
    plan.process(
        SAMPLE_RATE,
        FRAMES,
        &[PlanNoteInput {
            node: nodes.note_node(note_index).unwrap(),
            events,
        }],
    )
    .unwrap();
    plan.last_output().unwrap()[0].to_vec()
}

// 6
#[test]
fn the_built_graph_compiles_and_its_shape_matches_the_track_count() {
    let (list, _) = list_with(&[0.3, 0.3, 0.3]);
    let mut ids = IdGen::new(SEED);
    let (graph, nodes) = build_track_graph(&list, &mut ids).unwrap();
    let mut factory = track_device_factory(&list, &nodes);
    let plan = graph.compile(nodes.master.node, 256, &mut factory).unwrap();

    // instrument + gain per track, then the sum and the master. A builder that silently
    // dropped a track would still compile; the step count is what catches it
    assert_eq!(plan.step_count(), 2 * 3 + 2);
    assert_eq!(nodes.instruments.len(), 3);
    assert_eq!(nodes.track_gains.len(), 3);
}

// 7
#[test]
fn rebuilding_from_the_same_list_and_seed_produces_the_same_node_ids() {
    let (list, _) = list_with(&[0.3, 0.3]);
    let (_, first) = build_track_graph(&list, &mut IdGen::new(SEED)).unwrap();
    let (_, second) = build_track_graph(&list, &mut IdGen::new(SEED)).unwrap();
    // Fails if the builder ever iterates a HashMap instead of the list's own order
    assert_eq!(first, second);
}

// 8
#[test]
fn track_order_is_bus_order() {
    // One silent track and one sounding track; the sounding one moves from index 1 to index 0
    let (mut list, ids) = list_with(&[0.0, 0.5]);
    let events = note_events();

    let before = render_track(&list, 1, &events);
    assert!(before.iter().any(|s| *s != 0.0), "the fixture must sound");

    list.reorder(ids[1], 0).unwrap();
    let after = render_track(&list, 0, &events);

    assert_eq!(
        before, after,
        "moving the only sounding track must not change what the sum produces"
    );
}

// 8b — two sounding tracks reordered. Float addition is not associative, so this asserts the
// precision the arithmetic actually holds rather than an equality it does not guarantee
#[test]
fn reordering_two_sounding_tracks_changes_the_sum_by_at_most_one_ulp() {
    let (mut list, ids) = list_with(&[0.4, 0.7]);
    let events = note_events();

    let before = render_track(&list, 0, &events);
    list.reorder(ids[1], 0).unwrap();
    let after = render_track(&list, 1, &events);

    for (a, b) in before.iter().zip(&after) {
        if a == b {
            continue;
        }
        let ulps = (a.to_bits() as i64 - b.to_bits() as i64).abs();
        assert!(
            ulps <= 1,
            "a bus-order change may cost at most one ULP; {a} vs {b} differ by {ulps}"
        );
    }
}

// 9
#[test]
fn an_empty_track_list_compiles_to_a_plan_that_renders_exact_silence() {
    let list = TrackList::new();
    let mut ids = IdGen::new(SEED);
    let (graph, nodes) = build_track_graph(&list, &mut ids).unwrap();
    let mut factory = track_device_factory(&list, &nodes);
    let mut plan = graph
        .compile(nodes.master.node, FRAMES, &mut factory)
        .unwrap();

    assert!(plan.process(SAMPLE_RATE, FRAMES, &[]).is_ok());
    let output = plan.last_output().unwrap();
    for channel in output {
        for sample in channel {
            // Bit equality, so a -0.0 is caught: an empty project is exact silence by
            // construction, not an error state
            assert_eq!(sample.to_bits(), 0.0_f32.to_bits());
        }
    }
}
