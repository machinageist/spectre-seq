<!--
Author: Jeff
Date: 2026-07-11
Description: Evidence-based reuse disposition for major legacy Geist DAW components
Notes: Reuse permission is narrower than conceptual value; no component enters the rebuild without satisfying its stated gate
-->

# Reuse Disposition

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** major legacy crates, application paths, tests, documents, assets, and build infrastructure
- **Decision authority:** Jeff
- **Upstream sources:** all Phase A audits, Cargo metadata, active runtime trace, legacy document/reference audit
- **Downstream dependents:** active-workspace migration, crate boundaries, rebuild roadmap, R0/R1 implementation plans
- **Supersedes:** implicit reuse assumptions in `INITIAL_PLAN.md`, `PROPOSED_FILE_TREE.md`, and `PRODUCTION_PLAN.md`
- **Superseded by:** accepted implementation-specific reuse decisions linked from traceability
- **Open decisions:** workspace migration boundary, project encoding, UI stack, platform matrix, VST3 bindings/license, flagship synth relationship
- **Known gaps:** no benchmark qualification, cross-platform validation, current dependency-license audit, or clean-room source acceptance has been completed

## Disposition vocabulary

- **Preserve as-is:** contracts, implementation, and tests already fit the rebuild. Only mechanical namespace/build integration may change.
- **Preserve concept, rewrite implementation:** the responsibility or algorithmic idea remains useful, but ownership, API, failure behavior, or integration does not meet the new contract.
- **Salvage tests/fixtures only:** current production implementation must not enter the active rebuild; selected tests, signals, or edge cases may become independent acceptance evidence.
- **Archive for reference:** retain through Git/docs archive for historical or forensic value, but do not compile or treat as authority.
- **Remove from active architecture:** keep in Git history if needed, but exclude from the active workspace, dependency graph, docs authority, packaging, and feature work.
- **Undecided pending evidence:** a material choice or missing verification prevents a responsible reuse decision.

A disposition does not itself authorize copying code. Before reuse, the target requirement and architecture contract must exist, licenses must be compatible, and the component must pass the gate recorded here.

## Summary

No major production subsystem is approved **preserve as-is** at this stage. This is deliberate: the canonical ownership, timing, persistence, failure, and verification contracts do not yet exist. Some dependency-light algorithms may later earn as-is reuse after contract tests, but Phase A evidence is insufficient to grant that status now.

| Component | Disposition | Primary reason |
|---|---|---|
| fixed `geist-daw` application/runtime | remove from active architecture | canonical-path bypass and duplicated authority |
| app fixed sequencing/session/mixer engine | salvage tests/fixtures only | useful behavior cases, unsuitable ownership/timing architecture |
| `geist-core` | preserve concept, rewrite implementation | useful primitives, but stable identity/time/event contracts need redefinition |
| `geist-timeline` | preserve concept, rewrite implementation | useful tempo/transport logic; incomplete timing semantics |
| `geist-graph` | preserve concept, rewrite implementation | strongest candidate, but disconnected and contract-incomplete |
| `geist-audio-backend` | preserve concept, rewrite implementation | useful CPAL/reblocking patterns; missing duplex/reconfiguration/shutdown contract |
| `geist-dsp` | undecided pending evidence | algorithm-level decisions require per-module verification/license review |
| `geist-synth` | salvage tests/fixtures only | fixed-engine instrument, unclear long-term product role |
| `geist-fx` and app-local FX | salvage tests/fixtures only | DSP cases useful; lifecycle/parameter/device contracts incomplete |
| `geist-stacksynth` | archive for reference | disconnected alternate synth architecture |
| `geist-modular` | preserve concept, rewrite implementation | product identity relevant; current crate disconnected and limits unapproved |
| `geist-automation` | preserve concept, rewrite implementation | useful curves/evaluator ideas; no live parameter contract |
| `geist-project` | preserve concept, rewrite implementation | happy-path primitives useful; project authority/durability unresolved |
| `geist-config` | preserve concept, rewrite implementation | declarative profiles valuable; command/security/versioning contract absent |
| `geist-ui` | salvage tests/fixtures only | geometry/model cases useful; product interaction/accessibility unverified |
| `geist-vst-host` | undecided pending evidence | current bindings/license and lifecycle approach require official review |
| CLAP/LV2 host crates | remove from active architecture | conflict with VST3-only product constraint |
| legacy docs/plans/handoffs/ADRs | archive for reference | contradictory historical evidence, not current authority |
| clean-room dossiers | preserve concept, rewrite implementation | source discovery useful; claim-level provenance/requirements unsafe |
| test suite | salvage tests/fixtures only | narrow evidence useful; tied to legacy self-consistency |
| benchmarks | salvage tests/fixtures only | workloads useful; no accepted budgets or metadata policy |
| `xtask` | preserve concept, rewrite implementation | useful command entrypoint; packaging/quality behavior incomplete |
| assets/content | undecided pending evidence | existence, licensing, provenance, and product role not established |

## Application and runtime

### `app/geist-daw` fixed runtime

**Disposition:** remove from active architecture.

**Preserve:** Git history and runtime audit as forensic evidence. Selected workflow fixtures may be recreated independently.

**Do not carry forward:** `SynthProcessor` as the canonical engine, direct concrete-device ownership, fixed track/scene/event limits, UI-to-audio lossy state synchronization, synthetic persistence mapping, headless infinite loop, and callback-side ownership-drop behavior.

**Evidence:** startup installs `SynthProcessor` rather than a compiled graph; general automation/VST3/modular systems are absent; UI, callback, and serialized state have no common authority; timing is block-start accurate; `return_asset` may deallocate on the callback.

**Reconsideration gate:** none. Individual algorithms can be dispositioned separately, but this integration architecture must not become the rebuild shell.

### App sequencing, arrangement, session launcher, mixer, and audio-clip code

**Disposition:** salvage tests/fixtures only.

**Preserve:** behavioral edge cases for half-open clip windows, session quantization, mute/solo, mixing, asset playback, and bounded-capacity stress where those expectations survive requirements review.

**Do not carry forward:** fixed capacities as product limits, offset-zero scheduling, additive precedence by incidental call order, numeric command sprawl, or app-local ownership structures.

**Reconsideration gate:** convert each retained case into a requirement-linked black-box scenario against the new time/event/render contracts.

### App recorder and WAV writer

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** lock-free capture handoff concept and WAV round-trip fixture.

**Rewrite:** synchronized timestamp model, bounded disk writer, pre-created temporary recording, periodic flush, dropout markers, failure telemetry, crash salvage, asset transaction, and shutdown behavior.

**Reconsideration gate:** kill-and-recover fixture, disk-full/permission tests, latency-correction tests, and project reload of recorded media.

## Foundational crates

### `geist-core`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** dependency-light ownership goal; explicit newtypes; typed IDs, ports, events, parameter ranges, audio configuration, and borrowed process-context concepts.

**Rewrite/re-prove:** stable ID generation and persistence semantics; time rounding/overflow; event ordering; parameter identity/display/smoothing; channel/bus model; error policy; separation among project, prepared, and realtime types.

**Gate:** accepted R1 contracts plus property tests for identity, conversion boundaries, ordering, serialization compatibility, and invalid values. Reuse source only after symbol-by-symbol review.

### `geist-timeline`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** tempo-point lookup and transport-state test ideas.

**Rewrite/re-prove:** sample/beat mapping at tempo boundaries, meter maps, half-open intervals, loops/seeks, same-sample ordering, block-partition invariance, recording timestamps, and latency compensation.

**Gate:** randomized property tests proving identical event results across block partitionings and documented rounding rules.

### `geist-graph`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** editable-versus-compiled separation, topology validation, preplanned buffers, explicit feedback delay concept, executor fixture patterns, and off-thread swap/reclamation goal.

**Rewrite/re-prove:** ownership, typed audio/event/control buses, channel negotiation, aliasing, fan-in/out, sends/returns/sidechains, latency propagation, diagnostics, plan revisioning, reclamation under saturation, and offline/live identity.

**Why not preserve as-is:** it has never been the active runtime and its current tests do not cover required DAW graph semantics.

**Gate:** one requirement-traced offline vertical slice using the graph as the only renderer, then callback use of the identical prepared plan with allocation/deallocation guards.

### `geist-audio-backend`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** backend isolation, CPAL discovery concept, callback-size bridge tests, xrun counter idea, and bounded capture ring pattern.

**Rewrite/re-prove:** duplex stream model, input/output clock relationship, device selection/reconfiguration, format/channel negotiation, render-quantum policy, thread priority, error callbacks, orderly shutdown, and callback panic containment.

**Gate:** fake-backend deterministic tests plus named-platform device open/reconfigure/shutdown protocols and deadline measurements.

## DSP and native devices

### `geist-dsp`

**Disposition:** undecided pending evidence at module level.

A crate-wide disposition would be too coarse. Standard algorithms may be reusable, but each oscillator, envelope, filter, effect, math helper, random source, and SIMD path needs separate evidence.

**Potentially salvage:** deterministic signal fixtures, finite/silence properties, benchmark workloads, and standard literature references where present.

**Required review:** provenance/originality, numerical method, sample-rate behavior, reset/discontinuity, aliasing, oversampling, denormals, deterministic seeds, tolerance rationale, and long-run stability.

**Gate:** per-module DSP verification sheet. Modules that pass may move to preserve as-is or preserve concept/rewrite independently.

### `geist-synth`

**Disposition:** salvage tests/fixtures only.

**Preserve:** voice-allocation cases, note lifecycle scenarios, oscillator/filter/envelope signal fixtures, and early-slice requirements that can be expressed independently.

**Do not assume:** flagship status, current parameter IDs, unison/FM architecture, voice limits, fixed-engine node wrapper, or preset compatibility.

**Gate:** product decision defining the relationship among the small R4 synth, flagship synth, stacksynth, and modular system; accepted device/parameter/voice contracts.

### `geist-fx` and app-local FX chain

**Disposition:** salvage tests/fixtures only.

**Preserve:** neutral/silence/finite behavior cases and selected DSP reference signals.

**Rewrite/re-prove:** common device lifecycle, channel layouts, latency/tail, bypass/suspension, stable parameter descriptors, automation/modulation, quality modes, state migration, and realtime ownership.

**Gate:** per-effect DSP sheet plus one canonical graph device integration fixture.

### `geist-stacksynth`

**Disposition:** archive for reference.

**Reason:** active-workspace alternate synth development with no application dependency or approved product relationship. Continuing it before foundations would repeat speculative breadth.

**Gate to change disposition:** explicit flagship architecture decision and module-level DSP audit showing unique reusable value.

### `geist-modular`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** original Geist goal of visible, typed modular sound flow and selected pure utility-node test cases.

**Do not preserve automatically:** VCV-derived voltages, polyphony lanes, module limits, port semantics, or fixed capacities without Geist requirements.

**Gate:** accepted modular signal/port/polyphony/feedback contracts and original UI interaction requirements, followed by graph-integrated offline fixtures.

## Automation and configuration

### `geist-automation`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** curve interpolation, lane evaluation, and modulation-composition test ideas.

**Rewrite/re-prove:** stable target binding, sample/control-rate distinction, base/automation/modulation/smoothing composition, manual override, loop/discontinuity behavior, undo, recording, persistence, MPE, and graph/device integration.

**Gate:** parameter-ramp offline render through the canonical device model, block-partition invariance, and project round trip.

### `geist-config`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** declarative workflow profiles, keybinding data, command palette, and template concepts.

**Rewrite/re-prove:** versioning, validation, command ontology binding, scope/conflict handling, accessibility, platform layouts, migration, untrusted input, and prohibition on arbitrary code execution or command bypass.

**Gate:** accepted command ontology and security/configuration contracts with malformed/untrusted configuration tests.

## Project and assets

### `geist-project`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** version-envelope concept, typed error direction, malformed-input tests, content-hash idea, migration registry concept, same-filesystem temporary-write pattern, and recovery discovery tests.

**Do not preserve as decisions:** CBOR, current schema, synthetic node parameter mapping, current migration contract, one-file layout, current stable IDs, or opaque plugin-state assumptions. ADR 003 is not a decision.

**Rewrite/re-prove:** authoritative model snapshot, bundle/encoding choice, unknown/newer data policy, transaction and rollback, fsync/directory sync, autosave generations, asset references/collection, missing media, recording salvage, plugin placeholders/state, concurrency, and cross-platform paths.

**Gate:** accepted project-format ADR and failure-injected round-trip/migration/recovery corpus. The encoding choice requires Jeff approval because it creates long-term compatibility.

### Recorded media and asset cache

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** content hashing and disposable analysis-cache separation as candidate concepts.

**Rewrite:** ownership, relative paths, collection, hash policy, source-versus-cache distinction, case/Unicode behavior, missing media repair, recording temp files, waveform analysis, streaming, and archive/export.

**Gate:** project lifecycle and asset architecture contracts plus moved/missing/corrupt-media fixtures.

## UI

### `geist-ui` models/widgets and app GUI

**Disposition:** salvage tests/fixtures only.

**Preserve:** selected geometry, hit-testing, bounded visualization, and deterministic rendering fixtures after confirming they express original Geist interaction rather than copied composition.

**Do not preserve automatically:** screen layout, view hierarchy, Ableton-like composition, duplicated state models, widget APIs, hard-coded shortcuts, color choices, or claims of professional usability.

**Gate:** field-workflow research, accepted command ontology, focus/selection state machines, accessibility baseline, and prototype usability evidence. Only then review widgets individually.

### Arrange/Build/Shape/Mix/Browser/Modulation lens concept

**Disposition:** preserve concept, rewrite implementation.

This is a Geist product principle rather than reusable code. The lenses must become synchronized views over one project authority, with explicit selection/focus and command semantics.

**Gate:** accepted product/UI requirements and scenario prototypes.

## VST3 and legacy plugin hosts

### `geist-vst-host`

**Disposition:** undecided pending evidence.

**Potentially preserve:** strict host-boundary concept, bundle-path discovery tests, descriptor parsing cases, and concentration of unsafe FFI.

**Reasons to defer:** current official VST3 SDK/license status has not been verified; exact `vst3 0.3.0` binding provenance and maintenance posture are unaudited; no app integration or fixture matrix exists; process adapter omits event/parameter/context behavior; editor/state/latency/failure containment are absent.

**Gate:** official Steinberg source/license review, dependency/license/security audit, alternatives analysis, and a throwaway real-fixture spike. This decision materially affects licensing and compatibility and requires Jeff approval before acceptance.

### `geist-clap-host` and `geist-lv2-host`

**Disposition:** remove from active architecture.

**Action:** keep available through Git history or an explicitly non-active archive if archaeology is needed. Exclude from workspace, documentation authority, tests, packaging, and future implementation.

**Reason:** active third-party plugin support is VST3-only by Jeff’s current product constraint. No additional product decision is needed to prevent active reuse.

## Tests, fixtures, benchmarks, and tools

### Legacy unit/integration tests

**Disposition:** salvage tests/fixtures only.

**Preserve candidates:** pure conversion properties, standard DSP signals, graph topology cases, allocator methodology, queue saturation cases, serialization garbage, migration sequences, geometry/hit testing, and WAV round trips.

**Rule:** do not copy tests that merely encode accidental constants or implementation-specific structures. Rewrite retained tests against requirement-visible behavior and independent oracles.

**Gate:** each migrated test links to a requirement ID and states why its oracle is independent of the implementation.

### Benchmarks

**Disposition:** salvage tests/fixtures only.

**Preserve:** representative DSP and graph workloads where they remain relevant.

**Rewrite:** metadata capture, hardware/OS/build profile, quality settings, statistical method, baselines, budgets, and regression thresholds.

**Gate:** accepted performance budgets and reproducible benchmark protocol.

### Allocation guard

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** counting allocation, deallocation, and reallocation inside a marked scope.

**Rewrite:** canonical callback reachability, nested/thread semantics, graph swap/reclamation, saturation/error paths, plugin fixtures, and teardown coverage. Add lock/blocking guards where feasible.

**Gate:** active render plan passes the full realtime-verification matrix.

### `xtask`

**Disposition:** preserve concept, rewrite implementation.

**Preserve:** one repository-owned task entrypoint for checks, benchmarks, packaging, and fixture management.

**Rewrite:** command contracts, locked toolchain behavior, CI parity, artifact manifests, license/assets checks, package/install smoke tests, and reproducible metadata.

**Gate:** accepted R0 quality/tooling requirements and CI matrix.

## Documentation and research

### Root plans, handoff, old mega prompt, architecture summaries, UI notes, plugin notes

**Disposition:** archive for reference.

**Action:** preserve in Git, then move to `docs/archive/` or replace with short supersession pointers only after the documentation migration map exists. Do not silently rewrite historical claims.

**Reason:** these files contain useful intent and chronology but contradictory status, circular authority, stale decisions, and unsupported readiness claims.

### ADR 001

**Disposition:** preserve concept, rewrite implementation/document.

Preserve VST3-only external hosting and native-device isolation as current product constraints. Rebuild the ADR with official current sources, alternatives, consequences, validation, and reconsideration conditions. Do not retain unsupported validation or licensing claims.

### ADRs 002–004

**Disposition:** archive for reference.

They are pseudocode checklists, not decisions. ArcSwap, CBOR, egui, and wgpu remain undecided until proper ADRs are accepted.

### Existing clean-room specifications

**Disposition:** preserve concept, rewrite implementation/document.

**Preserve:** URL discovery, dated source tables, exclusions, coverage headings, and explicit source gaps after link/version revalidation.

**Rewrite:** claim-level provenance, source versions/sections, observed-versus-requirement-versus-implementation separation, requirement IDs, scenario criteria, and acceptance review. Remove direct promotion of vendor limits into Geist commitments.

**Gate:** the clean-room completion gate defined by Jeff’s mandate.

### `clean-room-spec-audit-metrics.json`

**Disposition:** archive for reference.

It may remain historical corpus metadata but must not determine correctness, completeness, requirement approval, or readiness.

## Assets and third-party content

### Fonts, icons, presets, samples, wavetables, impulse responses, fixture plugins

**Disposition:** undecided pending evidence.

The initial tree proposes assets, but provenance, licenses, source availability, redistribution rights, generation process, and product role have not been established.

**Gate:** machine-readable asset manifest with origin, author, license, modification/generation source, hashes, redistribution status, and test/product usage. Vendor factory content is excluded.

## Migration rules

1. The new active workspace MUST start minimal; legacy crates remain outside it until a requirement-traced slice needs them.
2. Reuse MUST happen symbol-by-symbol or fixture-by-fixture, never by copying an entire crate under a new name.
3. Every reuse change MUST cite this disposition, target requirement IDs, architecture contract, and verification evidence.
4. `Preserve concept, rewrite implementation` permits learning from responsibilities and tests, not importing accidental APIs or ownership.
5. `Salvage tests/fixtures only` prohibits production-source reuse unless this document is amended with new evidence.
6. `Archive for reference` material MUST NOT be imported by active build, generated docs, or contributor instruction paths.
7. `Remove from active architecture` material MUST NOT receive new feature work.
8. `Undecided pending evidence` MUST remain outside the active foundation until its decision gate closes.
9. No reused callback code may enter R2/R3 without allocation, deallocation, lock, bounded-work, and reclamation verification.
10. No reused persistent type or identifier may enter R1 without an accepted compatibility and migration policy.

## Phase A reuse conclusion

The legacy repository is most valuable as:

- forensic evidence of architectural drift;
- a source of independently rewritten tests and edge cases;
- a collection of concepts to evaluate against new contracts;
- proof that several algorithms can work in isolation.

It is not a suitable active foundation as a whole. The rebuild should not rename or rearrange the existing workspace and call that a clean start. The next phase must establish document authority, requirement IDs, decision gates, and architecture contracts before any legacy component is admitted into the new active workspace.
