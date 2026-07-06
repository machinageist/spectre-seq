// =============================================================================
// File: crates/geist-stacksynth/src/compile.rs
// Layer: internal synth device (generator-stack synth)
// Purpose: Compile a validated Patch into a per-voice render plan
// Status: S1 implemented; structural plan only, DSP render lands in S2+.
// Notes: The plan is built off the audio thread and swapped in whole; the
//        render loop iterates steps without allocating. Non-delayed audio-rate
//        edges are honored by topological step order; AuxIn edges read a
//        one-sample-delayed tap instead (spec §4.5, §7.3), so their ordering
//        is irrelevant and cycles through them are legal.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::HashMap;

use crate::schema::{AudioRateTarget, ModuleId, ModuleKind, Patch, SendDest};
use crate::validate::{validate, Validation};

// One resolved audio-rate modulation input on a step
#[derive(Clone, Debug, PartialEq)]
pub struct ModBinding {
    // Buffer index holding the source module's output
    pub source_buf: usize,
    pub param: AudioRateTarget,
    pub depth: f32,
    // True reads the source's previous-sample value (AuxIn one-sample delay)
    pub delayed: bool,
}

// One render step: a module with resolved input/output buffers
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub module: ModuleId,
    // Parameter snapshot; render state (phase, filter memory) lives elsewhere
    pub kind: ModuleKind,
    // Upstream chain buffer inside the same group; None at group top
    pub input: Option<usize>,
    // This step's output buffer; also the tap other steps read
    pub out: usize,
    pub mods: Vec<ModBinding>,
}

// One voice-to-bus send from an enabled output module
#[derive(Clone, Debug, PartialEq)]
pub struct OutputBinding {
    // Index into RenderPlan::steps
    pub step: usize,
    pub dest: SendDest,
}

// A compiled per-voice plan: ordered steps over fixed buffer slots
#[derive(Clone, Debug, PartialEq)]
pub struct RenderPlan {
    // Execution order; every non-delayed edge's source runs before its target
    pub steps: Vec<Step>,
    // Fixed buffer slot count, one per module
    pub buffer_count: usize,
    pub outputs: Vec<OutputBinding>,
}

// Compile a patch; validation errors abort with the full report
pub fn compile(patch: &Patch) -> Result<RenderPlan, Validation> {
    let validation = validate(patch);
    if !validation.is_valid() {
        return Err(validation);
    }

    // Buffer slot per module, in stack appearance order
    let mut buffer_of: HashMap<ModuleId, usize> = HashMap::new();
    for group in &patch.groups {
        for module in &group.modules {
            let next = buffer_of.len();
            buffer_of.insert(module.id, next);
        }
    }

    // Build steps in stack order with in-group chain inputs
    let mut steps: Vec<Step> = Vec::new();
    for group in &patch.groups {
        let mut prev: Option<usize> = None;
        for module in &group.modules {
            let out = buffer_of[&module.id];
            steps.push(Step {
                module: module.id,
                kind: module.kind.clone(),
                input: prev,
                out,
                mods: Vec::new(),
            });
            prev = Some(out);
        }
    }

    // Resolve enabled audio-rate routes onto their target steps
    let step_of: HashMap<ModuleId, usize> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.module, i))
        .collect();
    for route in &patch.audio_routes {
        if !route.enabled {
            continue;
        }
        let target = step_of[&route.target];
        steps[target].mods.push(ModBinding {
            source_buf: buffer_of[&route.source],
            param: route.target_param,
            depth: route.depth,
            delayed: route.target_param == AudioRateTarget::AuxIn,
        });
    }

    // Topologically order steps: implicit chain edges + non-delayed mod edges.
    // Kahn's algorithm with an index-ordered ready heap keeps compiles
    // deterministic for identical patches.
    let order = topo_order(&steps, &buffer_of, &step_of, patch);
    let steps: Vec<Step> = order.into_iter().map(|i| steps[i].clone()).collect();

    // Collect enabled bus sends after reordering
    let outputs = steps
        .iter()
        .enumerate()
        .filter_map(|(i, step)| {
            let out = match &step.kind {
                ModuleKind::EnvelopeOutput { out, .. } => out,
                ModuleKind::CurveOutput { out, .. } => out,
                _ => return None,
            };
            out.enabled.then_some(OutputBinding {
                step: i,
                dest: out.send_to,
            })
        })
        .collect();

    Ok(RenderPlan {
        buffer_count: steps.len(),
        steps,
        outputs,
    })
}

// Order steps so every non-delayed dependency runs before its dependent
fn topo_order(
    steps: &[Step],
    buffer_of: &HashMap<ModuleId, usize>,
    step_of: &HashMap<ModuleId, usize>,
    patch: &Patch,
) -> Vec<usize> {
    let n = steps.len();
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut indegree = vec![0usize; n];

    // Implicit in-group chain edges
    for group in &patch.groups {
        for pair in group.modules.windows(2) {
            let a = step_of[&pair[0].id];
            let b = step_of[&pair[1].id];
            dependents[a].push(b);
            indegree[b] += 1;
        }
    }
    // Explicit non-delayed mod edges
    let step_of_buffer: HashMap<usize, usize> = buffer_of
        .iter()
        .map(|(id, &buf)| (buf, step_of[id]))
        .collect();
    for (target, step) in steps.iter().enumerate() {
        for binding in &step.mods {
            if binding.delayed {
                continue;
            }
            let source = step_of_buffer[&binding.source_buf];
            dependents[source].push(target);
            indegree[target] += 1;
        }
    }

    // Kahn with smallest-index-first selection; the validator guarantees a DAG
    let mut ready: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while !ready.is_empty() {
        ready.sort_unstable_by(|a, b| b.cmp(a));
        let node = ready.pop().expect("non-empty ready set");
        order.push(node);
        for &next in &dependents[node] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                ready.push(next);
            }
        }
    }
    debug_assert_eq!(order.len(), n, "validator must reject cycles");
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::tests::fixture_patch;
    use crate::schema::*;

    #[test]
    fn fixture_compiles_and_is_deterministic() {
        let patch = fixture_patch();
        let a = compile(&patch).expect("valid fixture");
        let b = compile(&patch).expect("valid fixture");
        assert_eq!(a, b);
        assert_eq!(a.buffer_count, 4);
        assert_eq!(a.outputs.len(), 1);
    }

    #[test]
    fn invalid_patch_fails_compile_with_report() {
        let mut patch = fixture_patch();
        patch.lanes[2].poly = true;
        let err = compile(&patch).expect_err("poly gap must fail");
        assert!(!err.is_valid());
    }

    #[test]
    fn chain_inputs_never_cross_groups() {
        let patch = fixture_patch();
        let plan = compile(&patch).expect("valid fixture");
        // Map buffer -> group index, then check each step's input group
        let mut group_of_buffer = std::collections::HashMap::new();
        let mut buf = 0usize;
        for (gi, group) in patch.groups.iter().enumerate() {
            for _ in &group.modules {
                group_of_buffer.insert(buf, gi);
                buf += 1;
            }
        }
        for step in &plan.steps {
            if let Some(input) = step.input {
                assert_eq!(
                    group_of_buffer[&input], group_of_buffer[&step.out],
                    "chain input crossed a group boundary"
                );
            }
        }
    }

    #[test]
    fn group_top_steps_have_no_chain_input() {
        let patch = fixture_patch();
        let plan = compile(&patch).expect("valid fixture");
        // Fixture group 2 holds only module 4; its step must start clean
        let step = plan
            .steps
            .iter()
            .find(|s| s.module == ModuleId(4))
            .expect("module 4 present");
        assert_eq!(step.input, None);
    }

    #[test]
    fn fm_source_runs_before_its_target() {
        let patch = fixture_patch();
        let plan = compile(&patch).expect("valid fixture");
        let pos = |id: u32| {
            plan.steps
                .iter()
                .position(|s| s.module == ModuleId(id))
                .expect("module present")
        };
        // Module 4 phase-modulates module 1 undelayed, so it must run first
        assert!(pos(4) < pos(1));
        // The mod binding lands on module 1 with the route depth
        let target = &plan.steps[pos(1)];
        assert_eq!(target.mods.len(), 1);
        assert_eq!(target.mods[0].param, AudioRateTarget::Phase);
        assert!(!target.mods[0].delayed);
    }

    #[test]
    fn aux_feedback_compiles_with_delayed_binding() {
        let mut patch = fixture_patch();
        patch.groups[0].modules.insert(
            1,
            Module {
                id: ModuleId(9),
                kind: ModuleKind::Aux {
                    level: 1.0,
                    invert: false,
                },
            },
        );
        patch.audio_routes.push(AudioRateRoute {
            source: ModuleId(9),
            target: ModuleId(9),
            target_param: AudioRateTarget::AuxIn,
            depth: 0.5,
            enabled: true,
        });
        let plan = compile(&patch).expect("aux feedback is legal");
        let aux = plan
            .steps
            .iter()
            .find(|s| s.module == ModuleId(9))
            .expect("aux present");
        assert_eq!(aux.mods.len(), 1);
        assert!(aux.mods[0].delayed, "AuxIn binding must read delayed tap");
    }

    #[test]
    fn disabled_routes_produce_no_bindings() {
        let mut patch = fixture_patch();
        patch.audio_routes[0].enabled = false;
        let plan = compile(&patch).expect("valid fixture");
        assert!(plan.steps.iter().all(|s| s.mods.is_empty()));
    }

    #[test]
    fn disabled_output_is_not_bound_to_a_bus() {
        let mut patch = fixture_patch();
        for group in &mut patch.groups {
            for module in &mut group.modules {
                if let ModuleKind::EnvelopeOutput { out, .. } = &mut module.kind {
                    out.enabled = false;
                }
            }
        }
        let plan = compile(&patch).expect("still valid, just silent");
        assert!(plan.outputs.is_empty());
    }
}
