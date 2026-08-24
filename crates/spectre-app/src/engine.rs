// Author: Jeff
// Date: 2026-08-24
// Description: App-thread host for the live audio engine
// Notes: Every method here runs on the app thread and may allocate. Nothing in this file is
//   reachable from the audio callback; the only value that crosses is the RenderBridge moved
//   into the render closure at open. No DSP and no second render path are written here.

use spectre_audio::bridge::{BridgeTelemetry, RenderBridge, DEFAULT_NOTE_SCRATCH};
use spectre_audio::control::{
    control_channel, ControlError, ControlSender, DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY,
};
use spectre_audio::{AudioBackend, AudioStream, BackendError, RenderBlock, StreamConfig};
use spectre_core::{IdGen, TransportCommand};
use spectre_dsp::{
    AudioProcessor, DeviceParameterSnapshot, Gain, NoteEvent, NoteEventKind, PulseInstrument,
    Saturator, Waveform, GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS,
};
use spectre_graph::{Connection, EditableGraph, NodeId};
use std::sync::Arc;

use crate::DeviceParameterSnapshotError;

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
            Self::Control(error) => write!(formatter, "control transport: {error:?}"),
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

    // No parameter targets are registered: consuming the parameter lane is decision 22's seam
    // and lands at R4-2. The bridge continues to count observed changes as parameters_pending
    let (sender, receiver) =
        control_channel(&[], DEFAULT_NOTE_CAPACITY, DEFAULT_TRANSPORT_CAPACITY)
            .map_err(EngineUnavailable::Control)?;
    let bridge = RenderBridge::new(
        plan,
        receiver,
        pulse,
        f64::from(config.sample_rate),
        DEFAULT_NOTE_SCRATCH,
    );
    let telemetry = bridge.telemetry();

    Ok(EngineParts {
        bridge,
        sender,
        telemetry,
        config,
        plan_max_frames,
    })
}

// Open and start the default output device, consuming the parts' bridge
pub fn open_default(
    backend: &dyn AudioBackend,
    snapshot: &[DeviceParameterSnapshot],
) -> Result<LiveEngine, EngineUnavailable> {
    let device = backend
        .default_output_device()
        .map_err(EngineUnavailable::Backend)?;
    let sample_rate = backend
        .default_sample_rate(&device.id)
        .map_err(EngineUnavailable::Backend)?;
    let config = StreamConfig::stereo(sample_rate, ENGINE_BUFFER_FRAMES)
        .map_err(EngineUnavailable::Backend)?;

    // The plan is built before the device is touched, so an open failure can never leave a
    // half-built engine behind
    let parts = build_engine_parts(snapshot, config)?;
    let EngineParts {
        mut bridge,
        sender,
        telemetry,
        config,
        plan_max_frames: _,
    } = parts;

    let mut stream = backend
        .open_output(
            &device.id,
            config,
            Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),
        )
        .map_err(EngineUnavailable::Backend)?;
    stream.start().map_err(EngineUnavailable::Backend)?;

    Ok(LiveEngine::from_open_stream(
        stream,
        sender,
        telemetry,
        backend.name(),
        device.name,
        config,
    ))
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
            stream,
        }
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
