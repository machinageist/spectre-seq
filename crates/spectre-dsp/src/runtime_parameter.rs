// Author: Jeff
// Date: 2026-08-24
// Description: Callback-safe runtime parameter application failure for native devices
// Notes: Decision 22's seam refuses unknown keys as a recoverable error rather than a panic;
//   the variant carries the offending key by static reference so refusal never allocates

use crate::parameter::DeviceParameterKey;

// Recoverable failure from applying one already-validated parameter to a live device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterError {
    UnknownKey(DeviceParameterKey),
}

impl std::fmt::Display for ParameterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownKey(key) => {
                formatter.write_str("unknown device parameter key: ")?;
                formatter.write_str(key.as_str())
            }
        }
    }
}

impl std::error::Error for ParameterError {}
