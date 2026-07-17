<!--
Author: Jeff
Date: 2026-07-12
Description: Traceability from observations/rationale through requirements to code and evidence
Notes: One row per requirement that has moved past proposed; grows with each slice
-->

# Traceability

- **Status:** verified
- **Last verified:** 2026-07-16
- **Scope:** requirements with implementation or verification evidence
- **Decision authority:** Jeff
- **Upstream sources:** `requirements-ledger.md`; `../02-reference-research/*observations*.md`
- **Downstream dependents:** milestone exit reviews, release gates
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** none
- **Known gaps:** RT/GRAPH families have no implementation yet

Chain: provenance → requirement → implementation → evidence (repository-root workspace, 2026-07-12).

| Requirement | Implementation | Evidence | State |
|---|---|---|---|
| TIME-001 (explicit time types) | `crates/geist-core/src/time.rs` | type-level API (no raw time in public surface); unit tests | implemented |
| TIME-002 (half-open ranges, same-sample order) | `transport.rs` (`LoopRegion`), `event.rs` (rank ordering) | `loop_end_is_exclusive`, `same_sample_off_before_on`, property `event_order_is_permutation_invariant`, `loop_wrap_stays_in_region` | implemented |
| TIME-003 (tempo map conversion) | `tempo.rs` (`TempoMap`) | `tempo_time_003.rs`: 24-hour exact/fractional piecewise oracles, round-once discriminator, signed pre-roll, boundary ownership, monotonicity, exact tick round trip, nearest-tick sample bounds | verified |
| TIME-004 (meter map) | `crates/geist-core/src/meter.rs` | validated signatures/maps, Serde rejection and round trip, exact bar length, deterministic lookup properties | implemented |
| TIME-005 (transport state machine) | `transport.rs` | 5 unit tests + wrap property; no audio device needed | implemented |
| CORE-001 (stable IDs) | `id.rs` (`ObjectId`, `IdGen`) | uniqueness/determinism unit + property tests; save/load stability via project round trip | implemented |
| CORE-002 (parameter descriptors) | `param.rs` | validation, normalized round-trip, non-finite containment tests | implemented |
| CORE-003 (versioned envelope, forward preservation) | `crates/geist-project/src/lib.rs` | checked-in `tests/fixtures/r1-canonical.json`; byte-stable fixture rewrite, exact round trip, newer-schema rejection, tempo/meter/loop semantic rejection, project/envelope unknown-field preservation | verified |
| R1 beat representation | `crates/geist-core/src/time.rs` | `beat_ticks_contract.rs`: accepted 960 PPQ, exact common grids, checked positive/negative overflow boundaries, transparent signed-integer JSON round trip | verified |
| R0 offline harness | `crates/geist-offline/` | deterministic report test, malformed-project test, `cargo run --locked -p geist-offline -- --self-test` | verified |
| R1 command/undo seed | `crates/geist-project/src/command.rs` | atomic failure rollback, reversible rename, redo invalidation, bounded-history eviction, identity/unknown-field preservation | implemented |
| Interaction prototype | `crates/geist-app/`, `./geist` | seven app-model tests, process smoke test, verified native window startup, state-rich feedback report | prototype |
| DSP device I/O | `docs/03-architecture/dsp-device-io.md`, `crates/geist-dsp/src/io.rs` | layout, semantic event-order, bounded-capacity, overlap identity, buffer-shape, finite-output, deterministic-source, and sample-offset tests | implemented |
| Native device seed | `crates/geist-dsp/` | Pulse instrument, ToneSource, Gain, Saturator; six device tests | implemented |
| Native render fixture | `geist_offline::render_vertical_slice` | repeated render equality, stereo/peak/hash assertions | implemented |
| Backend-derived device UI | `geist_app::DeviceControl`, Build and Shape lenses | descriptor identity and backend clamping tests | prototype |
| GRAPH-001 (plan/graph type seam) | not started | — | proposed |
| RT-001..003 | workspace policy only | — | proposed |

Gate results (2026-07-16, repository root, rustc/cargo 1.96.1): formatting clean; strict Clippy clean; 88/88 tests pass; `./geist --smoke-test` and the offline self-test pass.
