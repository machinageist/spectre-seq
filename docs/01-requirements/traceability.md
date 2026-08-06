<!--
Author: Jeff
Date: 2026-07-12
Description: Traceability from observations/rationale through requirements to code and evidence
Notes: One row per requirement that has moved past proposed; grows with each slice
-->

# Traceability

- **Status:** verified
- **Last verified:** 2026-08-06
- **Scope:** requirements with implementation or verification evidence
- **Decision authority:** Jeff
- **Upstream sources:** `requirements-ledger.md`; `../02-reference-research/*observations*.md`
- **Downstream dependents:** milestone exit reviews, release gates
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** none
- **Known gaps:** RT family has no implementation yet; GRAPH-002 has only the implicit-cycle seed

Chain: provenance → requirement → implementation → evidence (repository-root workspace, 2026-08-06).

| Requirement | Implementation | Evidence | State |
|---|---|---|---|
| TIME-001 (explicit time types) | `crates/geist-core/src/time.rs` | type-level API (no raw time in public surface); unit tests | implemented |
| TIME-002 (half-open ranges, same-sample order) | `transport.rs` (`LoopRegion`), `event.rs` (rank ordering) | `loop_end_is_exclusive`, `same_sample_off_before_on`, property `event_order_is_permutation_invariant`, `loop_wrap_stays_in_region` | implemented |
| TIME-003 (tempo map conversion) | `tempo.rs` (`TempoMap`) | `tempo_time_003.rs`: 24-hour exact/fractional piecewise oracles, round-once discriminator, signed pre-roll, boundary ownership, monotonicity, exact tick round trip, nearest-tick sample bounds | verified |
| TIME-004 (meter map) | `crates/geist-core/src/meter.rs` | validated signatures/maps, Serde rejection and round trip, exact bar length, deterministic lookup properties | implemented |
| TIME-005 (transport state machine) | `transport.rs` | 5 unit tests + wrap property; no audio device needed | implemented |
| CORE-001 (stable IDs) | `id.rs` (`ObjectId`, `IdGen`), `geist_app::{DeviceControl, ParameterControl}`, `geist_dsp::DeviceParameterSnapshot` | generator uniqueness/determinism; validated transparent numeric decode rejects zero while preserving nonzero JSON round trips; save/load and undo identity; prototype device/parameter nonzero project-instance uniqueness; snapshot identity stability across edits; offline deserialization-boundary and duplicate/alias rejection. Reorder evidence remains gated on R4 and migration evidence on R5 | implemented |
| CORE-002 (parameter descriptors) | `param.rs`, `geist_dsp::DspParameter` | validation and non-finite containment; exact endpoints, finite monotonic mapping, nearest-`f32` plain quantization, nextafter endpoint behavior, and deterministic seeded normalized round trips within the declared 8,192-ULP fixture ceiling for every native descriptor | implemented |
| CORE-003 (versioned envelope, forward preservation) | `crates/geist-project/src/lib.rs` | checked-in `tests/fixtures/r1-canonical.json`; byte-stable fixture rewrite, exact round trip, newer-schema rejection, tempo/meter/loop semantic rejection, project/envelope unknown-field preservation | verified |
| R1 beat representation | `crates/geist-core/src/time.rs` | `beat_ticks_contract.rs`: accepted 960 PPQ, exact common grids, checked positive/negative overflow boundaries, transparent signed-integer JSON round trip | verified |
| R0 offline harness | `crates/geist-offline/` | deterministic report test, malformed-project test, `cargo run --locked -p geist-offline -- --self-test` | verified |
| R1 command/undo seed | `crates/geist-project/src/command.rs` | atomic failure rollback, reversible rename, redo invalidation, bounded-history eviction, identity/unknown-field preservation | implemented |
| Interaction prototype | `crates/geist-app/`, `./geist` | fifteen app-model tests, including nonzero unique project-instance identity, owned/stable typed snapshots, read-only device schema, identity-based value attribution, canonical descriptor clamping, and exact signed-zero/subnormal and non-finite containment through the setter seam; process smoke test; verified native window startup; state-rich feedback report | prototype |
| DSP device I/O | `docs/03-architecture/dsp-device-io.md`, `crates/geist-dsp/src/io.rs` | layout, semantic event-order, bounded-capacity, overlap identity, buffer-shape, finite-output, deterministic-source, and sample-offset tests | implemented |
| Native device seed | `crates/geist-dsp/` | Pulse instrument, ToneSource, Gain, Saturator; twelve device tests, including numeric mapping and boundary policy | implemented |
| Native render fixture | `geist_offline::render_vertical_slice` via `geist-graph` plan | repeated render equality; bit-identical to a hand-wired chain; exact-silence gate; impulse sample-exactness; allocation-free steady-state quanta (counting allocator); FNV hash determinism | verified |
| Backend-derived device UI | `geist_app::DeviceControl`, Build and Shape lenses, `geist_dsp::DeviceParameterSnapshot` | descriptor-derived controls with distinct stable device/parameter `ObjectId`s; renderer-neutral private-field DTO with instance/key getters and canonical clamping; app export preserves identities and exact signed-zero/subnormal bits; offline validation rejects incomplete, duplicate, unknown, aliased, inconsistent, and non-canonical identities/values | prototype (offline-integrated) |
| CORE-004 (atomic-save API design) | `docs/03-architecture/project-persistence.md` | accepted R1 design review: boundaries, ordered save algorithm, `SaveReceipt`/`SaveError`/`TargetState` vocabulary, failure-stage guarantees, deterministic fault-injection seam; filesystem implementation lands R4, crash qualification R5 | accepted (design) |
| GRAPH-001 (plan/graph type seam) | `crates/geist-graph/src/lib.rs`, `docs/03-architecture/graph-compilation.md`, `geist_offline::render_app_snapshot` | `graph_plan.rs`: deterministic repeated render, edit/compile validation, event-routing refusals, frame bounds, unreachable-node exclusion; plan exposes no mutation API; targeted app/offline tests prove complete validated app snapshots select processor values during immutable-plan construction and deterministically affect render reports | implemented |
| GRAPH-002 (explicit priced feedback) | implicit-cycle rejection only (`GraphError::Cycle` diagnostic) | cycle fixture in `graph_plan.rs`; explicit one-quantum-delay feedback edges not designed or implemented | proposed |
| RT-001..003 | workspace policy only | — | proposed |

Full-gate result (2026-08-06, repository root, rustc/cargo 1.97.1): formatting clean; strict Clippy clean; 143/143 tests pass; `./geist --smoke-test` and the offline self-test pass. The parameter-snapshot evidence includes 15/15 app-model tests and 21/21 offline harness tests; R2 silence/impulse/allocation/hash gates remain green on the compiled-plan path.
