// Author: Jeff
// Date: 2026-08-09
// Description: R3 slice 6 evidence — RT-003 denormal flush and NaN/Inf containment
// Notes: Injection fixtures per node type. Native devices contain non-finite values at their
//   own boundary, so proving plan-level containment requires processors that deliberately
//   emit poison; these live here as test-only devices rather than shipping in spectre-dsp.

use spectre_core::IdGen;
use spectre_dsp::{
    AudioProcessor, DeviceClass, DeviceIo, Filament, Gain, Gloam, NoteEvent, NoteEventKind,
    ProcessContext, ProcessError, PulseInstrument, SumBus, Waveform, FILAMENT_PARAMETERS,
    GLOAM_PARAMETERS,
};
use spectre_graph::{Connection, ContainmentStats, EditableGraph, NodeId, PlanNoteInput};

const FRAMES: usize = 64;
const SAMPLE_RATE: f64 = 48_000.0;

// What a poisoning device writes into its output
#[derive(Debug, Clone, Copy, PartialEq)]
enum Poison {
    Nan,
    PositiveInfinity,
    NegativeInfinity,
    PositiveDenormal,
    NegativeDenormal,
    Clean,
}

impl Poison {
    // Sample value this poison writes
    fn sample(self) -> f32 {
        match self {
            Self::Nan => f32::NAN,
            Self::PositiveInfinity => f32::INFINITY,
            Self::NegativeInfinity => f32::NEG_INFINITY,
            Self::PositiveDenormal => f32::MIN_POSITIVE / 4.0,
            Self::NegativeDenormal => -f32::MIN_POSITIVE / 4.0,
            Self::Clean => 0.5,
        }
    }
}

// Source-shaped device: no audio input, one stereo output
struct PoisonSource {
    poison: Poison,
}

impl AudioProcessor for PoisonSource {
    fn io(&self) -> DeviceIo {
        DeviceIo {
            class: DeviceClass::Source,
            audio_inputs: 0,
            audio_outputs: 2,
            accepts_notes: false,
        }
    }

    // Test-only device with no descriptors, so every key is refused
    fn set_parameter(
        &mut self,
        key: spectre_dsp::DeviceParameterKey,
        _value: f32,
    ) -> Result<(), spectre_dsp::ParameterError> {
        Err(spectre_dsp::ParameterError::UnknownKey(key))
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        _inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        let value = self.poison.sample();
        for channel in outputs.iter_mut() {
            for sample in channel.iter_mut().take(context.frames()) {
                *sample = value;
            }
        }
        Ok(())
    }
}

// Insert-effect-shaped device: passes input through, then writes poison into one sample
struct PoisonEffect {
    poison: Poison,
}

impl AudioProcessor for PoisonEffect {
    fn io(&self) -> DeviceIo {
        DeviceIo {
            class: DeviceClass::Effect,
            audio_inputs: 2,
            audio_outputs: 2,
            accepts_notes: false,
        }
    }

    // Test-only device with no descriptors, so every key is refused
    fn set_parameter(
        &mut self,
        key: spectre_dsp::DeviceParameterKey,
        _value: f32,
    ) -> Result<(), spectre_dsp::ParameterError> {
        Err(spectre_dsp::ParameterError::UnknownKey(key))
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        let frames = context.frames();
        for (channel, output) in outputs.iter_mut().enumerate() {
            output[..frames].copy_from_slice(&inputs[channel][..frames]);
        }
        // A single poisoned sample must contaminate the whole node, not just itself
        outputs[0][0] = self.poison.sample();
        Ok(())
    }
}

// Build a source-only plan and render one quantum
fn render_source(poison: Poison) -> (spectre_graph::CompiledPlan, NodeId) {
    let mut ids = IdGen::new(0x0000_5254_3030_3301);
    let source = NodeId::new(ids.next_id());
    let mut graph = EditableGraph::new();
    graph
        .add_node(source, PoisonSource { poison }.io())
        .unwrap();
    let mut plan = graph
        .compile(source, FRAMES, &mut |_| {
            Ok(Box::new(PoisonSource { poison }))
        })
        .unwrap();
    plan.process(SAMPLE_RATE, FRAMES, &[]).unwrap();
    (plan, source)
}

#[test]
fn non_finite_source_output_becomes_exact_silence() {
    for poison in [
        Poison::Nan,
        Poison::PositiveInfinity,
        Poison::NegativeInfinity,
    ] {
        let (plan, source) = render_source(poison);
        let output = plan.last_output().unwrap();

        for channel in output {
            assert!(
                channel.iter().all(|sample| *sample == 0.0),
                "{poison:?} must be contained as exact silence, not noise"
            );
        }
        let stats = plan.containment();
        assert_eq!(stats.contaminated_nodes, 1, "{poison:?} must be counted");
        assert_eq!(
            stats.last_contaminated,
            Some(source),
            "{poison:?} must name the offending node"
        );
    }
}

#[test]
fn denormal_output_is_flushed_with_its_sign_preserved() {
    for (poison, negative) in [
        (Poison::PositiveDenormal, false),
        (Poison::NegativeDenormal, true),
    ] {
        let (plan, _) = render_source(poison);
        let output = plan.last_output().unwrap();

        for channel in output {
            for sample in channel {
                assert_eq!(*sample, 0.0, "denormal must flush to zero");
                assert_eq!(
                    sample.is_sign_negative(),
                    negative,
                    "FTZ-equivalent flush preserves the sign"
                );
            }
        }
        let stats = plan.containment();
        // Flushing is not contamination; a denormal is a valid finite value
        assert_eq!(stats.contaminated_nodes, 0);
        assert_eq!(stats.last_contaminated, None);
        assert_eq!(stats.denormals_flushed, (FRAMES * 2) as u64);
    }
}

#[test]
fn clean_output_is_untouched_and_uncounted() {
    let (plan, _) = render_source(Poison::Clean);
    let output = plan.last_output().unwrap();
    for channel in output {
        assert!(channel.iter().all(|sample| *sample == 0.5));
    }
    assert_eq!(plan.containment(), Default::default());
}

#[test]
fn a_contaminated_node_is_isolated_from_downstream_nodes() {
    let mut ids = IdGen::new(0x0000_5254_3030_3302);
    let source = NodeId::new(ids.next_id());
    let effect = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            source,
            PoisonSource {
                poison: Poison::Nan,
            }
            .io(),
        )
        .unwrap();
    graph
        .add_node(effect, Gain::new(1.0).unwrap().io())
        .unwrap();
    graph
        .connect(Connection {
            from: source,
            from_bus: 0,
            to: effect,
            to_bus: 0,
        })
        .unwrap();

    let mut plan = graph
        .compile(effect, FRAMES, &mut |node| {
            if node == source {
                Ok(Box::new(PoisonSource {
                    poison: Poison::Nan,
                }))
            } else {
                Ok(Box::new(Gain::new(1.0)?))
            }
        })
        .unwrap();
    plan.process(SAMPLE_RATE, FRAMES, &[]).unwrap();

    // The downstream gain receives silence, so the final output is finite silence
    let output = plan.last_output().unwrap();
    for channel in output {
        assert!(
            channel.iter().all(|sample| sample.is_finite()),
            "isolation must stop NaN before it reaches downstream devices"
        );
        assert!(channel.iter().all(|sample| *sample == 0.0));
    }
    let stats = plan.containment();
    assert_eq!(stats.contaminated_nodes, 1, "only the source is at fault");
    assert_eq!(stats.last_contaminated, Some(source));
}

#[test]
fn one_poisoned_sample_silences_the_whole_node() {
    let mut ids = IdGen::new(0x0000_5254_3030_3303);
    let source = NodeId::new(ids.next_id());
    let effect = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            source,
            PoisonSource {
                poison: Poison::Clean,
            }
            .io(),
        )
        .unwrap();
    graph
        .add_node(
            effect,
            PoisonEffect {
                poison: Poison::PositiveInfinity,
            }
            .io(),
        )
        .unwrap();
    graph
        .connect(Connection {
            from: source,
            from_bus: 0,
            to: effect,
            to_bus: 0,
        })
        .unwrap();

    let mut plan = graph
        .compile(effect, FRAMES, &mut |node| {
            if node == source {
                Ok(Box::new(PoisonSource {
                    poison: Poison::Clean,
                }))
            } else {
                Ok(Box::new(PoisonEffect {
                    poison: Poison::PositiveInfinity,
                }))
            }
        })
        .unwrap();
    plan.process(SAMPLE_RATE, FRAMES, &[]).unwrap();

    let output = plan.last_output().unwrap();
    for channel in output {
        assert!(
            channel.iter().all(|sample| *sample == 0.0),
            "a single infinity must silence the node, not leak the other samples"
        );
    }
    assert_eq!(plan.containment().last_contaminated, Some(effect));
}

#[test]
fn containment_accumulates_across_quanta() {
    let mut ids = IdGen::new(0x0000_5254_3030_3304);
    let source = NodeId::new(ids.next_id());
    let mut graph = EditableGraph::new();
    graph
        .add_node(
            source,
            PoisonSource {
                poison: Poison::Nan,
            }
            .io(),
        )
        .unwrap();
    let mut plan = graph
        .compile(source, FRAMES, &mut |_| {
            Ok(Box::new(PoisonSource {
                poison: Poison::Nan,
            }))
        })
        .unwrap();

    for expected in 1..=5 {
        plan.process(SAMPLE_RATE, FRAMES, &[]).unwrap();
        assert_eq!(plan.containment().contaminated_nodes, expected);
    }
}

#[test]
fn healthy_native_devices_report_no_containment_activity() {
    let mut ids = IdGen::new(0x0000_5254_3030_3305);
    let pulse = NodeId::new(ids.next_id());
    let gain = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            pulse,
            PulseInstrument::new(Waveform::Saw, 0.3).unwrap().io(),
        )
        .unwrap();
    graph.add_node(gain, Gain::new(0.7).unwrap().io()).unwrap();
    graph
        .connect(Connection {
            from: pulse,
            from_bus: 0,
            to: gain,
            to_bus: 0,
        })
        .unwrap();
    let mut plan = graph
        .compile(gain, FRAMES, &mut |node| {
            if node == pulse {
                Ok(Box::new(PulseInstrument::new(Waveform::Saw, 0.3)?))
            } else {
                Ok(Box::new(Gain::new(0.7)?))
            }
        })
        .unwrap();

    let events: [NoteEvent; 0] = [];
    plan.process(
        SAMPLE_RATE,
        FRAMES,
        &[PlanNoteInput {
            node: pulse,
            events: &events,
        }],
    )
    .unwrap();

    // The shipping devices must not trip containment during ordinary rendering
    assert_eq!(plan.containment(), Default::default());
}

// R4-4 test 12 — a contaminated track is silenced before it can reach the sum
#[test]
fn a_contaminated_track_is_silenced_before_it_reaches_the_sum() {
    let mut ids = IdGen::new(0x0053_554d_4249_4e00);
    let poisoned = NodeId::new(ids.next_id());
    let clean = NodeId::new(ids.next_id());
    let sum = NodeId::new(ids.next_id());
    let master = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            poisoned,
            PoisonSource {
                poison: Poison::Nan,
            }
            .io(),
        )
        .unwrap();
    graph
        .add_node(
            clean,
            PoisonSource {
                poison: Poison::Clean,
            }
            .io(),
        )
        .unwrap();
    graph.add_node(sum, SumBus::new(2).unwrap().io()).unwrap();
    graph
        .add_node(master, Gain::new(1.0).unwrap().io())
        .unwrap();
    for (from, bus) in [(poisoned, 0), (clean, 1)] {
        graph
            .connect(Connection {
                from,
                from_bus: 0,
                to: sum,
                to_bus: bus,
            })
            .unwrap();
    }
    graph
        .connect(Connection {
            from: sum,
            from_bus: 0,
            to: master,
            to_bus: 0,
        })
        .unwrap();

    let mut plan = graph
        .compile(master, 16, &mut |node| {
            if node == poisoned {
                Ok(Box::new(PoisonSource {
                    poison: Poison::Nan,
                }))
            } else if node == clean {
                Ok(Box::new(PoisonSource {
                    poison: Poison::Clean,
                }))
            } else if node == sum {
                Ok(Box::new(SumBus::new(2)?))
            } else {
                Ok(Box::new(Gain::new(1.0)?))
            }
        })
        .unwrap();

    plan.process(48_000.0, 16, &[]).unwrap();

    // Containment silences the poisoned node whole before the sum sees it, so the clean track
    // survives at its own value rather than the whole master going silent
    let output = plan.last_output().unwrap();
    for channel in output {
        for sample in channel {
            assert!(sample.is_finite());
            assert_eq!(*sample, 0.5, "the clean track must reach master intact");
        }
    }
    assert_eq!(plan.containment().contaminated_nodes, 1);
}

// R4-6 — the alpha's own devices must not trip containment while sounding.
// Several quanta rather than one: Gloam is recursive, so a value that only goes non-finite after
// its state has accumulated would survive a single-block test
#[test]
fn filament_and_gloam_report_no_containment_activity() {
    const QUANTA: usize = 8;

    let mut ids = IdGen::new(0x0000_5254_3030_3306);
    let filament = NodeId::new(ids.next_id());
    let gloam = NodeId::new(ids.next_id());

    let lean = FILAMENT_PARAMETERS[0].default();
    let rise_ms = FILAMENT_PARAMETERS[1].default();
    let fall_ms = FILAMENT_PARAMETERS[2].default();
    let level = FILAMENT_PARAMETERS[3].default();
    let damp_hz = GLOAM_PARAMETERS[0].default();
    let depth = GLOAM_PARAMETERS[1].default();
    let track_ms = GLOAM_PARAMETERS[2].default();

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            filament,
            Filament::new(lean, rise_ms, fall_ms, level).unwrap().io(),
        )
        .unwrap();
    graph
        .add_node(gloam, Gloam::new(damp_hz, depth, track_ms).unwrap().io())
        .unwrap();
    graph
        .connect(Connection {
            from: filament,
            from_bus: 0,
            to: gloam,
            to_bus: 0,
        })
        .unwrap();
    let mut plan = graph
        .compile(gloam, FRAMES, &mut |node| {
            if node == filament {
                Ok(Box::new(Filament::new(lean, rise_ms, fall_ms, level)?)
                    as Box<dyn AudioProcessor>)
            } else {
                Ok(Box::new(Gloam::new(damp_hz, depth, track_ms)?))
            }
        })
        .unwrap();

    let held = [NoteEvent {
        frame_offset: 0,
        sequence: 0,
        kind: NoteEventKind::On {
            id: 1,
            channel: 0,
            note: 45,
            velocity: 0.8,
        },
    }];
    let empty: [NoteEvent; 0] = [];
    let mut peak = 0.0_f32;
    for quantum in 0..QUANTA {
        let events: &[NoteEvent] = if quantum == 0 { &held } else { &empty };
        plan.process(
            SAMPLE_RATE,
            FRAMES,
            &[PlanNoteInput {
                node: filament,
                events,
            }],
        )
        .unwrap();
        let output = plan.last_output().unwrap();
        for sample in output[0].iter().chain(output[1].iter()) {
            peak = peak.max(sample.abs());
        }
    }

    assert_eq!(plan.containment().contaminated_nodes, 0);
    assert!(plan.containment().last_contaminated.is_none());
    // Without this, two silent buffers would satisfy every assertion above
    assert!(
        peak > 0.0,
        "the voice chain must sound for the guard to mean anything"
    );
}

// R4 slice 8 evidence — containment is scoped to the quantum, so block size is audible
// The existing PoisonEffect writes outputs[0][0], the first sample of *every* quantum, which
// cannot express a fixed position on the timeline. This one counts frames across process calls
struct PoisonAtFrame {
    target: usize,
    elapsed: usize,
}

impl AudioProcessor for PoisonAtFrame {
    fn io(&self) -> DeviceIo {
        DeviceIo {
            class: DeviceClass::Effect,
            audio_inputs: 2,
            audio_outputs: 2,
            accepts_notes: false,
        }
    }

    // Test-only device with no descriptors, so every key is refused
    fn set_parameter(
        &mut self,
        key: spectre_dsp::DeviceParameterKey,
        _value: f32,
    ) -> Result<(), spectre_dsp::ParameterError> {
        Err(spectre_dsp::ParameterError::UnknownKey(key))
    }

    fn process(
        &mut self,
        context: &ProcessContext<'_>,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
    ) -> Result<(), ProcessError> {
        let frames = context.frames();
        for (channel, output) in outputs.iter_mut().enumerate() {
            output[..frames].copy_from_slice(&inputs[channel][..frames]);
        }
        if self.target >= self.elapsed && self.target < self.elapsed + frames {
            outputs[0][self.target - self.elapsed] = Poison::Nan.sample();
        }
        self.elapsed += frames;
        Ok(())
    }
}

// Render `total` clean frames through PoisonAtFrame in `block`-frame quanta, returning every
// output sample in frame order across both channels and the containment tally
fn render_poisoned_at(total: usize, block: usize, target: usize) -> (Vec<f32>, ContainmentStats) {
    let mut ids = IdGen::new(0x0000_5254_3034_3308);
    let source = NodeId::new(ids.next_id());
    let effect = NodeId::new(ids.next_id());

    let mut graph = EditableGraph::new();
    graph
        .add_node(
            source,
            PoisonSource {
                poison: Poison::Clean,
            }
            .io(),
        )
        .unwrap();
    graph
        .add_node(effect, PoisonAtFrame { target, elapsed: 0 }.io())
        .unwrap();
    graph
        .connect(Connection {
            from: source,
            from_bus: 0,
            to: effect,
            to_bus: 0,
        })
        .unwrap();

    let mut plan = graph
        .compile(effect, block, &mut |node| {
            if node == source {
                Ok(Box::new(PoisonSource {
                    poison: Poison::Clean,
                }))
            } else {
                Ok(Box::new(PoisonAtFrame { target, elapsed: 0 }))
            }
        })
        .unwrap();

    let mut samples = Vec::with_capacity(total * 2);
    for _ in 0..total / block {
        plan.process(SAMPLE_RATE, block, &[]).unwrap();
        let output = plan.last_output().unwrap();
        for (left, right) in output[0][..block].iter().zip(&output[1][..block]) {
            samples.push(*left);
            samples.push(*right);
        }
    }
    (samples, plan.containment())
}

#[test]
fn a_contaminated_render_is_not_identical_across_block_sizes() {
    const TOTAL: usize = 512;
    const TARGET: usize = 300;

    let (one_quantum, one_stats) = render_poisoned_at(TOTAL, TOTAL, TARGET);
    let (two_quanta, two_stats) = render_poisoned_at(TOTAL, TOTAL / 2, TARGET);

    // Frame 0 is 300 frames before the poison and is silenced anyway in the single-quantum
    // render, because containment is scoped to the quantum and the poison is inside it
    assert_eq!(one_quantum[0], 0.0);
    // The same frame survives when the quantum boundary falls between it and the poison
    assert_eq!(two_quanta[0], 0.5);

    // One node contaminated either way; what changes is how much audio went with it
    assert_eq!(one_stats.contaminated_nodes, 1);
    assert_eq!(two_stats.contaminated_nodes, 1);
    assert_eq!(one_quantum.iter().filter(|s| **s == 0.5).count(), 0);
    assert_eq!(
        two_quanta.iter().filter(|s| **s == 0.5).count(),
        TOTAL, // the first 256-frame quantum, both channels
    );

    // Same audio, same poison, different bytes — the whole reason a bounce must state its
    // block size rather than choose one. This fails if CompiledPlan::process ever narrows
    // containment from the quantum to the sample, which would be a good change to notice
    assert_ne!(one_quantum, two_quanta);
}

// The other half of the test above, and the half that evidences the accepted contract.
//
// `dsp-device-io.md`'s checklist says "identical initial state and input produce bit-identical
// offline output". R4-8 read the cross-block-size result as narrowing that clause and routed it
// as an open item. It does not narrow it: block size is part of a render's configuration, not
// part of its input -- which is what `STATUS.md` already says in the words "block size is part of
// a render, not a preference". Two renders at different quanta are not the same render, so the
// clause never covered them.
//
// What the clause DOES claim is that one configuration reproduces itself, and it holds even when
// containment fires. Without this test the pair would show only that block size matters and never
// that determinism survives contamination at all
#[test]
fn a_contaminated_render_is_bit_identical_to_itself_at_the_same_block_size() {
    const TOTAL: usize = 512;
    const TARGET: usize = 300;

    for block in [TOTAL, TOTAL / 2, TOTAL / 4] {
        let (first, first_stats) = render_poisoned_at(TOTAL, block, TARGET);
        let (second, second_stats) = render_poisoned_at(TOTAL, block, TARGET);
        assert_eq!(
            first, second,
            "a contaminated render at block {block} did not reproduce itself"
        );
        assert_eq!(
            first_stats.contaminated_nodes,
            second_stats.contaminated_nodes
        );
        // A pair of all-silent renders would satisfy the equality above
        assert!(
            first.iter().any(|sample| *sample != 0.0) || block == TOTAL,
            "block {block} produced only silence, so the comparison proved nothing"
        );
    }
}
