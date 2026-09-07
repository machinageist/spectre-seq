// Author: Jeff
// Date: 2026-07-12
// Description: Renderer-neutral application model for the launchable Spectre prototype
// Notes: UI state delegates transport truth to spectre-core; the live engine lives in
//   engine.rs and AppModel stays renderer-neutral and audio-free

pub mod bounce_panel;
pub mod engine;
pub mod project;

use spectre_core::{
    BeatTicks, IdGen, MeterMap, ObjectId, TempoMap, TempoMapError, TimeSignature, Transport,
    TransportCommand, TransportState,
};
use spectre_dsp::{
    DeviceParameterSnapshot, DspParameter, FILAMENT_PARAMETERS, GAIN_PARAMETERS, GLOAM_PARAMETERS,
    PULSE_PARAMETERS, SATURATOR_PARAMETERS,
};
use spectre_project::command::{
    CommandError, EditHistory, EditScope, Editable, ProjectCommand, Transaction,
};
use spectre_project::{
    ClipError, ClipNote, ClipPlacement, MidiClip, Track, TrackError, TrackInsert, TrackInstrument,
    TrackList,
};

// How many edits the model can reverse. Bounded because an unbounded history is a memory leak
// that grows with the length of a session, and a delete's inverse carries the whole track. The
// number is Spectre's own: 64 covers a working stretch between saves without a rationale row
// borrowed from any reference product (PROD-003)
pub const UNDO_HISTORY_DEPTH: usize = 64;

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
    // The loop region in TICKS, which is the musical truth. Transport carries a sample-domain
    // region and persists it, but samples are a function of tempo -- a persisted sample loop
    // silently moves when the tempo changes, so this is the source of truth and the sample
    // region is derived from it wherever the sample rate is known.
    //
    // Session-only for now: persisting it needs a tick-domain field the schema does not have,
    // and adding one is a migration this slice does not carry
    loop_ticks: Option<(BeatTicks, BeatTicks)>,
    ids: IdGen,
    feedback: String,
    // Bounded undo/redo over the project's own edits. Held by the model rather than the shell so
    // an edit cannot reach the track list without passing through it -- a history the caller has
    // to remember to use is a history someone forgets at the next call site
    history: EditHistory,
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
            loop_ticks: None,
            tracks,
            devices,
            selected_device,
            ids,
            feedback: String::new(),
            history: EditHistory::new(UNDO_HISTORY_DEPTH).expect("a nonzero history depth"),
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

        // One transaction, so creating a clip is a single undo step rather than two. The clip
        // must exist before the placement, because insert_placement reads the clip's length from
        // the table -- and if the placement is refused by an overlap the transaction rolls the
        // clip back, so no orphaned material is left behind either way
        let group = Transaction::new(vec![
            ProjectCommand::add_clip(clip),
            ProjectCommand::insert_placement(track, placement),
        ])
        .expect("two commands is not empty");
        self.apply(group).map_err(unwrap_clip_error)?;
        self.selected_clip = Some(placement_id);
        Ok(placement_id)
    }

    // Delete a placement and, when nothing else places it, its clip material too. One
    // transaction, so an undo restores both
    pub fn delete_clip(&mut self, placement: ObjectId) -> Result<(), ClipError> {
        let track = self
            .clip_track(placement)
            .ok_or(ClipError::UnknownPlacement(placement))?;
        let clip_id = self
            .tracks
            .get(track)
            .and_then(|entry| entry.clips().get(placement))
            .map(|entry| entry.clip())
            .ok_or(ClipError::UnknownPlacement(placement))?;
        // Only this placement uses the material, so removing it leaves nothing dangling
        let sole_user = self
            .tracks
            .tracks()
            .iter()
            .flat_map(|entry| entry.clips().placements())
            .filter(|entry| entry.clip() == clip_id)
            .count()
            == 1;

        let mut commands = vec![ProjectCommand::remove_placement(track, placement)];
        if sole_user {
            commands.push(ProjectCommand::remove_clip(clip_id));
        }
        self.apply(Transaction::new(commands).expect("at least one command"))
            .map_err(unwrap_clip_error)?;
        if self.selected_clip == Some(placement) {
            self.selected_clip = None;
        }
        Ok(())
    }

    // Move a placement along its track's timeline. Refused by an overlap, which leaves it exactly
    // where it was
    pub fn move_clip(&mut self, placement: ObjectId, start: BeatTicks) -> Result<(), ClipError> {
        let track = self
            .clip_track(placement)
            .ok_or(ClipError::UnknownPlacement(placement))?;
        self.apply(Transaction::single(ProjectCommand::move_placement(
            track, placement, start,
        )))
        .map_err(unwrap_clip_error)
    }

    // Note authoring: what a piano roll calls.
    //
    // Both take a built ClipNote rather than its fields. ClipNote::new is the one place a note is
    // validated, so passing one through keeps that single point and keeps these signatures from
    // growing a parameter every time the note model does
    pub fn add_note(&mut self, clip: ObjectId, note: ClipNote) -> Result<(), ClipError> {
        self.apply(Transaction::single(ProjectCommand::insert_note(clip, note)))
            .map_err(unwrap_clip_error)
    }

    pub fn remove_note(&mut self, clip: ObjectId, index: usize) -> Result<(), ClipError> {
        self.apply(Transaction::single(ProjectCommand::remove_note(
            clip, index,
        )))
        .map_err(unwrap_clip_error)
    }

    // Change one note: a drag, a resize, or a velocity edit. Notes are kept sorted by
    // (start, note), so any such edit moves the note's index -- which is why this is a remove and
    // an insert grouped atomically rather than a mutation in place. A refused replacement leaves
    // the original note exactly where it was
    pub fn replace_note(
        &mut self,
        clip: ObjectId,
        index: usize,
        replacement: ClipNote,
    ) -> Result<(), ClipError> {
        let group = Transaction::new(vec![
            ProjectCommand::remove_note(clip, index),
            ProjectCommand::insert_note(clip, replacement),
        ])
        .expect("two commands is not empty");
        self.apply(group).map_err(unwrap_clip_error)
    }

    // The notes of one clip, in the order the model keeps them, which is the order an editor
    // addresses them by index
    pub fn clip_notes(&self, clip: ObjectId) -> Option<&[ClipNote]> {
        self.tracks.clip(clip).map(|entry| entry.notes())
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

    // Apply one edit through the history, so no product mutation can reach the model without
    // becoming reversible. Every mutator below goes through here rather than touching its field
    // directly, which is what stops the next one from quietly forgetting
    fn edit(&mut self, command: ProjectCommand) -> Result<(), CommandError> {
        self.apply(Transaction::single(command))
    }

    // Borrow the editable pieces as one target. Disjoint field borrows, so the history and the
    // state it mutates can be held at once.
    //
    // AppModel cannot implement Editable itself: the history lives on the same struct, so
    // `self.history.apply(&mut self, ..)` would borrow it twice
    fn apply(&mut self, transaction: Transaction) -> Result<(), CommandError> {
        let mut target = AppEditTarget {
            tracks: &mut self.tracks,
            tempo: &mut self.tempo_map,
        };
        self.history.apply(&mut target, transaction)
    }

    // The loop region a musician set, in ticks. None means looping is off
    pub fn loop_ticks(&self) -> Option<(BeatTicks, BeatTicks)> {
        self.loop_ticks
    }

    // Set or clear the loop. Refuses an empty or inverted region rather than storing one the
    // transport would reject when it was converted, so the refusal happens where it is visible.
    //
    // Not routed through the history: a loop region is where you are looking, not what the
    // project contains, and an undo stack full of loop moves buries the edits that matter
    pub fn set_loop(&mut self, region: Option<(BeatTicks, BeatTicks)>) -> Result<(), ClipError> {
        if let Some((start, end)) = region {
            if start.0 < 0 || end.0 <= start.0 {
                return Err(ClipError::NoteOutsideClip {
                    start: start.0,
                    length: end.0 - start.0,
                });
            }
        }
        self.loop_ticks = region;
        Ok(())
    }

    // Replace the project tempo. Reversible, and the inverse carries the whole previous map, so
    // undoing a tempo change on a project that had a curve restores the curve rather than a
    // constant taken from its first segment
    pub fn set_tempo(&mut self, bpm: f64) -> Result<(), TempoMapError> {
        self.set_tempo_map(TempoMap::constant(bpm)?)
    }

    // Replace the whole map, so a tempo curve can be set as one reversible edit rather than as a
    // sequence of constants that undo one segment at a time
    pub fn set_tempo_map(&mut self, map: TempoMap) -> Result<(), TempoMapError> {
        self.apply(Transaction::single(ProjectCommand::set_tempo_map(map)))
            // A tempo command cannot be refused by the history: the app target always carries a
            // tempo, so the only refusal is the one TempoMap::constant already made above
            .map_err(|_| TempoMapError::BpmOutOfRange)
    }

    // Reverse the latest edit; false means there was none. Selection is repaired afterwards
    // because an undone delete restores a track the selection may have moved off
    pub fn undo(&mut self) -> Result<bool, CommandError> {
        let mut target = AppEditTarget {
            tracks: &mut self.tracks,
            tempo: &mut self.tempo_map,
        };
        let moved = self.history.undo(&mut target)?;
        self.repair_selection();
        Ok(moved)
    }

    pub fn redo(&mut self) -> Result<bool, CommandError> {
        let mut target = AppEditTarget {
            tracks: &mut self.tracks,
            tempo: &mut self.tempo_map,
        };
        let moved = self.history.redo(&mut target)?;
        self.repair_selection();
        Ok(moved)
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    // Keep selection pointing at something that exists. An undo can remove the selected track
    // and a redo can restore one, so this runs after both rather than being inferred at draw
    fn repair_selection(&mut self) {
        if self
            .selected_track
            .is_none_or(|id| self.tracks.get(id).is_none())
        {
            self.selected_track = self.tracks.tracks().first().map(Track::id);
        }
        if self
            .selected_clip
            .is_some_and(|placement| self.clip_track(placement).is_none())
        {
            self.selected_clip = None;
        }
    }

    // Rename one track; a blank name leaves the model unchanged
    pub fn rename_track(&mut self, id: ObjectId, name: &str) -> Result<(), TrackError> {
        self.edit(ProjectCommand::set_track_name(id, name))
            .map_err(unwrap_track_error)
    }

    // Set one track's fader position. Returns the ids whose effective gain changed — exactly one
    pub fn set_track_level(
        &mut self,
        id: ObjectId,
        level: f32,
    ) -> Result<Vec<ObjectId>, TrackError> {
        self.edit(ProjectCommand::set_track_level(id, level))
            .map_err(unwrap_track_error)?;
        Ok(vec![id])
    }

    // Mute one track. Returns the ids whose effective gain changed — exactly one
    pub fn set_track_muted(
        &mut self,
        id: ObjectId,
        muted: bool,
    ) -> Result<Vec<ObjectId>, TrackError> {
        self.edit(ProjectCommand::set_track_muted(id, muted))
            .map_err(unwrap_track_error)?;
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
        self.edit(ProjectCommand::set_track_soloed(id, soloed))
            .map_err(unwrap_track_error)?;
        Ok(self.tracks.tracks().iter().map(Track::id).collect())
    }

    // Set the master fader. Publishes the master gain's own target, not a track's, so the
    // returned track set is empty
    pub fn set_master_level(&mut self, level: f32) -> Vec<ObjectId> {
        // Refusal is impossible: no target can be absent, so the history cannot reject it
        let _ = self.edit(ProjectCommand::set_master_level(level));
        Vec::new()
    }

    // Set one track's instrument level. Returns the ids whose instrument level changed
    pub fn set_track_instrument_level(
        &mut self,
        id: ObjectId,
        level: f32,
    ) -> Result<Vec<ObjectId>, TrackError> {
        self.edit(ProjectCommand::set_track_instrument_level(id, level))
            .map_err(unwrap_track_error)?;
        Ok(vec![id])
    }

    // Add one effect to a track's chain at a position. Order is the signal path, so appending
    // and inserting are the same operation with a different index rather than two methods
    pub fn add_effect(
        &mut self,
        track: ObjectId,
        position: usize,
        insert: TrackInsert,
    ) -> Result<(), TrackError> {
        self.edit(ProjectCommand::insert_effect(track, position, insert))
            .map_err(unwrap_track_error)
    }

    // Append to the end, which is what a browser's "add" means
    pub fn append_effect(
        &mut self,
        track: ObjectId,
        insert: TrackInsert,
    ) -> Result<(), TrackError> {
        let position = self
            .tracks
            .get(track)
            .ok_or(TrackError::UnknownTrack(track))?
            .inserts()
            .len();
        self.add_effect(track, position, insert)
    }

    pub fn remove_effect(&mut self, track: ObjectId, position: usize) -> Result<(), TrackError> {
        self.edit(ProjectCommand::remove_effect(track, position))
            .map_err(unwrap_track_error)
    }

    pub fn move_effect(
        &mut self,
        track: ObjectId,
        from: usize,
        to: usize,
    ) -> Result<(), TrackError> {
        self.edit(ProjectCommand::move_effect(track, from, to))
            .map_err(unwrap_track_error)
    }

    // Returns the ids whose effective gain changed, which is none: a depth edit reaches the
    // effect's own parameter target and no track's gain
    pub fn set_effect_depth(
        &mut self,
        track: ObjectId,
        position: usize,
        depth: f32,
    ) -> Result<(), TrackError> {
        self.edit(ProjectCommand::set_effect_depth(track, position, depth))
            .map_err(unwrap_track_error)
    }

    // Change which instrument a track hosts.
    //
    // Without this every track created in the product was a Pulse saw for the life of the
    // project: TrackList::set_instrument existed and had no caller, so Filament -- the alpha's
    // own synth -- was unreachable on any track a musician made.
    //
    // A shape change, so structure_revision advances and the shell reports PLAN STALE until the
    // engine is rebuilt. That is the accepted rule, not a limitation introduced here
    pub fn set_track_instrument(
        &mut self,
        id: ObjectId,
        instrument: TrackInstrument,
    ) -> Result<(), TrackError> {
        self.edit(ProjectCommand::set_track_instrument(id, instrument))
            .map_err(unwrap_track_error)
    }

    // Move one track to an absolute index; identity and every field survive (CORE-001)
    pub fn reorder_track(&mut self, id: ObjectId, to_index: usize) -> Result<usize, TrackError> {
        let from = self
            .tracks
            .index_of(id)
            .ok_or(TrackError::UnknownTrack(id))?;
        self.edit(ProjectCommand::reorder_tracks(id, to_index))
            .map_err(unwrap_track_error)?;
        Ok(from)
    }

    // Remove one track, returning it whole so an undo can reinsert it unchanged
    pub fn remove_track(&mut self, id: ObjectId) -> Result<Track, TrackError> {
        let removed = self
            .tracks
            .get(id)
            .cloned()
            .ok_or(TrackError::UnknownTrack(id))?;
        self.edit(ProjectCommand::remove_track(id))
            .map_err(unwrap_track_error)?;
        self.repair_selection();
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
        // Appended through the history, so the add is reversible and its inverse addresses the
        // identity the generator just minted rather than a position
        self.edit(ProjectCommand::insert_track(self.tracks.len(), track))
            .map_err(unwrap_track_error)?;
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

// Track edits reach the history through commands, and the only failure a track command can
// produce is a TrackError the list itself raised. The other CommandError variants are
// structurally unreachable here -- an empty transaction cannot be built by `edit`, the capacity
// is nonzero by construction, and no track command touches a project name -- so this converts
// rather than widening every mutator's error type with variants they cannot return
fn unwrap_track_error(error: CommandError) -> TrackError {
    match error {
        CommandError::Track(error) => error,
        other => unreachable!("a track command produced {other:?}"),
    }
}

// Clip authoring reaches the history through commands, and the only failure a clip command can
// produce is a ClipError the model itself raised. The structural variants are unreachable here
// for the same reasons unwrap_track_error states
fn unwrap_clip_error(error: CommandError) -> ClipError {
    match error {
        CommandError::Clip(error) => error,
        other => unreachable!("a clip command produced {other:?}"),
    }
}

// The editable pieces of AppModel, borrowed as one target so commands can reach tempo as well as
// tracks. A struct rather than an impl on AppModel because the history lives on AppModel too,
// and borrowing the whole model would borrow the history twice
struct AppEditTarget<'a> {
    tracks: &'a mut TrackList,
    tempo: &'a mut TempoMap,
}

impl Editable for AppEditTarget<'_> {
    fn edit_scope(&mut self) -> EditScope<'_> {
        EditScope::new(None, self.tracks, Some(self.tempo))
    }
}
