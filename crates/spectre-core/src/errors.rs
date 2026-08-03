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
pub enum GeistError {
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
pub type GeistResult<T> = Result<T, GeistError>;

impl fmt::Display for GeistError {
    // Render a human-readable diagnostic for logs and UI
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeistError::InvalidId(what) => write!(f, "invalid id: {what}"),
            GeistError::InvalidPort(id) => write!(f, "invalid port: {id:?}"),
            GeistError::TypeMismatch { expected, found } => {
                write!(
                    f,
                    "port type mismatch: expected {expected:?}, found {found:?}"
                )
            }
            GeistError::DirectionMismatch => {
                write!(
                    f,
                    "port direction mismatch: connect one output to one input"
                )
            }
            GeistError::ChannelMismatch {
                out_channels,
                in_channels,
            } => {
                write!(
                    f,
                    "channel mismatch: output has {out_channels}, input has {in_channels}"
                )
            }
            GeistError::BadConfig(why) => write!(f, "bad audio config: {why}"),
            GeistError::UnsupportedBackend(name) => write!(f, "unsupported backend: {name}"),
            GeistError::PluginError(why) => write!(f, "plugin error: {why}"),
            GeistError::Serialization(why) => write!(f, "serialization error: {why}"),
            GeistError::Internal(why) => write!(f, "internal error: {why}"),
        }
    }
}

impl std::error::Error for GeistError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_non_empty_for_each_variant() {
        let cases = [
            GeistError::InvalidId("node"),
            GeistError::InvalidPort(PortId::new(1)),
            GeistError::TypeMismatch {
                expected: PortType::Audio,
                found: PortType::Note,
            },
            GeistError::DirectionMismatch,
            GeistError::ChannelMismatch {
                out_channels: 1,
                in_channels: 2,
            },
            GeistError::BadConfig("zero sample rate"),
            GeistError::UnsupportedBackend("jack"),
            GeistError::PluginError("init failed"),
            GeistError::Serialization("truncated"),
            GeistError::Internal("unreachable"),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn error_is_copy_and_comparable() {
        let err = GeistError::DirectionMismatch;
        let copy = err;
        assert_eq!(err, copy);
    }
}
