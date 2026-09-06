// Author: Jeff
// Date: 2026-08-24
// Description: App-thread host for the live audio engine
// Notes: Every method here runs on the app thread and may allocate. Nothing in this file is
//   reachable from the audio callback; the only value that crosses is the RenderBridge moved
//   into the render closure at open. No DSP and no second render path are written here.

use spectre_audio::bridge::{BridgeTelemetry, RenderBridge, DEFAULT_NOTE_SCRATCH};
use spectre_audio::clip::{ClipPlayer, ClipSchedule, CLIP_EVENT_RESERVE};
use spectre_audio::control::{
    control_channel, ControlError, ControlSender, ParameterTarget, DEFAULT_NOTE_CAPACITY,
    DEFAULT_TRANSPORT_CAPACITY,
};
use spectre_audio::route::{ParameterRoute, ParameterRoutes, RouteError};
use spectre_audio::{AudioBackend, AudioStream, BackendError, RenderBlock, StreamConfig};
use spectre_core::{IdGen, TransportCommand};
use spectre_dsp::{
    AudioProcessor, DeviceParameterSnapshot, Gain, NoteEvent, NoteEventKind, PulseInstrument,
    Saturator, Waveform, GAIN_PARAMETERS, GLOAM_DEPTH, GLOAM_PARAMETERS, PULSE_PARAMETERS,
    SATURATOR_PARAMETERS,
};
use spectre_graph::{Connection, EditableGraph, NodeId};
use std::sync::Arc;

use crate::{AppModel, DeviceParameterSnapshotError};

// Numeric bounds this slice introduces, each with its own rationale (decision 16, PROD-003)

// Requested driver block size. Rationale: 256 frames is the only block size for which Spectre
// has its own measured hardware evidence — the macOS qualification recorded in
// docs/06-plans/current-milestone.md rendered 173 callbacks at 256/48 kHz with 0 xruns and
// 0.990 worst-case headroom. Reusing it compares the first live run of ./spectre against a
// baseline Spectre measured, not a number taken from another product. Re-open when R4-3's
// Linux qualification produces a second measurement
pub const ENGINE_BUFFER_FRAMES: usize = 256;

// Plan capacity margin over the requested block. Rationale: the seam asks cpal for
// BufferSize::Fixed but no Spectre evidence proves every host honors it, and the macOS record
// carries no frame-capacity rejection count, so a larger-than-requested block is
// unproven-absent rather than known-absent. The margin's entire cost is 6,144 additional
// preallocated bytes; any block beyond it still lands in the already-implemented counted
// silence refusal. Not derived from any reference product
pub const ENGINE_PLAN_FRAME_MARGIN: usize = 2;

// Node identity seed for the live plan. Rationale: node IDs must be stable across builds so a
// plan compiled twice from one snapshot is the same plan; the value itself is arbitrary and
// carries no meaning beyond determinism
const ENGINE_NODE_SEED: u64 = 0x0000_5245_4e44_4552;

// The single audition voice R4-1 sends on Play. Rationale: these are the values the offline
// fixture already renders (spectre-offline's fixture_events), so live audition and offline
// render the same voice rather than two voices that happen to coexist. R4-5 replaces this
// scaffolding with clip playback
const AUDITION_VOICE_ID: u32 = 1;
const AUDITION_CHANNEL: u8 = 0;
const AUDITION_NOTE: u8 = 45;
const AUDITION_VELOCITY: f32 = 0.8;

// The canonical four-parameter fixture, in the order the offline harness validates it
fn canonical_fixture() -> [(&'static str, spectre_dsp::DspParameter); 4] {
    [
        ("pulse", PULSE_PARAMETERS[0]),
        ("gain", GAIN_PARAMETERS[0]),
        ("saturator", SATURATOR_PARAMETERS[0]),
        ("saturator", SATURATOR_PARAMETERS[1]),
    ]
}

// Why the engine is not running, phrased for direct display
#[derive(Debug, Clone, PartialEq)]
pub enum EngineUnavailable {
    // Compiled without a real backend; the app declines to fake one with NullBackend
    NoBackendCompiled,
    // Headless or smoke invocation deliberately skipped the start attempt
    NotAttempted,
    Backend(BackendError),
    // The live plan could not be compiled from the app snapshot; a defect, not user error
    Plan(String),
    Control(ControlError),
    // Two parameter targets resolved to the same identity; a shadowed route would send every
    // edit of one parameter to the wrong device, so it is refused rather than tolerated
    Route(RouteError),
    // The track graph could not be built from the model
    Routing(spectre_project::RoutingError),
    // The app snapshot was not the complete canonical fixture
    Snapshot(DeviceParameterSnapshotError),
}

impl std::fmt::Display for EngineUnavailable {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoBackendCompiled => formatter
                .write_str("built without an audio backend; rebuild with the live-audio feature"),
            Self::NotAttempted => formatter.write_str("engine start was not attempted"),
            Self::Backend(error) => write!(formatter, "audio backend: {error}"),
            Self::Plan(error) => write!(formatter, "plan compilation: {error}"),
            Self::Control(error) => write!(formatter, "control transport: {error}"),
            Self::Route(error) => write!(formatter, "parameter routing: {error}"),
            Self::Routing(error) => write!(formatter, "track routing: {error}"),
            Self::Snapshot(error) => write!(formatter, "device snapshot: {error}"),
        }
    }
}

// What the transport bar renders; derived, never stored as a claim
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineState {
    // Stream opened and started, but the driver has not called back yet
    Opened {
        backend: &'static str,
        device: String,
        sample_rate: u32,
        frames: usize,
    },
    // Opened AND blocks_rendered > 0; the only state allowed to read as "running"
    Running {
        backend: &'static str,
        device: String,
        sample_rate: u32,
        frames: usize,
    },
}

// Owned copy of the render thread's counters, read once per frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineHealth {
    pub blocks_rendered: u64,
    pub xruns: u64,
    pub worst_headroom: f32,
    pub last_headroom: f32,
    pub plan_errors: u64,
    pub frame_capacity_rejections: u64,
    pub notes_deferred: u64,
    pub contaminated_nodes: u64,
    pub stream_errors: u64,
    // Parameter edits delivered to a live processor
    pub parameters_applied: u64,
    // Parameter edits observed but not delivered. In a correctly wired build this stays zero;
    // §1.3 of the R4-2 spec names it as the field that encodes the acceptance criterion
    pub parameters_pending: u64,
    // The playhead in samples, None until a block has rendered. None is drawn as "—"; drawing a
    // zero would be the fabricated position r4-qa-protocol.md row 4 exists to catch
    pub position_samples: Option<i64>,
    pub transport_rolling: bool,
    // Peak of the last rendered block. The objective half of "produces sound": a clean render
    // that emits silence satisfies every other field here
    pub last_peak: f32,
    // Highest peak since the engine opened; "did anything sound", not "is it sounding now"
    pub session_peak: f32,
}

// Render-side halves built together, before any device is touched
pub struct EngineParts {
    pub bridge: RenderBridge,
    pub sender: ControlSender,
    pub telemetry: Arc<BridgeTelemetry>,
    pub config: StreamConfig,
    // Frames the plan was compiled for. RenderBridge keeps its plan private and exposes only
    // new/telemetry/transport/render, so plan capacity is unreadable through the bridge; this
    // carries the value build_engine_parts passed to EditableGraph::compile instead
    pub plan_max_frames: usize,
    // Registered lane targets, so the app can address slots by stable identity without
    // rebuilding the set it already published
    pub targets: Box<[ParameterTarget]>,
    // The plan nodes build_engine_parts allocated, exposed so routes can be built against them.
    // None for a track-list plan, whose node identities live in spectre-project's TrackPathNodes
    pub nodes: Option<FixtureNodes>,
    // True when a clip player is attached to any note node. Play then sends the transport
    // command alone: the project's own material REPLACES the audition note rather than merging
    // with it, because merging two producers on one node would turn R4-1's clean Play/Stop
    // refusal into a stuck note
    pub has_clips: bool,
}

// The three plan nodes build_engine_parts allocates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixtureNodes {
    pub pulse: NodeId,
    pub gain: NodeId,
    pub saturator: NodeId,
}

impl FixtureNodes {
    // Resolve a canonical device key to its plan node
    fn node_for(&self, device_key: &str) -> Option<NodeId> {
        match device_key {
            "pulse" => Some(self.pulse),
            "gain" => Some(self.gain),
            "saturator" => Some(self.saturator),
            _ => None,
        }
    }
}

// Build the lane's target set and the render-side route table from the same validated snapshot
// the plan is built from. No new app-side identity plumbing is needed: DeviceParameterSnapshot
// already carries device_instance_id, device_key, parameter_instance_id, and parameter_key,
// which is exactly the join
pub fn build_parameter_wiring(
    snapshot: &[DeviceParameterSnapshot],
    nodes: &FixtureNodes,
) -> Result<(Vec<ParameterTarget>, ParameterRoutes), EngineUnavailable> {
    let mut targets = Vec::with_capacity(snapshot.len());
    let mut routes = Vec::with_capacity(snapshot.len());
    for entry in snapshot {
        let Some(node) = nodes.node_for(entry.device_key()) else {
            // A device outside the canonical fixture cannot be routed; fixture_values already
            // refused an incomplete snapshot, so this is an unknown extra rather than a gap
            return Err(EngineUnavailable::Snapshot(
                DeviceParameterSnapshotError::MissingDevice(entry.device_key()),
            ));
        };
        let target = ParameterTarget {
            device: entry.device_instance_id(),
            parameter: entry.parameter_instance_id(),
        };
        targets.push(target);
        routes.push(ParameterRoute {
            target,
            node,
            key: entry.parameter_key(),
        });
    }
    let routes = ParameterRoutes::new(routes).map_err(EngineUnavailable::Route)?;
    Ok((targets, routes))
}

// Which half of a two-send audition sequence was refused; the caller needs the distinction
// because a queued transport command cannot be recalled from a wait-free lane
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditionError {
    // The transport send was refused, so the render transport will not change and the UI must
    // not either. Dominant: returned even if the note send was also refused
    Transport(ControlError),
    // The transport command was queued; only the note send was refused, so the UI change stands
    Note(ControlError),
}

// Wrap any plan-construction failure with its own diagnostic; never reached on the render path
fn plan_error(error: impl std::fmt::Display) -> EngineUnavailable {
    EngineUnavailable::Plan(error.to_string())
}

// Resolve the four canonical fixture values out of an app snapshot, fail-closed
fn fixture_values(
    snapshot: &[DeviceParameterSnapshot],
) -> Result<[f32; 4], DeviceParameterSnapshotError> {
    let mut values = [0.0_f32; 4];
    for (slot, (device_key, descriptor)) in values.iter_mut().zip(canonical_fixture()) {
        let entry = snapshot
            .iter()
            .find(|entry| {
                entry.device_key() == device_key && entry.parameter_key() == descriptor.key
            })
            .ok_or(DeviceParameterSnapshotError::MissingParameter {
                device_key,
                parameter_key: descriptor.key,
            })?;
        *slot = entry.value();
    }
    Ok(values)
}

// Build the render-side halves from a validated app snapshot; touches no device
pub fn build_engine_parts(
    snapshot: &[DeviceParameterSnapshot],
    config: StreamConfig,
) -> Result<EngineParts, EngineUnavailable> {
    config.validate().map_err(EngineUnavailable::Backend)?;
    let [pulse_level, gain_value, drive, mix] =
        fixture_values(snapshot).map_err(EngineUnavailable::Snapshot)?;

    let mut ids = IdGen::new(ENGINE_NODE_SEED);
    let pulse = NodeId::new(ids.next_id());
    let gain = NodeId::new(ids.next_id());
    let saturator = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            pulse,
            PulseInstrument::new(Waveform::Saw, pulse_level)
                .map_err(plan_error)?
                .io(),
        )
        .map_err(plan_error)?;
    graph
        .add_node(gain, Gain::new(gain_value).map_err(plan_error)?.io())
        .map_err(plan_error)?;
    graph
        .add_node(
            saturator,
            Saturator::new(drive, mix).map_err(plan_error)?.io(),
        )
        .map_err(plan_error)?;
    for (from, to) in [(pulse, gain), (gain, saturator)] {
        graph
            .connect(Connection {
                from,
                from_bus: 0,
                to,
                to_bus: 0,
            })
            .map_err(plan_error)?;
    }

    let plan_max_frames = config.buffer_frames * ENGINE_PLAN_FRAME_MARGIN;
    let plan = graph
        .compile(saturator, plan_max_frames, &mut |node| {
            if node == pulse {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, pulse_level)?))
            } else if node == gain {
                Ok(Box::new(Gain::new(gain_value)?))
            } else {
                Ok(Box::new(Saturator::new(drive, mix)?))
            }
        })
        .map_err(plan_error)?;

    // The lane's target set and the render-side route table come from the same snapshot the plan
    // was built from, so an identity can never be registered without a node to apply it to
    let nodes = FixtureNodes {
        pulse,
        gain,
        saturator,
    };
    let (targets, routes) = build_parameter_wiring(snapshot, &nodes)?;
    let (sender, receiver) =
        control_channel(&targets, DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY)
            .map_err(EngineUnavailable::Control)?;
    let bridge = RenderBridge::with_parameter_routes(
        plan,
        receiver,
        pulse,
        f64::from(config.sample_rate),
        DEFAULT_NOTE_SCRATCH,
        routes,
    );
    let telemetry = bridge.telemetry();

    Ok(EngineParts {
        bridge,
        sender,
        telemetry,
        config,
        plan_max_frames,
        targets: targets.into_boxed_slice(),
        nodes: Some(nodes),
        // The fixture chain has no project clips; Play auditions, as it has since R4-1
        has_clips: false,
    })
}

// Open and start a named device over already-built parts, consuming the parts' bridge
pub fn open_with_parts(
    backend: &dyn AudioBackend,
    device_id: &spectre_audio::DeviceId,
    device_name: String,
    parts: EngineParts,
) -> Result<LiveEngine, EngineUnavailable> {
    let EngineParts {
        mut bridge,
        sender,
        telemetry,
        config,
        plan_max_frames: _,
        targets,
        nodes: _,
        has_clips,
    } = parts;

    let mut stream = backend
        .open_output(
            device_id,
            config,
            Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),
        )
        .map_err(EngineUnavailable::Backend)?;
    stream.start().map_err(EngineUnavailable::Backend)?;

    let mut engine = LiveEngine::from_open_stream(
        stream,
        sender,
        telemetry,
        backend.name(),
        device_name,
        config,
    );
    engine.set_targets(targets);
    engine.set_has_clips(has_clips);
    Ok(engine)
}

// App-thread owner of the live stream; not Send, because AudioStream is not Send. Generic over
// the stream with dyn AudioStream as the default: the app holds the erased
// LiveEngine<dyn AudioStream> that open_default returns, while a test can hold a concrete
// LiveEngine<NullStream> and still reach NullStream::pump, which is inherent to NullStream and
// absent from the AudioStream trait
pub struct LiveEngine<S: ?Sized = dyn AudioStream> {
    sender: ControlSender,
    telemetry: Arc<BridgeTelemetry>,
    backend_name: &'static str,
    device_name: String,
    config: StreamConfig,
    // Sequence counter for outgoing note events; the ordering key's tie-break
    next_sequence: u64,
    // Registered lane targets, so a Shape edit can be addressed by stable identity
    targets: Box<[ParameterTarget]>,
    // True when the render thread is playing the project's own clips. Play then sends the
    // transport command alone, because the project's material replaces the audition note
    has_clips: bool,
    // Box<S> is itself Sized even when S is not, so field order is unconstrained here
    stream: Box<S>,
}

impl<S: AudioStream + ?Sized> LiveEngine<S> {
    // Adopt an already-open, already-started stream and the app-thread halves of its parts.
    // The bridge is not passed: the caller must already have moved it into the render closure
    // at open, which is what erases it from the app thread
    pub fn from_open_stream(
        stream: Box<S>,
        sender: ControlSender,
        telemetry: Arc<BridgeTelemetry>,
        backend_name: &'static str,
        device_name: String,
        config: StreamConfig,
    ) -> Self {
        Self {
            sender,
            telemetry,
            backend_name,
            device_name,
            config,
            next_sequence: 0,
            targets: Box::new([]),
            has_clips: false,
            stream,
        }
    }

    // Record whether the render thread is playing project clips. Separate from from_open_stream
    // for the same reason set_targets is: R4-1's signature and its callers stay unchanged
    pub fn set_has_clips(&mut self, has_clips: bool) {
        self.has_clips = has_clips;
    }

    // Report whether Play will audition or play the project's own material
    pub fn has_clips(&self) -> bool {
        self.has_clips
    }

    // Record the lane targets the parts registered. Separate from from_open_stream so R4-1's
    // signature, and every test that calls it, is unchanged
    pub fn set_targets(&mut self, targets: Box<[ParameterTarget]>) {
        self.targets = targets;
    }

    // Report the registered lane targets
    pub fn targets(&self) -> &[ParameterTarget] {
        &self.targets
    }

    // Publish one already-clamped value to its latest-wins slot. Takes &self, not &mut self:
    // the write is an atomic store and needs no exclusivity
    pub fn send_parameter(&self, target: ParameterTarget, value: f32) -> Result<(), ControlError> {
        self.sender.parameters().set_target(target, value)
    }

    // Borrow the stream for callers that must drive it explicitly. The app never calls this; it
    // exists so a test holding LiveEngine<NullStream> can pump between state() reads
    pub fn stream_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    // Report what the transport bar should render, derived from the render thread's counters
    pub fn state(&self) -> EngineState {
        let backend = self.backend_name;
        let device = self.device_name.clone();
        let sample_rate = self.config.sample_rate;
        let frames = self.config.buffer_frames;
        if self.telemetry.blocks_rendered() > 0 {
            EngineState::Running {
                backend,
                device,
                sample_rate,
                frames,
            }
        } else {
            EngineState::Opened {
                backend,
                device,
                sample_rate,
                frames,
            }
        }
    }

    // Copy the render thread's counters once, so one UI frame reads one consistent set
    pub fn health(&self) -> EngineHealth {
        EngineHealth {
            blocks_rendered: self.telemetry.blocks_rendered(),
            xruns: self.telemetry.xruns(),
            worst_headroom: self.telemetry.worst_headroom(),
            last_headroom: self.telemetry.last_headroom(),
            plan_errors: self.telemetry.plan_errors(),
            frame_capacity_rejections: self.telemetry.frame_capacity_rejections(),
            notes_deferred: self.telemetry.notes_deferred(),
            contaminated_nodes: self.telemetry.contaminated_nodes(),
            stream_errors: self.stream.stream_errors(),
            parameters_applied: self.telemetry.parameters_applied(),
            parameters_pending: self.telemetry.parameters_pending(),
            position_samples: self.telemetry.position_samples(),
            transport_rolling: self.telemetry.transport_rolling(),
            last_peak: self.telemetry.last_peak(),
            session_peak: self.telemetry.session_peak(),
        }
    }

    // Report the configuration the stream was opened with
    pub fn config(&self) -> StreamConfig {
        self.config
    }

    // Queue a transport command; Err leaves the caller's UI state unchanged
    pub fn send_transport(&mut self, command: TransportCommand) -> Result<(), ControlError> {
        self.sender.send_transport(command)
    }

    // Queue one note event, stamped with frame_offset 0 and the next sequence. This is a
    // convenience, not an ordering guarantee: two events in one block whose ranks decrease are
    // refused by the plan and counted as a plan error
    pub fn send_note(&mut self, kind: NoteEventKind) -> Result<(), ControlError> {
        let event = NoteEvent {
            frame_offset: 0,
            sequence: self.next_sequence,
            kind,
        };
        self.next_sequence += 1;
        self.sender.send_note(event)
    }

    // Play the audition voice: Play command, then one held note-on. Transport first, and the
    // note is not attempted if the transport send is refused
    pub fn start_audition(&mut self) -> Result<(), AuditionError> {
        self.send_transport(TransportCommand::Play)
            .map_err(AuditionError::Transport)?;
        // With clips attached the project's own material REPLACES the audition note rather than
        // merging with it. Merging two producers on one node would put the all-notes-off before
        // the note-on by rank and leave a note sounding after Stop — a stuck note where R4-1's
        // evidence pins a clean refusal
        if self.has_clips {
            return Ok(());
        }
        self.send_note(NoteEventKind::On {
            id: AUDITION_VOICE_ID,
            channel: AUDITION_CHANNEL,
            note: AUDITION_NOTE,
            velocity: AUDITION_VELOCITY,
        })
        .map_err(AuditionError::Note)
    }

    // Stop it: all-notes-off, then Stop. The note goes first so the release lands on the same
    // block, and Stop is sent even if the release was refused — a refused release plus a
    // stopped transport is recoverable, a running transport the UI thinks is stopped is not
    pub fn stop_audition(&mut self) -> Result<(), AuditionError> {
        // The release is still sent when clips are playing: the clip player releases its own
        // sounding notes on Stop, and an all-notes-off on the lane costs nothing and closes the
        // case where a live note was played into the same node
        let note = self.send_note(NoteEventKind::AllNotesOff {
            channel: Some(AUDITION_CHANNEL),
        });
        self.send_transport(TransportCommand::Stop)
            .map_err(AuditionError::Transport)?;
        note.map_err(AuditionError::Note)
    }

    // Drop retired render state on the app thread; called once per UI frame
    pub fn reclaim(&mut self) -> usize {
        self.sender.reclaim()
    }

    // Release the device; the engine is unusable afterward
    pub fn close(&mut self) -> Result<(), BackendError> {
        self.stream.close()
    }
}

// Apply a Shape edit to the model and publish the accepted value to the live lane.
//
// A free function rather than inline in main.rs so it can be tested against a real AppModel:
// main.rs is a binary target and its types are unreachable from crates/spectre-app/tests/.
//
// Generic over the stream rather than taking `Option<&LiveEngine>` as the spec declared. The
// declared form cannot be called with the concrete `LiveEngine<NullStream>` a test holds —
// `Option<&LiveEngine>` defaults to `LiveEngine<dyn AudioStream>`, and since `stream: Box<S>`
// the two have different layouts, so the reference does not coerce. The app still passes the
// erased type; only the tests need the concrete one.
// Which lane target, if any, a flat-device-list edit addresses on the selected track.
//
// The app's Build/Shape list is flat and its ObjectIds are the model's own; the track engine's
// lane targets are ObjectIds the graph builder allocated. The two spaces are disjoint, so an edit
// published under a model ID addressed a target that was never registered and every Shape edit
// reported "stored but did not reach live audio". This binds the two by ROLE on the selected
// track instead: the flat list's instrument row drives that track's instrument, its gloam row
// that track's insert, its gain row that track's gain.
//
// This is deliberately NOT R4-4 §8 Q2's device-browser re-parenting, which stays R6 work. Build
// and Shape still present the same flat list and no surface moves; only the destination of an
// edit is resolved. A device with no counterpart on the track -- `saturator`, which no track
// hosts -- returns None and its edit stays model-only, which is the honest answer rather than a
// silent misroute onto some other node.
fn selected_track_target_index(
    tracks: &spectre_project::TrackList,
    selected: Option<spectre_core::ObjectId>,
    device_key: &str,
    parameter_key: &str,
) -> Option<usize> {
    use spectre_project::{TrackEffect, TrackInstrument};

    let index = tracks.index_of(selected?)?;
    let track = tracks.tracks().get(index)?;

    let instrument_key = match track.instrument() {
        TrackInstrument::Pulse => "pulse",
        TrackInstrument::Filament => "filament",
    };
    if device_key == instrument_key {
        // Each instrument's own first descriptor is its level; naming the index rather than the
        // key literal keeps this correct if a descriptor is renamed
        let level = match track.instrument() {
            TrackInstrument::Pulse => PULSE_PARAMETERS[0].key,
            TrackInstrument::Filament => spectre_dsp::FILAMENT_PARAMETERS[0].key,
        };
        return (parameter_key == level.as_str()).then(|| tracks.instrument_target_index(index));
    }
    if device_key == "gloam"
        && matches!(
            track.insert().map(|slot| slot.effect()),
            Some(TrackEffect::Gloam)
        )
        && parameter_key == GLOAM_PARAMETERS[GLOAM_DEPTH].key.as_str()
    {
        return tracks.insert_target_index(index);
    }
    if device_key == "gain" && parameter_key == GAIN_PARAMETERS[0].key.as_str() {
        return Some(tracks.gain_target_index(index));
    }
    None
}

pub fn apply_parameter_edit<S: AudioStream + ?Sized>(
    model: &mut AppModel,
    engine: Option<&LiveEngine<S>>,
    device_key: &str,
    parameter_key: &str,
    value: f32,
    feedback_status: &mut String,
) {
    // The model is the value of record and the single clamping site; the lane only ever carries
    // a copy of what the model already accepted
    let edit = match model.edit_device_parameter(device_key, parameter_key, value) {
        Ok(edit) => edit,
        Err(error) => {
            *feedback_status =
                format!("Could not set {device_key}.{parameter_key}: {error}. Nothing changed.");
            return;
        }
    };
    let Some(engine) = engine else {
        return;
    };
    // Resolve the live destination by role on the selected track when one is registered there,
    // falling back to the model's own identity for the fixture engine, whose targets ARE the
    // model's IDs. Without the first branch nothing a user drags reaches the render thread
    let target = selected_track_target_index(
        model.track_list(),
        model.selected_track_id(),
        device_key,
        parameter_key,
    )
    .and_then(|index| engine.targets().get(index).copied())
    .unwrap_or(ParameterTarget {
        device: edit.device_instance_id,
        parameter: edit.parameter_instance_id,
    });
    if let Err(error) = engine.send_parameter(target, edit.value) {
        // The model keeps the edit: it is the value of record, and offline rendering will use it.
        // Only the live copy failed to publish, and the message says exactly that
        *feedback_status = format!(
            "{device_key}.{parameter_key} was stored but did not reach live audio: {error}"
        );
    }
}

// Apply the transport binding rule: send first, mutate second.
//
// Lives here rather than in main.rs because main.rs is a binary target and its functions are
// unreachable from crates/spectre-app/tests/. R4-1's own implementation review found that the
// test written against the in-main.rs version could not fail — it asserted on a model the rule
// had never touched — so the rule moved to where a falsifiable test can reach it.
//
// Returns what happened, so a caller that is not the UI can assert on it.
pub fn toggle_transport<S: AudioStream + ?Sized>(
    model: &mut AppModel,
    engine: Option<&mut LiveEngine<S>>,
    feedback_status: &mut String,
) -> Result<(), AuditionError> {
    let playing = model.is_playing();
    let Some(engine) = engine else {
        // With no engine the prototype behaves exactly as it did before R4-1
        model.toggle_play();
        return Ok(());
    };
    let result = if playing {
        engine.stop_audition()
    } else {
        engine.start_audition()
    };
    match result {
        Ok(()) => {
            model.toggle_play();
            Ok(())
        }
        // The transport command was queued and the render thread will apply it, so the UI flips
        // anyway; a queued command cannot be recalled from a wait-free lane, and a UI reading
        // "stopped" while the render transport plays is the worse of the two divergences
        Err(AuditionError::Note(error)) => {
            model.toggle_play();
            *feedback_status = format!("Transport changed but the note was dropped: {error}");
            Err(AuditionError::Note(error))
        }
        // No transport command was queued, so the UI must not change either
        Err(AuditionError::Transport(error)) => {
            *feedback_status =
                format!("Transport change refused: {error}. Nothing changed; try again.");
            Err(AuditionError::Transport(error))
        }
    }
}

// Render the engine's state as one stable field for the headless smoke line.
//
// Derived from the engine the caller actually holds, never a literal: if engine startup is ever
// wired into the headless path, this value changes and the smoke assertion fails. A hard-coded
// string here would make that assertion unfalsifiable, which is what R4-1's review found.
pub fn engine_status_field<S: AudioStream + ?Sized>(
    engine: Option<&LiveEngine<S>>,
) -> &'static str {
    match engine.map(LiveEngine::state) {
        None => "not-started",
        Some(EngineState::Opened { .. }) => "opened",
        Some(EngineState::Running { .. }) => "running",
    }
}

// Build the render-side halves from a track list rather than the fixed fixture snapshot.
//
// Sits beside build_engine_parts; neither is a second render path — both produce a CompiledPlan
// executed by the one RenderBridge, and both hash identically to what the offline harness
// renders from the same input.
pub fn build_track_engine_parts(
    tracks: &spectre_project::TrackList,
    tempo: &spectre_core::TempoMap,
    seed: u64,
    note_track: Option<spectre_core::ObjectId>,
    config: StreamConfig,
) -> Result<EngineParts, EngineUnavailable> {
    config.validate().map_err(EngineUnavailable::Backend)?;

    let mut ids = IdGen::new(seed);
    let (graph, nodes) =
        spectre_project::build_track_graph(tracks, &mut ids).map_err(EngineUnavailable::Routing)?;
    let plan_max_frames = config.buffer_frames * ENGINE_PLAN_FRAME_MARGIN;
    let mut factory = spectre_project::track_device_factory(tracks, &nodes);
    let plan = graph
        .compile(nodes.master.node, plan_max_frames, &mut factory)
        .map_err(plan_error)?;

    // Every automatable value on the path — each track's instrument level and gain, then the
    // master — is registered as a lane target, so a mixer edit is addressable the moment it happens
    let mut targets = Vec::new();
    let mut routes = Vec::new();
    let pairs = nodes.parameter_targets();
    for ((device, parameter), route_node) in pairs.iter().zip(parameter_route_nodes(&nodes)) {
        let target = ParameterTarget {
            device: *device,
            parameter: *parameter,
        };
        targets.push(target);
        routes.push(ParameterRoute {
            target,
            node: route_node.0,
            key: route_node.1,
        });
    }
    let routes = ParameterRoutes::new(routes).map_err(EngineUnavailable::Route)?;

    // The primary note node keeps the lane's live ingress. When the selected track is not a
    // note node, the master takes the role so the lane still has a valid destination
    let primary_index = note_track.and_then(|id| tracks.index_of(id)).unwrap_or(0);
    let note_node = nodes.note_node(primary_index).unwrap_or(nodes.master.node);

    let (sender, receiver) =
        control_channel(&targets, DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY)
            .map_err(EngineUnavailable::Control)?;
    let mut bridge = RenderBridge::with_parameter_routes(
        plan,
        receiver,
        note_node,
        f64::from(config.sample_rate),
        DEFAULT_NOTE_SCRATCH,
        routes,
    );

    // Clip playback, baked on the app thread before the stream opens. A project with no clip
    // placements attaches no player at all, so an empty project behaves exactly as it did
    // before this change — which is what keeps R4-1's Play/Stop refusal evidence valid
    let rate =
        spectre_core::SampleRate::new(config.sample_rate).ok_or(EngineUnavailable::Routing(
            spectre_project::RoutingError::Device("sample rate must be nonzero"),
        ))?;
    let mut has_clips = false;
    if let Some(primary) = bake_track_schedule(tracks, primary_index, tempo, rate) {
        has_clips = true;
        let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
        let _ = player.install(Box::new(primary));
        bridge = bridge.with_clip_player(player);
    }
    let voices: Vec<_> = (0..tracks.len())
        .filter(|index| *index != primary_index)
        .filter_map(|index| {
            let schedule = bake_track_schedule(tracks, index, tempo, rate)?;
            let node = nodes.note_node(index)?;
            let mut player = ClipPlayer::new(CLIP_EVENT_RESERVE);
            let _ = player.install(Box::new(schedule));
            Some((node, player))
        })
        .collect();
    if !voices.is_empty() {
        has_clips = true;
        bridge = bridge.with_clip_voices(voices, DEFAULT_NOTE_SCRATCH);
    }

    let telemetry = bridge.telemetry();

    Ok(EngineParts {
        bridge,
        sender,
        telemetry,
        config,
        plan_max_frames,
        targets: targets.into_boxed_slice(),
        nodes: None,
        has_clips,
    })
}

// Bake one track's active clip placements into absolute sample positions, or None when the
// track carries no clip material. The conversion goes through TempoMap::ticks_to_samples and
// nowhere else, which is what keeps one time conversion in the workspace
fn bake_track_schedule(
    tracks: &spectre_project::TrackList,
    index: usize,
    tempo: &spectre_core::TempoMap,
    rate: spectre_core::SampleRate,
) -> Option<ClipSchedule> {
    let track = tracks.tracks().get(index)?;
    let mut notes = Vec::new();
    for placement in track.clips().placements() {
        if !placement.is_active() {
            continue;
        }
        let clip = tracks.clip(placement.clip())?;
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
    if notes.is_empty() {
        return None;
    }
    ClipSchedule::bake(notes.into_iter(), tempo, rate, CLIP_EVENT_RESERVE).ok()
}

// Pair each parameter target with the plan node and key that applies it, in the same order
// TrackPathNodes::parameter_targets emits them: per track, instrument level then track gain,
// then the master gain
fn parameter_route_nodes(
    nodes: &spectre_project::TrackPathNodes,
) -> Vec<(NodeId, spectre_dsp::DeviceParameterKey)> {
    let mut routes = Vec::with_capacity(nodes.instruments.len() * 3 + 1);
    for ((instrument, chain), gain) in nodes
        .instruments
        .iter()
        .zip(&nodes.inserts)
        .zip(&nodes.track_gains)
    {
        routes.push((instrument.node, PULSE_PARAMETERS[0].key));
        // Without this an effect's depth reaches no live node, which is exactly what
        // r4-qa-protocol.md row 3 drags. Emitted in parameter_targets' order, one per chained
        // effect in signal order, which this function must match pair for pair
        for insert in chain.iter() {
            routes.push((insert.node, GLOAM_PARAMETERS[GLOAM_DEPTH].key));
        }
        routes.push((gain.node, GAIN_PARAMETERS[0].key));
    }
    routes.push((nodes.master.node, GAIN_PARAMETERS[0].key));
    routes
}

// Format a playhead as bars and beats, one-based, the way a musician counts.
//
// R4-5 §3.1 specifies this readout as derived from MeterMap::signature_at and TempoMap, and it
// lives here rather than in main.rs so it is reachable from a test -- main.rs is a binary target
// and R4-1's review already found one rule that could not be tested because it lived there.
//
// Returns None for no position, which the caller draws as the same "—" it drew before any
// position existed. A negative position is pre-roll and counts backwards from bar 1.
pub fn bars_beats(
    position: Option<i64>,
    tempo: &spectre_core::TempoMap,
    meter: &spectre_core::MeterMap,
    rate: spectre_core::SampleRate,
) -> Option<String> {
    let samples = position?;
    let ticks = tempo.samples_to_ticks(spectre_core::SampleTime(samples), rate);
    let signature = meter.signature_at(ticks);
    let per_bar = signature.ticks_per_bar().0;
    let per_beat = spectre_core::TICKS_PER_BEAT;
    // div_euclid, not `/`: a pre-roll tick is negative, and truncating division would place
    // -1 tick in bar 1 alongside +1 tick instead of in the bar before it
    let bar = ticks.0.div_euclid(per_bar);
    let within = ticks.0.rem_euclid(per_bar);
    let beat = within.div_euclid(per_beat);
    let remainder = within.rem_euclid(per_beat);
    // Sixteenths of a beat, so the readout moves visibly without implying sample precision it
    // does not have -- the position is published once per block, not once per sample
    let sixteenths = (remainder * 4) / per_beat;
    Some(format!("{}.{}.{}", bar + 1, beat + 1, sixteenths + 1))
}

// Node identity seed for the app's track graph. Rationale: identities must be stable across a
// rebuild so the same list produces the same graph; the value itself carries no meaning.
//
// Defined here rather than in main.rs so the hardware drill opens the engine with the SAME seed
// the binary does. It lived in main.rs until 2026-08-28, where no test could reach it
pub const APP_GRAPH_SEED: u64 = 0x0053_5045_4354_5245;

// Open and start the default output device running a track list.
//
// The seed fixes node identity, so the same list rebuilds to the same graph
pub fn open_track_engine(
    backend: &dyn AudioBackend,
    tracks: &spectre_project::TrackList,
    tempo: &spectre_core::TempoMap,
    seed: u64,
    note_track: Option<spectre_core::ObjectId>,
) -> Result<LiveEngine, EngineUnavailable> {
    let device = backend
        .default_output_device()
        .map_err(EngineUnavailable::Backend)?;
    let sample_rate = backend
        .default_sample_rate(&device.id)
        .map_err(EngineUnavailable::Backend)?;
    let config = StreamConfig::stereo(sample_rate, ENGINE_BUFFER_FRAMES)
        .map_err(EngineUnavailable::Backend)?;
    let parts = build_track_engine_parts(tracks, tempo, seed, note_track, config)?;
    open_with_parts(backend, &device.id, device.name, parts)
}

// Publish every effective gain a mixer edit invalidated.
//
// The publication set is part of the contract, not an implementation detail: effective_gain
// depends on any_soloed(), so ONE solo edit changes the value for EVERY track. Publishing only
// the edited id would produce a solo that silences nothing. Over-publishing is free — the lane
// is one preallocated latest-wins slot per target, with no capacity to exhaust — so when in
// doubt, republish.
pub fn publish_track_gains<S: AudioStream + ?Sized>(
    engine: &LiveEngine<S>,
    tracks: &spectre_project::TrackList,
    invalidated: &[spectre_core::ObjectId],
) -> Result<(), ControlError> {
    for id in invalidated {
        let Some(index) = tracks.index_of(*id) else {
            continue;
        };
        let Some(gain) = tracks.effective_gain(*id) else {
            continue;
        };
        let Some(target) = engine.targets().get(tracks.gain_target_index(index)) else {
            continue;
        };
        engine.send_parameter(*target, gain)?;
    }
    Ok(())
}

// Publish the master fader position
pub fn publish_master_gain<S: AudioStream + ?Sized>(
    engine: &LiveEngine<S>,
    tracks: &spectre_project::TrackList,
) -> Result<(), ControlError> {
    let Some(target) = engine.targets().get(tracks.master_target_index()) else {
        return Ok(());
    };
    engine.send_parameter(*target, tracks.master_level())
}
