// Author: Jeff
// Date: 2026-08-24
// Description: Filament — monophonic phase-warped voice behind a linear amplitude contour
// Notes: One voice, one oscillator, no modulation. Phase accumulates in f64; every other value
//   is f32. Nothing here allocates, locks, formats, logs, or panics. Numeric bounds are
//   DEV-001..DEV-005 and DEV-010 in docs/01-requirements/requirements-ledger.md

use crate::io::{
    validate_buffers, AudioProcessor, DeviceClass, DeviceIo, NoteEventKind, ParameterError,
    ProcessContext, ProcessError,
};
use crate::parameter::{parameter, DeviceParameterKey, DspParameter};
use spectre_core::ParamUnit;
use std::f64::consts::TAU;

// Instrument layout fixed by the v1 device contract, not chosen by this device
const FILAMENT_IO: DeviceIo = DeviceIo {
    class: DeviceClass::Instrument,
    audio_inputs: 0,
    audio_outputs: 2,
    accepts_notes: true,
};

// Descriptor slots; one definition each, shared by the constructor, the setter, and the reader
const LEAN: usize = 0;
const RISE_MS: usize = 1;
const FALL_MS: usize = 2;
const LEVEL: usize = 3;

pub const FILAMENT_PARAMETERS: [DspParameter; 4] = [
    parameter("lean", "Lean", ParamUnit::Linear, 0.0, 1.0, 0.5),
    parameter(
        "rise_ms",
        "Rise",
        ParamUnit::Milliseconds,
        1.0,
        1_000.0,
        31.6,
    ),
    parameter(
        "fall_ms",
        "Fall",
        ParamUnit::Milliseconds,
        1.0,
        1_000.0,
        31.6,
    ),
    parameter("level", "Level", ParamUnit::Percent, 0.0, 1.0, 0.2),
];

// Milliseconds-to-seconds conversion; a unit factor, not a bound
const SECONDS_PER_MILLISECOND: f64 = 0.001;
// Contour endpoints; both are reached exactly, which is what makes silence exact silence
const CONTOUR_FLOOR: f32 = 0.0;
const CONTOUR_CEILING: f32 = 1.0;
// Degenerate-rate guard: a contour spanning fewer than one sample period still steps once
const MIN_CONTOUR_SAMPLES: f64 = 1.0;
// Phase-map breakpoint image; lean always maps to the half-cycle point
const PHASE_MAP_MIDPOINT: f64 = 0.5;
// Normalized phase and lean domains are both [0, 1]
const NORMALIZED_CEILING: f64 = 1.0;
// Equal temperament, mirroring the mapping the shipped instrument already uses
const REFERENCE_PITCH_HZ: f64 = 440.0;
const REFERENCE_MIDI_NOTE: f64 = 69.0;
const SEMITONES_PER_OCTAVE: f64 = 12.0;
const OCTAVE_RATIO: f64 = 2.0;

// Monophonic note-driven voice: one warped oscillator behind one linear amplitude contour
#[derive(Debug, Clone)]
pub struct Filament {
    lean: f32,
    rise_ms: f32,
    fall_ms: f32,
    level: f32,
    // Normalized oscillator phase in [0, 1); f64 for the same reason ToneSource uses it
    phase: f64,
    // Active note identity and number, matching the note-ID contract the event slice carries
    active: Option<(u32, u8)>,
    // Held across release so the fall ramp keeps the note's own loudness
    velocity: f32,
    // Equal-tempered frequency of the last note-on; survives release for the same reason
    note_hz: f64,
    // Amplitude contour in [0, 1]; reaches its target exactly, so silence is exact
    contour: f32,
}

impl Filament {
    // Validate against the device's own descriptors, exactly as the shipped constructors do
    pub fn new(lean: f32, rise_ms: f32, fall_ms: f32, level: f32) -> Result<Self, &'static str> {
        let lean = FILAMENT_PARAMETERS[LEAN]
            .validate(lean)
            .map_err(|_| "lean must be finite and normalized")?;
        let rise_ms = FILAMENT_PARAMETERS[RISE_MS]
            .validate(rise_ms)
            .map_err(|_| "rise must be finite and within 1..=1000 ms")?;
        let fall_ms = FILAMENT_PARAMETERS[FALL_MS]
            .validate(fall_ms)
            .map_err(|_| "fall must be finite and within 1..=1000 ms")?;
        let level = FILAMENT_PARAMETERS[LEVEL]
            .validate(level)
            .map_err(|_| "level must be finite and normalized")?;
        Ok(Self {
            lean,
            rise_ms,
            fall_ms,
            level,
            phase: 0.0,
            active: None,
            velocity: 0.0,
            note_hz: 0.0,
            contour: CONTOUR_FLOOR,
        })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &FILAMENT_PARAMETERS
    }

    // Read one applied value back; the symmetric half of the setter, and callback-safe too
    pub fn parameter_value(&self, key: DeviceParameterKey) -> Option<f32> {
        if key == FILAMENT_PARAMETERS[LEAN].key {
            Some(self.lean)
        } else if key == FILAMENT_PARAMETERS[RISE_MS].key {
            Some(self.rise_ms)
        } else if key == FILAMENT_PARAMETERS[FALL_MS].key {
            Some(self.fall_ms)
        } else if key == FILAMENT_PARAMETERS[LEVEL].key {
            Some(self.level)
        } else {
            None
        }
    }

    // Apply one note event to the single voice; note-off releases without silencing
    fn apply_event(&mut self, kind: NoteEventKind) {
        match kind {
            NoteEventKind::On {
                id, note, velocity, ..
            } => {
                self.active = Some((id, note));
                self.velocity = velocity;
                self.note_hz = note_frequency_hz(note);
                self.phase = 0.0;
            }
            NoteEventKind::Off { id, .. } if self.active.is_some_and(|active| active.0 == id) => {
                self.active = None;
            }
            NoteEventKind::AllNotesOff { .. } => {
                self.active = None;
            }
            NoteEventKind::Off { .. } => {}
        }
    }
}

// Map a MIDI note number to its equal-tempered frequency
fn note_frequency_hz(note: u8) -> f64 {
    REFERENCE_PITCH_HZ
        * OCTAVE_RATIO.powf((f64::from(note) - REFERENCE_MIDI_NOTE) / SEMITONES_PER_OCTAVE)
}

// Compute one contour increment per quantum; total for every rate the context admits
fn contour_step(time_ms: f32, sample_rate: f64) -> f32 {
    let samples =
        (f64::from(time_ms) * SECONDS_PER_MILLISECOND * sample_rate).max(MIN_CONTOUR_SAMPLES);
    (1.0 / samples) as f32
}

// Reparameterize the phase ramp around lean, then read one sine at the warped position.
// The strict `<` and the `else` are load-bearing: at lean = 0 only the second branch runs and
// its denominator is 1, at lean = 1 only the first runs and its denominator is 1, so neither
// degenerate divisor is reachable for any phase in [0, 1)
fn warped(phase: f64, lean: f64) -> f64 {
    let mapped = if phase < lean {
        PHASE_MAP_MIDPOINT * phase / lean
    } else {
        PHASE_MAP_MIDPOINT + PHASE_MAP_MIDPOINT * (phase - lean) / (NORMALIZED_CEILING - lean)
    };
    (mapped * TAU).sin()
}

impl AudioProcessor for Filament {
    fn io(&self) -> DeviceIo {
        FILAMENT_IO
    }

    // Apply one already-validated value to a live device; assignment and one clamp, nothing else
    fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError> {
        if key == FILAMENT_PARAMETERS[LEAN].key {
            self.lean = FILAMENT_PARAMETERS[LEAN].clamp(value);
        } else if key == FILAMENT_PARAMETERS[RISE_MS].key {
            self.rise_ms = FILAMENT_PARAMETERS[RISE_MS].clamp(value);
        } else if key == FILAMENT_PARAMETERS[FALL_MS].key {
            self.fall_ms = FILAMENT_PARAMETERS[FALL_MS].clamp(value);
        } else if key == FILAMENT_PARAMETERS[LEVEL].key {
            self.level = FILAMENT_PARAMETERS[LEVEL].clamp(value);
        } else {
            return Err(ParameterError::UnknownKey(key));
        }
        Ok(())
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        validate_buffers(self.io(), context, inputs, outputs)?;
        let rise_step = contour_step(self.rise_ms, context.sample_rate());
        let fall_step = contour_step(self.fall_ms, context.sample_rate());
        let lean = f64::from(self.lean);
        let (left_slice, right_slice) = outputs.split_at_mut(1);
        let left = &mut left_slice[0];
        let right = &mut right_slice[0];
        let mut event_index = 0;
        for frame in 0..context.frames() {
            while event_index < context.events().len()
                && context.events()[event_index].frame_offset == frame
            {
                self.apply_event(context.events()[event_index].kind);
                event_index += 1;
            }
            self.contour = if self.active.is_some() {
                (self.contour + rise_step).min(CONTOUR_CEILING)
            } else {
                (self.contour - fall_step).max(CONTOUR_FLOOR)
            };
            let sample = if self.active.is_none() && self.contour == CONTOUR_FLOOR {
                // Exact silence, and the phase stays where it is rather than decaying forever
                CONTOUR_FLOOR
            } else {
                let value =
                    warped(self.phase, lean) as f32 * self.contour * self.velocity * self.level;
                // Contain the accumulator rather than the output: an unbounded rate would make
                // the increment infinite, and a non-finite phase would poison every later block
                let advanced = self.phase + self.note_hz / context.sample_rate();
                self.phase = if advanced.is_finite() {
                    advanced.fract()
                } else {
                    0.0
                };
                value
            };
            left[frame] = sample;
            right[frame] = sample;
        }
        Ok(())
    }
}
