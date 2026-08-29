// Author: Jeff
// Date: 2026-08-29
// Description: Saves a project in a tight loop so a parent can kill it mid-save
// Notes: R5 crash qualification needs process DEATH, not an unwound fault. fs.rs's FsOps seam is
//   private by contract and returns Err; a crash returns nothing and runs no cleanup, which is the
//   difference the requirement is about. This binary exists only to be killed.

use spectre_core::{BeatTicks, IdGen, TempoMap, Transport};
use spectre_project::{
    save_project_atomic, ClipNote, ClipPlacement, MidiClip, ProjectDoc, ProjectEnvelope, Track,
    TrackInstrument, TrackList, ViewDoc, SCHEMA_VERSION,
};
use std::path::PathBuf;

// Sized so one save takes long enough for a randomized kill to land inside its write, and short
// enough that many saves complete per trial. Both halves matter and the drill's negative control
// enforces the first: at three tracks and a kilobyte, every torn write still decoded cleanly,
// because the write finished before the parent could kill anything
const TRACKS: usize = 4;
const NOTES_PER_CLIP: usize = 512;

// Build the one project every trial writes, so the parent knows exactly what a completed save
// must contain and can reject anything else as partial
fn envelope(seed: u64) -> ProjectEnvelope {
    let mut ids = IdGen::new(seed);
    let project_id = ids.next_id();
    let mut tracks = TrackList::new();
    // Deliberately large. A three-track project encodes to about a kilobyte, which one write
    // syscall completes far inside the parent's finest kill granularity -- the drill's own
    // negative control proved that window is never sampled at that size, so the drill would have
    // passed against a truncating save. Twelve tracks each carrying a clip of MAX_NOTES_PER_CLIP
    // notes encodes to megabytes, which takes long enough that a random kill lands inside it
    for index in 0..TRACKS {
        let id = ids.next_id();
        let mut track = Track::new(id, &format!("T{index}"), TrackInstrument::Pulse)
            .expect("the generated name is valid");
        let clip_id = ids.next_id();
        let placement_id = ids.next_id();
        let mut clip = MidiClip::new(
            clip_id,
            &format!("C{index}"),
            BeatTicks(NOTES_PER_CLIP as i64 * 4),
        )
        .expect("the clip length is valid");
        for note in 0..NOTES_PER_CLIP {
            clip.insert_note(
                ClipNote::new(BeatTicks(note as i64 * 4), BeatTicks(2), 0, 60, 0.8)
                    .expect("the note is valid"),
            )
            .expect("the clip holds MAX_NOTES_PER_CLIP notes");
        }
        track
            .clips_mut()
            .insert(
                ClipPlacement::new(placement_id, clip_id, BeatTicks(0)).expect("valid placement"),
                clip.length(),
            )
            .expect("one placement fits");
        tracks
            .push(track)
            .expect("twelve tracks fit under MAX_TRACKS");
        tracks.add_clip(clip).expect("one clip per track fits");
    }
    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: project_id,
            name: "Crash Drill".into(),
            tempo_map: TempoMap::constant(120.0).expect("a constant tempo is valid"),
            transport: Transport::new(),
            id_gen_state: ids.state(),
            tracks,
            devices: Vec::new(),
            view: ViewDoc::default(),
            unknown: serde_json::Map::new(),
        },
        unknown: serde_json::Map::new(),
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(args.next().expect("usage: crash_saver <path> <seed>"));
    let seed: u64 = args
        .next()
        .expect("usage: crash_saver <path> <seed>")
        .parse()
        .expect("seed must parse");
    let snapshot = envelope(seed);
    // "torn" writes the destination directly, the way a naive save would. It exists so the crash
    // drill can prove it detects non-atomicity; nothing in Spectre saves this way
    let torn = args.next().is_some_and(|mode| mode == "torn");

    // Signal readiness so the parent's kill window starts at the first save rather than during
    // process startup, where nothing is being written and a kill proves nothing
    println!("ready");

    let encoded = spectre_project::to_bytes(&snapshot).expect("a validated envelope encodes");
    loop {
        if torn {
            // Truncate-then-write: the destination is observable half-written for as long as the
            // write takes, which is exactly what atomic replacement exists to prevent
            use std::io::Write;
            if let Ok(mut file) = std::fs::File::create(&path) {
                for chunk in encoded.chunks(64) {
                    if file.write_all(chunk).is_err() {
                        break;
                    }
                }
            }
        } else {
            // Errors are ignored on purpose: this process exists to be killed, and a save that
            // returns at all has already satisfied its own contract
            let _ = save_project_atomic(&path, &snapshot);
        }
    }
}
