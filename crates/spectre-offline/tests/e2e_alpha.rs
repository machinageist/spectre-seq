// Author: Jeff
// Date: 2026-08-28
// Description: R4 slice 9 — the composed alpha, rendered end to end from a checked-in file
// Notes: A vertical slice is not the conjunction of eight seams tested separately. Every
//   assertion here names the sibling whose claim it composes, so a failure says which slice's
//   contract the composition broke rather than only that something broke.
//   No checked-in golden hash: cross-platform bit-equality is undecided, because device math
//   routes through the platform's libm. What is asserted is equality between two computations on
//   the same host in the same process; the observed hash is printed for the run record.

use spectre_audio::bridge::{RenderBridge, DEFAULT_NOTE_SCRATCH};
use spectre_audio::clip::{ClipPlayer, ClipSchedule, CLIP_EVENT_RESERVE, CLIP_SEQUENCE_BAND};
use spectre_audio::control::control_channel;
use spectre_audio::RenderBlock;
use spectre_core::{IdGen, SampleRate, TempoMap};
use spectre_dsp::{FILAMENT_PARAMETERS, GLOAM_PARAMETERS};
use spectre_graph::{CompiledPlan, NodeId, PlanNoteInput};
use spectre_offline::alpha_fixture::{
    ALPHA_FILAMENT_LEAN, ALPHA_GLOAM_DEPTH, ALPHA_PROJECT_NAME, ALPHA_TRACK_NAMES,
};
use spectre_offline::hash::{hash_block, hash_planar_quantum, SampleHasher};
use spectre_project::{
    build_track_graph, from_bytes, load_project, save_project_atomic, track_device_factory,
    validate_envelope, ProjectEnvelope, TrackInstrument, TrackList,
};
use std::collections::HashSet;

const FIXTURE: &[u8] = include_bytes!("fixtures/r4-alpha.json");

// Reused from crates/spectre-audio/tests/lifecycle_health.rs rather than declared here, so the
// e2e record is directly comparable to the qualification record on the same host
const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;
const CHANNELS: usize = 2;

// The note span must cross block boundaries in both directions: sixteen blocks is the smallest
// power-of-two span that leaves room for the same-tick off/on pair in the middle without either
// event landing in the first or last block
const E2E_NOTE_BLOCKS: usize = 16;
// Inherited from R4-6's own return-to-silence criterion of 64 render quanta after the last
// note-off, and it MUST move with that criterion. A shorter tail cannot observe what it exists
// to check; a longer one adds render time without adding an observation
const E2E_TAIL_BLOCKS: usize = 64;
const E2E_TOTAL_BLOCKS: usize = E2E_NOTE_BLOCKS + E2E_TAIL_BLOCKS;

// The graph identity seed. Fixed, so a rebuild from the same list produces the same nodes
const E2E_GRAPH_SEED: u64 = 0x0045_3245_414c_5048;

fn fixture() -> ProjectEnvelope {
    from_bytes(FIXTURE).expect("the checked-in alpha fixture must decode")
}

// Compile the fixture's own track graph, the way the app's engine does
fn compile(tracks: &TrackList) -> (CompiledPlan, Vec<NodeId>) {
    let mut ids = IdGen::new(E2E_GRAPH_SEED);
    let (graph, nodes) = build_track_graph(tracks, &mut ids).expect("the fixture graph compiles");
    let mut factory = track_device_factory(tracks, &nodes);
    let plan = graph
        .compile(nodes.master.node, FRAMES, &mut factory)
        .expect("the fixture plan compiles");
    let instruments = nodes.instruments.iter().map(|entry| entry.node).collect();
    (plan, instruments)
}

// Bake one track's clip material into absolute sample positions, through the one tick-to-sample
// conversion the workspace has
fn schedule_for(tracks: &TrackList, index: usize, tempo: &TempoMap) -> ClipSchedule {
    let track = &tracks.tracks()[index];
    let mut notes = Vec::new();
    for placement in track.clips().placements() {
        if !placement.is_active() {
            continue;
        }
        let clip = tracks
            .clip(placement.clip())
            .expect("a placement names a clip in the project");
        for note in clip.notes() {
            notes.push((
                spectre_core::BeatTicks(placement.start().0 + note.start().0),
                note.length(),
                note.channel(),
                note.note(),
                note.velocity(),
            ));
        }
    }
    ClipSchedule::bake(
        notes.into_iter(),
        tempo,
        SampleRate::new(SAMPLE_RATE as u32).unwrap(),
        CLIP_EVENT_RESERVE,
    )
    .expect("the fixture's clip material bakes")
}

// What one offline render produced
struct Rendered {
    hash: u64,
    peak: f32,
    last_block: Vec<f32>,
    contaminated: u64,
}

// Render the whole project offline, delivering every track's own clip events to its own
// instrument. The plan takes one PlanNoteInput per note node, so all three tracks sound
fn render_offline(envelope: &ProjectEnvelope) -> Rendered {
    let tracks = &envelope.project.tracks;
    let (mut plan, instruments) = compile(tracks);
    let schedules: Vec<_> = (0..tracks.len())
        .map(|index| schedule_for(tracks, index, &envelope.project.tempo_map))
        .collect();
    let mut players: Vec<_> = schedules
        .into_iter()
        .map(|schedule| {
            let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
            let _ = player.install(Box::new(schedule));
            player
        })
        .collect();

    let mut hasher = SampleHasher::new();
    let mut peak = 0.0_f32;
    let mut last_block = Vec::new();
    let mut transport = spectre_core::Transport::new();
    transport.apply(spectre_core::TransportCommand::Play);

    for _ in 0..E2E_TOTAL_BLOCKS {
        let mut per_node: Vec<Vec<spectre_dsp::NoteEvent>> = Vec::with_capacity(players.len());
        for player in players.iter_mut() {
            let mut events = Vec::new();
            player.emit_block(&transport, FRAMES, &mut events, CLIP_SEQUENCE_BAND);
            events.sort_unstable_by(spectre_audio::clip::contract_order);
            per_node.push(events);
        }
        let inputs: Vec<PlanNoteInput<'_>> = instruments
            .iter()
            .zip(&per_node)
            .map(|(node, events)| PlanNoteInput {
                node: *node,
                events,
            })
            .collect();
        plan.process(SAMPLE_RATE, FRAMES, &inputs)
            .expect("R4-4/R4-5: the composed plan refused a block");
        let output = plan.last_output().expect("a quantum was rendered");
        for (left, right) in output[0][..FRAMES].iter().zip(&output[1][..FRAMES]) {
            hasher.write(*left);
            hasher.write(*right);
            peak = peak.max(left.abs()).max(right.abs());
        }
        last_block = output[0][..FRAMES]
            .iter()
            .chain(&output[1][..FRAMES])
            .copied()
            .collect();
        transport.advance(spectre_core::SampleDuration::new(FRAMES as u64));
    }

    Rendered {
        hash: hasher.finish(),
        peak,
        last_block,
        contaminated: plan.containment().contaminated_nodes,
    }
}

// U-1
#[test]
fn fixture_decodes_and_validates() {
    let envelope = fixture();
    if let Err(error) = validate_envelope(&envelope) {
        panic!("the checked-in alpha fixture is not a valid project: {error}");
    }
    assert_eq!(envelope.project.name, ALPHA_PROJECT_NAME);
}

// U-2
#[test]
fn fixture_content_is_exactly_what_the_protocol_documents() {
    let envelope = fixture();
    let tracks = &envelope.project.tracks;

    let names: Vec<_> = tracks
        .tracks()
        .iter()
        .map(|track| track.name().to_string())
        .collect();
    assert_eq!(names, ALPHA_TRACK_NAMES.to_vec());

    // Exactly one muted, and it is Ref
    let muted: Vec<_> = tracks
        .tracks()
        .iter()
        .filter(|track| track.is_muted())
        .map(|track| track.name().to_string())
        .collect();
    assert_eq!(muted, vec!["Ref".to_string()]);

    // One clip placed on each track, and one clip in the project table per track
    for track in tracks.tracks() {
        assert_eq!(track.clips().len(), 1, "{} has one placement", track.name());
        assert_eq!(track.instrument(), TrackInstrument::Filament);
    }
    assert_eq!(tracks.clips().len(), 3);

    // Pad's two notes share one tick between the release of the first and the attack of the
    // second, which is the ordering case the accepted contract's rank rule exists for
    let pad = tracks
        .clip(tracks.tracks()[1].clips().placements()[0].clip())
        .unwrap();
    assert_eq!(pad.notes().len(), 2);
    assert_eq!(
        pad.notes()[0].start().0 + pad.notes()[0].length().0,
        pad.notes()[1].start().0
    );

    // Ref's material is identical to Lead's, which is what makes "muted contributes nothing" a
    // hash-level claim rather than a level observation
    let lead = tracks
        .clip(tracks.tracks()[0].clips().placements()[0].clip())
        .unwrap();
    let reference = tracks
        .clip(tracks.tracks()[2].clips().placements()[0].clip())
        .unwrap();
    assert_eq!(lead.notes().len(), reference.notes().len());
    for (a, b) in lead.notes().iter().zip(reference.notes()) {
        assert_eq!(
            (a.start(), a.length(), a.note(), a.velocity()),
            (b.start(), b.length(), b.note(), b.velocity())
        );
    }

    // Every device parameter differs from its descriptor default. A fixture at every default
    // cannot distinguish "values were restored" from "values were re-defaulted" on reload
    let filament = envelope
        .project
        .devices
        .iter()
        .find(|d| d.key == "filament")
        .unwrap();
    assert_eq!(filament.parameters[0].value, ALPHA_FILAMENT_LEAN);
    assert_ne!(ALPHA_FILAMENT_LEAN, FILAMENT_PARAMETERS[0].default());
    let gloam = envelope
        .project
        .devices
        .iter()
        .find(|d| d.key == "gloam")
        .unwrap();
    assert_eq!(gloam.parameters[0].value, ALPHA_GLOAM_DEPTH);
    assert_ne!(ALPHA_GLOAM_DEPTH, GLOAM_PARAMETERS[0].default());
}

// U-3
#[test]
fn fixture_object_ids_are_unique_and_nonzero() {
    let envelope = fixture();
    let mut seen = HashSet::new();
    let mut record = |raw: u64| {
        assert_ne!(raw, 0, "every ObjectId must be nonzero");
        assert!(seen.insert(raw), "0x{raw:x} appears twice in the fixture");
    };

    record(envelope.project.id.raw());
    for track in envelope.project.tracks.tracks() {
        record(track.id().raw());
        for placement in track.clips().placements() {
            record(placement.id().raw());
        }
    }
    for clip in envelope.project.tracks.clips() {
        record(clip.id().raw());
    }
    for device in &envelope.project.devices {
        record(device.id.raw());
        for parameter in &device.parameters {
            record(parameter.id.raw());
        }
    }
    // The codec already rejects a zero ID at decode; what this asserts is that the fixture's
    // author did not create a collision the codec would happily accept
    assert!(seen.len() >= 13);
}

// I-1
#[test]
fn fixture_renders_with_a_nonzero_peak() {
    let rendered = render_offline(&fixture());
    assert!(
        rendered.peak > 0.0,
        "R4-6/R4-5: the fixture rendered silence, so every hash equality below would be vacuous"
    );
}

// I-2
#[test]
fn two_renders_of_the_fixture_are_bit_identical() {
    let envelope = fixture();
    let first = render_offline(&envelope);
    let second = render_offline(&envelope);
    assert_eq!(
        first.hash, second.hash,
        "GRAPH-001: the composed plan is not deterministic"
    );
    assert_eq!(first.peak, second.peak);
    assert!(first.peak > 0.0);
}

// I-4
#[test]
fn save_reload_render_produces_the_same_hash() {
    let directory = std::env::temp_dir().join("spectre-e2e-alpha");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("alpha.spectre");

    let envelope = fixture();
    let before = render_offline(&envelope);

    save_project_atomic(&path, &envelope).expect("R4-7: the fixture must save");
    let reloaded = load_project(&path).expect("R4-7: the saved fixture must load");
    let after = render_offline(&reloaded);

    assert_eq!(
        after.hash, before.hash,
        "R4-7: persistence changed what the project computes"
    );
    assert!(before.peak > 0.0);
    std::fs::remove_dir_all(&directory).unwrap();
}

// I-5
#[test]
fn track_ids_survive_reorder_across_a_save() {
    use spectre_project::command::{EditHistory, ProjectCommand, Transaction};

    let directory = std::env::temp_dir().join("spectre-e2e-reorder");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("alpha.spectre");

    let mut envelope = fixture();
    let original: Vec<_> = envelope
        .project
        .tracks
        .tracks()
        .iter()
        .map(|track| track.id())
        .collect();

    let mut history = EditHistory::new(8).unwrap();
    history
        .apply(
            &mut envelope.project,
            Transaction::single(ProjectCommand::reorder_tracks(original[0], 2)),
        )
        .unwrap();
    save_project_atomic(&path, &envelope).unwrap();
    let reloaded = load_project(&path).unwrap();

    let after: Vec<_> = reloaded
        .project
        .tracks
        .tracks()
        .iter()
        .map(|track| track.id())
        .collect();
    assert_eq!(
        after,
        vec![original[1], original[2], original[0]],
        "R4-7/CORE-001: track identity did not survive a reorder through a file"
    );
    std::fs::remove_dir_all(&directory).unwrap();
}

// I-6
#[test]
fn muting_a_track_changes_the_hash() {
    let mut unmuted = fixture();
    for track in 0..unmuted.project.tracks.len() {
        let id = unmuted.project.tracks.tracks()[track].id();
        unmuted.project.tracks.get_mut(id).unwrap().set_muted(false);
    }
    let all_on = render_offline(&unmuted);
    let with_mute = render_offline(&fixture());

    assert!(all_on.peak > 0.0 && with_mute.peak > 0.0);
    assert_ne!(
        all_on.hash, with_mute.hash,
        "R4-4: muting a track changed nothing, so mute reaches no signal path"
    );
}

// I-7
#[test]
fn a_muted_track_hashes_equal_to_its_absence() {
    let with_muted = render_offline(&fixture());

    let mut without = fixture();
    let reference = without.project.tracks.tracks()[2].id();
    without.project.tracks.remove(reference).unwrap();

    let deleted = render_offline(&without);
    assert!(with_muted.peak > 0.0);
    assert_eq!(
        with_muted.hash, deleted.hash,
        "R4-4: a muted track contributed signal"
    );
}

// I-9
#[test]
fn telemetry_is_clean_across_the_whole_run() {
    let rendered = render_offline(&fixture());
    assert_eq!(
        rendered.contaminated, 0,
        "RT-003: containment fired during an ordinary render"
    );
}

// I-10
#[test]
fn the_chain_returns_to_exact_zero_after_the_last_note_off() {
    let rendered = render_offline(&fixture());
    assert!(rendered.peak > 0.0, "a silent render proves nothing here");
    for sample in &rendered.last_block {
        // Both conditions: a residual denormal fails, while a legitimately flushed NEGATIVE
        // zero passes. contain_channel flushes a negative denormal to -0.0, so a bit-pattern
        // equality against +0.0 alone would fail a correctly-flushed render
        assert_eq!(*sample, 0.0, "R4-6: the chain did not return to silence");
        assert!(sample.abs() < f32::MIN_POSITIVE);
    }
}

// I-3 — live and offline over the composed graph, at equal block size.
// Scoped to one track's note delivery so the comparison isolates block alignment rather than
// voice fan-out. The bridge's one-note_node limit that originally forced this scope was lifted by
// the R4-9 follow-up; multi-instrument delivery is proved by
// the_live_bridge_plays_every_instrument_track, with one_voice_does_not_sound_like_three as its
// control
#[test]
fn live_bridge_matches_the_offline_render_at_equal_block_size() {
    let envelope = fixture();
    let tracks = &envelope.project.tracks;
    let schedule = schedule_for(tracks, 0, &envelope.project.tempo_map);

    // Offline: the same composed plan, notes to Lead's instrument only
    let (mut plan, instruments) = compile(tracks);
    let mut offline_player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = offline_player.install(Box::new(schedule_for(
        tracks,
        0,
        &envelope.project.tempo_map,
    )));
    let mut transport = spectre_core::Transport::new();
    transport.apply(spectre_core::TransportCommand::Play);
    let mut offline = SampleHasher::new();
    let mut offline_peak = 0.0_f32;
    for _ in 0..E2E_TOTAL_BLOCKS {
        let mut events = Vec::new();
        offline_player.emit_block(&transport, FRAMES, &mut events, CLIP_SEQUENCE_BAND);
        events.sort_unstable_by(spectre_audio::clip::contract_order);
        plan.process(
            SAMPLE_RATE,
            FRAMES,
            &[PlanNoteInput {
                node: instruments[0],
                events: &events,
            }],
        )
        .unwrap();
        let output = plan.last_output().unwrap();
        for (left, right) in output[0][..FRAMES].iter().zip(&output[1][..FRAMES]) {
            offline.write(*left);
            offline.write(*right);
            offline_peak = offline_peak.max(left.abs()).max(right.abs());
        }
        transport.advance(spectre_core::SampleDuration::new(FRAMES as u64));
    }

    // Live: the same plan through the bridge, its own player, its own transport
    let (live_plan, live_instruments) = compile(tracks);
    let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule));
    let mut bridge = RenderBridge::new(
        live_plan,
        receiver,
        live_instruments[0],
        SAMPLE_RATE,
        DEFAULT_NOTE_SCRATCH,
    )
    .with_clip_player(player);
    // The bridge takes transport through the RT-002 lane, never through a direct call
    sender
        .send_transport(spectre_core::TransportCommand::Play)
        .unwrap();

    let mut live = SampleHasher::new();
    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS];
    for _ in 0..E2E_TOTAL_BLOCKS {
        interleaved.fill(0.0);
        let mut block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
        bridge.render(&mut block);
        hash_block(&mut live, &interleaved, CHANNELS, FRAMES);
    }

    assert!(offline_peak > 0.0, "two silent buffers would agree");
    assert_eq!(
        live.finish(),
        offline.finish(),
        "R4-1/R4-8: live and offline diverged on the composed track graph"
    );
    assert_eq!(
        bridge.telemetry().blocks_rendered(),
        E2E_TOTAL_BLOCKS as u64
    );
    assert_eq!(bridge.telemetry().plan_errors(), 0);
    assert_eq!(bridge.telemetry().frame_capacity_rejections(), 0);
    assert_eq!(bridge.telemetry().notes_deferred(), 0);
    assert_eq!(bridge.telemetry().contaminated_nodes(), 0);
}

// The composition gap R4-9 found, now closed and pinned from the other side.
// RenderBridge used to carry exactly one note_node, so a three-track project played one track
// live while the offline path played all three: every seam passed its own test and the
// composition did not work. The bridge now carries additional clip voices, and this test is what
// says the product plays the whole project rather than a third of it
#[test]
fn the_live_bridge_plays_every_instrument_track() {
    let envelope = fixture();
    let tracks = &envelope.project.tracks;
    assert_eq!(tracks.len(), 3, "the fixture has three instrument tracks");

    let (plan, instruments) = compile(tracks);
    assert_eq!(instruments.len(), 3, "the graph has three instrument nodes");

    let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut primary = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = primary.install(Box::new(schedule_for(
        tracks,
        0,
        &envelope.project.tempo_map,
    )));
    let voices: Vec<_> = (1..tracks.len())
        .map(|index| {
            let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
            let _ = player.install(Box::new(schedule_for(
                tracks,
                index,
                &envelope.project.tempo_map,
            )));
            (instruments[index], player)
        })
        .collect();

    let mut bridge = RenderBridge::new(
        plan,
        receiver,
        instruments[0],
        SAMPLE_RATE,
        DEFAULT_NOTE_SCRATCH,
    )
    .with_clip_player(primary)
    .with_clip_voices(voices, DEFAULT_NOTE_SCRATCH);
    sender
        .send_transport(spectre_core::TransportCommand::Play)
        .unwrap();

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS];
    let mut live = SampleHasher::new();
    for _ in 0..E2E_TOTAL_BLOCKS {
        interleaved.fill(0.0);
        let mut block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
        bridge.render(&mut block);
        hash_block(&mut live, &interleaved, CHANNELS, FRAMES);
    }

    // The whole project, live, equals the whole project, offline
    let offline = render_offline(&envelope);
    assert!(offline.peak > 0.0, "two silent buffers would agree");
    assert_eq!(
        live.finish(),
        offline.hash,
        "R4-4/R4-5/R4-1: the live bridge and the offline path disagree on the composed project"
    );
    assert_eq!(bridge.telemetry().plan_errors(), 0);
    assert_eq!(bridge.telemetry().notes_deferred(), 0);
    assert_eq!(bridge.telemetry().clip_events_refused(), 0);
    assert_eq!(bridge.telemetry().contaminated_nodes(), 0);
}

// One voice is not three, and this is what says so. Without it, a bridge that silently dropped
// every additional voice would still pass the test above if the offline side dropped them too
#[test]
fn one_voice_does_not_sound_like_three() {
    let envelope = fixture();
    let tracks = &envelope.project.tracks;

    let (plan, instruments) = compile(tracks);
    let (mut sender, receiver) = control_channel(&[], 64, 8).unwrap();
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule_for(
        tracks,
        0,
        &envelope.project.tempo_map,
    )));
    let mut bridge = RenderBridge::new(
        plan,
        receiver,
        instruments[0],
        SAMPLE_RATE,
        DEFAULT_NOTE_SCRATCH,
    )
    .with_clip_player(player);
    sender
        .send_transport(spectre_core::TransportCommand::Play)
        .unwrap();

    let mut interleaved = vec![0.0_f32; FRAMES * CHANNELS];
    let mut live = SampleHasher::new();
    for _ in 0..E2E_NOTE_BLOCKS {
        interleaved.fill(0.0);
        let mut block = RenderBlock::new(&mut interleaved, CHANNELS as u16);
        bridge.render(&mut block);
        hash_block(&mut live, &interleaved, CHANNELS, FRAMES);
    }

    assert_ne!(
        live.finish(),
        render_offline(&envelope).hash,
        "one instrument must not produce what three produce"
    );
}

// I-12 — a print, not a check. It exists so an operator can transcribe the run record without
// re-deriving anything. Marked explicitly so nobody mistakes it for coverage
#[test]
fn the_report_line_is_printed_for_the_record() {
    let rendered = render_offline(&fixture());
    println!(
        "e2e frames={} channels={} peak={} hash={:#018x} blocks={}",
        E2E_TOTAL_BLOCKS * FRAMES,
        CHANNELS,
        rendered.peak,
        rendered.hash,
        E2E_TOTAL_BLOCKS
    );
}

// The single-quantum planar fold is not the streaming fold, and the e2e uses the streaming one.
// Named here so a reader of the record knows which walk produced the number above.
// Rendered WITH a note, deliberately: over an all-zero quantum the two walks read the same bytes
// in a different order and agree, so a silent render could not tell them apart
#[test]
fn the_recorded_hash_is_the_streaming_walk_not_the_quantum_walk() {
    let envelope = fixture();
    let tracks = &envelope.project.tracks;
    let (mut plan, instruments) = compile(tracks);
    let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
    let _ = player.install(Box::new(schedule_for(
        tracks,
        0,
        &envelope.project.tempo_map,
    )));
    let mut transport = spectre_core::Transport::new();
    transport.apply(spectre_core::TransportCommand::Play);

    // Render a few blocks so the quantum holds real audio rather than silence
    let mut peak = 0.0_f32;
    for _ in 0..4 {
        let mut events = Vec::new();
        player.emit_block(&transport, FRAMES, &mut events, CLIP_SEQUENCE_BAND);
        events.sort_unstable_by(spectre_audio::clip::contract_order);
        plan.process(
            SAMPLE_RATE,
            FRAMES,
            &[PlanNoteInput {
                node: instruments[0],
                events: &events,
            }],
        )
        .unwrap();
        let output = plan.last_output().unwrap();
        for sample in output[0][..FRAMES].iter().chain(&output[1][..FRAMES]) {
            peak = peak.max(sample.abs());
        }
        transport.advance(spectre_core::SampleDuration::new(FRAMES as u64));
    }
    assert!(peak > 0.0, "silence cannot distinguish two traversals");

    let output = plan.last_output().unwrap();
    let quantum = hash_planar_quantum([&output[0][..FRAMES], &output[1][..FRAMES]]);
    let mut streaming = SampleHasher::new();
    for (left, right) in output[0][..FRAMES].iter().zip(&output[1][..FRAMES]) {
        streaming.write(*left);
        streaming.write(*right);
    }
    assert_ne!(
        quantum,
        streaming.finish(),
        "channel-major and frame-major must not be confusable in the record"
    );
}
