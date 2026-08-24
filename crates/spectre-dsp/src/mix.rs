// Author: Jeff
// Date: 2026-08-24
// Description: Stereo summing bus with a declared number of stereo input buses
// Notes: Accumulates in f64 per decision 6 and rounds once to f32 at store. Bus order is
//   track order and is part of the contract, because float addition is not associative.

use crate::io::{
    validate_buffers, AudioProcessor, DeviceClass, DeviceIo, ParameterError, ProcessContext,
    ProcessError,
};
use crate::parameter::DeviceParameterKey;

// Maximum stereo input buses one summing device may declare
// Rationale row in docs/01-requirements/requirements-ledger.md; equals MAX_TRACKS and is the
// same bound expressed in the DSP layer. Kept as a separate const so a layout error is caught
// at device construction with a device-level message, before graph validation
pub const MAX_SUM_BUSES: usize = 16;

// Stereo summing bus; zero buses is legal and renders exact silence
#[derive(Debug, Clone)]
pub struct SumBus {
    buses: usize,
}

impl SumBus {
    // Reject a bus count beyond MAX_SUM_BUSES; zero is accepted
    pub fn new(buses: usize) -> Result<Self, &'static str> {
        if buses > MAX_SUM_BUSES {
            return Err("summing bus count exceeds MAX_SUM_BUSES");
        }
        Ok(Self { buses })
    }

    // Report the declared stereo input bus count
    pub fn buses(&self) -> usize {
        self.buses
    }
}

impl AudioProcessor for SumBus {
    // Declare one stereo output fed by `buses` stereo inputs
    fn io(&self) -> DeviceIo {
        DeviceIo {
            class: DeviceClass::Effect,
            audio_inputs: self.buses * 2,
            audio_outputs: 2,
            accepts_notes: false,
        }
    }

    // The bus has no automatable parameters; its width is structural, set at construction
    fn set_parameter(
        &mut self,
        key: DeviceParameterKey,
        _value: f32,
    ) -> Result<(), ParameterError> {
        Err(ParameterError::UnknownKey(key))
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
                // f64 accumulation buys exactly one thing: the intermediate rounding of an
                // N-term f32 sum no longer compounds, because there is one rounding, at the
                // store. It does not make the sum order-independent — f64 addition is not
                // associative either. Determinism comes from the bus order being fixed
                let mut sum = 0.0_f64;
                for bus in 0..self.buses {
                    let sample = inputs[bus * 2 + channel][frame];
                    // Contain non-finite input at this device's own boundary, the way Gain and
                    // Saturator already do, rather than relying on downstream containment
                    if sample.is_finite() {
                        sum += f64::from(sample);
                    }
                }
                outputs[channel][frame] = sum as f32;
            }
        }
        Ok(())
    }
}
