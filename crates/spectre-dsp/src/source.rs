// Author: Jeff
// Date: 2026-07-12
// Description: Deterministic native audio sources and note-driven instrument
// Notes: Phase accumulation uses f64; process paths allocate nothing

use crate::io::{
    validate_buffers, AudioProcessor, DeviceClass, DeviceIo, NoteEventKind, ParameterError,
    ProcessContext, ProcessError, MAX_VOICES,
};
use crate::parameter::{parameter, DeviceParameterKey, DspParameter};
use spectre_core::ParamUnit;
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

    // Changing frequency mid-stream deliberately does not reset `phase`, so a frequency change
    // is phase-continuous; resetting it would click
    fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError> {
        if key == TONE_PARAMETERS[0].key {
            self.frequency = TONE_PARAMETERS[0].clamp(value);
            return Ok(());
        }
        if key == TONE_PARAMETERS[1].key {
            self.level = TONE_PARAMETERS[1].clamp(value);
            return Ok(());
        }
        Err(ParameterError::UnknownKey(key))
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

// One sounding note. Pulse has no amplitude contour, so a released voice is immediately free --
// unlike Filament, whose voice stays busy through its fall ramp
#[derive(Debug, Clone, Copy)]
struct PulseVoice {
    active: Option<(u32, u8)>,
    phase: f64,
    velocity: f32,
    // Allocation order, used only to choose which voice to steal. A counter rather than a
    // measured amplitude, so the same input always steals the same voice
    started: u64,
}

impl PulseVoice {
    const fn silent() -> Self {
        Self {
            active: None,
            phase: 0.0,
            velocity: 0.0,
            started: 0,
        }
    }
}

// Polyphonic note-driven instrument for the first vertical slice.
//
// Its own pool and its own stealing policy, per the accepted per-instrument voicing decision;
// it shares only the MAX_VOICES count, which is one musical argument rather than two
#[derive(Debug, Clone)]
pub struct PulseInstrument {
    waveform: Waveform,
    level: f32,
    voices: [PulseVoice; MAX_VOICES],
    next_started: u64,
}

impl PulseInstrument {
    pub fn new(waveform: Waveform, level: f32) -> Result<Self, &'static str> {
        let level = PULSE_PARAMETERS[0]
            .validate(level)
            .map_err(|_| "level must be finite and normalized")?;
        Ok(Self {
            waveform,
            level,
            voices: [PulseVoice::silent(); MAX_VOICES],
            next_started: 0,
        })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &PULSE_PARAMETERS
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    // Choose the voice a new note takes: a free one, else the oldest sounding one. Scans the
    // fixed pool and allocates nothing
    fn allocate(&self) -> usize {
        let mut oldest = 0;
        for (index, voice) in self.voices.iter().enumerate() {
            if voice.active.is_none() {
                return index;
            }
            if voice.started < self.voices[oldest].started {
                oldest = index;
            }
        }
        oldest
    }

    fn sample_at(&self, voice_phase: f64) -> f32 {
        let phase = voice_phase as f32;
        match self.waveform {
            Waveform::Sine => (voice_phase * TAU).sin() as f32,
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

    // Level applies at the next block boundary; the phase and the active note are untouched, so
    // a level change during a held note does not restart it
    fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError> {
        if key == PULSE_PARAMETERS[0].key {
            self.level = PULSE_PARAMETERS[0].clamp(value);
            return Ok(());
        }
        Err(ParameterError::UnknownKey(key))
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
                        let slot = self.allocate();
                        let started = self.next_started;
                        self.next_started = self.next_started.wrapping_add(1);
                        self.voices[slot] = PulseVoice {
                            active: Some((id, note)),
                            phase: 0.0,
                            velocity,
                            started,
                        };
                    }
                    // Every voice holding this id, so a duplicate id cannot strand one sounding
                    NoteEventKind::Off { id, .. } => {
                        for voice in self.voices.iter_mut() {
                            if voice.active.is_some_and(|active| active.0 == id) {
                                voice.active = None;
                                voice.velocity = 0.0;
                            }
                        }
                    }
                    NoteEventKind::AllNotesOff { .. } => {
                        for voice in self.voices.iter_mut() {
                            voice.active = None;
                            voice.velocity = 0.0;
                        }
                    }
                }
                event_index += 1;
            }
            // Voices sum in pool order, which is fixed for the life of the device. A single
            // sounding voice sums into a zero accumulator, which is exact, so one note renders
            // bit-identically to the monophonic device this replaced
            let mut sample = 0.0;
            for index in 0..MAX_VOICES {
                let Some((_, note)) = self.voices[index].active else {
                    continue;
                };
                let voice_phase = self.voices[index].phase;
                sample += self.sample_at(voice_phase) * self.level * self.voices[index].velocity;
                let frequency = 440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0);
                self.voices[index].phase =
                    (voice_phase + frequency / context.sample_rate()).fract();
            }
            left[frame] = sample;
            right[frame] = sample;
        }
        Ok(())
    }
}
