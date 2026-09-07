// Author: Jeff
// Date: 2026-09-06
// Description: M3 evidence — a track chains effects in any order, and schema 2 files survive it
// Notes: The chain is serial, so every node in it has exactly one input bus and none of the
//   graph's fan-in bounds apply. What has to be proved is that order is the signal path, that
//   the parameter index arithmetic follows a real prefix sum rather than a fixed stride, and
//   that a file written when a track could hold one effect still loads with that effect intact.

use spectre_core::{IdGen, TempoMap, Transport};
use spectre_project::{
    build_track_graph, from_bytes, migrate_to_current, to_bytes, ProjectDoc, ProjectEnvelope,
    Track, TrackEffect, TrackInsert, TrackInstrument, TrackList, MAX_CHAIN_DEVICES, SCHEMA_VERSION,
};

const SEED: u64 = 0x0043_4841_494e;

fn track_with(count: usize) -> Track {
    let mut ids = IdGen::new(SEED);
    let mut track =
        Track::new(ids.next_id(), "Chained", TrackInstrument::Filament).expect("a valid track");
    for index in 0..count {
        track
            .push_insert(TrackInsert::new(TrackEffect::Gloam, 0.1 * index as f32))
            .expect("the effect fits");
    }
    track
}

// One project holding the given track. Identities come from a generator seeded past the
// track's own, so the project id cannot collide with the track id -- the validator refuses a
// duplicate, which is what the first version of this helper tripped over
fn envelope_with(track: Track) -> ProjectEnvelope {
    let mut ids = IdGen::new(SEED ^ 0xFFFF);
    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: ids.next_id(),
            name: "Chained".into(),
            tempo_map: TempoMap::constant(120.0).expect("valid"),
            transport: Transport::new(),
            id_gen_state: ids.state(),
            tracks: list_with(track),
            devices: Vec::new(),
            view: Default::default(),
            unknown: Default::default(),
        },
        unknown: Default::default(),
    }
}

fn list_with(track: Track) -> TrackList {
    let mut list = TrackList::new();
    list.push(track).expect("the track fits");
    list
}

#[test]
fn a_track_chains_more_than_one_effect() {
    let track = track_with(4);
    assert_eq!(track.inserts().len(), 4);
    let list = list_with(track);
    let mut ids = IdGen::new(SEED);
    let (_, nodes) = build_track_graph(&list, &mut ids).expect("the graph builds");
    assert_eq!(
        nodes.inserts[0].len(),
        4,
        "the chain did not reach the graph"
    );
}

// The chain is the signal path, so its order is a fact about the audio and every node must be
// distinct and in sequence
#[test]
fn chain_order_reaches_the_graph_in_signal_order() {
    let list = list_with(track_with(3));
    let mut ids = IdGen::new(SEED);
    let (_, nodes) = build_track_graph(&list, &mut ids).expect("the graph builds");
    let chain = &nodes.inserts[0];
    let identities: Vec<_> = chain.iter().map(|node| node.node.object_id()).collect();
    let mut unique = identities.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        identities.len(),
        "two chained effects share a node identity"
    );
}

// The arithmetic that broke: `index * 2 + inserts` addressed the wrong parameter from the second
// track onward the moment a chain could be longer than one
#[test]
fn parameter_indices_follow_a_prefix_sum_over_chain_lengths() {
    let mut ids = IdGen::new(SEED);
    let mut list = TrackList::new();
    for (index, chain_len) in [0usize, 3, 1, 2].iter().enumerate() {
        let mut track = Track::new(
            ids.next_id(),
            &format!("T{index}"),
            TrackInstrument::Filament,
        )
        .expect("a valid track");
        for _ in 0..*chain_len {
            track
                .push_insert(TrackInsert::new(TrackEffect::Gloam, 0.5))
                .expect("the effect fits");
        }
        list.push(track).expect("the track fits");
    }

    let mut graph_ids = IdGen::new(SEED);
    let (_, nodes) = build_track_graph(&list, &mut graph_ids).expect("the graph builds");
    let targets = nodes.parameter_targets();

    for (index, chain) in nodes.inserts.iter().enumerate() {
        // Every instrument parameter, by descriptor position
        for (position, identity) in nodes.instruments[index].parameters.iter().enumerate() {
            let at = list
                .instrument_parameter_index(index, position)
                .expect("the descriptor position has a target");
            assert_eq!(
                targets[at],
                (nodes.instruments[index].node.object_id(), *identity),
                "instrument index disagrees at track {index} parameter {position}"
            );
        }
        for (position, insert) in chain.iter().enumerate() {
            // Every parameter of every chained effect, by descriptor position
            for (parameter, identity) in insert.parameters.iter().enumerate() {
                let at = list
                    .insert_parameter_index(index, position, parameter)
                    .unwrap_or_else(|| {
                        panic!("no target for track {index} effect {position} param {parameter}")
                    });
                assert_eq!(
                    targets[at],
                    (insert.node.object_id(), *identity),
                    "effect index disagrees at track {index} position {position} param {parameter}"
                );
            }
        }
        assert_eq!(
            targets[list.gain_target_index(index)],
            (
                nodes.track_gains[index].node.object_id(),
                nodes.track_gains[index].gain_parameter
            ),
            "gain index disagrees at track {index}"
        );
    }
    assert_eq!(
        targets[list.master_target_index()],
        (nodes.master.node.object_id(), nodes.master.gain_parameter)
    );
    assert_eq!(targets.len(), list.master_target_index() + 1);
}

#[test]
fn a_chain_can_be_reordered_and_trimmed() {
    let mut track = track_with(3);
    let first = track.inserts()[0].depth();
    let last = track.inserts()[2].depth();
    track.reorder_insert(0, 2).expect("the move applies");
    assert_eq!(track.inserts()[2].depth(), first);
    assert_eq!(track.inserts()[1].depth(), last);

    let removed = track.remove_insert(1).expect("the removal applies");
    assert_eq!(removed.depth(), last);
    assert_eq!(track.inserts().len(), 2);

    track
        .insert_at(0, TrackInsert::new(TrackEffect::Gloam, 0.9))
        .expect("the insert applies");
    assert_eq!(track.inserts()[0].depth(), 0.9);
    assert_eq!(track.inserts().len(), 3);
}

#[test]
fn a_chain_refuses_past_its_bound_and_past_its_end() {
    let mut track = track_with(MAX_CHAIN_DEVICES);
    track
        .push_insert(TrackInsert::new(TrackEffect::Gloam, 0.5))
        .expect_err("a chain must refuse past MAX_CHAIN_DEVICES");
    let mut short = track_with(1);
    short
        .insert_at(5, TrackInsert::new(TrackEffect::Gloam, 0.5))
        .expect_err("a position past the end must be refused");
    short
        .remove_insert(5)
        .expect_err("removing past the end must be refused");
}

// The regression this whole change could most easily have caused: a file written when a track
// held one effect must still load with that effect, not silently lose it.
//
// The schema-2 document is DERIVED from a current one rather than hand-written. Hand-writing it
// means restating every field name, and the first attempt did exactly that and got `start_tick`
// wrong -- a fixture that fails for a reason unrelated to what it tests
#[test]
fn a_schema_two_document_keeps_its_effect_through_migration() {
    let current = envelope_with(track_with(1));
    let mut json: serde_json::Value =
        serde_json::from_slice(&to_bytes(&current).expect("encodes")).expect("is json");

    // Downgrade to the shape schema 2 wrote: one "insert" object, no "inserts" array
    json["schema_version"] = serde_json::json!(2);
    let track = &mut json["project"]["tracks"]["tracks"][0];
    let only = track["inserts"][0].clone();
    track
        .as_object_mut()
        .expect("a track object")
        .remove("inserts");
    track["insert"] = only;

    let bytes = serde_json::to_vec(&json).expect("re-encodes");
    let decoded = from_bytes(&bytes).expect("the schema-2 document decodes");
    assert_eq!(decoded.schema_version, 2);
    assert!(
        decoded.project.tracks.tracks()[0].inserts().is_empty(),
        "the legacy slot must not reach the chain before migration, or this proves nothing"
    );

    let migrated = migrate_to_current(decoded).expect("it migrates");
    assert!(migrated.changed(), "the migration reported no change");
    assert_eq!(migrated.to_schema, SCHEMA_VERSION);

    let track = &migrated.envelope.project.tracks.tracks()[0];
    assert_eq!(
        track.inserts().len(),
        1,
        "the schema-2 effect was lost in migration"
    );
    assert_eq!(track.inserts()[0].effect(), TrackEffect::Gloam);
    assert_eq!(
        track.inserts()[0].depth(),
        current.project.tracks.tracks()[0].inserts()[0].depth()
    );

    // And it survives a re-encode, now in the chain form rather than the slot form
    let round_tripped =
        from_bytes(&to_bytes(&migrated.envelope).expect("encodes")).expect("decodes");
    assert_eq!(round_tripped.project.tracks.tracks()[0].inserts().len(), 1);
}

// Migration must be a no-op on a document that already carries a chain
#[test]
fn migrating_an_already_current_document_changes_nothing() {
    let envelope = envelope_with(track_with(3));
    let migrated = migrate_to_current(envelope.clone()).expect("it migrates");
    assert!(!migrated.changed());
    assert_eq!(migrated.envelope, envelope);
}
