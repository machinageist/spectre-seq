// Author: Jeff
// Date: 2026-07-12
// Description: Renderer-neutral application model for the launchable Geist prototype
// Notes: UI state delegates transport truth to geist-core; no audio engine is implied

use geist_core::{IdGen, ObjectId, Transport, TransportCommand, TransportState};
use geist_dsp::{
    DeviceParameterSnapshot, DspParameter, GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS,
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

// Minimal track state rendered by every lens
#[derive(Debug, Clone, PartialEq)]
pub struct TrackView {
    pub id: ObjectId,
    pub name: String,
    pub muted: bool,
    pub solo: bool,
    pub armed: bool,
    pub level: f32,
}

// One backend-defined parameter and its current app-thread value
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterControl {
    pub descriptor: DspParameter,
    pub value: f32,
}

// One device card rendered from the stabilized backend contract
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceControl {
    pub key: &'static str,
    pub name: &'static str,
    pub role: &'static str,
    pub parameters: Vec<ParameterControl>,
}

impl DeviceControl {
    fn from_descriptors(
        key: &'static str,
        name: &'static str,
        role: &'static str,
        descriptors: &[DspParameter],
    ) -> Self {
        Self {
            key,
            name,
            role,
            parameters: descriptors
                .iter()
                .copied()
                .map(|descriptor| ParameterControl {
                    value: descriptor.default(),
                    descriptor,
                })
                .collect(),
        }
    }
}

// Internal fixture-schema failure; public mutation cannot create this state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceParameterSnapshotError {
    MissingDevice(&'static str),
    MissingParameter {
        device_key: &'static str,
        parameter_key: geist_dsp::DeviceParameterKey,
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
    transport: Transport,
    lens: Lens,
    tracks: Vec<TrackView>,
    devices: Vec<DeviceControl>,
    selected_track: Option<ObjectId>,
    ids: IdGen,
    feedback: String,
}

impl AppModel {
    // Create the deterministic first-launch state
    pub fn prototype() -> Self {
        let mut ids = IdGen::new(0x0047_4549_5354_5549);
        let track = TrackView {
            id: ids.next_id(),
            name: "Pulse".into(),
            muted: false,
            solo: false,
            armed: false,
            level: 0.78,
        };
        Self {
            transport: Transport::new(),
            lens: Lens::Arrange,
            selected_track: Some(track.id),
            tracks: vec![track],
            devices: vec![
                DeviceControl::from_descriptors(
                    "pulse",
                    "Pulse",
                    "Instrument · stereo out · note input",
                    &PULSE_PARAMETERS,
                ),
                DeviceControl::from_descriptors(
                    "gain",
                    "Gain",
                    "Effect · stereo in/out",
                    &GAIN_PARAMETERS,
                ),
                DeviceControl::from_descriptors(
                    "saturator",
                    "Saturator",
                    "Effect · stereo in/out",
                    &SATURATOR_PARAMETERS,
                ),
            ],
            ids,
            feedback: String::new(),
        }
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

    pub fn tracks(&self) -> &[TrackView] {
        &self.tracks
    }

    pub fn selected_track_id(&self) -> Option<ObjectId> {
        self.selected_track
    }

    pub fn select_track(&mut self, id: ObjectId) {
        if self.tracks.iter().any(|track| track.id == id) {
            self.selected_track = Some(id);
        }
    }

    pub fn selected_track(&self) -> Option<&TrackView> {
        let id = self.selected_track?;
        self.tracks.iter().find(|track| track.id == id)
    }

    pub fn selected_track_mut(&mut self) -> Option<&mut TrackView> {
        let id = self.selected_track?;
        self.tracks.iter_mut().find(|track| track.id == id)
    }

    pub fn devices(&self) -> &[DeviceControl] {
        &self.devices
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
                    device_key,
                    descriptor,
                    parameter.value,
                ))
            })
            .collect()
    }

    pub fn set_device_parameter(
        &mut self,
        device_key: &str,
        parameter_key: &str,
        value: f32,
    ) -> Result<(), &'static str> {
        let device = self
            .devices
            .iter_mut()
            .find(|device| device.key == device_key)
            .ok_or("unknown device")?;
        let parameter = device
            .parameters
            .iter_mut()
            .find(|parameter| parameter.descriptor.key.as_str() == parameter_key)
            .ok_or("unknown parameter")?;
        parameter.value = parameter.descriptor.clamp(value);
        Ok(())
    }

    pub fn add_track(&mut self, name: impl Into<String>) -> Result<ObjectId, &'static str> {
        let name = name.into();
        let name = name.trim();
        if name.is_empty() {
            return Err("track name must not be blank");
        }
        let id = self.ids.next_id();
        self.tracks.push(TrackView {
            id,
            name: name.into(),
            muted: false,
            solo: false,
            armed: false,
            level: 0.72,
        });
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
            "Geist prototype feedback\nlens: {}\ntransport: {}\ntracks: {}\nselected track: {}\n\n{}",
            self.lens,
            if self.is_playing() { "playing" } else { "stopped" },
            self.tracks.len(),
            self.selected_track()
                .map(|track| track.name.as_str())
                .unwrap_or("none"),
            self.feedback.trim()
        )
    }
}
