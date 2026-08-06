<!--
Author: Jeff
Date: 2026-07-12
Description: Current verified state of Geist DAW
Notes: Claims here link to live evidence; optimistic language is prohibited
-->

# Status

- **Status:** accepted
- **Last verified:** 2026-08-06
- **Scope:** current implementation, documentation, and research state
- **Decision authority:** Jeff
- **Upstream sources:** workspace tests, `../01-requirements/traceability.md`, research ledgers
- **Downstream dependents:** `NEXT.md`, `../06-plans/current-milestone.md`
- **Supersedes:** all removed prototype-era status and handoff material
- **Superseded by:** none
- **Open decisions:** milestone-gated decisions in `../01-requirements/decision-gates.md`
- **Known gaps:** live callback/audio remains R3 work

## Repository state

The prototype implementation and all prototype-only plans, assets, CI, audits, archives, feature lanes, and agent scaffolding were removed on 2026-07-12. The former namespaced workspace was promoted to the repository root. Git history retains committed historical material.

The active workspace contains:

- `geist-core`: stable IDs, explicit time types, tempo and meter maps, transport, bounded event ordering, and parameter descriptors;
- `geist-dsp`: planar-buffer processing contract, bounded note events, deterministic tone source, Pulse instrument, Gain, and Saturator;
- `geist-graph`: app-thread editable graph and immutable compiled plan (GRAPH-001 split) with validated compilation, implicit-cycle diagnostics, and measured allocation-free execution;
- `geist-project`: versioned JSON envelope, semantic validation after decode, atomic command transactions, and bounded undo/redo;
- `geist-offline`: deterministic project inspection and a Pulse → Gain → Saturator fixture rendered through the compiled plan, including an offline snapshot entrypoint that requires the complete four-parameter fixture and validates exact backend identities and authoritative values before processor construction;
- `geist-app`: native egui interaction prototype with backend-derived Build/Shape device surfaces, an owned offline device-parameter snapshot seam, and a feedback-report seam.

`./geist` launches the graphical interaction prototype. Build shows the native device signal path; Shape exposes controls derived from backend parameter descriptors. App parameter edits can be transferred as an owned snapshot to deterministic offline rendering, where the complete fixed fixture is validated before values construct processors in the immutable compiled plan. This is not a live engine: no audio backend, callback bridge, VST3 host, recording path, or project editing canvas exists. Play changes the model's transport state but produces no sound.

The accepted JSON project codec and 960-PPQ `BeatTicks` representation have checked-in R1 fixtures. Tempo conversion evidence now covers signed pre-roll, fractional piecewise boundaries, 24-hour positions, unrounded-anchor accumulation, and nearest-tick sample quantization without claiming impossible one-sample arbitrary-sample round trips.

## Validation

The current gate is:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Latest full-gate result (2026-08-06): formatting, strict Clippy, and all 143 tests pass; the headless launch check and offline self-test pass. The parameter-snapshot evidence includes 15/15 app-model tests and 21/21 offline harness tests. They cover validated nonzero `ObjectId` decode at the public snapshot/render boundary, stable project-instance device/parameter identity, private-field DTO access through getters, canonical constructor containment of NaN/infinities/out-of-range input, exact signed-zero/subnormal publication, read-only app schema with identity-based value attribution and explicit invariant errors, exact complete fixture membership, duplicate/alias/partial/unknown/mismatched snapshot rejection, exact hand-wired render equivalence for all four mappings, backend-default equivalence, and deterministic repeated rendering. Native descriptor tests also pin the documented `f32` normalization and boundary policy. The existing R2 silence, impulse, allocation, and deterministic-hash evidence remains on the unchanged compiled-plan process path.

R0/R1 exited 2026-07-17; R2 (offline graph) is the active milestone.

The R1 exit disposition is complete: CORE-004's atomic-save API design is accepted via the project-persistence contract (implementation at R4, crash qualification at R5), and CORE-001 remains implemented with reorder evidence explicitly gated on the first persisted collection (R4) and migration evidence on the first schema migration (R5).

## Product and requirements

The product vision, requirement seed, decision defaults, roadmap, and current milestone are accepted. Decisions explicitly assigned to later intakes remain gated there rather than blocking R1.

## Research

- 29 unique ledger sources across 10 products.
- Four timestamped FL Studio action-sequence observations.
- Two Ableton thematic self-reports, not action-sequence evidence.
- No frequency, convergence, or priority claim is authorized by the current corpus.
- Visible-session Bitwig and Ableton evidence remains the highest-value workflow gap.