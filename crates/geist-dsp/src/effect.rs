// Author: Jeff
// Date: 2026-07-12
// Description: Native stereo gain and soft-saturation effects
// Notes: Effects contain non-finite input and fully overwrite output buffers

use crate::io::{
    validate_buffers, AudioProcessor, DeviceClass, DeviceIo, ProcessContext, ProcessError,
};
use crate::parameter::{parameter, DspParameter};
use geist_core::ParamUnit;

const EFFECT_IO: DeviceIo = DeviceIo {
    class: DeviceClass::Effect,
    audio_inputs: 2,
    audio_outputs: 2,
    accepts_notes: false,
};

pub const GAIN_PARAMETERS: [DspParameter; 1] =
    [parameter("gain", "Gain", ParamUnit::Linear, 0.0, 2.0, 1.0)];

pub const SATURATOR_PARAMETERS: [DspParameter; 2] = [
    parameter("drive", "Drive", ParamUnit::Linear, 1.0, 24.0, 1.0),
    parameter("mix", "Mix", ParamUnit::Percent, 0.0, 1.0, 1.0),
];

// Stereo gain with callback-ready target state
#[derive(Debug, Clone)]
pub struct Gain {
    gain: f32,
}

impl Gain {
    pub fn new(gain: f32) -> Result<Self, &'static str> {
        let gain = GAIN_PARAMETERS[0]
            .validate(gain)
            .map_err(|_| "gain must be finite and within 0..=2")?;
        Ok(Self { gain })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &GAIN_PARAMETERS
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = GAIN_PARAMETERS[0].clamp(gain);
    }
}

impl AudioProcessor for Gain {
    fn io(&self) -> DeviceIo {
        EFFECT_IO
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        validate_buffers(self.io(), context, inputs, outputs)?;
        for channel in 0..2 {
            for frame in 0..context.frames() {
                let input = inputs[channel][frame];
                outputs[channel][frame] = if input.is_finite() {
                    input * self.gain
                } else {
                    0.0
                };
            }
        }
        Ok(())
    }
}

// Stereo normalized soft clip with parallel dry/wet blend
#[derive(Debug, Clone)]
pub struct Saturator {
    drive: f32,
    mix: f32,
}

impl Saturator {
    pub fn new(drive: f32, mix: f32) -> Result<Self, &'static str> {
        let drive = SATURATOR_PARAMETERS[0]
            .validate(drive)
            .map_err(|_| "drive must be finite and within 1..=24")?;
        let mix = SATURATOR_PARAMETERS[1]
            .validate(mix)
            .map_err(|_| "mix must be finite and normalized")?;
        Ok(Self { drive, mix })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &SATURATOR_PARAMETERS
    }

    pub fn set_drive(&mut self, drive: f32) {
        self.drive = SATURATOR_PARAMETERS[0].clamp(drive);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = SATURATOR_PARAMETERS[1].clamp(mix);
    }
}

impl AudioProcessor for Saturator {
    fn io(&self) -> DeviceIo {
        EFFECT_IO
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        validate_buffers(self.io(), context, inputs, outputs)?;
        let normalization = self.drive.tanh();
        for channel in 0..2 {
            for frame in 0..context.frames() {
                let dry = if inputs[channel][frame].is_finite() {
                    inputs[channel][frame].clamp(-1.0, 1.0)
                } else {
                    0.0
                };
                let wet = (dry * self.drive).tanh() / normalization;
                outputs[channel][frame] = dry + (wet - dry) * self.mix;
            }
        }
        Ok(())
    }
}
