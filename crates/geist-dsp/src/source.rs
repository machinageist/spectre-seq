// Author: Jeff
// Date: 2026-07-12
// Description: Deterministic native audio sources and note-driven instrument
// Notes: Phase accumulation uses f64; process paths allocate nothing

use crate::io::{
    validate_buffers, AudioProcessor, DeviceClass, DeviceIo, NoteEventKind, ProcessContext,
    ProcessError,
};
use crate::parameter::{parameter, DspParameter};
use geist_core::ParamUnit;
use std::f64::consts::TAU;

const SOURCE_IO: DeviceIo = DeviceIo {
    class: DeviceClass::Source,
    audio_inputs: 0,
    audio_outputs: 2,
    accepts_notes: false,
};
const INSTRUMENT_IO: DeviceIo = DeviceIo {
    class: DeviceClass::Instrument,
    audio_inputs: 0,
    audio_outputs: 2,
    accepts_notes: true,
};

pub const TONE_PARAMETERS: [DspParameter; 2] = [
    parameter(
        "frequency_hz",
        "Frequency",
        ParamUnit::Hertz,
        20.0,
        20_000.0,
        440.0,
    ),
    parameter("level", "Level", ParamUnit::Percent, 0.0, 1.0, 0.25),
];

pub const PULSE_PARAMETERS: [DspParameter; 1] = [parameter(
    "level",
    "Level",
    ParamUnit::Percent,
    0.0,
    1.0,
    0.2,
)];

// Deterministic stereo sine fixture source
#[derive(Debug, Clone)]
pub struct ToneSource {
    frequency: f32,
    level: f32,
    phase: f64,
}

impl ToneSource {
    pub fn new(frequency: f32, level: f32) -> Result<Self, &'static str> {
        let frequency = TONE_PARAMETERS[0]
            .validate(frequency)
            .map_err(|_| "frequency must be finite and within 20..=20000 Hz")?;
        let level = TONE_PARAMETERS[1]
            .validate(level)
            .map_err(|_| "level must be finite and normalized")?;
        Ok(Self {
            frequency,
            level,
            phase: 0.0,
        })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &TONE_PARAMETERS
    }
}

impl AudioProcessor for ToneSource {
    fn io(&self) -> DeviceIo {
        SOURCE_IO
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        validate_buffers(self.io(), context, inputs, outputs)?;
        let step = self.frequency as f64 / context.sample_rate();
        let (left_slice, right_slice) = outputs.split_at_mut(1);
        let left = &mut left_slice[0];
        let right = &mut right_slice[0];
        for frame in 0..context.frames() {
            let sample = (self.phase * TAU).sin() as f32 * self.level;
            left[frame] = sample;
            right[frame] = sample;
            self.phase = (self.phase + step).fract();
        }
        Ok(())
    }
}

// Original oscillator waveform family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Triangle,
    Saw,
    Square,
}

// Monophonic note-driven instrument for the first vertical slice
#[derive(Debug, Clone)]
pub struct PulseInstrument {
    waveform: Waveform,
    level: f32,
    phase: f64,
    active_note: Option<(u32, u8)>,
    velocity: f32,
}

impl PulseInstrument {
    pub fn new(waveform: Waveform, level: f32) -> Result<Self, &'static str> {
        let level = PULSE_PARAMETERS[0]
            .validate(level)
            .map_err(|_| "level must be finite and normalized")?;
        Ok(Self {
            waveform,
            level,
            phase: 0.0,
            active_note: None,
            velocity: 0.0,
        })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &PULSE_PARAMETERS
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    fn sample(&self) -> f32 {
        let phase = self.phase as f32;
        match self.waveform {
            Waveform::Sine => (self.phase * TAU).sin() as f32,
            Waveform::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
            Waveform::Saw => phase * 2.0 - 1.0,
            Waveform::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }
}

impl AudioProcessor for PulseInstrument {
    fn io(&self) -> DeviceIo {
        INSTRUMENT_IO
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        validate_buffers(self.io(), context, inputs, outputs)?;
        let (left_slice, right_slice) = outputs.split_at_mut(1);
        let left = &mut left_slice[0];
        let right = &mut right_slice[0];
        let mut event_index = 0;
        for frame in 0..context.frames() {
            while event_index < context.events().len()
                && context.events()[event_index].frame_offset == frame
            {
                match context.events()[event_index].kind {
                    NoteEventKind::On {
                        id, note, velocity, ..
                    } => {
                        self.active_note = Some((id, note));
                        self.velocity = velocity;
                        self.phase = 0.0;
                    }
                    NoteEventKind::Off { id, .. }
                        if self.active_note.is_some_and(|active| active.0 == id) =>
                    {
                        self.active_note = None;
                        self.velocity = 0.0;
                    }
                    NoteEventKind::AllNotesOff { .. } => {
                        self.active_note = None;
                        self.velocity = 0.0;
                    }
                    NoteEventKind::Off { .. } => {}
                }
                event_index += 1;
            }
            let sample = if let Some((_, note)) = self.active_note {
                let value = self.sample() * self.level * self.velocity;
                let frequency = 440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0);
                self.phase = (self.phase + frequency / context.sample_rate()).fract();
                value
            } else {
                0.0
            };
            left[frame] = sample;
            right[frame] = sample;
        }
        Ok(())
    }
}
