// Author: Jeff
// Date: 2026-09-06
// Description: M3 evidence — summing scales past one node's fan-in without changing the RT path
// Notes: MAX_TRACKS was 16 because one SumBus cannot declare more input buses than the graph's
//   fixed input map allows. The product requirement is an unbounded number of TRACKS, not an
//   unbounded fan-in on one node, and a tree of summing nodes satisfies the first while every
//   node stays inside the second. Nothing on the render path changes: no fixed array grows and
//   the callback still allocates nothing.
//
//   The compatibility half is the one that matters most. At or below one node's fan-in the tree
//   must collapse to exactly the single node it always built, or R4's render evidence -- taken
//   on three-track projects -- would no longer describe the graph the product compiles.

use spectre_core::{IdGen, ObjectId};
use spectre_dsp::MAX_SUM_BUSES;
use spectre_dsp::{NoteEvent, NoteEventKind};
use spectre_graph::PlanNoteInput;
use spectre_project::{
    build_track_graph, track_device_factory, Track, TrackInstrument, TrackList, MAX_TRACKS,
};

// If the track limit ever falls back to one node's fan-in, every tree assertion below becomes
// vacuous. A const assertion fails the build rather than letting the suite pass on nothing
const _: () = assert!(
    MAX_TRACKS > MAX_SUM_BUSES,
    "MAX_TRACKS no longer exceeds one summing node's fan-in, so the tree tests prove nothing"
);

const SEED: u64 = 0x5E_4D_7E_11;
const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 64;

fn list_of(count: usize) -> TrackList {
    let mut ids = IdGen::new(SEED);
    let mut tracks = TrackList::new();
    for index in 0..count {
        let mut track = Track::new(ids.next_id(), &format!("T{index}"), TrackInstrument::Pulse)
            .expect("a valid track");
        track.set_instrument_level(0.5);
        tracks.push(track).expect("the track fits");
    }
    tracks
}

fn note() -> [NoteEvent; 1] {
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

fn steps(count: usize) -> usize {
    let tracks = list_of(count);
    let mut ids = IdGen::new(SEED);
    let (graph, nodes) = build_track_graph(&tracks, &mut ids).expect("the graph builds");
    let mut factory = track_device_factory(&tracks, &nodes);
    graph
        .compile(nodes.master.node, FRAMES, &mut factory)
        .expect("the plan compiles")
        .step_count()
}

// Render one quantum with a note addressed to exactly one track's instrument
fn render_one(tracks: &TrackList, note_index: usize) -> Vec<f32> {
    let mut ids = IdGen::new(SEED);
    let (graph, nodes) = build_track_graph(tracks, &mut ids).expect("the graph builds");
    let mut factory = track_device_factory(tracks, &nodes);
    let mut plan = graph
        .compile(nodes.master.node, FRAMES, &mut factory)
        .expect("the plan compiles");
    plan.process(
        SAMPLE_RATE,
        FRAMES,
        &[PlanNoteInput {
            node: nodes
                .note_node(note_index)
                .expect("the track has a note node"),
            events: &note(),
        }],
    )
    .expect("the quantum renders");
    plan.last_output().expect("a rendered plan has output")[0].to_vec()
}

// The compatibility claim: at or below one node's fan-in, nothing changed
#[test]
fn at_or_below_one_nodes_fan_in_the_graph_is_the_one_it_always_was() {
    for count in [0, 1, 3, MAX_SUM_BUSES - 1, MAX_SUM_BUSES] {
        // instrument + gain per track, one summing node, one master
        assert_eq!(
            steps(count),
            count * 2 + 2,
            "{count} tracks built a tree where a single summing node was expected"
        );
    }
}

// Past the fan-in exactly one level is added, and only the nodes the count needs
#[test]
fn past_one_nodes_fan_in_exactly_one_level_is_added() {
    let count = MAX_SUM_BUSES + 1;
    // 17 sources -> two group nodes -> one root: three summing nodes rather than one
    assert_eq!(steps(count), count * 2 + 3 + 1);
}

#[test]
fn the_largest_legal_project_compiles() {
    // 32 sources -> two group nodes -> one root
    assert_eq!(steps(MAX_TRACKS), MAX_TRACKS * 2 + 3 + 1);
}

// The claim a node count cannot make: every group actually reaches the root. A tree that wired
// only its first group would compile, and would silently drop every track past the sixteenth
#[test]
fn a_track_in_every_group_reaches_the_master() {
    let tracks = list_of(MAX_TRACKS);
    let first = render_one(&tracks, 0);
    assert!(
        first.iter().any(|sample| *sample != 0.0),
        "the first track is silent, so the comparisons below prove nothing"
    );
    // One index inside each group the tree builds, including the last
    for index in [MAX_SUM_BUSES - 1, MAX_SUM_BUSES, MAX_TRACKS - 1] {
        let rendered = render_one(&tracks, index);
        assert_eq!(
            rendered, first,
            "track {index} does not reach the master; its summing group is not wired to the root"
        );
    }
}

// Bus order is source order at every level. Float addition is not associative, so this is a
// contract rather than a nicety
#[test]
fn the_same_list_builds_the_same_graph_twice() {
    let tracks = list_of(MAX_TRACKS);
    let identities = |list: &TrackList| -> Vec<ObjectId> {
        let mut ids = IdGen::new(SEED);
        let (_, nodes) = build_track_graph(list, &mut ids).expect("the graph builds");
        nodes
            .track_gains
            .iter()
            .map(|gain| gain.node.object_id())
            .chain(std::iter::once(nodes.sum.object_id()))
            .chain(std::iter::once(nodes.master.node.object_id()))
            .collect()
    };
    assert_eq!(identities(&tracks), identities(&tracks));
}

#[test]
fn a_list_refuses_more_than_max_tracks() {
    let mut tracks = list_of(MAX_TRACKS);
    let mut ids = IdGen::new(0xFF);
    let extra = Track::new(ids.next_id(), "Overflow", TrackInstrument::Pulse).expect("valid");
    tracks
        .push(extra)
        .expect_err("a list must refuse more than MAX_TRACKS");
}
