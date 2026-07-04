// Author: Jeff
// Date: 2026-07-03
// Description: Synth native parameter descriptor behavior tests
// Notes: Clean-room synth specs require stable typed parameter metadata before UI/automation wiring.

use geist_core::ids::ParamId;
use geist_synth::engine::params::{param_info, SynthParam, SYNTH_PARAM_DESCRIPTORS};

#[test]
fn descriptors_have_stable_order_and_ids() {
    assert_eq!(SYNTH_PARAM_DESCRIPTORS.len(), SynthParam::COUNT);

    for (index, descriptor) in SYNTH_PARAM_DESCRIPTORS.iter().enumerate() {
        assert_eq!(descriptor.id, ParamId::new(index as u64));
        assert_eq!(descriptor.param as usize, index);
    }

    assert_eq!(SYNTH_PARAM_DESCRIPTORS[0].param, SynthParam::OscAMix);
    assert_eq!(
        SYNTH_PARAM_DESCRIPTORS[11].param,
        SynthParam::FilterCutoffHz
    );
    assert_eq!(SYNTH_PARAM_DESCRIPTORS[20].param, SynthParam::Polyphony);
}

#[test]
fn descriptor_defaults_are_valid_and_normalized() {
    for descriptor in SYNTH_PARAM_DESCRIPTORS {
        assert_eq!(
            descriptor.info.clamp(descriptor.info.default),
            descriptor.info.default
        );
        let normalized = descriptor.info.default_normalized();
        assert!((0.0..=1.0).contains(&normalized));
    }
}

#[test]
fn param_info_lookup_uses_typed_destination() {
    let cutoff = param_info(SynthParam::FilterCutoffHz);
    assert_eq!(cutoff.id, ParamId::new(SynthParam::FilterCutoffHz as u64));
    assert_eq!(cutoff.unit, "Hz");
    assert_eq!(cutoff.clamp(100_000.0), 20_000.0);

    let polyphony = param_info(SynthParam::Polyphony);
    assert_eq!(polyphony.unit, "voices");
    assert_eq!(polyphony.clamp(0.0), 1.0);
    assert_eq!(polyphony.clamp(99.0), 32.0);
}
