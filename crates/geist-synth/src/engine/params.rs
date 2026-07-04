// Author: Jeff
// Date: 2026-07-03
// Description: Typed parameter descriptors for Geist's first-party synth engine.
// Notes: Descriptor order is stable API for UI, automation, and preset state.

use geist_core::ids::ParamId;
use geist_core::params::{ParamInfo, ParamRange};

// Stable synth parameter destinations for native UI, automation, and modulation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum SynthParam {
    OscAMix = 0,
    OscASemitones = 1,
    OscACents = 2,
    OscBLevel = 3,
    OscBSemitones = 4,
    OscBCents = 5,
    FmAmount = 6,
    UnisonVoices = 7,
    UnisonDetuneCents = 8,
    AmpLevel = 9,
    FilterEnvOctaves = 10,
    FilterCutoffHz = 11,
    FilterResonance = 12,
    AmpAttackSeconds = 13,
    AmpDecaySeconds = 14,
    AmpSustain = 15,
    AmpReleaseSeconds = 16,
    FilterAttackSeconds = 17,
    FilterDecaySeconds = 18,
    FilterSustain = 19,
    Polyphony = 20,
}

impl SynthParam {
    pub const COUNT: usize = 21;
}

// Static synth descriptor preserving typed destination alongside shared metadata.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SynthParamDescriptor {
    pub param: SynthParam,
    pub id: ParamId,
    pub info: ParamInfo,
}

impl SynthParamDescriptor {
    const fn new(
        param: SynthParam,
        name: &'static str,
        unit: &'static str,
        range: ParamRange,
        default: f32,
        modulatable: bool,
    ) -> Self {
        let id = ParamId::new(param as u64);
        Self {
            param,
            id,
            info: ParamInfo {
                id,
                name,
                unit,
                range,
                default,
                automatable: true,
                modulatable,
            },
        }
    }
}

pub const SYNTH_PARAM_DESCRIPTORS: &[SynthParamDescriptor] = &[
    SynthParamDescriptor::new(
        SynthParam::OscAMix,
        "Osc A shape mix",
        "norm",
        ParamRange::Linear { min: 0.0, max: 1.0 },
        0.5,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::OscASemitones,
        "Osc A coarse tune",
        "st",
        ParamRange::Linear {
            min: -48.0,
            max: 48.0,
        },
        0.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::OscACents,
        "Osc A fine tune",
        "ct",
        ParamRange::Linear {
            min: -100.0,
            max: 100.0,
        },
        0.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::OscBLevel,
        "Osc B level",
        "norm",
        ParamRange::Linear { min: 0.0, max: 1.0 },
        0.5,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::OscBSemitones,
        "Osc B coarse tune",
        "st",
        ParamRange::Linear {
            min: -48.0,
            max: 48.0,
        },
        0.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::OscBCents,
        "Osc B fine tune",
        "ct",
        ParamRange::Linear {
            min: -100.0,
            max: 100.0,
        },
        0.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::FmAmount,
        "FM amount",
        "norm",
        ParamRange::Linear { min: 0.0, max: 1.0 },
        0.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::UnisonVoices,
        "Unison voices",
        "voices",
        ParamRange::Stepped { min: 1, max: 8 },
        1.0,
        false,
    ),
    SynthParamDescriptor::new(
        SynthParam::UnisonDetuneCents,
        "Unison detune",
        "ct",
        ParamRange::Linear {
            min: 0.0,
            max: 100.0,
        },
        0.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::AmpLevel,
        "Amp level",
        "norm",
        ParamRange::Linear { min: 0.0, max: 1.0 },
        0.8,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::FilterEnvOctaves,
        "Filter env amount",
        "oct",
        ParamRange::Linear {
            min: -8.0,
            max: 8.0,
        },
        4.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::FilterCutoffHz,
        "Filter cutoff",
        "Hz",
        ParamRange::Logarithmic {
            min: 20.0,
            max: 20_000.0,
        },
        2_000.0,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::FilterResonance,
        "Filter resonance",
        "Q",
        ParamRange::Linear { min: 0.1, max: 2.0 },
        0.7,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::AmpAttackSeconds,
        "Amp attack",
        "s",
        ParamRange::Logarithmic {
            min: 0.001,
            max: 10.0,
        },
        0.005,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::AmpDecaySeconds,
        "Amp decay",
        "s",
        ParamRange::Logarithmic {
            min: 0.001,
            max: 10.0,
        },
        0.1,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::AmpSustain,
        "Amp sustain",
        "norm",
        ParamRange::Linear { min: 0.0, max: 1.0 },
        0.8,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::AmpReleaseSeconds,
        "Amp release",
        "s",
        ParamRange::Logarithmic {
            min: 0.001,
            max: 10.0,
        },
        0.3,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::FilterAttackSeconds,
        "Filter attack",
        "s",
        ParamRange::Logarithmic {
            min: 0.001,
            max: 10.0,
        },
        0.01,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::FilterDecaySeconds,
        "Filter decay",
        "s",
        ParamRange::Logarithmic {
            min: 0.001,
            max: 10.0,
        },
        0.2,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::FilterSustain,
        "Filter sustain",
        "norm",
        ParamRange::Linear { min: 0.0, max: 1.0 },
        0.3,
        true,
    ),
    SynthParamDescriptor::new(
        SynthParam::Polyphony,
        "Polyphony",
        "voices",
        ParamRange::Stepped { min: 1, max: 32 },
        16.0,
        false,
    ),
];

// Return shared parameter metadata for one typed synth destination.
pub fn param_info(param: SynthParam) -> &'static ParamInfo {
    &SYNTH_PARAM_DESCRIPTORS[param as usize].info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_descriptor_matches_its_index() {
        assert_eq!(SYNTH_PARAM_DESCRIPTORS.len(), SynthParam::COUNT);
        for (index, descriptor) in SYNTH_PARAM_DESCRIPTORS.iter().enumerate() {
            assert_eq!(descriptor.param as usize, index);
            assert_eq!(descriptor.info.id, ParamId::new(index as u64));
        }
    }

    #[test]
    fn lookup_returns_static_descriptor_info() {
        assert_eq!(param_info(SynthParam::OscAMix).name, "Osc A shape mix");
        assert_eq!(param_info(SynthParam::Polyphony).clamp(64.0), 32.0);
    }
}
