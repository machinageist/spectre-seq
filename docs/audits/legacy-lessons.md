<!--
Author: Jeff
Date: 2026-07-11
Description: Evidence-based lessons and prevention rules for the Geist DAW rebuild
Notes: Candidly records failed patterns without discarding useful legacy evidence
-->

# Legacy Lessons

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** recurring planning, architecture, implementation, and verification failure patterns
- **Decision authority:** Jeff
- **Upstream sources:** `architecture-drift.md`, `runtime-reachability.md`, `repository-baseline.md`, legacy plans/ADRs/status files
- **Downstream dependents:** requirements policy, architecture contracts, verification strategy, rebuild roadmap, contributor guidance
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** final reuse disposition and migration boundary
- **Known gaps:** full clean-room and UI field-research audits are not yet incorporated

## How to use this document

Each lesson identifies evidence, impact, and a prevention rule. These rules constrain the rebuild; they are not claims that every legacy component is unusable.

## 1. Architecture was declared before it became the runtime

**Evidence:** `docs/architecture.md:4-6` calls itself current architecture. It assigns rendering to `geist-graph`, automation to `geist-automation`, and project safety to `geist-project`. Startup instead installs fixed `SynthProcessor`; the app has no automation or VST3 dependency, and autosave is disconnected.

**Impact:** contributors can implement against a documented system that does not exist, while tests continue passing on parallel paths.

**Prevention rule:** architecture documents MUST identify active, proposed, and test-only paths separately. A contract becomes implemented only when traceability links it to the canonical runtime and passing acceptance evidence.

## 2. Compilation and unit tests were used as integration status

**Evidence:** `INITIAL_PLAN.md` labels graph “implemented and tested,” sequencing “implemented,” synth/effects “implemented,” and the first vertical slice satisfied. `PRODUCTION_PLAN.md:29-32` simultaneously admits graph, VST3, and automation are disconnected.

**Impact:** maturity inflation hides the work needed to produce a dependable user workflow.

**Prevention rule:** status MUST use the mandated maturity scale. Unit-tested crates remain `unit-tested in isolation` until an active end-to-end path is exercised.

## 3. A demo engine bypassed the canonical graph

**Evidence:** `init.rs` creates fixed tracks and `SynthProcessor`; `geist-graph` is never constructed. The node UI and graph tests therefore represent a different system from audible routing.

**Impact:** routing edits, latency, feedback, device lifecycle, and visible signal flow can never be trusted as one system.

**Prevention rule:** the first render slice MUST use one canonical prepared render plan for offline fixtures and later live callbacks. No second “temporary” engine may become the product path.

## 4. State authority was duplicated across UI, audio, and persistence

**Evidence:** `StudioApp` owns mutable project-like state, sends lossy bounded commands to `SynthProcessor`, and serializes a separately reconstructed `StudioSession`.

**Impact:** dropped commands or partial failures can make the saved project, audible state, undo state, and visible state disagree.

**Prevention rule:** one authoritative project model MUST accept validated typed commands. UI renders snapshots; render state is prepared from committed project revisions; serialization snapshots that same authority.

## 5. Broad modules accumulated unrelated ownership

**Evidence:** `app/geist-daw/src/engine.rs` owns tracks, synth patches, sequencing, session launching, arrangement, modulation, assets, mixing, metering, command handling, and callback processing. `studio.rs` similarly combines UI, editing, persistence coordination, recording, and engine synchronization.

**Impact:** thread boundaries and invariants become implicit, tests require large fixtures, and safe replacement becomes difficult.

**Prevention rule:** modules MUST be divided by ownership and lifecycle, not screen or convenience. Cross-thread and persistence seams require named contracts and bounded data types.

## 6. Speculative abstractions were created without a proving slice

**Evidence:** graph, automation, device, modular, stacksynth, and VST3 abstractions accumulated independently before one canonical workflow used them together.

**Impact:** incompatible assumptions survive because each crate agrees only with its own tests.

**Prevention rule:** add abstractions only when a current vertical slice requires the seam. Each abstraction MUST ship with an active caller and acceptance fixture.

## 7. TODOs and decision gates lived inside “implemented” plans

**Evidence:** `INITIAL_PLAN.md` labels phases implemented while listing fundamental next refinements. `PRODUCTION_PLAN.md` contains unresolved Jeff gates for reverb, graph adoption, plugin QA, and architecture while describing a verified production-readiness starting position.

**Impact:** unresolved product and architecture decisions become invisible prerequisites.

**Prevention rule:** unresolved material choices belong in `decision-gates.md`; affected requirements remain blocked or proposed. Accepted documents may have gaps, but implemented status may not conceal them.

## 8. Placeholder or model-tested UI was treated as product UI

**Evidence:** `INITIAL_PLAN.md:161-170` calls the UI implemented beyond placeholder level based on shells, models, and interaction tests. No named-platform accessibility, focus, high-DPI, plugin-window, keyboard-only, or workflow QA evidence exists.

**Impact:** surface count substitutes for usability, and musician workflows remain unproven.

**Prevention rule:** UI maturity requires scenario-based interaction evidence, accessibility checks, and named-platform manual protocols in addition to deterministic model tests.

## 9. Constants encoded product behavior without a product contract

**Evidence:** fixed track counts, scene counts, reserved clip-ID ranges, fixed four-beat session slots, numeric parameter blocks, and fixed callback quantum appear in app code and persistence mappings.

**Impact:** prototype limits leak into project compatibility and user workflows without rationale or migration strategy.

**Prevention rule:** every externally observable limit MUST have a requirement or engineering-budget rationale. Persistent IDs and ranges MUST be typed, namespaced, versioned, and migration-safe.

## 10. Tests often proved implementation self-consistency

**Evidence:** graph tests exercise graph internals disconnected from startup; project tests round-trip the schema they define; VST3 tests cover helper paths without a real fixture; UI tests exercise models without live application QA.

**Impact:** green tests can coexist with absent workflows and incompatible subsystem contracts.

**Prevention rule:** retain unit tests, but add independent or cross-layer oracles: offline renders, scenario fixtures, migration corpora, real redistributable plugins, failure injection, and UI workflow protocols.

## 11. Failure, recovery, stress, and compatibility evidence was missing

**Evidence:** no disk-full/interrupted-save test, recording salvage, callback soak, plugin crash/hang containment, cross-platform project fixture, or package/install test was found. Strict Clippy and formatting currently fail despite green tests.

**Impact:** the project can appear stable while losing work, failing under load, or breaking outside one developer environment.

**Prevention rule:** each milestone MUST define negative, recovery, stress, and compatibility gates proportionate to its risk before implementation begins.

## 12. “Atomic save” was narrower than durable project safety

**Evidence:** `atomic_write_cbor` serializes, writes a sibling temporary file, and renames it. It does not sync file/directory data, preserve the original on every platform failure mode, report autosave worker errors, or coordinate recording/assets.

**Impact:** a happy-path rename test can be mistaken for crash-safe project lifecycle.

**Prevention rule:** project safety MUST specify transaction boundaries, flush/sync policy, rollback, failure reporting, autosave generations, recording salvage, asset consistency, and recovery drills.

## 13. Realtime law exceeded realtime enforcement

**Evidence:** `docs/realtime_rules.md` states broad callback law. The allocator guard covers a busy `SynthProcessor` fixture only. It excludes the CPAL boundary, graph swaps/reclamation, VST3, input recording, teardown, lock detection, deadlines, and future device paths.

**Impact:** a narrow passing guard can legitimize callback-reachable operations it never observes.

**Prevention rule:** maintain a machine-readable callback reachability inventory. Enforce allocation/deallocation and forbidden synchronization across every canonical node, swap, error, and teardown path; pair safety guards with deadline benchmarks.

## 14. Handoff logs became architecture and status authority

**Evidence:** `docs/architecture.md` points contributors to `HANDOFF.md` for validation notes; plans require appending handoff iterations and phase updates.

**Impact:** chronological narrative accumulates contradictions and obscures the latest verified state.

**Prevention rule:** handoffs are historical context only. `STATUS.md`, `NEXT.md`, `VALIDATION.md`, requirements, ADRs, and subsystem ledgers each own one current information class.

## 15. Stale documents remained live without status metadata

**Evidence:** root plans, architecture summaries, plugin notes, clean-room specs, ADR pseudocode, and source headers each declare partial authority with incompatible status vocabularies.

**Impact:** contributors cannot determine which statement governs a decision.

**Prevention rule:** every authoritative document MUST carry required metadata. Superseded material moves to archive or becomes a short pointer recorded in a migration map.

## 16. Generated metrics substituted for source quality

**Evidence:** `clean-room-spec-audit-metrics.json` measures lines, headings, URLs, and marker words. Legacy readiness language uses document/test counts without proving source completeness or runtime behavior.

**Impact:** quantity creates false confidence while claims remain untraceable or mixed with implementation inference.

**Prevention rule:** metrics MAY measure corpus maintenance but MUST NOT determine clean-room acceptance, requirement status, or subsystem maturity. Acceptance requires source/version coverage and claim-level traceability.

## 17. Plugin scaffolding was described too close to support

**Evidence:** architecture calls VST3 the supported format while the app never scans or loads it. `VstPluginNode` omits parameter and event queues, editor, state, context, latency, and failure behavior.

**Impact:** users and contributors can mistake an ABI experiment for compatibility.

**Prevention rule:** use “target” until a fixture matrix proves the complete minimum host lifecycle. Compatibility claims MUST name versions, platforms, fixtures, and known limitations.

## 18. Product experiments lacked explicit reuse disposition

**Evidence:** `geist-synth`, `geist-stacksynth`, and modular synth/rack efforts coexist without a settled relationship. All remain active workspace code despite different product roles and integration states.

**Impact:** effort fragments across architectures and broadens maintenance before foundations are trustworthy.

**Prevention rule:** every legacy experiment MUST be classified as preserve, concept-only rewrite, fixture-only salvage, archive, removal, or undecided before entering the rebuild workspace.

## Summary prevention gate

Before R0/R1 implementation begins, the rebuild MUST have:

1. one document authority hierarchy;
2. stable requirement IDs and traceability;
3. explicit ownership/threading/persistence contracts;
4. a reuse disposition for every major legacy component;
5. verification gates defined before code;
6. no claim that isolated code is integrated;
7. no canonical runtime bypass.
