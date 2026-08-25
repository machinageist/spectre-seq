// Author: Jeff
// Date: 2026-08-24
// Description: Gloam — stereo one-pole damping whose corner opens with the signal's own level
// Notes: Recursive state, so non-finite input is contained before the state update rather than
//   after it; a latched NaN would poison every later block. Coefficients are computed once per
//   quantum, never per sample. Numeric bounds are DEV-006..DEV-009 and DEV-011..DEV-012 in
//   docs/01-requirements/requirements-ledger.md

use crate::io::{
    validate_buffers, AudioProcessor, DeviceClass, DeviceIo, ParameterError, ProcessContext,
    ProcessError,
};
use crate::parameter::{parameter, DeviceParameterKey, DspParameter};
use spectre_core::ParamUnit;
use std::f64::consts::TAU;

// Insert-effect layout fixed by the v1 device contract; restated here rather than shared, so
// adding a device does not refactor effect.rs
const GLOAM_IO: DeviceIo = DeviceIo {
    class: DeviceClass::Effect,
    audio_inputs: 2,
    audio_outputs: 2,
    accepts_notes: false,
};

// Descriptor slots; one definition each, shared by the constructor, the setter, and the reader
const DAMP_HZ: usize = 0;
const DEPTH: usize = 1;
const TRACK_MS: usize = 2;

pub const GLOAM_PARAMETERS: [DspParameter; 3] = [
    parameter("damp_hz", "Damp", ParamUnit::Hertz, 20.0, 20_000.0, 632.5),
    parameter("depth", "Depth", ParamUnit::Percent, 0.0, 1.0, 0.0),
    parameter(
        "track_ms",
        "Track",
        ParamUnit::Milliseconds,
        1.0,
        1_000.0,
        31.6,
    ),
];

// Stereo layout, from GLOAM_IO rather than restated per loop
const CHANNELS: usize = GLOAM_IO.audio_outputs;
// Milliseconds-to-seconds conversion; a unit factor, not a bound
const SECONDS_PER_MILLISECOND: f64 = 0.001;
// Coefficient domain; a one-pole in [0, 1] is a convex combination and cannot ring
const COEFFICIENT_FLOOR: f32 = 0.0;
const COEFFICIENT_CEILING: f32 = 1.0;
// Follower time constant expressed as one exponential lifetime
const ONE_TIME_CONSTANT: f64 = 1.0;

// Stereo level-tracked damping; two scalars of state per channel and nothing else
#[derive(Debug, Clone)]
pub struct Gloam {
    damp_hz: f32,
    depth: f32,
    track_ms: f32,
    // Per-channel magnitude follower output in [0, max |x|]
    follower: [f32; CHANNELS],
    // Per-channel one-pole state; a convex combination of past inputs, so |state| <= max |x|
    damped: [f32; CHANNELS],
}

impl Gloam {
    // Validate against the device's own descriptors, exactly as the shipped constructors do
    pub fn new(damp_hz: f32, depth: f32, track_ms: f32) -> Result<Self, &'static str> {
        let damp_hz = GLOAM_PARAMETERS[DAMP_HZ]
            .validate(damp_hz)
            .map_err(|_| "damp must be finite and within 20..=20000 Hz")?;
        let depth = GLOAM_PARAMETERS[DEPTH]
            .validate(depth)
            .map_err(|_| "depth must be finite and normalized")?;
        let track_ms = GLOAM_PARAMETERS[TRACK_MS]
            .validate(track_ms)
            .map_err(|_| "track must be finite and within 1..=1000 ms")?;
        Ok(Self {
            damp_hz,
            depth,
            track_ms,
            follower: [0.0; CHANNELS],
            damped: [0.0; CHANNELS],
        })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &GLOAM_PARAMETERS
    }

    // Read one applied value back; the symmetric half of the setter, and callback-safe too
    pub fn parameter_value(&self, key: DeviceParameterKey) -> Option<f32> {
        if key == GLOAM_PARAMETERS[DAMP_HZ].key {
            Some(self.damp_hz)
        } else if key == GLOAM_PARAMETERS[DEPTH].key {
            Some(self.depth)
        } else if key == GLOAM_PARAMETERS[TRACK_MS].key {
            Some(self.track_ms)
        } else {
            None
        }
    }

    // Settle recursive state once per quantum: contain non-finite, then flush denormals.
    // The plan's RT-003 pass sees output buffers and never this state, so an unflushed tail
    // would sit in denormal range on the callback path indefinitely after a signal stops
    fn settle_state(&mut self) {
        for value in self.follower.iter_mut().chain(self.damped.iter_mut()) {
            if !value.is_finite() {
                *value = 0.0;
            } else if *value != 0.0 && value.abs() < f32::MIN_POSITIVE {
                // FTZ-equivalent: magnitude collapses, sign survives
                *value = if value.is_sign_negative() { -0.0 } else { 0.0 };
            }
        }
    }
}

// Compute the one-pole corner coefficient; total for every rate the context admits.
// An enormous ratio underflows exp to 0 and yields exactly 1 (transparent), a vanishing ratio
// yields exactly 0 (frozen); neither is NaN and both stay inside [0, 1]
fn corner_coefficient(corner_hz: f32, sample_rate: f64) -> f32 {
    (1.0 - (-TAU * f64::from(corner_hz) / sample_rate).exp()) as f32
}

// Compute the magnitude-follower coefficient over one exponential lifetime; total for the
// same reasons, and bounded to [0, 1] by the same argument
fn follower_coefficient(time_ms: f32, sample_rate: f64) -> f32 {
    let samples = f64::from(time_ms) * SECONDS_PER_MILLISECOND * sample_rate;
    (1.0 - (-ONE_TIME_CONSTANT / samples).exp()) as f32
}

impl AudioProcessor for Gloam {
    fn io(&self) -> DeviceIo {
        GLOAM_IO
    }

    // Apply one already-validated value to a live device; assignment and one clamp, nothing else
    fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError> {
        if key == GLOAM_PARAMETERS[DAMP_HZ].key {
            self.damp_hz = GLOAM_PARAMETERS[DAMP_HZ].clamp(value);
        } else if key == GLOAM_PARAMETERS[DEPTH].key {
            self.depth = GLOAM_PARAMETERS[DEPTH].clamp(value);
        } else if key == GLOAM_PARAMETERS[TRACK_MS].key {
            self.track_ms = GLOAM_PARAMETERS[TRACK_MS].clamp(value);
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
        let damp_a = corner_coefficient(self.damp_hz, context.sample_rate());
        let track_k = follower_coefficient(self.track_ms, context.sample_rate());
        for channel in 0..CHANNELS {
            for frame in 0..context.frames() {
                // Containment precedes the state update; the two shipped effects are stateless
                // and can afford to contain afterwards, a recursion cannot
                let input = inputs[channel][frame];
                let sample = if input.is_finite() { input } else { 0.0 };
                self.follower[channel] += track_k * (sample.abs() - self.follower[channel]);
                // The clamp is mandatory: a follower above 1 would push the coefficient past 1,
                // which is the one way this filter could be made unstable
                let opening = (self.depth * self.follower[channel])
                    .clamp(COEFFICIENT_FLOOR, COEFFICIENT_CEILING);
                let coefficient = damp_a + (COEFFICIENT_CEILING - damp_a) * opening;
                self.damped[channel] += coefficient * (sample - self.damped[channel]);
                if !self.damped[channel].is_finite() {
                    self.damped[channel] = 0.0;
                }
                outputs[channel][frame] = self.damped[channel];
            }
        }
        self.settle_state();
        Ok(())
    }
}
