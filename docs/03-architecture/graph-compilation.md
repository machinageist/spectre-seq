<!--
Author: Jeff
Date: 2026-07-17
Description: V1 editable-graph and compiled-plan contract for offline rendering
Notes: GRAPH-001 type seam; live callback execution and explicit feedback are later contracts
-->

# Graph Compilation Contract

- **Status:** implemented for offline R2; live-callback execution arrives at R3
- **Last verified:** 2026-07-17
- **Scope:** editable-graph semantics, compile-time validation, and plan execution
- **Decision authority:** Jeff
- **Upstream sources:** [GRAPH-001..002](../01-requirements/requirements-ledger.md), [DSP device I/O contract](dsp-device-io.md), realtime rules
- **Downstream dependents:** `geist-graph`, offline render fixtures, future live engine
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** input-bus summing/mixing semantics; explicit priced feedback edges (GRAPH-002); buffer-reuse optimization
- **Known gaps:** live callback bridge, latency compensation, parameter snapshot publication

## Type seam

`EditableGraph` and `CompiledPlan` are distinct types in `geist-graph` (GRAPH-001). The editable graph is app-thread-only, owns no processors and no buffers, and cannot render. The compiled plan exposes no node or edge mutation API; `process` is its only execution surface.

## V1 editable-graph semantics

- Buses are stereo. A device layout is valid when flattened outputs equal two and flattened inputs are even and at most four (source/instrument: zero inputs; insert: one bus; sidechain: two buses).
- Node identity wraps the project-stable `ObjectId` (`NodeId`), preserving CORE-001 identity through graph edits.
- Each input bus accepts exactly one connection. Summing multiple sources into one input is an explicit later decision, not an implicit behavior.
- Self-connections are rejected at edit time. Multi-node implicit cycles are rejected at compile time with a diagnostic naming a node on the cycle; feedback will require an explicit priced edge (GRAPH-002).

## Compilation

`compile(output, max_frames, factory)`:

1. includes exactly the ancestors of the designated output node; disconnected nodes are excluded and never built;
2. rejects any included node with an unconnected input bus;
3. orders included nodes by Kahn's algorithm with insertion-order tie-break, so plan order is deterministic;
4. builds each processor through the caller-supplied factory and rejects any processor whose `io()` contradicts the node's declared layout;
5. preallocates one dedicated planar `f32` channel pair per included output bus at `max_frames`; buffers are never reused across nodes in v1 and never resized afterward.

## Plan execution

`CompiledPlan::process(sample_rate, frames, note_inputs)`:

- accepts `1..=max_frames` frames; note inputs must name distinct, note-accepting plan nodes;
- executes steps in frozen order; each step gets a validated `ProcessContext` and borrowed buffers;
- buffer handoff uses pointer-swap take/restore, so the loop allocates nothing, takes no locks, and performs no I/O;
- buffers return to the pool before any error propagates, so a failed quantum cannot poison the next;
- devices fully write every output channel per the DSP I/O contract, so no inter-quantum zeroing is required;
- `last_output` borrows the output node's channels from the latest successful quantum.

## Acceptance evidence

`crates/geist-graph/tests/graph_plan.rs`: deterministic repeated render, implicit-cycle diagnostic, edit-time validation (unknown nodes, bus ranges, double-feed, self-connection), compile-time validation (missing input, factory layout mismatch), event routing refusals, frame-capacity bounds, impulse sample-exactness, and unreachable-node exclusion. `tests/plan_alloc.rs`: steady-state process quanta measured allocation- and deallocation-free by a counting allocator. `crates/geist-offline/tests/harness.rs`: the fixture renders through the plan bit-identically to a hand-wired chain, with exact-silence and deterministic-hash gates.
