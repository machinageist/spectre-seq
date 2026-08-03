// Author: Jeff
// Date: 2026-05-27
// Description: Shared error and result types for recoverable app-thread failures.
// Notes: Audio-thread code reports status through preallocated counters/events, not rich errors.

use crate::ids::PortId;
use crate::port::PortType;
use std::fmt;

// Recoverable failure reported on the app thread
// All payloads are Copy and allocation-free so returns stay cheap
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SpectreError {
    // Referenced identifier does not exist in its owning collection
    InvalidId(&'static str),
    // Referenced port is unknown to the graph
    InvalidPort(PortId),
    // Connection joins incompatible port signal types
    TypeMismatch { expected: PortType, found: PortType },
    // Connection joins two inputs or two outputs
    DirectionMismatch,
    // Connection joins ports with differing channel counts
    ChannelMismatch { out_channels: u16, in_channels: u16 },
    // Audio configuration failed validation
    BadConfig(&'static str),
    // Requested audio backend is unavailable on this platform
    UnsupportedBackend(&'static str),
    // Hosted plugin reported a failure
    PluginError(&'static str),
    // Project serialization or deserialization failed
    Serialization(&'static str),
    // Internal invariant was violated; indicates a bug
    Internal(&'static str),
}

// Standard result alias for app-thread fallible operations
pub type SpectreResult<T> = Result<T, SpectreError>;

impl fmt::Display for SpectreError {
    // Render a human-readable diagnostic for logs and UI
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpectreError::InvalidId(what) => write!(f, "invalid id: {what}"),
            SpectreError::InvalidPort(id) => write!(f, "invalid port: {id:?}"),
            SpectreError::TypeMismatch { expected, found } => {
                write!(
                    f,
                    "port type mismatch: expected {expected:?}, found {found:?}"
                )
            }
            SpectreError::DirectionMismatch => {
                write!(
                    f,
                    "port direction mismatch: connect one output to one input"
                )
            }
            SpectreError::ChannelMismatch {
                out_channels,
                in_channels,
            } => {
                write!(
                    f,
                    "channel mismatch: output has {out_channels}, input has {in_channels}"
                )
            }
            SpectreError::BadConfig(why) => write!(f, "bad audio config: {why}"),
            SpectreError::UnsupportedBackend(name) => write!(f, "unsupported backend: {name}"),
            SpectreError::PluginError(why) => write!(f, "plugin error: {why}"),
            SpectreError::Serialization(why) => write!(f, "serialization error: {why}"),
            SpectreError::Internal(why) => write!(f, "internal error: {why}"),
        }
    }
}

impl std::error::Error for SpectreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_non_empty_for_each_variant() {
        let cases = [
            SpectreError::InvalidId("node"),
            SpectreError::InvalidPort(PortId::new(1)),
            SpectreError::TypeMismatch {
                expected: PortType::Audio,
                found: PortType::Note,
            },
            SpectreError::DirectionMismatch,
            SpectreError::ChannelMismatch {
                out_channels: 1,
                in_channels: 2,
            },
            SpectreError::BadConfig("zero sample rate"),
            SpectreError::UnsupportedBackend("jack"),
            SpectreError::PluginError("init failed"),
            SpectreError::Serialization("truncated"),
            SpectreError::Internal("unreachable"),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn error_is_copy_and_comparable() {
        let err = SpectreError::DirectionMismatch;
        let copy = err;
        assert_eq!(err, copy);
    }
}
