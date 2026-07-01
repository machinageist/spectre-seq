// =============================================================================
// File: crates/geist-graph/src/node.rs
// Layer: audio graph
// Purpose: AudioNode trait definition
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;
use geist_core::devices::{DeviceDescriptor, DeviceState};
use geist_core::errors::GeistResult;
use geist_core::params::ParamInfo;

// Unit of audio processing in the graph
// Send so a compiled graph can move to the audio thread; never required to be Sync
// process runs on the audio thread and must not allocate, lock, or block
pub trait AudioNode: Send {
    // Process one block in place through the borrowed context
    fn process(&mut self, ctx: &mut ProcessContext);

    // Configure for a sample rate and block size before streaming starts
    // Runs on the app thread; allocation is permitted here, not in process
    fn prepare(&mut self, _config: &AudioConfig) {}

    // Clear internal state back to silence
    fn reset(&mut self) {}
}

// Common internal device surface shared by native devices and hosted wrappers.
// State/descriptor methods run on app or graph-compile threads, not the callback.
pub trait AudioDevice: AudioNode {
    // Describe this device without exposing native or VST implementation details.
    fn descriptor(&self) -> DeviceDescriptor<'_>;

    // Borrow the parameter surface used by automation, modulation, and UI.
    fn parameters(&self) -> &[ParamInfo];

    // Report fixed processing latency for graph compensation.
    #[inline]
    fn latency_samples(&self) -> u32 {
        0
    }

    // Snapshot opaque device state outside the audio callback.
    fn state(&self) -> DeviceState;

    // Restore opaque device state outside the audio callback.
    fn load_state(&mut self, state: &DeviceState) -> GeistResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::devices::{DeviceKind, DeviceState};
    use geist_core::ids::{DeviceId, ParamId};
    use geist_core::params::{ParamInfo, ParamRange};

    const GAIN: ParamInfo = ParamInfo {
        id: ParamId::new(7),
        name: "Gain",
        unit: "dB",
        range: ParamRange::Linear {
            min: -60.0,
            max: 12.0,
        },
        default: 0.0,
        automatable: true,
        modulatable: true,
    };

    struct DummyDevice {
        state: DeviceState,
    }

    impl AudioNode for DummyDevice {
        fn process(&mut self, _ctx: &mut ProcessContext) {}
    }

    impl AudioDevice for DummyDevice {
        fn descriptor(&self) -> DeviceDescriptor<'_> {
            DeviceDescriptor::new(
                DeviceId::new(1),
                DeviceKind::NativeEffect,
                "geist.test.dummy",
                "Dummy",
                &[GAIN],
            )
        }

        fn parameters(&self) -> &[ParamInfo] {
            &[GAIN]
        }

        fn state(&self) -> DeviceState {
            self.state.clone()
        }

        fn load_state(&mut self, state: &DeviceState) -> GeistResult<()> {
            self.state = state.clone();
            Ok(())
        }
    }

    #[test]
    fn audio_device_exposes_descriptor_params_latency_and_state() {
        let mut device = DummyDevice {
            state: DeviceState::empty(DeviceId::new(1)),
        };
        let next = DeviceState::from_bytes(DeviceId::new(1), DeviceKind::NativeEffect, [9, 8, 7]);

        assert_eq!(device.descriptor().kind, DeviceKind::NativeEffect);
        assert_eq!(device.parameters()[0].id, ParamId::new(7));
        assert_eq!(device.latency_samples(), 0);

        device.load_state(&next).unwrap();
        assert_eq!(device.state().bytes(), &[9, 8, 7]);
    }
}
