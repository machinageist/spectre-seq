<!--
Author: Jeff
Date: 2026-07-12
Description: Current verified state of Spectre
Notes: Claims here link to live evidence; optimistic language is prohibited
-->

# Status

- **Status:** accepted
- **Last verified:** 2026-08-09
- **Scope:** current implementation, documentation, and research state
- **Decision authority:** Jeff
- **Upstream sources:** workspace tests, `../01-requirements/traceability.md`, research ledgers
- **Downstream dependents:** `NEXT.md`, `../06-plans/current-milestone.md`
- **Supersedes:** all removed prototype-era status and handoff material
- **Superseded by:** none
- **Open decisions:** milestone-gated decisions in `../01-requirements/decision-gates.md`; the R3 audio backend has no decision row yet
- **Known gaps:** R3 opens with no implementation; no audio backend, callback bridge, or MIDI ingress exists

## Repository state

The prototype implementation and all prototype-only plans, assets, CI, audits, archives, feature lanes, and agent scaffolding were removed on 2026-07-12. The former namespaced workspace was promoted to the repository root. Git history retains committed historical material.

The project was renamed from Geist to Spectre on 2026-08-09, aligning the code with the `spectre-seq` repository. The rename covered all six crate directories and package names (`spectre-core`, `spectre-dsp`, `spectre-graph`, `spectre-project`, `spectre-offline`, `spectre-app`), the `./spectre` launcher and its binary, in-app strings, the research taxonomy tags `SPECTRE-CANDIDATE`/`SPECTRE-REQ`, the local agent skills, and all documentation. It was a pure identifier and prose rename: no behavior, contract, schema, or fixture content changed, and the checked-in project fixtures never carried the old name. The copyright holder `machinageist` is unrelated to the product name and is unchanged.

The active workspace contains:

- `spectre-core`: stable IDs, explicit time types, tempo and meter maps, transport, bounded event ordering, and parameter descriptors;
- `spectre-dsp`: planar-buffer processing contract, bounded note events, deterministic tone source, Pulse instrument, Gain, and Saturator;
- `spectre-graph`: app-thread editable graph and immutable compiled plan (GRAPH-001 split) with validated compilation, implicit-cycle diagnostics, and measured allocation-free execution;
- `spectre-project`: versioned JSON envelope, semantic validation after decode, atomic command transactions, and bounded undo/redo;
- `spectre-offline`: deterministic project inspection and a Pulse → Gain → Saturator fixture rendered through the compiled plan, including an offline snapshot entrypoint that requires the complete four-parameter fixture and validates exact backend identities and authoritative values before processor construction;
- `spectre-audio`: audio backend trait seam with validated stream configuration, a deterministic pump-driven null backend for CI and offline, a cpal implementation behind a default-on feature, and the RT-002 split-lane control transport over a bounded wait-free SPSC ring;
- `spectre-app`: native egui interaction prototype with backend-derived Build/Shape device surfaces, stable project-instance device focus, an owned offline device-parameter snapshot seam, and a selection-aware feedback-report seam.

`./spectre` launches the graphical interaction prototype. Build shows the native device signal path and offers a visible `Open in Shape` action on every card. The action atomically preserves track selection, focuses that existing device by stable `ObjectId`, and enters Shape; Shape renders only the selected device using backend parameter descriptors and setters. Ordinary lens changes and parameter edits preserve device focus. App parameter edits can be transferred as an owned snapshot to deterministic offline rendering independently of focus, where the complete fixed fixture is validated before values construct processors in the immutable compiled plan. This is not a live engine: no audio backend, callback bridge, VST3 host, recording path, or project editing canvas exists. Play changes the model's transport state but produces no sound.

The accepted JSON project codec and 960-PPQ `BeatTicks` representation have checked-in R1 fixtures. Tempo conversion evidence now covers signed pre-roll, fractional piecewise boundaries, 24-hour positions, unrounded-anchor accumulation, and nearest-tick sample quantization without claiming impossible one-sample arbitrary-sample round trips.

## Validation

The current gate is:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Latest full-gate result (2026-08-09, after the Geist → Spectre rename and R3 slices 2-4): formatting, strict Clippy, and all 196 tests pass; the selected-device headless launch check and offline self-test pass. The app evidence includes 27/27 model tests and 1/1 process smoke test; the offline harness retains 21/21 tests. Selection tests cover deterministic Pulse focus, atomic valid drill-in, invalid-ID rollback of lens/selection/device state, all-lens and parameter-edit continuity, selected descriptor and parameter identity, focus-independent complete snapshots with correct edited-value attribution, renderer-neutral Build/Shape presentation, recoverable UI-thread error reporting, and selected-device feedback/smoke reporting. Existing snapshot tests continue to cover validated nonzero `ObjectId` decode at the public snapshot/render boundary, stable project-instance device/parameter identity, private-field DTO access through getters, canonical constructor containment of NaN/infinities/out-of-range input, exact signed-zero/subnormal publication, read-only app schema with identity-based value attribution and explicit invariant errors, exact complete fixture membership, duplicate/alias/partial/unknown/mismatched snapshot rejection, exact hand-wired render equivalence for all four mappings, backend-default equivalence, and deterministic repeated rendering. Native descriptor tests also pin the documented `f32` normalization and boundary policy. The existing R2 silence, impulse, allocation, and deterministic-hash evidence remains on the unchanged compiled-plan process path.

The rename changed no test count and no assertion: the same 155 tests pass before and after, and the pre-rename run on 2026-08-09 reproduced the 2026-08-06 result exactly. The only source churn beyond identifiers was rustfmt reordering `use` blocks, because `spectre_*` sorts differently than `geist_*`.

R0/R1 exited 2026-07-17 and R2 (offline graph) exited 2026-08-09 on its four render gates; R3 (live shell) is the active milestone and opens with no implementation. R2's exit does not claim full GRAPH-002 satisfaction: only implicit-cycle rejection exists, and explicit priced feedback edges stay gated at decision row 7 before R11.

The R1 exit disposition is complete: CORE-004's atomic-save API design is accepted via the project-persistence contract (implementation at R4, crash qualification at R5), and CORE-001 remains implemented with reorder evidence explicitly gated on the first persisted collection (R4) and migration evidence on the first schema migration (R5).

## Product and requirements

The product vision, requirement seed, decision defaults, roadmap, and current milestone are accepted. Decisions explicitly assigned to later intakes remain gated there rather than blocking R1.

## Research

- 29 unique ledger sources across 10 products.
- Four timestamped FL Studio action-sequence observations.
- Two Ableton thematic self-reports, not action-sequence evidence.
- No frequency, convergence, or priority claim is authorized by the current corpus.
- Visible-session Bitwig and Ableton evidence remains the highest-value workflow gap.