// Author: Jeff
// Date: 2026-07-12
// Description: Borrowed planar-buffer and bounded-event DSP process contract
// Notes: Validation allocates nothing and process callers own all storage

// Native device processing role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Source,
    Instrument,
    Effect,
}

// Immutable compiled-plan I/O declaration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceIo {
    pub class: DeviceClass,
    pub audio_inputs: usize,
    pub audio_outputs: usize,
    pub accepts_notes: bool,
}

// One bounded note event
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteEvent {
    pub frame_offset: usize,
    pub sequence: u64,
    pub kind: NoteEventKind,
}

// V1 instrument event payload
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteEventKind {
    On {
        id: u32,
        channel: u8,
        note: u8,
        velocity: f32,
    },
    Off {
        id: u32,
        channel: u8,
        note: u8,
        velocity: f32,
    },
    AllNotesOff {
        channel: Option<u8>,
    },
}

// Engine-owned event storage limit for one device bus and render quantum
pub const MAX_NOTE_EVENTS_PER_BLOCK: usize = 1_024;

// Callback-safe validation failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    InvalidSampleRate,
    InvalidFrameCount,
    InvalidEvent,
    EventCapacity,
    UnsortedEvents,
    InputCount,
    OutputCount,
    BufferLength,
}

// Validated render-quantum metadata and borrowed events
#[derive(Debug, Clone, Copy)]
pub struct ProcessContext<'a> {
    sample_rate: f64,
    frames: usize,
    events: &'a [NoteEvent],
}

impl<'a> ProcessContext<'a> {
    // Validate one process quantum outside device arithmetic
    pub fn new(
        sample_rate: f64,
        frames: usize,
        events: &'a [NoteEvent],
    ) -> Result<Self, ProcessError> {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return Err(ProcessError::InvalidSampleRate);
        }
        if frames == 0 {
            return Err(ProcessError::InvalidFrameCount);
        }
        if events.len() > MAX_NOTE_EVENTS_PER_BLOCK {
            return Err(ProcessError::EventCapacity);
        }
        let mut previous = None;
        for event in events {
            if event.frame_offset >= frames || !event.is_valid() {
                return Err(ProcessError::InvalidEvent);
            }
            let order = (event.frame_offset, event.kind.rank(), event.sequence);
            if previous.is_some_and(|before| order <= before) {
                return Err(ProcessError::UnsortedEvents);
            }
            previous = Some(order);
        }
        Ok(Self {
            sample_rate,
            frames,
            events,
        })
    }

    pub fn sample_rate(self) -> f64 {
        self.sample_rate
    }

    pub fn frames(self) -> usize {
        self.frames
    }

    pub fn events(self) -> &'a [NoteEvent] {
        self.events
    }
}

impl NoteEvent {
    fn is_valid(self) -> bool {
        match self.kind {
            NoteEventKind::On {
                id,
                channel,
                note,
                velocity,
            }
            | NoteEventKind::Off {
                id,
                channel,
                note,
                velocity,
            } => {
                id != 0
                    && channel <= 15
                    && note <= 127
                    && velocity.is_finite()
                    && (0.0..=1.0).contains(&velocity)
            }
            NoteEventKind::AllNotesOff { channel } => channel.is_none_or(|value| value <= 15),
        }
    }
}

impl NoteEventKind {
    fn rank(self) -> u8 {
        match self {
            Self::Off { .. } | Self::AllNotesOff { .. } => 0,
            Self::On { .. } => 1,
        }
    }
}

// Allocation-free native device process seam
pub trait AudioProcessor {
    fn io(&self) -> DeviceIo;

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError>;
}

// Validate flattened planar buffers against the compiled layout
pub(crate) fn validate_buffers(
    io: DeviceIo,
    context: &ProcessContext<'_>,
    inputs: &[&[f32]],
    outputs: &[&mut [f32]],
) -> Result<(), ProcessError> {
    if inputs.len() != io.audio_inputs {
        return Err(ProcessError::InputCount);
    }
    if outputs.len() != io.audio_outputs {
        return Err(ProcessError::OutputCount);
    }
    if inputs.iter().any(|buffer| buffer.len() != context.frames())
        || outputs
            .iter()
            .any(|buffer| buffer.len() != context.frames())
    {
        return Err(ProcessError::BufferLength);
    }
    if !io.accepts_notes && !context.events().is_empty() {
        return Err(ProcessError::InvalidEvent);
    }
    Ok(())
}
