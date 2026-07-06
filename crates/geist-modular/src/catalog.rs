// =============================================================================
// File: crates/geist-modular/src/catalog.rs
// Layer: modular utilities
// Purpose: Rack node registry: constructible nodes with tag and port metadata
// Status: Implemented; existing utility families registered.
// Notes: Tags come from the spec §10.3 vocabulary. Builders run on the app
//        thread (construction may allocate; process never does). Port lists
//        are the browser/patching contract, not a hard channel shape — any
//        output patches to any input, validation is feedback-only.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_graph::node::AudioNode;

use crate::bridge::{MidiCvNode, RackOutNode, TransportClockNode};
use crate::logic::{AndNode, ComparatorNode, FlipFlopNode, NotNode, OrNode};
use crate::math::{AbsNode, AddNode, ClipNode, MultiplyNode, RescaleNode};
use crate::rack_nodes::{EnvNode, LfoRackNode, VcaNode, VcfNode, VcoNode};
use crate::sample_hold::{SampleAndHoldNode, TrackAndHoldNode};
use crate::signal::{AttenuverterNode, DcOffsetNode, DemuxNode, MuxNode};
use crate::timing::{ClockDividerNode, GateDelayNode, SlewLimiterNode};

// One constructible rack node: identity, search tag, ports, and builder
pub struct RackNodeSpec {
    pub name: &'static str,
    // Spec §10.3 tag vocabulary; drives browser categories and search
    pub tag: &'static str,
    pub inputs: &'static [&'static str],
    pub outputs: &'static [&'static str],
    pub build: fn() -> Box<dyn AudioNode>,
}

// Every node the rack can construct, existing utility families first
pub fn rack_catalog() -> &'static [RackNodeSpec] {
    &CATALOG
}

// Look up a catalog entry by its display name
pub fn rack_node(name: &str) -> Option<&'static RackNodeSpec> {
    CATALOG.iter().find(|spec| spec.name == name)
}

static CATALOG: [RackNodeSpec; 27] = [
    // Bridge family (DAW <-> rack)
    RackNodeSpec {
        name: "MIDI to CV",
        tag: "External",
        inputs: &[],
        outputs: &["V/Oct", "Gate", "Vel", "Rtrg"],
        build: || Box::new(MidiCvNode::new()),
    },
    RackNodeSpec {
        name: "Clock",
        tag: "Clock generator",
        inputs: &[],
        outputs: &["Clock", "Clock/N"],
        build: || Box::new(TransportClockNode::new()),
    },
    RackNodeSpec {
        name: "Rack Out",
        tag: "Utility",
        inputs: &["Audio"],
        outputs: &["Audio"],
        build: || Box::new(RackOutNode),
    },
    // Generator/processor family (geist-dsp adapters)
    RackNodeSpec {
        name: "Oscillator",
        tag: "Oscillator",
        inputs: &["V/Oct"],
        outputs: &["Audio"],
        build: || Box::new(VcoNode::new()),
    },
    RackNodeSpec {
        name: "LFO",
        tag: "Low-frequency oscillator",
        inputs: &["Rate"],
        outputs: &["CV"],
        build: || Box::new(LfoRackNode::new()),
    },
    RackNodeSpec {
        name: "Envelope",
        tag: "Envelope generator",
        inputs: &["Gate"],
        outputs: &["CV"],
        build: || Box::new(EnvNode::new()),
    },
    RackNodeSpec {
        name: "Filter",
        tag: "Filter",
        inputs: &["Audio", "Cutoff"],
        outputs: &["Audio"],
        build: || Box::new(VcfNode::new()),
    },
    RackNodeSpec {
        name: "VCA",
        tag: "Voltage-controlled amplifier",
        inputs: &["Audio", "Level"],
        outputs: &["Audio"],
        build: || Box::new(VcaNode),
    },
    // Logic family
    RackNodeSpec {
        name: "And",
        tag: "Logic",
        inputs: &["A", "B"],
        outputs: &["Gate"],
        build: || Box::new(AndNode),
    },
    RackNodeSpec {
        name: "Or",
        tag: "Logic",
        inputs: &["A", "B"],
        outputs: &["Gate"],
        build: || Box::new(OrNode),
    },
    RackNodeSpec {
        name: "Not",
        tag: "Logic",
        inputs: &["Gate"],
        outputs: &["Gate"],
        build: || Box::new(NotNode),
    },
    RackNodeSpec {
        name: "Comparator",
        tag: "Logic",
        inputs: &["In"],
        outputs: &["Gate"],
        build: || Box::new(ComparatorNode::default()),
    },
    RackNodeSpec {
        name: "Flip-Flop",
        tag: "Logic",
        inputs: &["Toggle"],
        outputs: &["Gate"],
        build: || Box::new(FlipFlopNode::default()),
    },
    // Math family
    RackNodeSpec {
        name: "Add",
        tag: "Utility",
        inputs: &["A", "B"],
        outputs: &["Out"],
        build: || Box::new(AddNode::default()),
    },
    RackNodeSpec {
        name: "Multiply",
        tag: "Utility",
        inputs: &["A", "B"],
        outputs: &["Out"],
        build: || Box::new(MultiplyNode::default()),
    },
    RackNodeSpec {
        name: "Abs",
        tag: "Utility",
        inputs: &["In"],
        outputs: &["Out"],
        build: || Box::new(AbsNode),
    },
    RackNodeSpec {
        name: "Clip",
        tag: "Utility",
        inputs: &["In"],
        outputs: &["Out"],
        build: || Box::new(ClipNode::default()),
    },
    RackNodeSpec {
        name: "Rescale",
        tag: "Utility",
        inputs: &["In"],
        outputs: &["Out"],
        build: || Box::new(RescaleNode::default()),
    },
    // Sample & hold family
    RackNodeSpec {
        name: "Sample & Hold",
        tag: "Sample and hold",
        inputs: &["Signal", "Trigger"],
        outputs: &["Out"],
        build: || Box::new(SampleAndHoldNode::default()),
    },
    RackNodeSpec {
        name: "Track & Hold",
        tag: "Sample and hold",
        inputs: &["Signal", "Gate"],
        outputs: &["Out"],
        build: || Box::new(TrackAndHoldNode::default()),
    },
    // Signal routing family
    RackNodeSpec {
        name: "Attenuverter",
        tag: "Attenuator",
        inputs: &["In"],
        outputs: &["Out"],
        build: || Box::new(AttenuverterNode::default()),
    },
    RackNodeSpec {
        name: "DC Offset",
        tag: "Utility",
        inputs: &["In"],
        outputs: &["Out"],
        build: || Box::new(DcOffsetNode::default()),
    },
    RackNodeSpec {
        name: "Mux",
        tag: "Switch",
        inputs: &["A", "B", "Select"],
        outputs: &["Out"],
        build: || Box::new(MuxNode::default()),
    },
    RackNodeSpec {
        name: "Demux",
        tag: "Switch",
        inputs: &["In", "Select"],
        outputs: &["A", "B"],
        build: || Box::new(DemuxNode::default()),
    },
    // Timing family
    RackNodeSpec {
        name: "Clock Divider",
        tag: "Clock modulator",
        inputs: &["Clock"],
        outputs: &["Clock"],
        build: || Box::new(ClockDividerNode::default()),
    },
    RackNodeSpec {
        name: "Gate Delay",
        tag: "Clock modulator",
        inputs: &["Gate"],
        outputs: &["Gate"],
        build: || Box::new(GateDelayNode::default()),
    },
    RackNodeSpec {
        name: "Slew Limiter",
        tag: "Slew limiter",
        inputs: &["In"],
        outputs: &["Out"],
        build: || Box::new(SlewLimiterNode::default()),
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use geist_core::config::AudioConfig;
    use geist_core::context::ProcessContext;
    use geist_core::transport::TransportSnapshot;

    // Spec §10.3 tag subset the catalog is allowed to use
    const ALLOWED_TAGS: [&str; 14] = [
        "Logic",
        "Utility",
        "Sample and hold",
        "Attenuator",
        "Switch",
        "Clock modulator",
        "Slew limiter",
        "Oscillator",
        "Low-frequency oscillator",
        "Envelope generator",
        "Filter",
        "Voltage-controlled amplifier",
        "External",
        "Clock generator",
    ];

    #[test]
    fn catalog_names_are_unique_and_tagged_from_the_vocabulary() {
        let catalog = rack_catalog();
        assert!(!catalog.is_empty());
        for (i, spec) in catalog.iter().enumerate() {
            assert!(
                ALLOWED_TAGS.contains(&spec.tag),
                "{} uses an off-vocabulary tag {}",
                spec.name,
                spec.tag
            );
            assert!(!spec.inputs.is_empty() || !spec.outputs.is_empty());
            assert!(
                !catalog[..i].iter().any(|prev| prev.name == spec.name),
                "duplicate catalog name {}",
                spec.name
            );
        }
    }

    #[test]
    fn every_catalog_node_builds_prepares_and_runs_a_block() {
        let config = AudioConfig::new(48_000, 64, 0, 2).unwrap();
        let frames = 64;
        let input = vec![0.0f32; frames * 2];
        for spec in rack_catalog() {
            let mut node = (spec.build)();
            node.prepare(&config);
            let mut output = vec![0.0f32; frames * 2];
            let transport = TransportSnapshot::stopped(48_000);
            let mut ctx =
                ProcessContext::new(frames, 48_000, &input, &mut output, &[], &[], transport);
            node.process(&mut ctx);
            assert!(
                output.iter().all(|s| s.is_finite()),
                "{} produced non-finite output",
                spec.name
            );
        }
    }

    #[test]
    fn lookup_by_name_finds_registered_nodes() {
        assert!(rack_node("Sample & Hold").is_some());
        assert!(rack_node("Clock Divider").is_some());
        assert!(rack_node("Oscillator").is_some());
        assert!(rack_node("VCA").is_some());
        assert!(rack_node("Reverb").is_none(), "not a rack node");
    }
}
