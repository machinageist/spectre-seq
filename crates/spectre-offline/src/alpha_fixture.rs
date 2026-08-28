// Author: Jeff
// Date: 2026-08-28
// Description: The R4 end-to-end alpha project, built once so the checked-in file has an author
// Notes: The fixture file is the artifact; this builder is what wrote it and what a schema change
//   regenerates it with. Content is chosen to exercise the three places R4-5 identifies as
//   fragile — a note-off on a block boundary, a same-frame off/on pair, and identical material on
//   a muted track — and nothing more.

use spectre_core::{BeatTicks, IdGen, TempoMap, Transport};
use spectre_dsp::{FILAMENT_PARAMETERS, GLOAM_PARAMETERS};
use spectre_project::{
    ClipNote, ClipPlacement, DeviceDoc, MidiClip, ParameterDoc, ProjectDoc, ProjectEnvelope, Track,
    TrackInstrument, TrackList, ViewDoc, SCHEMA_VERSION,
};

// The seed this fixture's identities come from. A fixed value, so regenerating the file after a
// schema change reproduces every ObjectId rather than renaming the whole project's contents
pub const ALPHA_SEED: u64 = 0x0052_3441_414c_5048;

pub const ALPHA_PROJECT_NAME: &str = "R4 Alpha E2E";
pub const ALPHA_TRACK_NAMES: [&str; 3] = ["Lead", "Pad", "Ref"];

// Non-default device values, one per device. A fixture at every default cannot distinguish
// "values were restored" from "values were re-defaulted" on reload, which is the exact failure
// the round-trip assertion exists for
pub const ALPHA_FILAMENT_LEAN: f32 = 0.72;
pub const ALPHA_GLOAM_DEPTH: f32 = 0.44;

// Descriptor positions, named so a reordering of either array cannot silently swap two controls
const FILAMENT_LEAN: usize = 0;
const GLOAM_DEPTH: usize = 0;

// Build the R4 alpha fixture project
pub fn alpha_project() -> ProjectEnvelope {
    let mut ids = IdGen::new(ALPHA_SEED);
    let project_id = ids.next_id();
    let mut tracks = TrackList::new();

    // The material has to fit inside the e2e render's 16 note blocks. At 120 BPM and 48 kHz one
    // tick is exactly 25 samples, so a 256-frame block is 10.24 ticks and 16 blocks is 163.84
    // ticks. A 160-tick clip is 4,000 samples, comfortably inside 4,096.
    //
    // **An exact note-off on a block boundary is not expressible here, and the arithmetic says
    // why.** A boundary falls on a whole tick only where samples divide by both 256 and 25, and
    // the first such point is 6,400 samples — 25 blocks, past the derived 16-block note span.
    // The other fragile case, a release and an attack sharing one tick, IS expressible and is
    // what Pad exercises below
    let clip_length = BeatTicks(160);
    let half = BeatTicks(64);
    let lead_length = BeatTicks(128);

    for (index, name) in ALPHA_TRACK_NAMES.iter().enumerate() {
        let track_id = ids.next_id();
        let clip_id = ids.next_id();
        let placement_id = ids.next_id();

        let mut clip = MidiClip::new(clip_id, name, clip_length).expect("clip length is valid");
        match index {
            // Lead: one note sounding across eleven block boundaries, so a scheduler that
            // emitted its release into the wrong block would be visible in the hash
            0 => {
                clip.insert_note(ClipNote::new(BeatTicks(0), lead_length, 0, 57, 0.8).unwrap())
                    .unwrap();
            }
            // Pad: two notes whose release and attack share one tick, so the plan's own
            // validator has to see the release ordered before the attack at equal timestamp
            1 => {
                clip.insert_note(ClipNote::new(BeatTicks(0), half, 0, 60, 0.6).unwrap())
                    .unwrap();
                clip.insert_note(ClipNote::new(half, half, 0, 64, 0.6).unwrap())
                    .unwrap();
            }
            // Ref: identical material to Lead, on the muted track. Identical content is what
            // makes "muted contributes nothing" a hash-level claim rather than a level
            // observation
            _ => {
                clip.insert_note(ClipNote::new(BeatTicks(0), lead_length, 0, 57, 0.8).unwrap())
                    .unwrap();
            }
        }

        let mut track = Track::new(track_id, name, TrackInstrument::Filament)
            .expect("the literal names are not blank");
        if index == 2 {
            track.set_muted(true);
        }
        track
            .clips_mut()
            .insert(
                ClipPlacement::new(placement_id, clip_id, BeatTicks(0)).unwrap(),
                clip_length,
            )
            .expect("one placement fits");
        tracks.push(track).expect("three tracks fit");
        tracks.add_clip(clip).expect("one clip per track fits");
    }

    let filament_device = ids.next_id();
    let filament_parameter = ids.next_id();
    let gloam_device = ids.next_id();
    let gloam_parameter = ids.next_id();

    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: project_id,
            name: ALPHA_PROJECT_NAME.into(),
            tempo_map: TempoMap::constant(120.0).expect("a constant tempo is valid"),
            transport: Transport::new(),
            id_gen_state: ids.state(),
            tracks,
            devices: vec![
                DeviceDoc {
                    id: filament_device,
                    key: "filament".into(),
                    parameters: vec![ParameterDoc {
                        id: filament_parameter,
                        key: FILAMENT_PARAMETERS[FILAMENT_LEAN].key.as_str().into(),
                        value: ALPHA_FILAMENT_LEAN,
                        unknown: serde_json::Map::new(),
                    }],
                    unknown: serde_json::Map::new(),
                },
                DeviceDoc {
                    id: gloam_device,
                    key: "gloam".into(),
                    parameters: vec![ParameterDoc {
                        id: gloam_parameter,
                        key: GLOAM_PARAMETERS[GLOAM_DEPTH].key.as_str().into(),
                        value: ALPHA_GLOAM_DEPTH,
                        unknown: serde_json::Map::new(),
                    }],
                    unknown: serde_json::Map::new(),
                },
            ],
            view: ViewDoc::default(),
            unknown: serde_json::Map::new(),
        },
        unknown: serde_json::Map::new(),
    }
}
