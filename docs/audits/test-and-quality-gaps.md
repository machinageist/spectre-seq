<!--
Author: Jeff
Date: 2026-07-11
Description: Evidence-based assessment of Geist DAW test coverage and professional quality gaps
Notes: Passing tests are credited only for the paths and properties they directly exercise
-->

# Test and Quality Gaps

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** automated tests, realtime verification, failure/recovery coverage, compatibility, performance, UI QA, and release gates
- **Decision authority:** Jeff
- **Upstream sources:** `repository-baseline.md`, `runtime-reachability.md`, `architecture-drift.md`, Cargo targets, test source, legacy validation claims
- **Downstream dependents:** verification strategy, realtime verification, performance budgets, release gates, rebuild roadmap
- **Supersedes:** unqualified “green workspace” and test-count readiness claims
- **Superseded by:** none
- **Open decisions:** supported platforms, plugin fixture policy, benchmark hardware tiers, UI accessibility stack, project durability policy
- **Known gaps:** this audit did not run hardware audio/UI smoke tests, benchmarks, fuzzers, advisories, license scanning, or package installation

## Baseline gate results

| Gate | Result | What it proves | What it does not prove |
|---|---|---|---|
| Cargo metadata with lockfile | pass | active workspace and recorded dependency resolution load | declared nightly reproducibility, licensing, target-platform builds |
| formatting check | fail | formatting drift exists | functional correctness |
| workspace check, all targets/features | pass with warnings | active sources type-check on this Mac/toolchain | runtime behavior, declared nightly, other platforms |
| strict Clippy, all targets/features | fail | dead paths and a range-loop lint remain | absence of deeper design defects |
| workspace tests/all features | pass | current unit, integration, and doctest assertions pass | canonical runtime integration, hardware, recovery, usability, release readiness |
| ignored-test search | no ignored Rust tests found | no hidden `#[ignore]` Rust tests were discovered | existence of required real-plugin/hardware fixtures |

No benchmark result, application launch artifact, audio capture, UI recording, package artifact, or named-platform qualification report was recorded.

## Existing strengths worth preserving

The legacy suite contains useful narrow evidence:

- time and transport conversion tests;
- graph compilation, execution, feedback, and swap fixtures;
- deterministic DSP and synth behavior tests;
- fixed-engine scheduling, command, clip, session, effects, and mixer fixtures;
- a counting allocator test around a busy steady-state `SynthProcessor` block;
- bounded-ring and callback-size bridge tests;
- project serialization, malformed input, migration sequence, atomic-rename, autosave worker, and recovery-file discovery tests;
- UI model and geometry tests;
- WAV capture-ring and round-trip tests;
- VST3 path, descriptor, module-failure, and lifecycle helper tests.

These are salvageable evidence and fixture patterns. Their presence does not establish the missing cross-layer contracts below.

## Critical cross-layer gaps

### No canonical offline-to-live render fixture

The graph executor and fixed live processor are different paths. There is no fixture that loads one authoritative project, prepares one render plan, renders it offline, then exercises that same plan through the callback bridge and compares scheduling/audio semantics.

Required gate:

- deterministic project fixture;
- canonical graph compilation;
- offline render golden/property checks;
- live callback bridge using the same prepared plan;
- explicit tolerance and event-order assertions.

### No single-source project/UI/audio consistency test

The UI mutates local state and sends bounded commands whose failures are usually ignored. Persistence serializes UI-owned state rather than an acknowledged engine/project revision.

Required gate:

- command transaction accepted/rejected result;
- project revision and snapshot identity;
- forced queue saturation;
- proof that visible, saved, undo, and prepared-render states converge or report failure;
- reload into the same canonical model.

### No complete workflow tests

No automated or manual protocol proves empty project to sound, clip creation to arrangement, recording to saved/reloaded media, missing asset recovery, VST3 insertion/state recall, automation playback, export, or crash recovery.

Required gate: scenario fixtures with named preconditions, commands, resulting audio/state, persisted artifacts, and recovery behavior.

## Realtime verification gaps

### Allocation/deallocation coverage is incomplete

The counting allocator fixture covers steady-state `SynthProcessor::process_block`. It does not cover:

- the actual CPAL callback closure and all callback-size transitions;
- canonical graph plan swaps;
- old-plan/device reclamation;
- hosted VST3 nodes;
- queue saturation and error paths;
- stream reconfiguration;
- callback panic paths;
- teardown.

A concrete violation risk exists: `EngineSink::return_asset` may drop an `Arc` on the callback when its return ring is full (`control.rs:531-538`).

Required gate: allocation/deallocation guard over every callback-reachable success, overload, swap, error, and teardown path, with ownership-return saturation tests.

### Lock and blocking-I/O prohibitions are not enforced

The realtime document prohibits locks and I/O, but no lock interception, call-graph policy check, or runtime thread marker proves that callback-reachable code obeys those rules.

Required gate: callback-reachability inventory plus debug/runtime guards or model tests for prohibited synchronization and I/O boundaries.

### Queue overflow policy is unverified as product behavior

Command, event, scope, asset, and capture rings have bounded behavior, but overflow is often silently lossy. Tests do not establish which commands may be dropped, how state resynchronizes, how note-offs are protected, or what users see.

Required gate: capacity and overflow tests for each queue direction, including note-off/all-notes-off safety, command acknowledgement, telemetry, and ownership return.

### No deadline, jitter, or xrun qualification

No reproducible callback percentile/jitter benchmark exists by sample rate, buffer size, track/device load, or build profile. Xrun counting exists but forced-load behavior is not recorded.

Required gate: benchmark harness recording hardware, OS, sample rate, callback size, render quantum, fixture, build profile, percentile timing, jitter, deadline headroom, and xrun count.

### Numerical containment is incomplete

DSP tests cover selected finite/silence behavior, but there is no whole-plan policy test for NaN, Inf, denormals, runaway feedback, unstable plugin output, or panic containment.

Required gate: injected invalid numerical/plugin behavior with bounded containment, diagnostics outside the callback, and deterministic recovery.

## Time, event, and sequencing gaps

The current app scheduling is block-start accurate. Step events use offset zero, session transitions are processed at block boundaries, and synth `ProcessContext` receives no parameter-change events.

Missing tests include:

- half-open event intervals across arbitrary block boundaries;
- same-sample ordering across note, transport, automation, clip launch, and graph swap;
- loop-wrap note-off behavior;
- seek/stop/device-removal/project-close note flush;
- tempo and meter changes inside blocks;
- sample-rate changes;
- latency-compensated playback/record timestamps;
- MPE and future MIDI posture;
- external MIDI timestamp conversion and overflow.

Required gate: property tests over randomized block partitioning that produce invariant musical results independent of block size.

## Graph and device gaps

Graph tests do not currently prove:

- use by the application;
- channel-layout negotiation beyond current assumptions;
- buses, sends, returns, sidechains, monitoring taps, or resampling;
- latency propagation and compensation;
- stable device/parameter identity through reorder and migration;
- bypass, suspension, tail, silence, and reset contracts;
- offline/live semantic equivalence;
- bounded plan compilation diagnostics;
- off-callback reclamation under rapid swaps;
- plugin failure containment.

Required gate: project-command-to-compiled-plan integration fixtures and property/model tests for topology edits, swaps, latency, identity, and reclamation.

## Recording and asset gaps

Current tests prove capture-ring accumulation and float-WAV round trips. They do not prove professional recording safety.

Missing tests include:

- device/channel selection and loss;
- synchronized duplex input/output clocks;
- input and monitoring latency correction;
- punch, count-in, overdub, and dropout semantics;
- incremental disk writing and bounded buffering;
- disk-full and permission failures;
- periodic flush and durable close;
- crash salvage of an in-progress take;
- temp naming/collision handling;
- recording during save/autosave/render;
- project asset reference and reload;
- moved/missing/case-changed media repair;
- collected versus referenced asset behavior;
- hash verification and cache invalidation.

The Studio session schema currently omits audio-clip asset references, so a recorded clip cannot be proven to survive project reload.

Required gate: kill-and-recover recording fixtures plus project round trips with referenced and collected media.

## Project safety and compatibility gaps

Existing happy-path CBOR and temp-rename tests do not cover:

- file and parent-directory sync policy;
- interruption before/after temporary write and rename;
- preservation/rollback of the prior project;
- disk-full, quota, permission, and read-only volume behavior;
- autosave error reporting;
- generation retention and recovery choice;
- corrupt or partially migrated files;
- migration transaction rollback;
- newer-schema refusal and unknown-field preservation;
- plugin-state and missing-device envelopes;
- cross-platform path, case, and Unicode behavior;
- backwards compatibility fixture corpus;
- cross-platform project round trips;
- concurrent edit/save/autosave/render/recording snapshots.

Required gate: fault-injected filesystem tests and a versioned migration corpus independent of current serializer implementation.

## VST3 quality gaps

Crate-level VST3 tests do not establish host support. Missing verification includes:

- current official SDK/license provenance;
- scanner subprocess isolation, timeout, crash, and cache invalidation;
- redistributable or locally supplied real instrument/effect fixtures;
- component/controller lifecycle and thread rules;
- bus and speaker-arrangement negotiation;
- audio and event processing;
- sample-accurate parameter queues and ramps;
- transport/process context;
- state get/set ordering and project round trip;
- latency, tail, silence, bypass, programs, units, and restart notifications;
- missing/incompatible plugin placeholders;
- editor parenting, focus, scale, resize, close, and multi-monitor behavior;
- plugin hang/crash containment;
- named-platform fixture matrix.

Required gate: no VST3 support claim before a minimum instrument and effect pass the complete lifecycle and project-recall matrix on each qualified platform.

## Automation and modulation gaps

`geist-automation` tests are isolated and the live engine does not use the crate. Missing tests include:

- stable target identity through device edits and migration;
- base + automation + modulation + smoothing + clamping composition;
- sample-accurate and control-rate evaluation policy;
- manual override and re-enable;
- automation recording and undo;
- loop and transport discontinuities;
- per-note expression/MPE;
- persistence and missing-device behavior;
- visualization decimation independent from audio evaluation;
- block-partition invariance.

Required gate: an offline parameter-ramp render followed by the same canonical live-plan evaluation and project round trip.

## DSP verification gaps

Current DSP tests and benchmarks are useful but lack a repository-wide verification policy for:

- tolerance selection and numerical rationale;
- exact golden versus spectral/property testing;
- neutral-operation null tests;
- sample-rate and block-size invariance;
- oversampling and alias-rejection targets;
- reset and discontinuity behavior;
- deterministic random seeds;
- denormal policy;
- quality-mode equivalence contracts;
- long-run stability and accumulated error;
- standard reference-signal fixtures.

Required gate: per-device verification sheets linking algorithm claims, test method, tolerances, sample rates, quality settings, and benchmark fixtures.

## UI and accessibility gaps

Model tests do not prove the visible application is professionally usable. Missing evidence includes:

- keyboard-only completion of core workflows;
- focus and selection preservation across Arrange/Session/Mix/Build/Shape/Browser lenses;
- text-entry versus musical-typing shortcut conflicts;
- command search and remapping;
- screen-reader labels and accessibility tree;
- contrast and non-color status cues;
- reduced motion and scalable text;
- high-DPI and multi-monitor behavior;
- plugin editor focus/scale/lifecycle;
- touch/trackpad/controller behavior;
- empty/loading/missing/invalid/overload/recovery states;
- frame-time budgets under meters/scopes/waveforms;
- usability budgets for first sound, variation, split, duplicate, quantize, routing, and recovery.

Required gate: versioned scenario protocols and named-platform recordings/results, not screenshot inspection alone.

## Fuzzing, property testing, and concurrency gaps

No active fuzz targets were found. Property-style coverage is incomplete for the highest-risk invariants.

Required targets:

- project parser and migration envelopes;
- MIDI/event ingestion;
- plugin metadata/state envelopes;
- graph commands and compilation;
- tempo/sample/beat conversions;
- block partitioning and same-sample ordering;
- undo/redo transaction algebra;
- queue/swap/shutdown ownership models;
- save/autosave/recording concurrency.

## Platform, packaging, and stewardship gaps

No CI configuration, license, contributor policy, security policy, installer, signing workflow, or release matrix was found in the initial inventory. `xtask package-release` runs a release build and stages an executable/README/license if present; it is not a complete package or qualification system.

Missing gates include:

- pinned toolchain installation and reproducibility;
- macOS/Linux/Windows compile and runtime matrix;
- supported CPU architecture matrix;
- package/install/uninstall smoke tests;
- codesign/notarization or platform equivalents;
- permissions, sandbox, plugin-path, and user-data behavior;
- dependency advisory and license scans;
- bundled asset/font/sample provenance;
- security reporting and release process;
- reproducible generated assets and source availability.

## Quality-gate policy for the rebuild

1. Formatting, strict lint, targeted tests, and appropriate workspace tests MUST pass for attributable rebuild files.
2. A passing unit suite MUST NOT advance subsystem maturity beyond `unit-tested in isolation`.
3. Integration status requires the canonical active path and a recorded fixture.
4. Verification requires acceptance evidence linked to requirement IDs and the recorded environment.
5. Manual QA MUST name platform, hardware, build, project fixture, procedure, and result.
6. Performance claims MUST include reproducible benchmark metadata and budgets.
7. Project safety and recording milestones MUST include fault injection and recovery drills.
8. VST3 claims MUST name real fixtures and lifecycle coverage.
9. Release qualification MUST include packaging, installation, compatibility, security, and documentation gates.
10. Test counts MUST remain informational only; they are never a readiness metric.
