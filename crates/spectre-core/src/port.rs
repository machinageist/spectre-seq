// Author: Jeff
// Date: 2026-05-27
// Description: Port metadata, direction, and compatibility rules for graph connections.
// Notes: Port checks run on the app thread before a compiled graph reaches audio.

use crate::errors::{SpectreError, SpectreResult};
use crate::ids::{NodeId, ParamId, PortId};

// Whether a port consumes or produces signal
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PortDirection {
    Input,
    Output,
}

// Signal category carried by a port
// Determines connection compatibility and processing rate
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum PortType {
    // Audio-rate sample stream
    Audio,
    // Audio-rate control voltage for modular routing
    Cv,
    // Audio-rate trigger/gate signal
    Gate,
    // Musical note on/off event stream
    Note,
    // Raw MIDI message stream
    Midi,
    // Smoothed parameter value stream
    Parameter,
    // Metering feedback toward the UI
    Meter,
}

impl PortType {
    // Report whether two ports may carry the same signal across a connection
    #[inline]
    pub fn is_compatible_with(self, other: PortType) -> bool {
        self == other
    }
}

// Static description of one port on one node
// The graph registry owns these; the audio thread never mutates them
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PortDescriptor {
    pub id: PortId,
    pub node: NodeId,
    pub direction: PortDirection,
    pub port_type: PortType,
    pub channels: u16,
    pub name: &'static str,
    // Bound parameter when this port targets a parameter input
    pub param: Option<ParamId>,
}

// Validate that an output port may feed an input port
// Pure and allocation-free so graph edit code can call it directly
pub fn can_connect(out: &PortDescriptor, inp: &PortDescriptor) -> SpectreResult<()> {
    if out.direction != PortDirection::Output || inp.direction != PortDirection::Input {
        return Err(SpectreError::DirectionMismatch);
    }
    if !out.port_type.is_compatible_with(inp.port_type) {
        return Err(SpectreError::TypeMismatch {
            expected: out.port_type,
            found: inp.port_type,
        });
    }
    if out.channels != inp.channels {
        return Err(SpectreError::ChannelMismatch {
            out_channels: out.channels,
            in_channels: inp.channels,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a descriptor for connection tests
    fn port(
        raw: u64,
        direction: PortDirection,
        port_type: PortType,
        channels: u16,
    ) -> PortDescriptor {
        PortDescriptor {
            id: PortId::new(raw),
            node: NodeId::new(raw),
            direction,
            port_type,
            channels,
            name: "test",
            param: None,
        }
    }

    #[test]
    fn legal_audio_connection() {
        let out = port(1, PortDirection::Output, PortType::Audio, 2);
        let inp = port(2, PortDirection::Input, PortType::Audio, 2);
        assert!(can_connect(&out, &inp).is_ok());
    }

    #[test]
    fn rejects_direction_mismatch() {
        let a = port(1, PortDirection::Output, PortType::Audio, 2);
        let b = port(2, PortDirection::Output, PortType::Audio, 2);
        assert_eq!(can_connect(&a, &b), Err(SpectreError::DirectionMismatch));
    }

    #[test]
    fn rejects_type_mismatch() {
        let out = port(1, PortDirection::Output, PortType::Audio, 1);
        let inp = port(2, PortDirection::Input, PortType::Note, 1);
        assert_eq!(
            can_connect(&out, &inp),
            Err(SpectreError::TypeMismatch {
                expected: PortType::Audio,
                found: PortType::Note
            })
        );
    }

    #[test]
    fn rejects_channel_mismatch() {
        let out = port(1, PortDirection::Output, PortType::Audio, 1);
        let inp = port(2, PortDirection::Input, PortType::Audio, 2);
        assert_eq!(
            can_connect(&out, &inp),
            Err(SpectreError::ChannelMismatch {
                out_channels: 1,
                in_channels: 2
            })
        );
    }
}
