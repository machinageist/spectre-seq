// Author: Jeff
// Date: 2026-08-28
// Description: The app-thread bridge between AppModel and the persisted project document
// Notes: project_envelope reads the model and mutates nothing; adopt replaces the model only
//   after every mapping has succeeded. Both run on the app thread and neither is reachable from
//   a callback. This module writes AppModel's private fields directly, which is legal because it
//   is a descendant of the crate root that declares them — so persistence adds no field and no
//   accessor to AppModel.

use crate::{AppModel, DeviceControl, Lens, ParameterControl};
use spectre_core::{IdGen, TransportCommand};
use spectre_dsp::{
    DspParameter, FILAMENT_PARAMETERS, GAIN_PARAMETERS, GLOAM_PARAMETERS, PULSE_PARAMETERS,
    SATURATOR_PARAMETERS,
};
use spectre_project::{
    DeviceDoc, LensDoc, ParameterDoc, ProjectDoc, ProjectEnvelope, ViewDoc, SCHEMA_VERSION,
};

// Why a loaded project could not become the live one. Every variant is a refusal, never a
// silent repair: a file must not be able to smuggle a value the UI cannot produce
#[derive(Debug, Clone, PartialEq)]
pub enum AdoptError {
    UnknownDevice {
        key: String,
    },
    UnknownParameter {
        device_key: String,
        key: String,
    },
    ValueOutOfRange {
        device_key: String,
        key: String,
        value: f32,
    },
}

impl std::fmt::Display for AdoptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownDevice { key } => {
                write!(f, "this build has no device named {key}")
            }
            Self::UnknownParameter { device_key, key } => {
                write!(f, "{device_key} has no parameter named {key}")
            }
            Self::ValueOutOfRange {
                device_key,
                key,
                value,
            } => write!(
                f,
                "{device_key}.{key} value {value} is outside this build's accepted range"
            ),
        }
    }
}

impl std::error::Error for AdoptError {}

// The device catalogue this build can adopt into, by canonical key. Written here rather than
// derived from AppModel::prototype so a project naming a device the running model does not
// happen to hold is refused by the build's own catalogue, not by whatever is on screen
const CATALOGUE: [(&str, &str, &str, &[DspParameter]); 5] = [
    (
        "pulse",
        "Pulse",
        "Instrument · stereo out · note input",
        &PULSE_PARAMETERS,
    ),
    ("gain", "Gain", "Effect · stereo in/out", &GAIN_PARAMETERS),
    (
        "saturator",
        "Saturator",
        "Effect · stereo in/out",
        &SATURATOR_PARAMETERS,
    ),
    (
        "filament",
        "Filament",
        "Instrument · stereo out · note input",
        &FILAMENT_PARAMETERS,
    ),
    (
        "gloam",
        "Gloam",
        "Effect · stereo in/out",
        &GLOAM_PARAMETERS,
    ),
];

// Convert between the UI's lens and the document's. The document schema is a wire format with
// its own compatibility rules; the UI enum is not, and spectre-project must not depend on this
// crate, so the two are converted rather than shared
fn lens_doc(lens: Lens) -> LensDoc {
    match lens {
        Lens::Arrange => LensDoc::Arrange,
        Lens::Build => LensDoc::Build,
        Lens::Shape => LensDoc::Shape,
        Lens::Mix => LensDoc::Mix,
    }
}

fn lens_from(doc: LensDoc) -> Lens {
    match doc {
        LensDoc::Arrange => Lens::Arrange,
        LensDoc::Build => Lens::Build,
        LensDoc::Shape => Lens::Shape,
        LensDoc::Mix => Lens::Mix,
    }
}

// Build the immutable app-thread snapshot the save boundary consumes.
// The sole owner of the version stamp: this is the only component that builds an envelope from
// scratch, so it is the only one that can set schema_version, and it sets it unconditionally.
// save_project_atomic reproduces whatever version it is handed and never rewrites one
pub fn project_envelope(model: &AppModel, name: &str) -> ProjectEnvelope {
    ProjectEnvelope {
        schema_version: SCHEMA_VERSION,
        project: ProjectDoc {
            id: model.project_id(),
            name: name.to_string(),
            tempo_map: model.tempo_map().clone(),
            transport: model.transport(),
            // Persisting the generator position is what stops a reload from re-minting IDs the
            // project already holds. Without it the very next add_track produces a duplicate,
            // and TrackList::insert refuses it — a failed add the musician cannot get past
            id_gen_state: model.id_gen_state(),
            tracks: model.track_list().clone(),
            devices: model
                .devices()
                .iter()
                .map(|device| DeviceDoc {
                    id: device.instance_id,
                    key: device.key.to_string(),
                    parameters: device
                        .parameters
                        .iter()
                        .map(|parameter| ParameterDoc {
                            id: parameter.instance_id,
                            key: parameter.descriptor.key.as_str().to_string(),
                            value: parameter.value,
                            unknown: serde_json::Map::new(),
                        })
                        .collect(),
                    unknown: serde_json::Map::new(),
                })
                .collect(),
            view: ViewDoc {
                lens: lens_doc(model.lens()),
                selected_track: model.selected_track_id(),
                selected_device: model.selected_device_id(),
            },
            // AppModel has no field for either unknown map, so a snapshot built from it carries
            // empty ones. An unknown field read from a file survives load_project and survives a
            // crate-level rewrite, but does NOT survive open -> edit -> save through the shell.
            // A real hole in CORE-003's preservation at the product level; this slice does not
            // close it and no test or product string may say otherwise
            unknown: serde_json::Map::new(),
        },
        unknown: serde_json::Map::new(),
    }
}

// Replace the live model only after a load has fully succeeded. Every mapping is resolved into
// owned values first; the model is not touched until all of them are known good
pub fn adopt(model: &mut AppModel, envelope: ProjectEnvelope) -> Result<(), AdoptError> {
    let document = envelope.project;

    let mut devices = Vec::with_capacity(document.devices.len());
    for device in &document.devices {
        let entry = CATALOGUE
            .iter()
            .find(|(key, ..)| *key == device.key)
            .ok_or_else(|| AdoptError::UnknownDevice {
                key: device.key.clone(),
            })?;
        let (key, name, role, descriptors) = *entry;

        let mut parameters = Vec::with_capacity(device.parameters.len());
        for parameter in &device.parameters {
            // The canonical &'static str is required, not cosmetic: DspParameter's key is one,
            // and a String read from a file is not
            let descriptor = descriptors
                .iter()
                .find(|candidate| candidate.key.as_str() == parameter.key)
                .ok_or_else(|| AdoptError::UnknownParameter {
                    device_key: key.to_string(),
                    key: parameter.key.clone(),
                })?;
            // Refused, never clamped. Clamping on the load path would silently accept a file
            // the UI could not have produced and then present it as the musician's own work
            if !parameter.value.is_finite()
                || parameter.value < descriptor.minimum()
                || parameter.value > descriptor.maximum()
            {
                return Err(AdoptError::ValueOutOfRange {
                    device_key: key.to_string(),
                    key: parameter.key.clone(),
                    value: parameter.value,
                });
            }
            parameters.push(ParameterControl {
                instance_id: parameter.id,
                descriptor: *descriptor,
                value: parameter.value,
            });
        }

        devices.push(DeviceControl {
            instance_id: device.id,
            key,
            name,
            role,
            parameters,
        });
    }

    // Past this line nothing can fail, so nothing can leave the model half-adopted
    let mut transport = document.transport;
    // Opening a file never starts playback, and post R4-1 never starts audio
    transport.apply(TransportCommand::Stop);

    model.transport = transport;
    model.lens = lens_from(document.view.lens);
    model.tracks = document.tracks;
    model.devices = devices;
    model.selected_track = document.view.selected_track;
    model.selected_device = document.view.selected_device;
    // Session-only and deliberately not persisted, so an open clears it rather than restoring a
    // selection the file never carried
    model.selected_clip = None;
    model.project_id = document.id;
    model.tempo_map = document.tempo_map;
    // A schema-1 document carries no generator position. That case is bounded: schema 1 has no
    // collections, so the only ID in it is the project's own, which ObjectId guarantees nonzero
    model.ids = IdGen::new(if document.id_gen_state == 0 {
        document.id.raw()
    } else {
        document.id_gen_state
    });
    Ok(())
}

// The two shell decisions that are not drawing, extracted so they can be tested.
// main.rs is a binary target and nothing in crates/spectre-app/tests/ can reach it, which is
// exactly how R4-1 shipped a transport rule with no coverage at all. These live here for that
// reason, and main.rs holds only the egui calls around them

// True when the document differs from what was last written. Before any save there is nothing
// on disk, so the answer is yes: a marker reading "Saved" over work no file holds is the
// product-killing defect the project-safety pillar names
pub fn is_dirty(saved: Option<&[u8]>, current: Option<&[u8]>) -> bool {
    match (saved, current) {
        (Some(saved), Some(current)) => saved != current,
        _ => true,
    }
}

// What pressing Open should do next
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenGate {
    // Unsaved work: replace the button with a confirm for one interaction rather than a modal
    ArmDiscard,
    Proceed,
}

// One press never discards unsaved work; a second one does. Arming is only reachable while the
// project is dirty, so a clean project opens on the first press
pub fn open_gate(dirty: bool, armed: bool) -> OpenGate {
    if dirty && !armed {
        OpenGate::ArmDiscard
    } else {
        OpenGate::Proceed
    }
}
