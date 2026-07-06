// =============================================================================
// File: crates/geist-stacksynth/src/validate.rs
// Layer: internal synth device (generator-stack synth)
// Purpose: Patch validation: limits, structure, routing legality, cycles
// Status: S0 implemented; every rule maps to a spec section
//         (docs/specs/geist-modular-synth-spec.md).
// Notes: Errors block compilation (S1); warnings surface in UI but still play.
//        Cycle detection treats Aux-input edges as delayed and legal (§4.5).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::schema::{
    AudioRateTarget, ModuleId, ModuleKind, Patch, LANE_COUNT, MAX_GENERATOR_MODULES,
    MAX_MODULATORS,
};

// Hard validation failure; the patch cannot compile
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidateError {
    // Generator-area module count exceeds MAX_GENERATOR_MODULES
    TooManyModules { count: usize },
    // Modulator slot count exceeds MAX_MODULATORS
    TooManyModulators { count: usize },
    // Two modules share one id
    DuplicateModuleId { id: ModuleId },
    // Two modulator slots share one id
    DuplicateModulatorId { id: u32 },
    // Audio route endpoint does not exist in the stack
    MissingRouteEndpoint { source: ModuleId, target: ModuleId },
    // Audio route targets a parameter the module kind does not have
    InvalidRouteTarget { target: ModuleId, param: AudioRateTarget },
    // Poly lanes must form a contiguous prefix starting at lane 1 (§8.1)
    PolyLaneGap { lane: usize },
    // Lane send must go strictly rightward or to master (§8.1)
    LaneSendBackward { lane: usize, dest: u8 },
    // Undelayed audio-route cycle; the named edge closes the loop (§7.3)
    AudioRouteCycle { source: ModuleId, target: ModuleId },
    // SendDest::Lane index out of range
    InvalidSendLane { module: ModuleId, lane: u8 },
}

impl fmt::Display for ValidateError {
    // Name the offending entity so UI diagnostics stay actionable
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyModules { count } => write!(
                f,
                "generator area holds {count} modules; the limit is {MAX_GENERATOR_MODULES}"
            ),
            Self::TooManyModulators { count } => write!(
                f,
                "modulator lane holds {count} slots; the limit is {MAX_MODULATORS}"
            ),
            Self::DuplicateModuleId { id } => write!(f, "duplicate module id {}", id.0),
            Self::DuplicateModulatorId { id } => write!(f, "duplicate modulator id {id}"),
            Self::MissingRouteEndpoint { source, target } => write!(
                f,
                "audio route {} -> {} references a missing module",
                source.0, target.0
            ),
            Self::InvalidRouteTarget { target, param } => write!(
                f,
                "module {} has no audio-rate target {param:?}",
                target.0
            ),
            Self::PolyLaneGap { lane } => write!(
                f,
                "lane {} is poly but an earlier lane is not; poly lanes must be a prefix",
                lane + 1
            ),
            Self::LaneSendBackward { lane, dest } => write!(
                f,
                "lane {} sends to lane {}; sends must go rightward or to master",
                lane + 1,
                dest + 1
            ),
            Self::AudioRouteCycle { source, target } => write!(
                f,
                "audio route {} -> {} closes an undelayed feedback cycle; break it with an Aux input",
                source.0, target.0
            ),
            Self::InvalidSendLane { module, lane } => write!(
                f,
                "output module {} sends to lane {} which does not exist",
                module.0,
                lane + 1
            ),
        }
    }
}

// Non-blocking diagnostic; the patch still compiles and plays
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidateWarning {
    // Input-dependent module with nothing above it in its group (§2.2)
    MissingInput { module: ModuleId },
    // No enabled output module anywhere; the patch is silent (§5.1)
    NoActiveOutput,
}

impl fmt::Display for ValidateWarning {
    // Keep warning text short enough for inline UI badges
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInput { module } => {
                write!(f, "module {} has no input from above", module.0)
            }
            Self::NoActiveOutput => write!(f, "patch has no enabled output module"),
        }
    }
}

// Full validation result
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Validation {
    pub errors: Vec<ValidateError>,
    pub warnings: Vec<ValidateWarning>,
}

impl Validation {
    // Report whether the patch is compilable
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

// Validate a patch against every S0 rule; never panics on malformed data
pub fn validate(patch: &Patch) -> Validation {
    let mut v = Validation::default();
    check_limits(patch, &mut v);
    check_unique_ids(patch, &mut v);
    check_missing_inputs(patch, &mut v);
    check_outputs(patch, &mut v);
    check_lanes(patch, &mut v);
    check_audio_routes(patch, &mut v);
    check_cycles(patch, &mut v);
    v
}

// Enforce compatibility limits (§15.2)
fn check_limits(patch: &Patch, v: &mut Validation) {
    let count = patch.module_count();
    if count > MAX_GENERATOR_MODULES {
        v.errors.push(ValidateError::TooManyModules { count });
    }
    if patch.modulators.len() > MAX_MODULATORS {
        v.errors.push(ValidateError::TooManyModulators {
            count: patch.modulators.len(),
        });
    }
}

// Reject duplicate module/modulator ids
fn check_unique_ids(patch: &Patch, v: &mut Validation) {
    let mut seen = HashSet::new();
    for group in &patch.groups {
        for module in &group.modules {
            if !seen.insert(module.id) {
                v.errors.push(ValidateError::DuplicateModuleId { id: module.id });
            }
        }
    }
    let mut seen_mods = HashSet::new();
    for slot in &patch.modulators {
        if !seen_mods.insert(slot.id) {
            v.errors
                .push(ValidateError::DuplicateModulatorId { id: slot.id.0 });
        }
    }
}

// Warn on input-dependent modules with no upstream signal in their group (§2.2)
fn check_missing_inputs(patch: &Patch, v: &mut Validation) {
    for group in &patch.groups {
        let mut has_signal = false;
        for module in &group.modules {
            if module.kind.requires_input() && !has_signal {
                v.warnings
                    .push(ValidateWarning::MissingInput { module: module.id });
            }
            // Sources add signal; Aux injects its explicit input; processors pass through
            if module.kind.is_source() || matches!(module.kind, ModuleKind::Aux { .. }) {
                has_signal = true;
            }
        }
    }
}

// Warn when no enabled output exists and reject out-of-range send lanes (§5.1)
fn check_outputs(patch: &Patch, v: &mut Validation) {
    let mut any_active = false;
    for group in &patch.groups {
        for module in &group.modules {
            let out = match &module.kind {
                ModuleKind::EnvelopeOutput { out, .. } => out,
                ModuleKind::CurveOutput { out, .. } => out,
                _ => continue,
            };
            if out.enabled {
                any_active = true;
            }
            if let crate::schema::SendDest::Lane(lane) = out.send_to {
                if usize::from(lane) >= LANE_COUNT {
                    v.errors.push(ValidateError::InvalidSendLane {
                        module: module.id,
                        lane,
                    });
                }
            }
        }
    }
    if !any_active {
        v.warnings.push(ValidateWarning::NoActiveOutput);
    }
}

// Enforce poly-prefix and rightward-send lane rules (§8.1)
fn check_lanes(patch: &Patch, v: &mut Validation) {
    let mut prefix_ended = false;
    for (i, lane) in patch.lanes.iter().enumerate() {
        if lane.poly && prefix_ended {
            v.errors.push(ValidateError::PolyLaneGap { lane: i });
        }
        if !lane.poly {
            prefix_ended = true;
        }
        if let Some(dest) = lane.send_to {
            if usize::from(dest) <= i || usize::from(dest) >= LANE_COUNT {
                v.errors.push(ValidateError::LaneSendBackward { lane: i, dest });
            }
        }
    }
}

// Check audio-route endpoints exist and target parameters match the kind (§7)
fn check_audio_routes(patch: &Patch, v: &mut Validation) {
    for route in &patch.audio_routes {
        let source = patch.find_module(route.source);
        let target = patch.find_module(route.target);
        let (Some(_), Some(target)) = (source, target) else {
            v.errors.push(ValidateError::MissingRouteEndpoint {
                source: route.source,
                target: route.target,
            });
            continue;
        };
        if !target_accepts(&target.kind, route.target_param) {
            v.errors.push(ValidateError::InvalidRouteTarget {
                target: route.target,
                param: route.target_param,
            });
        }
    }
}

// Report whether a module kind exposes an audio-rate target parameter
fn target_accepts(kind: &ModuleKind, param: AudioRateTarget) -> bool {
    use AudioRateTarget as T;
    match kind {
        k if k.is_source() => matches!(
            param,
            T::Phase | T::Harmonic | T::Shift | T::Level | T::Pitch
        ),
        ModuleKind::Distortion { .. } => matches!(param, T::Drive),
        ModuleKind::Filter { .. } | ModuleKind::NonlinearFilter { .. } => {
            matches!(param, T::Cutoff | T::Drive)
        }
        ModuleKind::Mix { .. } => matches!(param, T::Level),
        ModuleKind::Aux { .. } => matches!(param, T::Level | T::AuxIn),
        // Output modules accept no audio-rate targets at S0
        _ => false,
    }
}

// Reject audio-route cycles that contain no delayed (AuxIn) edge (§7.3)
fn check_cycles(patch: &Patch, v: &mut Validation) {
    // Node set: every module id; edges: implicit in-group chain + explicit
    // routes, skipping AuxIn edges because they carry a one-sample delay
    let mut adjacency: HashMap<ModuleId, Vec<ModuleId>> = HashMap::new();
    for group in &patch.groups {
        for module in &group.modules {
            adjacency.entry(module.id).or_default();
        }
        for pair in group.modules.windows(2) {
            adjacency.entry(pair[0].id).or_default().push(pair[1].id);
        }
    }
    for route in &patch.audio_routes {
        if !route.enabled || route.target_param == AudioRateTarget::AuxIn {
            continue;
        }
        if adjacency.contains_key(&route.source) && adjacency.contains_key(&route.target) {
            adjacency
                .entry(route.source)
                .or_default()
                .push(route.target);
        }
    }

    // Iterative DFS with three-color marking; report the edge closing a cycle
    let mut color: HashMap<ModuleId, u8> = HashMap::new();
    for &start in adjacency.keys() {
        if color.get(&start).copied().unwrap_or(0) != 0 {
            continue;
        }
        let mut stack = vec![(start, 0usize)];
        color.insert(start, 1);
        while let Some(&(node, next)) = stack.last() {
            let neighbors = &adjacency[&node];
            if next < neighbors.len() {
                stack.last_mut().expect("non-empty stack").1 += 1;
                let peer = neighbors[next];
                match color.get(&peer).copied().unwrap_or(0) {
                    // Grey peer = back edge = undelayed cycle
                    1 => {
                        v.errors.push(ValidateError::AudioRouteCycle {
                            source: node,
                            target: peer,
                        });
                        return;
                    }
                    0 => {
                        color.insert(peer, 1);
                        stack.push((peer, 0));
                    }
                    _ => {}
                }
            } else {
                color.insert(node, 2);
                stack.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;

    // Build a minimal one-osc one-output patch
    fn small_patch() -> Patch {
        Patch {
            groups: vec![Group {
                id: GroupId(1),
                name: "Main".into(),
                minimized: false,
                modules: vec![
                    Module {
                        id: ModuleId(1),
                        kind: ModuleKind::AnalogOsc {
                            common: CommonGenParams::default(),
                            waveform: AnalogWaveform::Sine,
                            sync: 1.0,
                            pulse_width: 0.5,
                            unison: UnisonSettings::default(),
                        },
                    },
                    Module {
                        id: ModuleId(2),
                        kind: ModuleKind::EnvelopeOutput {
                            env: AdsrParams::default(),
                            out: OutputCommon::default(),
                        },
                    },
                ],
            }],
            ..Patch::default()
        }
    }

    // Build an analog osc module with a given id
    fn osc(id: u32) -> Module {
        Module {
            id: ModuleId(id),
            kind: ModuleKind::AnalogOsc {
                common: CommonGenParams::default(),
                waveform: AnalogWaveform::Sine,
                sync: 1.0,
                pulse_width: 0.5,
                unison: UnisonSettings::default(),
            },
        }
    }

    #[test]
    fn valid_patch_passes() {
        let v = validate(&small_patch());
        assert!(v.is_valid(), "{:?}", v.errors);
        assert!(v.warnings.is_empty(), "{:?}", v.warnings);
    }

    #[test]
    fn module_limit_boundary() {
        let mut patch = Patch::default();
        patch.groups.push(Group {
            id: GroupId(1),
            name: String::new(),
            minimized: false,
            modules: (0..MAX_GENERATOR_MODULES as u32).map(osc).collect(),
        });
        assert!(validate(&patch).is_valid());
        patch.groups[0].modules.push(osc(999));
        let v = validate(&patch);
        assert!(v
            .errors
            .contains(&ValidateError::TooManyModules { count: 33 }));
    }

    #[test]
    fn modulator_limit_boundary() {
        let slot = |id: u32| ModulatorSlot {
            id: ModulatorId(id),
            kind: ModulatorKind::Note,
            output_range: OutputRange::Unipolar,
            trigger: TriggerMode::Auto,
            depth: 1.0,
        };
        let mut patch = small_patch();
        patch.modulators = (0..MAX_MODULATORS as u32).map(slot).collect();
        assert!(validate(&patch).is_valid());
        patch.modulators.push(slot(999));
        let v = validate(&patch);
        assert!(v
            .errors
            .contains(&ValidateError::TooManyModulators { count: 33 }));
    }

    #[test]
    fn duplicate_module_ids_rejected() {
        let mut patch = small_patch();
        patch.groups[0].modules.push(osc(1));
        let v = validate(&patch);
        assert!(v
            .errors
            .contains(&ValidateError::DuplicateModuleId { id: ModuleId(1) }));
    }

    #[test]
    fn processor_at_group_top_warns_missing_input() {
        let mut patch = small_patch();
        patch.groups[0].modules.insert(
            0,
            Module {
                id: ModuleId(10),
                kind: ModuleKind::Filter {
                    filter_type: FilterType::LowPass,
                    cutoff_hz: 1000.0,
                    q: 0.7,
                    gain_db: 0.0,
                    slope: FilterSlope::TwoPole,
                },
            },
        );
        let v = validate(&patch);
        assert!(v
            .warnings
            .contains(&ValidateWarning::MissingInput { module: ModuleId(10) }));
        assert!(v.is_valid());
    }

    #[test]
    fn silent_patch_warns_no_output() {
        let mut patch = small_patch();
        patch.groups[0].modules.pop();
        let v = validate(&patch);
        assert!(v.warnings.contains(&ValidateWarning::NoActiveOutput));
    }

    #[test]
    fn poly_lanes_must_be_prefix() {
        let mut patch = small_patch();
        patch.lanes[1].poly = true;
        let v = validate(&patch);
        assert!(v.errors.contains(&ValidateError::PolyLaneGap { lane: 1 }));
        patch.lanes[0].poly = true;
        assert!(validate(&patch).is_valid());
    }

    #[test]
    fn lane_sends_must_go_rightward() {
        let mut patch = small_patch();
        patch.lanes[1].send_to = Some(0);
        let v = validate(&patch);
        assert!(v
            .errors
            .contains(&ValidateError::LaneSendBackward { lane: 1, dest: 0 }));
        patch.lanes[1].send_to = Some(2);
        assert!(validate(&patch).is_valid());
    }

    #[test]
    fn route_to_missing_module_rejected() {
        let mut patch = small_patch();
        patch.audio_routes.push(AudioRateRoute {
            source: ModuleId(1),
            target: ModuleId(99),
            target_param: AudioRateTarget::Phase,
            depth: 1.0,
            enabled: true,
        });
        let v = validate(&patch);
        assert!(v.errors.contains(&ValidateError::MissingRouteEndpoint {
            source: ModuleId(1),
            target: ModuleId(99),
        }));
    }

    #[test]
    fn route_target_param_must_match_kind() {
        let mut patch = small_patch();
        // Phase is not a valid target on an output module
        patch.audio_routes.push(AudioRateRoute {
            source: ModuleId(1),
            target: ModuleId(2),
            target_param: AudioRateTarget::Phase,
            depth: 1.0,
            enabled: true,
        });
        let v = validate(&patch);
        assert!(v.errors.contains(&ValidateError::InvalidRouteTarget {
            target: ModuleId(2),
            param: AudioRateTarget::Phase,
        }));
    }

    #[test]
    fn undelayed_cycle_rejected_and_named() {
        let mut patch = small_patch();
        // Osc modulates itself through the implicit chain: 1 -> 2 (implicit),
        // then explicit 2-target back onto 1 is not possible (2 is output), so
        // build a two-osc loop instead
        patch.groups[0].modules.insert(1, osc(5));
        patch.audio_routes.push(AudioRateRoute {
            source: ModuleId(5),
            target: ModuleId(1),
            target_param: AudioRateTarget::Phase,
            depth: 1.0,
            enabled: true,
        });
        let v = validate(&patch);
        assert!(matches!(
            v.errors.first(),
            Some(ValidateError::AudioRouteCycle { .. })
        ));
    }

    #[test]
    fn aux_input_edge_breaks_cycle_legally() {
        let mut patch = small_patch();
        patch.groups[0].modules.insert(
            1,
            Module {
                id: ModuleId(6),
                kind: ModuleKind::Aux {
                    level: 1.0,
                    invert: false,
                },
            },
        );
        // Feedback into the Aux explicit input is delayed and therefore legal
        patch.audio_routes.push(AudioRateRoute {
            source: ModuleId(6),
            target: ModuleId(6),
            target_param: AudioRateTarget::AuxIn,
            depth: 1.0,
            enabled: true,
        });
        assert!(validate(&patch).is_valid());
    }

    #[test]
    fn disabled_routes_do_not_form_cycles() {
        let mut patch = small_patch();
        patch.groups[0].modules.insert(1, osc(5));
        patch.audio_routes.push(AudioRateRoute {
            source: ModuleId(5),
            target: ModuleId(1),
            target_param: AudioRateTarget::Phase,
            depth: 1.0,
            enabled: false,
        });
        assert!(validate(&patch).is_valid());
    }

    #[test]
    fn cross_group_fm_fixture_is_valid() {
        let patch = crate::schema::tests::fixture_patch();
        let v = validate(&patch);
        assert!(v.is_valid(), "{:?}", v.errors);
    }
}
