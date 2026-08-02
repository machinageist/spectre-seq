<!--
Author: Jeff
Date: 2026-08-01
Description: Implementation plan for the typed multi-rate realtime graph, blocked pending owner decisions.
Notes: Slice order is provisional and contingent on the recommended answers in SPEC.md; do not start work from this file yet.
-->

# Typed Multi-Rate Realtime Graph Implementation Plan

## Status

**Blocked.** This plan cannot be written honestly yet.

`SPEC.md` is a draft carrying seventeen open decisions. Four of them change the shape of the work rather than a value inside it:

- **Decision 8 (device instance ownership)** determines whether a generation carries node instances or only a plan. That is the ownership skeleton every later slice builds on. Answering it after T2 means rewriting T2.
- **Decision 1 (CV rate model)** determines whether `SignalDomain` has six variants or seven, and whether the arena count is two or three. It is the first line of T1.
- **Decision 10 (minimum feedback delay)** determines whether the compiler needs a strongly-connected-component chunk scheduler. That is the difference between one slice and three.
- **Decision 12 (note identity on the audio thread)** determines the size and shape of the note event, which is the type every note device and every MIDI adapter is written against.

Decisions 2, 5, 15, and 16 are constants and conventions. They are cheap to answer and expensive to change after patches persist, so they should be answered before T1 rather than deferred.

Decision 17 changes the *order* of the whole milestone, not the content of one slice: it asks whether ten engine slices land before any of them can be heard.

Writing a task-level plan now would mean writing tasks that a single answer deletes. What follows is the gating matrix and a provisional order, clearly marked as contingent.

## What must be decided before any slice starts

Ordered by how much rework a late answer costs.

| Priority | Decision | Blocks | Cost of answering late |
| --- | --- | --- | --- |
| 1 | 8 — device instance ownership | T2, and the shape of every slice after it | Full rewrite of the executor and `swap.rs` payload |
| 2 | 17 — crate boundary and sequencing | The whole milestone's order | Ten slices built against the wrong owner, or ten slices unvalidated by real audio |
| 3 | 1 — CV rate model | T1, T3 | Domain enum, arena count, and every adapter signature |
| 4 | 12 — note identity | T4 | Every note device and MIDI adapter |
| 5 | 10 — minimum feedback delay | T6 | A scheduler that either exists or does not |
| 6 | 6 — fan-in order persistence | T3, and a durable identity domain in Milestone 1 | A persisted ordering added after projects exist |
| 7 | 9 — control addressing | T5, and project-document D5 | Control-stream message format |
| 8 | 4 — meter as outlet or domain | T1, T9 | A variant added to or removed from the domain enum |
| 9 | 2, 5, 15, 16 — conventions and constants | T1, T3, T7, T8 | Persisted patches and rendered levels change meaning |
| 10 | 3, 7, 11, 13, 14 — policies | T1, T3, T4, T6, T8 | Localized; each affects one slice |

## Decision-to-slice gating matrix

| Slice | Gated by |
| --- | --- |
| T1 — Descriptors and validation | 1, 2, 3, 4, 15 |
| T2 — Ownership skeleton | 8, 17 |
| T3 — Streams and control-rate buffers | 1, 5, 6, 7 |
| T4 — Event domain | 12, 13 |
| T5 — Parameter domain | 9 |
| T5a — Single-track spike | 17 |
| T6 — Cycles, delay, latency | 10, 11 |
| T7 — Buses | 16 |
| T8 — Polyphonic lanes | 14, 15 |
| T9 — Meters and analysis | 4 |
| T10 — Fixtures and gates | none |

## Work that is decision-independent

Two pieces of work can proceed regardless of every open question, because they only make the current state honest. They are small and neither changes behavior.

### T0a — Correct the stale feedback comments

`crates/geist-graph/src/nodes/delay_node.rs:4` and `:16` claim `DelayNode` is auto-inserted by topology compilation. Nothing inserts it. `crates/geist-graph/src/process_list.rs:5` claims the file implements "feedback routing" without saying the routing is an implicit reordering. The comment contract in `.claude/skills/geist-realtime-rust.md` requires comments to state actual behavior.

This slice changes comments only. Verification: `cargo test -p geist-graph`, `git diff --stat` shows no `.rs` line outside comments.

### T0b — Characterize current behavior before changing it

The three test files under `crates/geist-graph/src/tests/` are pseudocode scaffolds with zero tests. Before the engine is rewritten, pin what it actually does today so the rewrite's diffs are legible:

- cycle handling as implemented, including that `compile` never fails on a cycle;
- fan-in rejection at `crates/geist-graph/src/graph.rs:106`;
- buffer assignment order and the contiguous-output-run invariant;
- swap publish, adopt, and reclaim under repeated publication.

These tests are expected to be inverted or deleted by later slices. Their value is proving the rewrite changed exactly what it intended to change.

Verification: `cargo test -p geist-graph`, `cargo check --workspace`.

## Provisional slice order

Contingent on the recommended answers in `SPEC.md`. If any of decisions 1, 8, 10, 12, or 17 is answered differently, this order is void and must be rewritten.

1. **T0a, T0b** — comment correction and characterization. No decisions required.
2. **T1 — Descriptors and validation.** `geist-core` only. `SignalDomain`, declared `SignalRate`, `BusLayout` with `Declared` rejected at compile, `LaneSpec`, `FanInPolicy`, `EventCapacity`, and typed connection errors replacing `Internal`. The executor does not change; `geist-graph` is updated only enough to keep compiling.
3. **T2 — Ownership skeleton.** Device table, plan-and-arena generation payload, `GenerationId` and `requires_control_sequence` on the swap, acknowledgement ring, debug allocator guard. Audio domain only, behavior otherwise preserved, so the diff is ownership and nothing else. This slice also settles the boundary with project-document Slice D5 and should be reviewed jointly with it.
4. **T3 — Streams.** Audio, CV, and gate arenas; control-rate buffers; ordered summing fan-in; `CvUpsample` and `CvDownsample`.
5. **T4 — Events.** Note and MIDI arenas, per-route delivery, deterministic merge, `NoteInstanceId`, expression variants, per-port capacity and overflow counters. Deletes `ctx.notes()` and `ctx.param_changes()`; updates `plugins/geist-synth/src/daw_node.rs`.
6. **T5 — Parameters.** `(DeviceId, ParamKey)` addressing, sorted route index, control-stream resolution, base/modulation/clamp/smooth layering.
7. **T5a — Single-track spike.** One instrument, one effect, one meter, driven end to end by the compiled graph. This is the first slice that makes a sound through the typed engine, and it exists to catch contract errors before four more layers sit on top of them.
8. **T6 — Cycles and latency.** Feedback-break declaration, cycle rejection, `AudioFeedbackDelay` and `EventFeedbackDelay`, declared latency, compensation. Inverts `feedback_cycle_compiles_with_one_block_delay` into a rejection test.
9. **T7 — Buses.** Mono and stereo as first-class layouts; `MonoToStereo` and `StereoToMono`; `Declared` rejection path proven.
10. **T8 — Lanes.** Lane propagation and validation, lane-major layout, split, merge, reduce, broadcast adapters.
11. **T9 — Meters and analysis.** Node-declared outlets, atomic and ring publication, reclaim.
12. **T10 — Fixtures and gates.** Reproducible performance fixtures, overload tests, graph-swap stress, callback benchmarks at 48 kHz / 128 frames and the 64-frame stress mode.

## Standing verification for every slice

- `cargo check -p <touched crate>` at minimum.
- `cargo test -p geist-core`, `cargo test -p geist-graph`, `cargo test --workspace`.
- `cargo bench -p geist-graph` when the compiled plan or executor changes.
- The debug allocator guard, once T2 lands, on every slice after it.
- `git diff --check`.

## Stop conditions

Stop before a slice when:

- a decision gating it in `SPEC.md` `## Decisions required` is unanswered;
- the design would require the compiler to insert a semantic conversion;
- a graph edit would reset device DSP state;
- a plan could reference a device slot that is not installed;
- the callback would allocate, deallocate, lock, or drop;
- fan-in order would not be reproducible across a reload;
- a bounded queue would saturate without a counter and an explicit reconciliation path;
- project-document Slice D5 has moved and this spec's protocol addition no longer composes with it;
- independent review has unresolved findings.
