// Author: Jeff
// Date: 2026-07-12
// Description: Renderer-neutral application model for the launchable Spectre prototype
// Notes: UI state delegates transport truth to spectre-core; the live engine lives in
//   engine.rs and AppModel stays renderer-neutral and audio-free

pub mod bounce_panel;
pub mod engine;
pub mod project;

use spectre_core::{
    BeatTicks, IdGen, MeterMap, ObjectId, TempoMap, TimeSignature, Transport, TransportCommand,
    TransportState,
};
use spectre_dsp::{
    DeviceParameterSnapshot, DspParameter, FILAMENT_PARAMETERS, GAIN_PARAMETERS, GLOAM_PARAMETERS,
    PULSE_PARAMETERS, SATURATOR_PARAMETERS,
};
use spectre_project::{
    ClipError, ClipPlacement, MidiClip, Track, TrackError, TrackInstrument, TrackList,
};

// Persistent workspace lenses over one project selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lens {
    Arrange,
    Build,
    Shape,
    Mix,
}

impl Lens {
    pub const ALL: [Self; 4] = [Self::Arrange, Self::Build, Self::Shape, Self::Mix];
}

impl std::fmt::Display for Lens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Arrange => "Arrange",
            Self::Build => "Build",
            Self::Shape => "Shape",
            Self::Mix => "Mix",
        };
        f.write_str(label)
    }
}

// The tempo the first-launch project carries. The transport bar has shown this number since R3
// and calls it a fixed project default; naming it here is what lets the project persist it
// rather than re-derive it. Not a limit, so it owes no PROD-003 row
pub const PROTOTYPE_BPM: f64 = 120.0;

// The canonical result of one accepted Shape edit: what the model stored, and which stable
// identities name it. Public fields, matching ParameterControl, because this is an app-thread
// value rather than a published cross-crate DTO like DeviceParameterSnapshot
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterEdit {
    pub device_instance_id: ObjectId,
    pub parameter_instance_id: ObjectId,
    // Already clamped by the parameter's own descriptor; this is what offline rendering would use
    pub value: f32,
}

// One backend-defined parameter and its current app-thread value
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterControl {
    pub instance_id: ObjectId,
    pub descriptor: DspParameter,
    pub value: f32,
    // Forward fields travel with the stable parameter identity through open → edit → save.
    // Private because the UI must not interpret fields this build does not own
    unknown: serde_json::Map<String, serde_json::Value>,
}

// One device card rendered from the stabilized backend contract
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceControl {
    pub instance_id: ObjectId,
    pub key: &'static str,
    pub name: &'static str,
    pub role: &'static str,
    pub parameters: Vec<ParameterControl>,
    // Same preservation boundary as ParameterControl; never used to invent device behaviour
    unknown: serde_json::Map<String, serde_json::Value>,
}

impl DeviceControl {
    fn from_descriptors(
        ids: &mut IdGen,
        key: &'static str,
        name: &'static str,
        role: &'static str,
        descriptors: &[DspParameter],
    ) -> Self {
        Self {
            instance_id: ids.next_id(),
            key,
            name,
            role,
            parameters: descriptors
                .iter()
                .copied()
                .map(|descriptor| ParameterControl {
                    instance_id: ids.next_id(),
                    value: descriptor.default(),
                    descriptor,
                    unknown: serde_json::Map::new(),
                })
                .collect(),
            unknown: serde_json::Map::new(),
        }
    }
}

pub const OPEN_IN_SHAPE_ACTION_LABEL: &str = "Open in Shape";
pub const SHAPE_EMPTY_MESSAGE: &str = "No device selected. Open a device from Build to shape it.";
// A truthful empty surface: it states the fact and names the command as text, and does not draw
// a populated-looking lane. The vision's release bar prohibits fake surfaces
pub const CLIP_LANE_EMPTY_MESSAGE: &str = "No clips on this track.";
pub const CLIP_INSPECTOR_EMPTY_MESSAGE: &str =
    "No clip selected. Select a clip in Arrange to edit its notes.";

// Borrow Build card content and its direct Shape action from the app model
#[derive(Debug, Clone, Copy)]
pub struct BuildDeviceCard<'a> {
    device: &'a DeviceControl,
    selected: bool,
}

impl<'a> BuildDeviceCard<'a> {
    pub fn device(self) -> &'a DeviceControl {
        self.device
    }

    pub fn is_selected(self) -> bool {
        self.selected
    }

    pub fn action(self) -> OpenInShapeAction {
        OpenInShapeAction {
            device_id: self.device.instance_id,
        }
    }
}

// Identify the visible direct action without renderer-specific state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenInShapeAction {
    device_id: ObjectId,
}

impl OpenInShapeAction {
    pub fn label(self) -> &'static str {
        OPEN_IN_SHAPE_ACTION_LABEL
    }

    pub fn device_id(self) -> ObjectId {
        self.device_id
    }
}

// Iterate existing devices without cloning or allocating presentation state
#[derive(Debug, Clone, Copy)]
pub struct BuildPresentation<'a> {
    devices: &'a [DeviceControl],
    selected_device: Option<ObjectId>,
}

impl<'a> BuildPresentation<'a> {
    pub fn cards(
        &self,
    ) -> impl ExactSizeIterator<Item = BuildDeviceCard<'a>> + DoubleEndedIterator + '_ {
        self.devices.iter().map(move |device| BuildDeviceCard {
            device,
            selected: self.selected_device == Some(device.instance_id),
        })
    }
}

// Expose only the selected Shape device and truthful empty-state copy
#[derive(Debug, Clone, Copy)]
pub struct ShapePresentation<'a> {
    selected_device: Option<&'a DeviceControl>,
}

impl<'a> ShapePresentation<'a> {
    pub fn from_selected_device(selected_device: Option<&'a DeviceControl>) -> Self {
        Self { selected_device }
    }

    pub fn selected_device(self) -> Option<&'a DeviceControl> {
        self.selected_device
    }

    pub fn empty_state_message(self) -> Option<&'static str> {
        self.selected_device
            .is_none()
            .then_some(SHAPE_EMPTY_MESSAGE)
    }
}

// Internal fixture-schema failure; public mutation cannot create this state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceParameterSnapshotError {
    MissingDevice(&'static str),
    MissingParameter {
        device_key: &'static str,
        parameter_key: spectre_dsp::DeviceParameterKey,
    },
}

impl std::fmt::Display for DeviceParameterSnapshotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDevice(device_key) => {
                write!(formatter, "missing canonical device: {device_key}")
            }
            Self::MissingParameter {
                device_key,
                parameter_key,
            } => write!(
                formatter,
                "missing canonical parameter: {device_key}.{}",
                parameter_key.as_str()
            ),
        }
    }
}

impl std::error::Error for DeviceParameterSnapshotError {}

// Single source of truth for prototype interactions
#[derive(Debug)]
pub struct AppModel {
    // The project's own identity and tempo. R4-7 added both, deviating from its spec's claim
    // that persistence adds no field: without them an app-built envelope has nothing to put in
    // ProjectDoc.id or ProjectDoc.tempo_map, so every save would mint a new project identity and
    // reset the tempo to the prototype's. A project ID that changes on every save makes
    // CORE-001's project-level identity meaningless, which is worse than one field
    project_id: ObjectId,
    // Unknown envelope and project fields survive the product path without becoming app state.
    // Known behavior still comes only from typed fields below
    envelope_unknown: serde_json::Map<String, serde_json::Value>,
    project_unknown: serde_json::Map<String, serde_json::Value>,
    tempo_map: TempoMap,
    // The project's meter. Held rather than assumed so the transport's bars-beats readout and
    // its "4 / 4" label read one source; meter editing arrives with the arrangement
    meter_map: MeterMap,
    transport: Transport,
    lens: Lens,
    tracks: TrackList,
    devices: Vec<DeviceControl>,
    selected_track: Option<ObjectId>,
    selected_device: Option<ObjectId>,
    // Session-only, never persisted, and deliberately independent of track selection: selecting
    // a clip must not move the user's track context
    selected_clip: Option<ObjectId>,
    ids: IdGen,
    feedback: String,
}

impl AppModel {
    // Create the deterministic first-launch state
    pub fn prototype() -> Self {
        let mut ids = IdGen::new(0x0047_4549_5354_5549);
        let mut tracks = TrackList::new();
        let track_id = ids.next_id();
        tracks
            .push(
                Track::new(track_id, "Pulse", TrackInstrument::Pulse)
                    .expect("the literal name is not blank"),
            )
            .expect("an empty list accepts one track");
        let devices = vec![
            DeviceControl::from_descriptors(
                &mut ids,
                "pulse",
                "Pulse",
                "Instrument · stereo out · note input",
                &PULSE_PARAMETERS,
            ),
            DeviceControl::from_descriptors(
                &mut ids,
                "gain",
                "Gain",
                "Effect · stereo in/out",
                &GAIN_PARAMETERS,
            ),
            DeviceControl::from_descriptors(
                &mut ids,
                "saturator",
                "Saturator",
                "Effect · stereo in/out",
                &SATURATOR_PARAMETERS,
            ),
            // Appended rather than inserted: prototype() allocates every identity from one
            // seeded IdGen in declaration order, so a new device in the middle would renumber
            // every device after it and break the offline fixture's stable IDs
            DeviceControl::from_descriptors(
                &mut ids,
                "filament",
                "Filament",
                "Instrument · stereo out · note input",
                &FILAMENT_PARAMETERS,
            ),
            DeviceControl::from_descriptors(
                &mut ids,
                "gloam",
                "Gloam",
                "Effect · stereo in/out",
                &GLOAM_PARAMETERS,
            ),
        ];
        let selected_device = devices.first().map(|device| device.instance_id);
        Self {
            project_id: ids.next_id(),
            envelope_unknown: serde_json::Map::new(),
            project_unknown: serde_json::Map::new(),
            tempo_map: TempoMap::constant(PROTOTYPE_BPM).expect("a constant tempo is valid"),
            meter_map: MeterMap::constant(
                TimeSignature::new(4, 4).expect("4/4 is a valid signature"),
            ),
            transport: Transport::new(),
            lens: Lens::Arrange,
            selected_track: Some(track_id),
            selected_clip: None,
            tracks,
            devices,
            selected_device,
            ids,
            feedback: String::new(),
        }
    }

    // Stable identity of the open project, restored on load rather than re-minted on save
    pub fn project_id(&self) -> ObjectId {
        self.project_id
    }

    pub fn tempo_map(&self) -> &TempoMap {
        &self.tempo_map
    }

    pub fn meter_map(&self) -> &MeterMap {
        &self.meter_map
    }

    pub fn transport(&self) -> Transport {
        self.transport
    }

    // Generator position, so a reload resumes the sequence rather than restarting it
    pub fn id_gen_state(&self) -> u64 {
        self.ids.state()
    }

    pub fn is_playing(&self) -> bool {
        self.transport.state == TransportState::Playing
    }

    pub fn toggle_play(&mut self) {
        let command = if self.is_playing() {
            TransportCommand::Stop
        } else {
            TransportCommand::Play
        };
        self.transport.apply(command);
    }

    pub fn stop(&mut self) {
        self.transport.apply(TransportCommand::Stop);
    }

    pub fn lens(&self) -> Lens {
        self.lens
    }

    pub fn select_lens(&mut self, lens: Lens) {
        self.lens = lens;
    }

    // Borrow the ordered track collection
    pub fn track_list(&self) -> &TrackList {
        &self.tracks
    }

    pub fn tracks(&self) -> &[Track] {
        self.tracks.tracks()
    }

    pub fn selected_track_id(&self) -> Option<ObjectId> {
        self.selected_track
    }

    pub fn select_track(&mut self, id: ObjectId) {
        if self.tracks.get(id).is_some() {
            self.selected_track = Some(id);
        }
    }

    pub fn selected_track(&self) -> Option<&Track> {
        self.tracks.get(self.selected_track?)
    }

    pub fn selected_clip(&self) -> Option<ObjectId> {
        self.selected_clip
    }

    // Create clip material and place it on one track, selecting the placement.
    // Track selection is untouched: a new clip must not move the user's context
    pub fn create_clip(
        &mut self,
        track: ObjectId,
        name: &str,
        length: BeatTicks,
        start: BeatTicks,
    ) -> Result<ObjectId, ClipError> {
        if self.tracks.get(track).is_none() {
            return Err(ClipError::UnknownClip(track));
        }
        let clip_id = self.ids.next_id();
        let clip = MidiClip::new(clip_id, name, length)?;
        let placement_id = self.ids.next_id();
        let placement = ClipPlacement::new(placement_id, clip_id, start)?;

        // Placement first: it is the edit that can be refused by an overlap, and a refused
        // placement must not leave orphaned clip material behind
        self.tracks
            .get_mut(track)
            .expect("presence checked above")
            .clips_mut()
            .insert(placement, length)?;
        self.tracks.add_clip(clip).expect("the id was just minted");
        self.selected_clip = Some(placement_id);
        Ok(placement_id)
    }

    // Select an existing placement. Unlike open_device_in_shape this changes no lens, because a
    // clip is edited where it lives rather than in a separate surface
    pub fn select_clip(&mut self, placement: ObjectId) -> Result<(), ClipError> {
        if self.clip_track(placement).is_none() {
            return Err(ClipError::UnknownPlacement(placement));
        }
        self.selected_clip = Some(placement);
        Ok(())
    }

    // Which track holds one placement
    pub fn clip_track(&self, placement: ObjectId) -> Option<ObjectId> {
        self.tracks
            .tracks()
            .iter()
            .find(|track| track.clips().get(placement).is_some())
            .map(|track| track.id())
    }

    // Activate or deactivate one placement. A deactivated placement contributes no notes to a
    // bake, which is how it goes silent without being deleted
    pub fn set_clip_active(&mut self, placement: ObjectId, active: bool) -> Result<(), ClipError> {
        let track = self
            .clip_track(placement)
            .ok_or(ClipError::UnknownPlacement(placement))?;
        self.tracks
            .get_mut(track)
            .expect("clip_track returned a track in the list")
            .clips_mut()
            .get_mut(placement)
            .expect("clip_track found the placement on this track")
            .set_active(active);
        Ok(())
    }

    // One accessible label for a placement: name, position, length, note count, and active state.
    // Built from the model rather than from a view, so no second description of a clip exists
    // The selected placement's clip length, its notes as plain rows, and whether the placement
    // is active. Returned as owned rows so the UI can hold them while it mutates the model --
    // the inspector's active checkbox writes back through set_clip_active in the same frame
    #[allow(clippy::type_complexity)]
    pub fn selected_clip_detail(&self) -> Option<(BeatTicks, Vec<(i64, i64, u8, f32, u8)>, bool)> {
        let placement_id = self.selected_clip?;
        let track = self.tracks.get(self.clip_track(placement_id)?)?;
        let placement = track
            .clips()
            .placements()
            .iter()
            .find(|entry| entry.id() == placement_id)?;
        let clip = self.tracks.clip(placement.clip())?;
        let notes = clip
            .notes()
            .iter()
            .map(|note| {
                (
                    note.start().0,
                    note.length().0,
                    note.note(),
                    note.velocity(),
                    note.channel(),
                )
            })
            .collect();
        Some((clip.length(), notes, placement.is_active()))
    }

    pub fn clip_label(&self, placement: ObjectId) -> Option<String> {
        let track = self.tracks.get(self.clip_track(placement)?)?;
        let index = track
            .clips()
            .placements()
            .iter()
            .position(|entry| entry.id() == placement)?;
        let entry = &track.clips().placements()[index];
        let length = track.clips().length_at(index)?;
        let clip = self.tracks.clip(entry.clip())?;
        Some(format!(
            "{}, starts at beat {:.3}, {:.3} beats long, {} notes, {}",
            clip.name(),
            entry.start().as_beats_f64(),
            length.as_beats_f64(),
            clip.note_count(),
            if entry.is_active() {
                "active"
            } else {
                "inactive"
            },
        ))
    }

    // Rename one track; a blank name leaves the model unchanged
    pub fn rename_track(&mut self, id: ObjectId, name: &str) -> Result<(), TrackError> {
        self.tracks
            .get_mut(id)
            .ok_or(TrackError::UnknownTrack(id))?
            .set_name(name)
    }

    // Set one track's fader position. Returns the ids whose effective gain changed — exactly one
    pub fn set_track_level(
        &mut self,
        id: ObjectId,
        level: f32,
    ) -> Result<Vec<ObjectId>, TrackError> {
        self.tracks
            .get_mut(id)
            .ok_or(TrackError::UnknownTrack(id))?
            .set_level(level);
        Ok(vec![id])
    }

    // Mute one track. Returns the ids whose effective gain changed — exactly one
    pub fn set_track_muted(
        &mut self,
        id: ObjectId,
        muted: bool,
    ) -> Result<Vec<ObjectId>, TrackError> {
        self.tracks
            .get_mut(id)
            .ok_or(TrackError::UnknownTrack(id))?
            .set_muted(muted);
        Ok(vec![id])
    }

    // Solo one track. Returns EVERY track id, in list order, because effective_gain depends on
    // any_soloed(): turning a solo on silences every other track and clearing the last solo
    // restores them. Returning only the edited id produces a solo that silences nothing
    pub fn set_track_soloed(
        &mut self,
        id: ObjectId,
        soloed: bool,
    ) -> Result<Vec<ObjectId>, TrackError> {
        self.tracks
            .get_mut(id)
            .ok_or(TrackError::UnknownTrack(id))?
            .set_soloed(soloed);
        Ok(self.tracks.tracks().iter().map(Track::id).collect())
    }

    // Set the master fader. Publishes the master gain's own target, not a track's, so the
    // returned track set is empty
    pub fn set_master_level(&mut self, level: f32) -> Vec<ObjectId> {
        self.tracks.set_master_level(level);
        Vec::new()
    }

    // Set one track's instrument level. Returns the ids whose instrument level changed
    pub fn set_track_instrument_level(
        &mut self,
        id: ObjectId,
        level: f32,
    ) -> Result<Vec<ObjectId>, TrackError> {
        self.tracks
            .get_mut(id)
            .ok_or(TrackError::UnknownTrack(id))?
            .set_instrument_level(level);
        Ok(vec![id])
    }

    // Move one track to an absolute index; identity and every field survive (CORE-001)
    pub fn reorder_track(&mut self, id: ObjectId, to_index: usize) -> Result<usize, TrackError> {
        self.tracks.reorder(id, to_index)
    }

    // Remove one track, returning it whole so an undo can reinsert it unchanged
    pub fn remove_track(&mut self, id: ObjectId) -> Result<Track, TrackError> {
        let removed = self.tracks.remove(id)?;
        if self.selected_track == Some(id) {
            self.selected_track = self.tracks.tracks().first().map(Track::id);
        }
        Ok(removed)
    }

    pub fn devices(&self) -> &[DeviceControl] {
        &self.devices
    }

    pub fn build_presentation(&self) -> BuildPresentation<'_> {
        BuildPresentation {
            devices: &self.devices,
            selected_device: self.selected_device,
        }
    }

    pub fn selected_device_id(&self) -> Option<ObjectId> {
        self.selected_device
    }

    pub fn selected_device(&self) -> Option<&DeviceControl> {
        let id = self.selected_device?;
        self.devices.iter().find(|device| device.instance_id == id)
    }

    pub fn shape_presentation(&self) -> ShapePresentation<'_> {
        ShapePresentation::from_selected_device(self.selected_device())
    }

    // Focus one existing device and enter Shape as one app-thread operation
    pub fn open_device_in_shape(&mut self, id: ObjectId) -> Result<(), &'static str> {
        if !self.devices.iter().any(|device| device.instance_id == id) {
            return Err("unknown device");
        }
        self.selected_device = Some(id);
        self.lens = Lens::Shape;
        Ok(())
    }

    // Publish the fixed offline fixture by stable device and parameter identity
    pub fn device_parameter_snapshot(
        &self,
    ) -> Result<Vec<DeviceParameterSnapshot>, DeviceParameterSnapshotError> {
        let fixture = [
            ("pulse", PULSE_PARAMETERS[0]),
            ("gain", GAIN_PARAMETERS[0]),
            ("saturator", SATURATOR_PARAMETERS[0]),
            ("saturator", SATURATOR_PARAMETERS[1]),
        ];

        fixture
            .into_iter()
            .map(|(device_key, descriptor)| {
                let device = self
                    .devices
                    .iter()
                    .find(|device| device.key == device_key)
                    .ok_or(DeviceParameterSnapshotError::MissingDevice(device_key))?;
                let parameter = device
                    .parameters
                    .iter()
                    .find(|parameter| parameter.descriptor.key == descriptor.key)
                    .ok_or(DeviceParameterSnapshotError::MissingParameter {
                        device_key,
                        parameter_key: descriptor.key,
                    })?;
                Ok(DeviceParameterSnapshot::new(
                    device.instance_id,
                    device_key,
                    parameter.instance_id,
                    descriptor,
                    parameter.value,
                ))
            })
            .collect()
    }

    // Apply a Shape edit and report the canonical stored value with its stable identities.
    // This is the single clamping site in the app, so a value cannot be clamped in one place and
    // published unclamped from another
    pub fn edit_device_parameter(
        &mut self,
        device_key: &str,
        parameter_key: &str,
        value: f32,
    ) -> Result<ParameterEdit, &'static str> {
        let device = self
            .devices
            .iter_mut()
            .find(|device| device.key == device_key)
            .ok_or("unknown device")?;
        let device_instance_id = device.instance_id;
        let parameter = device
            .parameters
            .iter_mut()
            .find(|parameter| parameter.descriptor.key.as_str() == parameter_key)
            .ok_or("unknown parameter")?;
        parameter.value = parameter.descriptor.clamp(value);
        Ok(ParameterEdit {
            device_instance_id,
            parameter_instance_id: parameter.instance_id,
            value: parameter.value,
        })
    }

    pub fn set_device_parameter(
        &mut self,
        device_key: &str,
        parameter_key: &str,
        value: f32,
    ) -> Result<(), &'static str> {
        self.edit_device_parameter(device_key, parameter_key, value)
            .map(|_| ())
    }

    // Append a track and select it; the ID comes from the model's own generator (CORE-001)
    pub fn add_track(&mut self, name: impl AsRef<str>) -> Result<ObjectId, TrackError> {
        let id = self.ids.next_id();
        let track = Track::new(id, name.as_ref(), TrackInstrument::Pulse)?;
        self.tracks.push(track)?;
        self.selected_track = Some(id);
        Ok(id)
    }

    pub fn feedback(&self) -> &str {
        &self.feedback
    }

    pub fn set_feedback(&mut self, feedback: impl Into<String>) {
        self.feedback = feedback.into();
    }

    pub fn feedback_report(&self) -> String {
        format!(
            "Spectre prototype feedback\nlens: {}\ntransport: {}\ntracks: {}\nselected track: {}\nselected device: {}\n\n{}",
            self.lens,
            if self.is_playing() { "playing" } else { "stopped" },
            self.tracks.len(),
            self.selected_track()
                .map(Track::name)
                .unwrap_or("none"),
            self.selected_device()
                .map(|device| format!("{} ({})", device.name, device.key))
                .unwrap_or_else(|| "none".into()),
            self.feedback.trim()
        )
    }
}

// Surface recoverable focus failures on the UI thread
pub fn open_device_in_shape_from_ui(
    model: &mut AppModel,
    id: ObjectId,
    feedback_status: &mut String,
) {
    if let Err(error) = model.open_device_in_shape(id) {
        *feedback_status =
            format!("Could not open device in Shape: {error}. Return to Build and try again.");
    }
}

// Surface recoverable parameter failures on the UI thread
pub fn set_device_parameter_from_ui(
    model: &mut AppModel,
    device_key: &str,
    parameter_key: &str,
    value: f32,
    feedback_status: &mut String,
) {
    if let Err(error) = model.set_device_parameter(device_key, parameter_key, value) {
        *feedback_status = format!(
            "Could not update {device_key}.{parameter_key}: {error}. Reopen the device from Build and try again."
        );
    }
}
