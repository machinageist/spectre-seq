// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 2 evidence — the cpal backend enumerates without hardware assumptions
// Notes: CI runners have no sound card, so these assert shape and total absence of panics,
//   never that a device exists. Opening a real stream is the slice 7 lifecycle drill,
//   which runs on qualification hardware rather than here.

#![cfg(feature = "cpal-backend")]

use spectre_audio::cpal_backend::CpalBackend;
use spectre_audio::{AudioBackend, BackendError, CPAL_BACKEND_NAME};

#[test]
fn cpal_backend_reports_its_identity() {
    assert_eq!(CpalBackend::new().name(), CPAL_BACKEND_NAME);
}

#[test]
fn cpal_enumeration_is_well_formed_with_or_without_devices() {
    let backend = CpalBackend::new();
    // An error is an acceptable outcome on a headless runner; a panic is not
    let Ok(devices) = backend.output_devices() else {
        return;
    };
    for device in &devices {
        assert!(
            !device.id.as_str().is_empty(),
            "device key must be nonempty"
        );
        assert_eq!(device.name, device.id.as_str());
    }
    // cpal reports at most one default output device
    assert!(devices.iter().filter(|device| device.is_default).count() <= 1);
}

#[test]
fn cpal_default_device_absence_is_reported_not_panicked() {
    let backend = CpalBackend::new();
    match backend.default_output_device() {
        Ok(device) => assert!(device.is_default),
        Err(error) => assert!(matches!(
            error,
            BackendError::NoDefaultDevice | BackendError::EnumerationFailed(_)
        )),
    }
}

#[test]
fn cpal_open_rejects_invalid_config_without_a_device() {
    let backend = CpalBackend::new();
    let Ok(default) = backend.default_output_device() else {
        return;
    };
    let bad = spectre_audio::StreamConfig {
        sample_rate: 48_000,
        channels: 2,
        buffer_frames: 0,
    };
    // Config validation runs before any device lookup or driver call
    assert_eq!(
        backend
            .open_output(&default.id, bad, Box::new(|_| {}))
            .err()
            .unwrap(),
        BackendError::UnsupportedBufferSize(0)
    );
}
