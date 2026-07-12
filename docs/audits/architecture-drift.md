<!--
Author: Jeff
Date: 2026-07-11
Description: Comparison of legacy architectural claims with the active Geist DAW runtime
Notes: Treats plans and ADRs as evidence; runtime source and recorded verification control maturity claims
-->

# Architecture Drift

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** legacy architecture and planning claims compared with active code reachability
- **Decision authority:** Jeff
- **Upstream sources:** `INITIAL_PLAN.md`, `PRODUCTION_PLAN.md`, `PROPOSED_FILE_TREE.md`, `docs/architecture.md`, `docs/realtime_rules.md`, ADRs 001–004, `runtime-reachability.md`, `repository-baseline.md`
- **Downstream dependents:** `legacy-lessons.md`, `reuse-disposition.md`, requirements and architecture contracts
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** active-workspace migration strategy, canonical graph/device model, project format, UI stack, VST3 bindings/license posture
- **Known gaps:** remaining handoff/spec documents and historical revisions still require full classification

## Executive finding

The repository contains useful isolated implementations, but its declared architecture is not the active architecture. The live DAW is a fixed-track application engine whose UI, audio state, and persistence model grew together. Separately tested graph, automation, modular, stacksynth, project-safety, and VST3 components were described as implementation progress without being adopted by the canonical runtime.

This is architectural drift, not merely unfinished breadth. Multiple documents describe intended dependency and ownership boundaries as current facts while the application continues through a different path.

## Claim-to-runtime comparison

| Intended or claimed architecture | Evidence in documents | Active evidence | Finding |
|---|---|---|---|
| compiled graph is the DAW render architecture | `docs/architecture.md:16-29`; `INITIAL_PLAN.md:118-132` | `init.rs:41-74` installs `SynthProcessor`; no graph construction | graph exists only as isolated library/test architecture |
| UI is command/snapshot driven and owns no engine truth | `PROPOSED_FILE_TREE.md:160-167`; `docs/architecture.md:25-29` | `studio.rs` owns editable session/rack/timeline/mixer state, then mirrors commands; `session.rs` serializes UI-owned `StudioSession` | authority is duplicated and reconciliation is implicit |
| project crate supplies versioned schema, migration, assets, autosave | `docs/architecture.md:22`; `INITIAL_PLAN.md:88-101` | manual save/load uses app-specific synthetic node parameters; autosaver/recovery are never instantiated | primitives exist, but project lifecycle architecture is disconnected |
| automation/modulation is owned by dedicated crate | `docs/architecture.md:21`; `INITIAL_PLAN.md:35` | app does not depend on `geist-automation`; fixed track uses local synth modulation | architecture claim is not live |
| VST3 plugins adapt into internal graph/device nodes | ADR 001:39-45; `docs/architecture.md:41-43` | host crate is not an app dependency; wrapper has null event/parameter/context queues | boundary prototype exists; host lifecycle and user workflow are absent |
| native devices share a common internal device abstraction | `INITIAL_PLAN.md:50-63`, 94-105 | fixed engine owns concrete synth and app-local effects directly | abstraction is not canonical in the live engine |
| modular routing is a core sound-flow surface | `INITIAL_PLAN.md:185-195`; production P6 | graph UI/model and modular crate are disconnected from rendered audio | visible representation can diverge from actual sound flow |
| realtime path consumes precompiled state | `docs/realtime_rules.md:11-24`; `docs/architecture.md:27-29` | fixed processor mutates broad app-local state directly from command traffic | bounded behavior has tests, but “precompiled state” overstates the actual model |
| autosave and crash recovery are project capabilities | `docs/architecture.md:22`; production P8 acknowledges future work | library worker exists but app never starts it; recording remains memory-only until stop | project-safety language is premature |
| implementation plan phases represent integrated progress | `INITIAL_PLAN.md:88-205` | production plan itself admits fixed graph, scaffold VST3, dormant automation at lines 29-32 | status vocabulary collapses isolated and integrated maturity |

## Verified drift hypotheses

### Fixed engine bypasses the graph — confirmed

`init::start` builds `Vec<Track>` and `SynthProcessor`, then gives that processor to `BlockBridge`. No editable graph, compiler, process-list executor, or graph swap enters startup. The graph tests prove graph code behavior, not DAW render integration.

Impact: routing, device lifecycle, latency, sidechain, feedback, offline rendering, and graph UI cannot share one enforceable contract.

### VST3 is not a functioning DAW host — confirmed

The host crate includes discovery, raw lifecycle work, and a graph node wrapper. The application never invokes it. The wrapper passes null event and parameter queues, exposes no editor, and has no state/persistence/missing-plugin path.

Impact: wording such as “supported external plugin format” must mean architectural target, not supported user capability.

### General automation is disconnected — confirmed

`geist-automation` is absent from the app dependency graph. App-local LFO modulation does not satisfy automation ownership, parameter identity, recording, persistence, or sample-accurate evaluation.

### UI models exceed their authority contract — confirmed

`StudioApp` maintains editable project-like state and emits engine commands. `StudioSession` is reconstructed from that UI state for serialization. There is no authoritative project command model from which both render state and UI snapshots derive.

Impact: save/reload, undo, callback state, and visible state can disagree after partial command delivery or failure.

### Persistence is a bounded demo mapping — confirmed

Session persistence stores global and per-track values as numeric parameters on a synthetic `geist-macros` node, plus reserved clip-ID ranges. Atomic CBOR helpers and migration tests do not make this mapping a robust DAW project contract.

### Realtime enforcement is narrow — confirmed

The allocation guard exercises `SynthProcessor::process_block` in a controlled test. It does not cover CPAL callback adaptation, graph swaps and reclamation, hosted plugins, recording input, shutdown, or future canonical graph/device paths. No lock guard, callback deadline benchmark, or runtime debug boundary is active.

### Multiple synth architectures lack product disposition — confirmed

The fixed app uses `geist-synth`. `geist-stacksynth` and `geist-modular` are active workspace crates but not app dependencies. The plans call `geist-synth` flagship while later work starts another synth architecture without a documented relationship or reuse decision.

### Historical plugin architectures remain structurally visible — confirmed

CLAP and LV2 crates remain in the repository and ADR 001 retains a filename that says “clap over vst” while its content says VST3-only. They are excluded, but filenames, docs, and source trees still create ambiguity for contributors and tooling.

### “Green workspace” was not a readiness gate — confirmed

`PRODUCTION_PLAN.md:21-32` cites 586 green tests and release builds while acknowledging the live graph, VST3, and automation gaps. The 2026-07-11 baseline found formatting failure and strict Clippy failure despite all tests passing. No named-platform GUI/audio QA, plugin fixture matrix, stress test, or release qualification evidence was found.

## ADR audit

### ADR 001 — decision direction retained; acceptance suspended pending current evidence

The VST3-only boundary and internal-native-device separation agree with Jeff’s current constraints. However:

- time-sensitive SDK 3.8.0 licensing claims require current official Steinberg verification;
- the exact `vst3 0.3.0` crate/license and generated-binding relationship require dependency review;
- “validated against real `.vst3` binaries on a dev box” has no linked result;
- opaque plugin-state support is asserted against ADR 003, which is not an accepted decision record.

Disposition: preserve the product decision, rewrite the ADR after source/licensing review.

### ADRs 002–004 — not decisions

These files contain generic pseudocode checklists and no context, alternatives, consequences, validation, or reconsideration conditions. Their filenames must not confer acceptance on ArcSwap, CBOR, egui, or a wgpu migration.

Disposition: mark superseded/archive after the new decision register exists; re-decide each subject independently.

## Documentation authority drift

Current documents assign authority to each other circularly:

- `CLAUDE.md` points to `INITIAL_PLAN.md` phase order and `PROPOSED_FILE_TREE.md` architecture;
- `docs/architecture.md` points back to those files and `HANDOFF.md`;
- `PRODUCTION_PLAN.md` treats `INITIAL_PLAN.md` as completed starting position;
- `HANDOFF.md` is both an activity log and architectural evidence;
- source headers use “Implemented” independently of repository-wide maturity gates.

No document declares a complete status metadata contract, requirement provenance, or verification link chain. The rebuild must replace this circle with one authority per decision class.

## Rebuild prevention constraints derived from drift

1. The active application MUST render through the same canonical prepared plan exercised by offline and integration tests.
2. UI, persistence, and render preparation MUST derive from one authoritative project model through typed commands.
3. A crate MUST NOT receive “integrated” status until an active application path exercises it.
4. A plugin format MUST NOT be advertised as supported until scanning, instantiation, processing, state, missing-plugin behavior, and editor/user workflow gates pass.
5. Realtime verification MUST cover the entire callback-reachable graph and reclamation path, not one processor fixture.
6. Project safety MUST be demonstrated through failure and recovery drills, not helper-unit tests.
7. ADR status MUST be determined by decision content and current evidence, never filename or sequence number.
8. Synth/device experiments MUST receive explicit product and reuse dispositions before entering the active workspace.
