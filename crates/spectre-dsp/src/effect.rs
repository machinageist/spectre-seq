// Author: Jeff
// Date: 2026-07-12
// Description: Native stereo gain and soft-saturation effects
// Notes: Effects contain non-finite input and fully overwrite output buffers

use crate::io::{
    validate_buffers, AudioProcessor, DeviceClass, DeviceIo, ParameterError, ProcessContext,
    ProcessError,
};
use crate::parameter::{parameter, DeviceParameterKey, DspParameter};
use spectre_core::ParamUnit;

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
    // The value currently multiplying samples. Equal to `target` except during the one block
    // that follows a parameter change
    gain: f32,
    target: f32,
}

impl Gain {
    pub fn new(gain: f32) -> Result<Self, &'static str> {
        let gain = GAIN_PARAMETERS[0]
            .validate(gain)
            .map_err(|_| "gain must be finite and within 0..=2")?;
        Ok(Self { gain, target: gain })
    }

    pub fn parameters(&self) -> &'static [DspParameter] {
        &GAIN_PARAMETERS
    }

    // Aim at a new gain. The next block ramps to it; the current value is left alone, which is
    // what makes the change inaudible as a click rather than a step
    pub fn set_gain(&mut self, gain: f32) {
        self.target = GAIN_PARAMETERS[0].clamp(gain);
    }
}

impl AudioProcessor for Gain {
    fn io(&self) -> DeviceIo {
        EFFECT_IO
    }

    // Compare against the device's own const descriptor table so the setter cannot drift from
    // the descriptors the UI and the offline snapshot both read
    fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError> {
        if key == GAIN_PARAMETERS[0].key {
            self.set_gain(value);
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
        // Ramp across exactly one block, so a parameter change is fully applied by the time the
        // next block begins -- the same boundary the accepted seam already applies parameters on.
        // The ramp length is the block, so this introduces NO new numeric bound; that was one of
        // R4-2's two stated reasons for leaving the contract's smoothing clause unimplemented.
        //
        // The other was that smoothing would change rendered output for identical inputs. It does
        // not: `new` sets target == gain, so a render with no parameter change takes the delta==0
        // path below and is bit-identical to the unsmoothed device. Only a block that follows an
        // actual edit differs, which is the entire point.
        let frames = context.frames();
        let start = self.gain;
        let delta = self.target - start;
        let step = if frames == 0 {
            0.0
        } else {
            delta / frames as f32
        };
        for channel in 0..2 {
            for frame in 0..context.frames() {
                let input = inputs[channel][frame];
                // Lands exactly on target at the final sample: the last term is start + delta
                let gain = if delta == 0.0 {
                    start
                } else {
                    start + step * (frame + 1) as f32
                };
                outputs[channel][frame] = if input.is_finite() { input * gain } else { 0.0 };
            }
        }
        // Settle exactly on target rather than on the accumulated sum, so a ramp cannot leave a
        // residue that makes the next block start a hair off the value the model holds
        self.gain = self.target;
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

    // Reuses the existing inherent setters, which clamp against the same const table
    fn set_parameter(&mut self, key: DeviceParameterKey, value: f32) -> Result<(), ParameterError> {
        if key == SATURATOR_PARAMETERS[0].key {
            self.set_drive(value);
            return Ok(());
        }
        if key == SATURATOR_PARAMETERS[1].key {
            self.set_mix(value);
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
