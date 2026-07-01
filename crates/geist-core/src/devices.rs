// Author: Jeff
// Date: 2026-06-30
// Description: Internal device descriptors and state envelopes.
// Notes: VST-hosted devices adapt into this model; native devices do not depend on VST.

use crate::ids::DeviceId;
use crate::params::ParamInfo;

// Origin/class of a device after it has entered the internal DAW model.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum DeviceKind {
    NativeSynth,
    NativeEffect,
    NativeMidiTool,
    NativeModulator,
    NativeUtility,
    HostedVst,
}

impl DeviceKind {
    // True when the device is first-party internal code.
    #[inline]
    pub const fn is_native(self) -> bool {
        !matches!(self, Self::HostedVst)
    }

    // True when the device wraps a third-party VST instance.
    #[inline]
    pub const fn is_hosted(self) -> bool {
        matches!(self, Self::HostedVst)
    }
}

// Static or scan-derived description of one internal device surface.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DeviceDescriptor<'a> {
    pub id: DeviceId,
    pub kind: DeviceKind,
    pub stable_key: &'a str,
    pub name: &'a str,
    pub params: &'a [ParamInfo],
}

impl<'a> DeviceDescriptor<'a> {
    // Build a descriptor without exposing plugin-specific implementation details.
    #[inline]
    pub const fn new(
        id: DeviceId,
        kind: DeviceKind,
        stable_key: &'a str,
        name: &'a str,
        params: &'a [ParamInfo],
    ) -> Self {
        Self {
            id,
            kind,
            stable_key,
            name,
            params,
        }
    }

    // Borrow the parameter surface.
    #[inline]
    pub const fn parameters(&self) -> &'a [ParamInfo] {
        self.params
    }

    // True when the descriptor names a first-party internal device.
    #[inline]
    pub const fn is_native(&self) -> bool {
        self.kind.is_native()
    }

    // True when the descriptor names a hosted VST wrapper.
    #[inline]
    pub const fn is_hosted(&self) -> bool {
        self.kind.is_hosted()
    }
}

// Opaque state bytes for either native devices or hosted VST wrappers.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DeviceState {
    id: DeviceId,
    kind: DeviceKind,
    bytes: Vec<u8>,
}

impl DeviceState {
    // Build an empty native-utility state envelope.
    #[inline]
    pub fn empty(id: DeviceId) -> Self {
        Self {
            id,
            kind: DeviceKind::NativeUtility,
            bytes: Vec::new(),
        }
    }

    // Build an opaque state envelope from app-thread owned bytes.
    #[inline]
    pub fn from_bytes(id: DeviceId, kind: DeviceKind, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            id,
            kind,
            bytes: bytes.into(),
        }
    }

    // Return the owning device id.
    #[inline]
    pub const fn id(&self) -> DeviceId {
        self.id
    }

    // Return the device kind for persistence/routing decisions.
    #[inline]
    pub const fn kind(&self) -> DeviceKind {
        self.kind
    }

    // Borrow opaque state bytes.
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    // Consume the envelope into opaque bytes.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{DeviceId, ParamId};
    use crate::params::{ParamInfo, ParamRange};

    const GAIN: ParamInfo = ParamInfo {
        id: ParamId::new(1),
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

    #[test]
    fn descriptor_hides_native_or_hosted_origin() {
        let native = DeviceDescriptor::new(
            DeviceId::new(10),
            DeviceKind::NativeSynth,
            "geist.synth",
            "Geist Synth",
            &[GAIN],
        );
        let hosted = DeviceDescriptor::new(
            DeviceId::new(11),
            DeviceKind::HostedVst,
            "vst3.example",
            "Example VST",
            &[GAIN],
        );

        assert!(native.is_native());
        assert!(!native.is_hosted());
        assert!(hosted.is_hosted());
        assert_eq!(native.parameters()[0].id, ParamId::new(1));
    }

    #[test]
    fn device_state_keeps_native_and_hosted_blobs_opaque() {
        let empty = DeviceState::empty(DeviceId::new(1));
        let state = DeviceState::from_bytes(DeviceId::new(2), DeviceKind::HostedVst, [1, 2, 3, 4]);

        assert!(empty.bytes().is_empty());
        assert_eq!(state.kind(), DeviceKind::HostedVst);
        assert_eq!(state.bytes(), &[1, 2, 3, 4]);
    }
}
