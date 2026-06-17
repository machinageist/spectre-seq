// =============================================================================
// File: crates/geist-audio-backend/src/device.rs
// Layer: audio I/O
// Purpose: AudioDevice, DeviceInfo
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Static description of an audio device enumerated from a backend
// Owned, allocation-friendly data built on the app thread, never on audio
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub max_input_channels: u16,
    pub max_output_channels: u16,
    pub default_sample_rate_hz: u32,
    pub min_sample_rate_hz: u32,
    pub max_sample_rate_hz: u32,
}

impl DeviceInfo {
    // Report whether the device can run at a sample rate
    pub fn supports_sample_rate(&self, hz: u32) -> bool {
        (self.min_sample_rate_hz..=self.max_sample_rate_hz).contains(&hz)
    }

    // Whether the device can play audio out
    pub fn is_output(&self) -> bool {
        self.max_output_channels > 0
    }

    // Whether the device can capture audio in
    pub fn is_input(&self) -> bool {
        self.max_input_channels > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> DeviceInfo {
        DeviceInfo {
            name: "Built-in Output".to_string(),
            max_input_channels: 0,
            max_output_channels: 2,
            default_sample_rate_hz: 48_000,
            min_sample_rate_hz: 44_100,
            max_sample_rate_hz: 192_000,
        }
    }

    #[test]
    fn reports_direction_from_channel_counts() {
        let d = device();
        assert!(d.is_output());
        assert!(!d.is_input());
    }

    #[test]
    fn sample_rate_support_is_inclusive_range() {
        let d = device();
        assert!(d.supports_sample_rate(44_100));
        assert!(d.supports_sample_rate(192_000));
        assert!(d.supports_sample_rate(96_000));
        assert!(!d.supports_sample_rate(22_050));
        assert!(!d.supports_sample_rate(384_000));
    }
}
