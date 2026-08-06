// Author: Jeff
// Date: 2026-07-12
// Description: Backend-owned native-device parameter metadata
// Notes: Project-instance identity is an ObjectId; this module's keys are stable per device type

use geist_core::{ObjectId, ParamError, ParamSpec, ParamUnit};

// R2 evidence-corpus ceiling; endpoint neighborhoods retain the plain-domain bound
pub const NORMALIZED_ROUND_TRIP_MAX_ULPS: u32 = 8_192;

// Stable parameter identity within a device type, distinct from project-instance ObjectId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceParameterKey(&'static str);

impl DeviceParameterKey {
    pub const fn new(value: &'static str) -> Option<Self> {
        if value.is_empty() {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

// Static UI metadata backed by geist-core's validated numerical contract
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DspParameter {
    pub key: DeviceParameterKey,
    pub name: &'static str,
    pub spec: ParamSpec,
}

impl DspParameter {
    pub const fn new(key: DeviceParameterKey, name: &'static str, spec: ParamSpec) -> Self {
        Self { key, name, spec }
    }

    pub fn validate(self, value: f32) -> Result<f32, ParamError> {
        self.spec
            .validate(f64::from(value))
            .map(|value| value.plain() as f32)
    }

    // Contain app-thread input before publishing a prevalidated f32 snapshot
    pub fn clamp(self, value: f32) -> f32 {
        self.spec.clamp(f64::from(value)).plain() as f32
    }

    pub fn to_normalized(self, value: f32) -> f32 {
        let value = self.spec.clamp(f64::from(value));
        self.spec.to_normalized(value) as f32
    }

    pub fn from_normalized(self, normalized: f32) -> f32 {
        self.spec.from_normalized(f64::from(normalized)).plain() as f32
    }

    pub const fn unit(self) -> ParamUnit {
        self.spec.unit()
    }

    pub const fn minimum(self) -> f32 {
        self.spec.minimum() as f32
    }

    pub const fn maximum(self) -> f32 {
        self.spec.maximum() as f32
    }

    pub const fn default(self) -> f32 {
        self.spec.default_value() as f32
    }
}

// Owned renderer-neutral parameter value contained by one canonical DSP descriptor
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceParameterSnapshot {
    device_instance_id: ObjectId,
    device_key: &'static str,
    parameter_instance_id: ObjectId,
    parameter_key: DeviceParameterKey,
    value: f32,
}

impl DeviceParameterSnapshot {
    // Contain app-thread input while preserving canonical device and parameter identity
    pub fn new(
        device_instance_id: ObjectId,
        device_key: &'static str,
        parameter_instance_id: ObjectId,
        parameter: DspParameter,
        value: f32,
    ) -> Self {
        Self {
            device_instance_id,
            device_key,
            parameter_instance_id,
            parameter_key: parameter.key,
            value: parameter.clamp(value),
        }
    }

    pub const fn device_instance_id(&self) -> ObjectId {
        self.device_instance_id
    }

    pub const fn device_key(&self) -> &'static str {
        self.device_key
    }

    pub const fn parameter_key(&self) -> DeviceParameterKey {
        self.parameter_key
    }

    pub const fn parameter_instance_id(&self) -> ObjectId {
        self.parameter_instance_id
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

pub(crate) const fn parameter(
    key: &'static str,
    name: &'static str,
    unit: ParamUnit,
    minimum: f32,
    maximum: f32,
    default: f32,
) -> DspParameter {
    let key = match DeviceParameterKey::new(key) {
        Some(key) => key,
        None => panic!("device parameter key must not be empty"),
    };
    let spec = match ParamSpec::new(unit, minimum as f64, maximum as f64, default as f64) {
        Ok(spec) => spec,
        Err(_) => panic!("device parameter specification must be valid"),
    };
    DspParameter::new(key, name, spec)
}
