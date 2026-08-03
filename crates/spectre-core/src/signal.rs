// Author: Jeff
// Date: 2026-05-27
// Description: Signal domain vocabulary for audio, modulation, gates, notes, MIDI, and parameters.
// Notes: Signal variants describe data shape; buffers remain borrowed by ProcessContext.

use crate::port::PortType;

// Processing cadence a port type is evaluated at
// Drives how the compiled graph schedules and buffers each edge
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SignalRate {
    // Per-sample continuous stream
    Audio,
    // Per-block scalar value
    Control,
    // Sparse timestamped events within a block
    Event,
}

impl PortType {
    // Classify the processing rate this port type carries
    #[inline]
    pub fn rate(self) -> SignalRate {
        match self {
            PortType::Audio | PortType::Cv | PortType::Gate => SignalRate::Audio,
            PortType::Parameter | PortType::Meter => SignalRate::Control,
            PortType::Note | PortType::Midi => SignalRate::Event,
        }
    }

    // Report whether this port carries a per-sample stream
    #[inline]
    pub fn is_audio_rate(self) -> bool {
        matches!(self.rate(), SignalRate::Audio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_classify_as_audio_rate() {
        assert_eq!(PortType::Audio.rate(), SignalRate::Audio);
        assert_eq!(PortType::Cv.rate(), SignalRate::Audio);
        assert_eq!(PortType::Gate.rate(), SignalRate::Audio);
        assert!(PortType::Audio.is_audio_rate());
    }

    #[test]
    fn scalars_classify_as_control_rate() {
        assert_eq!(PortType::Parameter.rate(), SignalRate::Control);
        assert_eq!(PortType::Meter.rate(), SignalRate::Control);
        assert!(!PortType::Parameter.is_audio_rate());
    }

    #[test]
    fn messages_classify_as_event_rate() {
        assert_eq!(PortType::Note.rate(), SignalRate::Event);
        assert_eq!(PortType::Midi.rate(), SignalRate::Event);
    }
}
