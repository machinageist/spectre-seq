<!--
Author: Jeff
Date: 2026-08-01
Description: Implementation plan for the typed multi-rate realtime graph.
Notes: Unblocked 2026-08-03 when all seventeen SPEC decisions were settled; one checkpoint remains inside T6a.
-->

# Typed Multi-Rate Realtime Graph Implementation Plan

## Status

**Unblocked 2026-08-03.** All seventeen decisions in `SPEC.md` are settled and the slice order below is no longer contingent.

Three answers went against the spec's recommendation and reshaped this plan rather than a value inside it:

- **Decision 10** put sub-block strongly-connected-component scheduling in this milestone at a one-sample floor, where the spec had it at Milestone 12. That is a new slice, **T6a**, and it is the largest single piece of work here.
- **Decision 5** set `CONTROL_PERIOD_FRAMES = 8` and **decision 15** set `MAX_LANES = 32`. Neither adds a slice, but both raise the cost that T10's fixtures must measure.

One checkpoint remains open and it blocks T6a, not the milestone: how control-rate values behave inside a component when chunk boundaries no longer align to the control grid.

## Preconditions

- Product and architecture contract: `SPEC.md`, accepted.
- The publication protocol half is `docs/changes/project-document/SPEC.md`; T2 must be reviewed jointly with its slice D5.
- The durable graph model belongs to `geist_document::graph` (decision 17). `geist-graph` is compile-and-execute only. Do not add durable editing state to `geist-graph` in any slice.
- `crates/geist-graph/src/swap.rs` is reused, not replaced. It already refuses to drop on the callback.
- Every slice keeps the app runnable and `#![deny(unsafe_code)]` intact.

## Slice T0a — Correct the stale feedback comments

`crates/geist-graph/src/nodes/delay_node.rs:4` and `:16` claim `DelayNode` is auto-inserted by topology compilation. Nothing inserts it. `crates/geist-graph/src/process_list.rs:5` claims the file implements "feedback routing" without saying the routing is an implicit reordering.

Comments only, no behavior change.

Verification: `cargo test -p geist-graph`; `git diff` shows no changed line outside a comment.

## Slice T0b — Characterize current behavior before changing it

The three test files under `crates/geist-graph/src/tests/` are pseudocode scaffolds with zero tests. Pin what the engine does today so the rewrite's diffs are legible:

- cycle handling as implemented, including that `compile` never fails on a cycle;
- fan-in rejection at `crates/geist-graph/src/graph.rs:106`;
- buffer assignment order and the contiguous-output-run invariant;
- swap publish, adopt, and reclaim under repeated publication.

These tests are expected to be inverted or deleted by later slices. Their value is proving the rewrite changed exactly what it intended to change.

Verification: `cargo test -p geist-graph`; `cargo check --workspace`.

## Slice T1 — Descriptors and validation

`geist-core` only. `SignalDomain` (six variants; `Meter` is not among them), declared `SignalRate`, `BusLayout` with `Declared` parsed and compiler-rejected, `LaneSpec`, `FanInPolicy`, `EventCapacity`, and typed connection errors replacing `GeistError::Internal`. Normalized CV convention with the pitch-port octave relationship. `MAX_LANES = 32` lands here as a constant.

The executor does not change; `geist-graph` is updated only enough to keep compiling.

Verification: `cargo test -p geist-core`; `cargo check --workspace`.

## Slice T2 — Ownership skeleton

The audio-thread `DeviceTable`, the plan-and-arena generation payload, `GenerationId` and `requires_control_sequence` on the swap, the acknowledgement ring, and the debug allocator guard. Audio domain only, behavior otherwise preserved, so the diff is ownership and nothing else.

Review jointly with project-document slice D5; this is the slice where the two specs meet. ADR 005 records the decision this implements.

Must prove: a graph edit that adds or removes an unrelated device leaves an existing device's filter state, delay line, and sounding voices intact.

Verification: `cargo test -p geist-graph`; allocator guard under graph-swap stress; `cargo bench -p geist-graph`.

## Slice T3 — Streams

Audio, CV, and gate arenas; control-rate buffers at `CONTROL_PERIOD_FRAMES = 8`; ordered summing fan-in driven by persisted route ordinals; `CvUpsample` and `CvDownsample` adapters.

Must prove: the same project renders bit-identical output across processes and reloads, because contributor order is persisted rather than derived.

Verification: `cargo test -p geist-graph`; bit-identity fixture across two processes.

## Slice T4 — Events

Note and MIDI arenas, per-route delivery, deterministic k-way merge, `NoteInstanceId`, expression variants, per-port capacity and overflow counters, and the reserved note-off headroom in note runs. Deletes `ctx.notes()` and `ctx.param_changes()`; updates `plugins/geist-synth/src/daw_node.rs`.

Must prove: a note run saturated with note-ons still delivers every note-off, so overflow cannot produce a stuck note.

Verification: `cargo test -p geist-graph`; `cargo test -p geist-synth`; overflow saturation test.

## Slice T5 — Parameters

`(DeviceId, ParamKey)` addressing, the sorted per-generation route index, control-stream resolution by bounded binary search, base/modulation/clamp/smooth layering, `accepts_audio_rate` declarations.

Must prove: a control message sent before a swap is still applied to the right target after it.

Verification: `cargo test -p geist-graph`; swap-during-control-traffic test.

## Slice T5a — Single-track spike

One instrument, one effect, one meter, driven end to end by the compiled graph. The first slice that makes a sound through the typed engine, and the reason it sits here rather than at the end: it catches contract errors before cycles, buses, and lanes are built on top of them (decision 17).

Verification: `cargo run -p geist-daw -- --headless` produces audio through the compiled path; callback benchmark at 128 and 64 frames.

## Slice T6 — Cycles and latency

Feedback-break declaration, cycle rejection naming every participating node, `AudioFeedbackDelay` and `EventFeedbackDelay`, declared node latency, and compiler-inserted compensating delay with every insertion recorded in the plan. Inverts `feedback_cycle_compiles_with_one_block_delay` into a rejection test.

This slice removes the silent one-block conversion at `crates/geist-graph/src/process_list.rs:54` and `crates/geist-graph/src/topology.rs:93,136`, and gives `topological_order` (`topology.rs:18`) its first real caller.

Verification: `cargo test -p geist-graph`; the inverted feedback test; latency-report fixture.

## Slice T6a — Sub-block SCC scheduler

**Blocking checkpoint before this slice starts.** Two questions the one-sample floor creates, neither answerable by implementation preference:

1. **Control values inside a component.** At `SCC_FRAME_FLOOR = 1`, chunk boundaries no longer align to the 8-frame control grid, so one control point spans eight iterations. Interpolate per sample, or hold per chunk? They sound different. Decide before writing the scheduler.
2. **Per-sample dispatch shape.** Whether the second execution path calls `AudioNode::process` with single-frame slices, or whether `AudioNode` gains a per-sample entry point. The first keeps one trait; the second avoids per-sample slice construction. Measure before choosing.

Scope once resolved: component detection, chunked iteration at the declared floor, the per-sample dispatch path in `process_list.rs`, and plan recording of component membership, floor, and iteration count.

Must prove: a graph with no cycles produces identical output through both dispatch paths.

Verification: `cargo test -p geist-graph`; dual-path equivalence test; `cargo bench -p geist-graph` with a large-SCC fixture.

## Slice T7 — Buses

Mono and stereo as first-class layouts; `MonoToStereo` defaulting to copy and `StereoToMono` defaulting to mean, both as adapter parameters; the `Declared` rejection path proven to round-trip through validation and persistence.

Verification: `cargo test -p geist-graph`; layout round-trip fixture.

## Slice T8 — Lanes

Lane propagation and validation, lane-major layout at `MAX_LANES = 32`, and the `LaneSplit`, `LaneMerge`, `LaneReduce`, `LaneBroadcast` adapters. No implicit broadcast anywhere; the editor auto-inserts the adapter on an illegal drag.

Verification: `cargo test -p geist-graph`; lane-resolution rejection tests.

## Slice T9 — Meters and analysis

Node-declared outlets, atomic and ring publication, reclaim. `Meter` is not a connectable domain, so this slice adds no edge type.

Verification: `cargo test -p geist-graph`.

## Slice T10 — Fixtures and gates

Reproducible performance fixtures, overload tests, graph-swap stress, and callback benchmarks at 48 kHz / 128 frames and the 64-frame stress mode.

**One fixture is mandatory and specific to the decisions taken here: a patch with a large strongly-connected component.** `CONTROL_PERIOD_FRAMES = 8`, `MAX_LANES = 32`, and `SCC_FRAME_FLOOR = 1` were each chosen independently and each spend from the same 64-frame budget. Nothing in the current bench set would find that cliff.

Verification: all fixtures green at both block sizes; allocator guard clean throughout.

## Standing verification for every slice

- `cargo check -p <touched crate>` at minimum.
- `cargo test -p geist-core`, `cargo test -p geist-graph`, `cargo test --workspace`.
- `cargo bench -p geist-graph` when the compiled plan or executor changes.
- The debug allocator guard, once T2 lands, on every slice after it.
- `git diff --check`.

## Stop conditions

Stop before a slice when:

- the T6a control-value checkpoint is unresolved;
- the design would require the compiler to insert a semantic conversion;
- a graph edit would reset device DSP state;
- a plan could reference a device slot that is not installed;
- the callback would allocate, deallocate, lock, or drop;
- fan-in order would not be reproducible across a reload;
- a bounded queue would saturate without a counter and an explicit reconciliation path;
- durable graph editing state would land in `geist-graph` rather than `geist-document`;
- project-document Slice D5 has moved and this spec's protocol addition no longer composes with it;
- independent review has unresolved findings.
