// Author: Jeff
// Date: 2026-09-06
// Description: M5 evidence — a track added the way the product adds one can play a chord
// Notes: polyphony.rs proves each instrument sounds a chord. This proves the claim the product
//   actually makes: a track created with the default instrument, compiled through the real track
//   graph, sounds three notes together. It is the end of the chain DEV-010's re-open trigger
//   names -- "MIDI clips producing overlapping notes a user expects to hear together" -- and it
//   would have failed while PulseInstrument was monophonic even with Filament already polyphonic.

use spectre_core::{IdGen, ObjectId};
use spectre_dsp::{NoteEvent, NoteEventKind};
use spectre_graph::PlanNoteInput;
use spectre_project::{build_track_graph, track_device_factory, Track, TrackInstrument, TrackList};

const SEED: u64 = 0x0043_484f_5244;
const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;

fn on(sequence: u64, id: u32, note: u8) -> NoteEvent {
    NoteEvent {
        frame_offset: 0,
        sequence,
        kind: NoteEventKind::On {
            id,
            channel: 0,
            note,
            velocity: 0.8,
        },
    }
}

// One track carrying the instrument AppModel::add_track gives a new track
fn default_track_list() -> (TrackList, ObjectId) {
    let mut ids = IdGen::new(SEED);
    let id = ids.next_id();
    let mut list = TrackList::new();
    list.push(Track::new(id, "Added", TrackInstrument::Pulse).expect("a valid track"))
        .expect("the track fits");
    (list, id)
}

fn render(list: &TrackList, events: &[NoteEvent]) -> Vec<f32> {
    let mut ids = IdGen::new(SEED);
    let (graph, nodes) = build_track_graph(list, &mut ids).expect("the graph builds");
    let mut factory = track_device_factory(list, &nodes);
    let mut plan = graph
        .compile(nodes.master.node, FRAMES, &mut factory)
        .expect("the plan compiles");
    plan.process(
        SAMPLE_RATE,
        FRAMES,
        &[PlanNoteInput {
            node: nodes.note_node(0).expect("the track has a note node"),
            events,
        }],
    )
    .expect("the quantum renders");
    plan.last_output().expect("a rendered plan has output")[0].to_vec()
}

#[test]
fn a_default_track_plays_a_chord_through_the_real_graph() {
    let (list, _) = default_track_list();
    let chord = render(&list, &[on(0, 1, 60), on(1, 2, 64), on(2, 3, 67)]);
    let single = render(&list, &[on(0, 1, 60)]);

    assert!(
        chord.iter().any(|sample| *sample != 0.0),
        "the chord is silent through the track graph"
    );
    assert_ne!(
        chord, single,
        "a track added the way the product adds one still plays one note at a time"
    );
    let chord_peak = chord.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    let single_peak = single.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    assert!(
        chord_peak > single_peak,
        "the chord peaks no higher than its root: {chord_peak} vs {single_peak}"
    );
}

// The alpha's own voice, through the same path
#[test]
fn a_filament_track_plays_a_chord_through_the_real_graph() {
    let mut ids = IdGen::new(SEED);
    let mut list = TrackList::new();
    list.push(Track::new(ids.next_id(), "Lead", TrackInstrument::Filament).expect("a valid track"))
        .expect("the track fits");

    let chord = render(&list, &[on(0, 1, 60), on(1, 2, 64), on(2, 3, 67)]);
    let single = render(&list, &[on(0, 1, 60)]);
    assert!(chord.iter().any(|sample| *sample != 0.0));
    assert_ne!(chord, single);
}
