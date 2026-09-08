<!--
Author: Jeff
Date: 2026-09-07
Description: Architecture and plan reconciliation at d3c4c56
Notes: Static source findings distinguished from automated gate evidence and unrun reproduction tests
-->

# Architecture audit — 2026-09-07

- **Status:** proposed (findings and corrective sequence for review)
- **Last verified:** 2026-09-07
- **Scope:** latest handoff, product authority, production paths and full-product alignment
- **Decision authority:** Jeff
- **Upstream sources:** `HANDOFF.md`, source at `d3c4c56`, direct interview and independent read-only reviews
- **Downstream dependents:** `../06-plans/product-convergence.md`, corrected handoff and intake plans
- **Supersedes:** none; narrows the confidence of broad prior completion claims
- **Superseded by:** none
- **Open decisions:** final release staging and product-contract interview axes
- **Known gaps:** no fresh GUI, ear, hardware or behavioral reproduction of the findings below

## Verdict

REQUEST_CHANGES on plan/implementation alignment. Useful foundations exist; the current handoff
and roadmap do not establish a faithful path to the expanded final-product specification without
state/wiring repairs, explicit capability coverage, and additional product decisions.

Automated validation was executed by the parent in this session:

```
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

The combined command exited 0. This report does not independently re-count aggregate tests or
reclassify ignored hardware/operator drills. The previous handoff's test count is historical.
No GUI or speaker verification was performed. Passing existing tests does not disprove uncovered
production-wiring defects. No runtime implementation changes were made in this audit.

## Architecture actually present

- `spectre-core`: identity, beat/sample time, tempo/meter, transport and ordered events.
- `spectre-project`: track/clip state, schema/validation, commands/history, atomic files,
  sidecars/recovery, and construction of the instrument/insert/gain/summing/master graph.
- `spectre-dsp`: processing/parameter contract and native DSP devices.
- `spectre-graph`: editable graph → validated immutable execution plan with prepared buffers.
- `spectre-audio`: backend seam, bounded controls, clip schedules, bridge and telemetry.
- `spectre-app`: egui shell, application model, project mapping and live-engine lifecycle.
- `spectre-offline`: shared rendering helpers and fixture-oriented bounce functionality.

Live production path: `main.rs::open_engine` → `engine.rs::open_track_engine` /
`build_track_engine_parts` → `spectre-project::routing::build_track_graph` /
`track_device_factory` → compiled plan → backend-owned `RenderBridge::render` → native DSP.
This is a useful common engine, not yet the complete typed modular/plugin graph of the vision.

## Parent-checked source findings

### A-01 — Shell export is fixture export

`crates/spectre-app/src/main.rs` starts Bounce with `device_parameter_snapshot()` and
`spectre_offline::fixture_events`. The snapshot in `crates/spectre-app/src/lib.rs` explicitly
selects Pulse/Gain/Saturator fixture parameters. It does not pass project tracks, clips and tempo.
The nearby comment that what Shape plays is what bounce renders is not valid for general projects.

Required reproduction: author different Filament arrangements through the product, export each,
and compare the decoded WAVE with its corresponding live/null project render, not a fixture hash.

### A-02 — Competing parameter authorities

`AppModel::edit_device_parameter` writes the global `devices` collection directly, not history or
track state. `engine.rs::selected_track_target_index` resolves those controls onto the selected
track; `publish_stored_parameters` republishes them to that selected track at startup/rebuild.
`routing.rs::instrument_for` reconstructs Filament shape from descriptor defaults, keeping only
track instrument level; `effect_for` keeps depth and defaults other Gloam controls.

Independent per-instance sound persistence/undo is therefore not established by current global
parameter save tests. Startup republishing can also compete with track-owned level/depth/gain.
Required reproduction: two distinct track sounds and repeated inserts through selection,
rebuild, undo and save/reload. Broad “undo all of it” claims exclude Shape in the current source.

### A-03 — Model-relative indices against an old plan

`main.rs::engine_is_stale` compares structure revisions, but `apply_mix_edit` still publishes.
`engine.rs::apply_parameter_edit` and `publish_effect_parameter` compute offsets from the current
track list, then index the existing engine's target array. They contain no matching-generation
check at that join.

Static risk: a structural edit can redirect subsequent publications to an old positional target.
A reproduction test must pin actual target/value and refuse or preserve identity on stale plans;
this audit has not executed that test. A stale badge is not an authority boundary.

## Independent-review findings requiring bounded reproduction

These were traced by the independent read-only reviewer; consequences are not freshly exercised:

- **A-04 lifecycle:** `main.rs::rebuild_engine/start_engine/open_project` and
  `project.rs::adopt` do not provide a complete shared old/new transport and loop reconciliation.
  Test Open while playing, rebuild while playing, and loop state after restart/reload.
- **A-05 unsaved recovery:** recovery inspection in `spectre-project/src/recovery.rs` requires
  the saved project to exist, while sidecar autosave can precede its creation. Test a crash after
  autosaving a never-saved project and recover through the actual shell path.
- **A-06 silent schedule refusal:** `engine.rs::bake_track_schedule` converts failure to absence;
  publication can turn absence into an empty schedule. Test invalid/dense material with visible
  diagnostics rather than accepting unexplained silence.
- **A-07 meter/loop persistence:** `project.rs::project_envelope` does not serialize typed meter
  state; adoption conditionally reads unknown meter data. Loop persistence/reset also needs a
  contract. Do not expose more editing over state that cannot round-trip.
- **A-08 comparison:** the bounce checkbox controls hash logging; the reviewer found no production
  comparison against the live engine. Logging is not a comparison result.

## Documentation and dependency findings

- `NEXT.md` and `../06-plans/current-milestone.md` contain old unreachable-recovery claims alongside records
  of recovery integration. These must not guide new implementation as current facts.
- `../06-plans/rebuild-roadmap.md` permits concurrent carry only for external/operator/qualification evidence.
  The missing-media type in R5 is unfinished engineering; current-milestone prose cannot silently
  widen the rule. Propose a scope split or dependency reorder for Jeff's approval.
- The requirements ledger's track/summing limits lag implemented 32-track/tree-summing changes.
  “Unbounded” is not an accurate present product limit. Reconcile numeric rationale and acceptance.
- Proposed requirement rows are not automatically ratified by code existing; some, such as the
  launcher, are absent rather than implemented-but-unratified.
- Note-routing §5's no-migration assertion is unproven against the current instrument/effect enums.
- The September 6 interview is only secondarily summarized in this repository. Do not reconstruct
  an accepted complete MVP contract from scattered references.

## Disposition

The direct 2026-09-07 interview now supplies accepted UI direction and full destination scope in
`../00-product/product-contract.md`. The complete supplied source and section-level map are
preserved there. The corrective sequence and proposed expanded phase map are in
`../06-plans/product-convergence.md`; they still require release/order decisions. No aggregate
product PASS, R4 exit, UI qualification or architecture rewrite is authorized by this report.
