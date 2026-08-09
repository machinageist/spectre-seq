// Author: Jeff
// Date: 2026-07-11
// Description: Parameter descriptors with stable identity, range, default, and normalized mapping (CORE-002)
// Notes: Linear mapping only in R1; curve/taper variants arrive with the device model

use crate::id::ObjectId;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

// Display unit for a parameter value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamUnit {
    Linear,
    Decibels,
    Hertz,
    Milliseconds,
    Percent,
    Semitones,
}

// Validated range, default, unit, and normalized mapping shared by every parameter surface
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ParamSpec {
    unit: ParamUnit,
    min: f64,
    max: f64,
    default: f64,
}

// Immutable description of one project-instance parameter
#[derive(Debug, Clone, PartialEq)]
pub struct ParamDescriptor {
    instance_id: ObjectId,
    name: String,
    spec: ParamSpec,
}

// A validated plain value for a specific parameter specification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParamValue(f64);

// Parameter construction and plain-value validation failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamError {
    InvalidInstanceId,
    InvalidRange,
    DefaultOutOfRange,
    ValueOutOfRange,
    NonFiniteValue,
}

impl std::fmt::Display for ParamError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInstanceId => "parameter instance ID must be nonzero",
            Self::InvalidRange => "parameter maximum must be greater than its minimum",
            Self::DefaultOutOfRange => "parameter default is outside its range",
            Self::ValueOutOfRange => "parameter value is outside its range",
            Self::NonFiniteValue => "parameter values must be finite",
        })
    }
}

impl std::error::Error for ParamError {}

impl ParamSpec {
    // Build the single validated numerical contract used by core, DSP, and app layers
    pub const fn new(
        unit: ParamUnit,
        min: f64,
        max: f64,
        default: f64,
    ) -> Result<Self, ParamError> {
        if !min.is_finite() || !max.is_finite() || !default.is_finite() {
            return Err(ParamError::NonFiniteValue);
        }
        if max <= min {
            return Err(ParamError::InvalidRange);
        }
        if default < min || default > max {
            return Err(ParamError::DefaultOutOfRange);
        }
        Ok(Self {
            unit,
            min,
            max,
            default,
        })
    }

    pub const fn unit(self) -> ParamUnit {
        self.unit
    }

    pub const fn minimum(self) -> f64 {
        self.min
    }

    pub const fn maximum(self) -> f64 {
        self.max
    }

    pub const fn default_value(self) -> f64 {
        self.default
    }

    // Validate without silently changing a caller-supplied plain value
    pub fn validate(self, plain: f64) -> Result<ParamValue, ParamError> {
        if !plain.is_finite() {
            return Err(ParamError::NonFiniteValue);
        }
        if !(self.min..=self.max).contains(&plain) {
            return Err(ParamError::ValueOutOfRange);
        }
        Ok(ParamValue(plain))
    }

    // Clamp a plain value into range, mapping non-finite input to the default
    pub fn clamp(self, plain: f64) -> ParamValue {
        if !plain.is_finite() {
            return ParamValue(self.default);
        }
        ParamValue(plain.clamp(self.min, self.max))
    }

    // Convert a value from this specification to normalized [0, 1]
    pub fn to_normalized(self, value: ParamValue) -> f64 {
        (value.0 - self.min) / (self.max - self.min)
    }

    // Convert normalized [0, 1] to a clamped plain value
    pub fn from_normalized(self, normalized: f64) -> ParamValue {
        if !normalized.is_finite() {
            return ParamValue(self.default);
        }
        let normalized = normalized.clamp(0.0, 1.0);
        ParamValue(self.min + normalized * (self.max - self.min))
    }
}

impl<'de> Deserialize<'de> for ParamSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EncodedSpec {
            unit: ParamUnit,
            min: f64,
            max: f64,
            default: f64,
        }

        let encoded = EncodedSpec::deserialize(deserializer)?;
        Self::new(encoded.unit, encoded.min, encoded.max, encoded.default).map_err(D::Error::custom)
    }
}

impl ParamDescriptor {
    // Build a validated project-instance descriptor
    pub fn new(
        instance_id: ObjectId,
        name: impl Into<String>,
        unit: ParamUnit,
        min: f64,
        max: f64,
        default: f64,
    ) -> Result<Self, ParamError> {
        if instance_id.raw() == 0 {
            return Err(ParamError::InvalidInstanceId);
        }
        Ok(Self {
            instance_id,
            name: name.into(),
            spec: ParamSpec::new(unit, min, max, default)?,
        })
    }

    pub const fn instance_id(&self) -> ObjectId {
        self.instance_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn spec(&self) -> ParamSpec {
        self.spec
    }

    pub fn clamp(&self, plain: f64) -> ParamValue {
        self.spec.clamp(plain)
    }

    pub fn to_normalized(&self, value: ParamValue) -> f64 {
        self.spec.to_normalized(value)
    }

    pub fn from_normalized(&self, normalized: f64) -> ParamValue {
        self.spec.from_normalized(normalized)
    }
}

// Preserve the R1 flattened descriptor representation while validating on decode
impl Serialize for ParamDescriptor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct EncodedDescriptor<'a> {
            id: ObjectId,
            name: &'a str,
            unit: ParamUnit,
            min: f64,
            max: f64,
            default: f64,
        }

        EncodedDescriptor {
            id: self.instance_id,
            name: &self.name,
            unit: self.spec.unit(),
            min: self.spec.minimum(),
            max: self.spec.maximum(),
            default: self.spec.default_value(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ParamDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EncodedDescriptor {
            id: ObjectId,
            name: String,
            unit: ParamUnit,
            min: f64,
            max: f64,
            default: f64,
        }

        let encoded = EncodedDescriptor::deserialize(deserializer)?;
        Self::new(
            encoded.id,
            encoded.name,
            encoded.unit,
            encoded.min,
            encoded.max,
            encoded.default,
        )
        .map_err(D::Error::custom)
    }
}

impl ParamValue {
    // Expose the plain value
    pub fn plain(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::IdGen;

    // Helper descriptor spanning -60..+6 dB
    fn gain() -> ParamDescriptor {
        let mut ids = IdGen::new(1);
        ParamDescriptor::new(ids.next_id(), "Gain", ParamUnit::Decibels, -60.0, 6.0, 0.0).unwrap()
    }

    // Normalized round trip preserves plain values within float tolerance
    #[test]
    fn normalized_round_trip() {
        let d = gain();
        for v in [-60.0, -18.0, 0.0, 6.0] {
            let plain = d.clamp(v);
            let back = d.from_normalized(d.to_normalized(plain));
            assert!((back.plain() - plain.plain()).abs() < 1e-12);
        }
    }

    // Non-finite input degrades to the default, never propagates
    #[test]
    fn non_finite_maps_to_default() {
        let d = gain();
        assert_eq!(d.clamp(f64::NAN).plain(), 0.0);
        assert_eq!(d.from_normalized(f64::INFINITY).plain(), 0.0);
    }

    // Construction validates range and default
    #[test]
    fn construction_is_validated() {
        let mut ids = IdGen::new(2);
        assert_eq!(
            ParamDescriptor::new(ids.next_id(), "x", ParamUnit::Linear, 1.0, 1.0, 1.0).unwrap_err(),
            ParamError::InvalidRange
        );
        assert_eq!(
            ParamDescriptor::new(ids.next_id(), "x", ParamUnit::Linear, 0.0, 1.0, 2.0).unwrap_err(),
            ParamError::DefaultOutOfRange
        );
    }

    // Persistence must not bypass the same invariants as construction
    #[test]
    fn descriptor_deserialization_rejects_invalid_range() {
        let encoded = r#"{
            "id": 7,
            "name": "Broken",
            "unit": "Linear",
            "min": 1.0,
            "max": 1.0,
            "default": 1.0
        }"#;

        assert!(serde_json::from_str::<ParamDescriptor>(encoded).is_err());
    }

    #[test]
    fn descriptor_deserialization_rejects_out_of_range_default() {
        let encoded = r#"{
            "id": 8,
            "name": "Broken",
            "unit": "Percent",
            "min": 0.0,
            "max": 1.0,
            "default": 2.0
        }"#;

        assert!(serde_json::from_str::<ParamDescriptor>(encoded).is_err());
    }

    #[test]
    fn descriptor_deserialization_rejects_zero_instance_id() {
        let encoded = r#"{
            "id": 0,
            "name": "Broken",
            "unit": "Percent",
            "min": 0.0,
            "max": 1.0,
            "default": 0.5
        }"#;

        assert!(serde_json::from_str::<ParamDescriptor>(encoded).is_err());
    }

    #[test]
    fn parameter_spec_owns_validation_clamping_and_normalization() {
        let spec = ParamSpec::new(ParamUnit::Percent, 0.0, 100.0, 25.0).unwrap();

        assert_eq!(spec.validate(80.0).unwrap().plain(), 80.0);
        assert_eq!(spec.validate(101.0), Err(ParamError::ValueOutOfRange));
        assert_eq!(spec.clamp(f64::NAN).plain(), 25.0);
        assert_eq!(spec.clamp(120.0).plain(), 100.0);
        assert_eq!(spec.to_normalized(spec.clamp(25.0)), 0.25);
        assert_eq!(spec.from_normalized(0.75).plain(), 75.0);
    }

    #[test]
    fn descriptor_exposes_instance_identity_separately_from_its_spec() {
        let descriptor = gain();

        assert_ne!(descriptor.instance_id().raw(), 0);
        assert_eq!(descriptor.name(), "Gain");
        assert_eq!(descriptor.spec().unit(), ParamUnit::Decibels);
        assert_eq!(descriptor.spec().minimum(), -60.0);
    }

    #[test]
    fn parameter_spec_deserialization_is_validated() {
        let invalid = r#"{
            "unit": "Linear",
            "min": 2.0,
            "max": 1.0,
            "default": 1.5
        }"#;

        assert!(serde_json::from_str::<ParamSpec>(invalid).is_err());
    }
}
