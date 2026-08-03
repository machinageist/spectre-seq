<!--
Author: Jeff
Date: 2026-08-03
Description: Record the feedback-scheduling decision — sub-block SCC chunking at a one-sample floor, in Milestone 3.
Notes: Decision 10 of docs/changes/typed-realtime-graph/SPEC.md, taken against that spec's own recommendation on both timing and floor.
-->

# ADR 006 — Sub-block SCC scheduling at a one-sample floor

- Status: **Accepted** (2026-08-03)
- Source: `docs/changes/typed-realtime-graph/SPEC.md` decision 10
- Supersedes: that spec's recommendation of a one-block floor with sub-block scheduling deferred to Milestone 12, and the roadmap's placement of the same work
- Delivery: roadmap Milestone 3, slice T6a in `docs/changes/typed-realtime-graph/PLAN.md`

## Context

The compiler today converts feedback cycles into hidden one-block delay, and does it silently. `compile` calls `schedule(graph)` and keeps only `.order`, discarding `.feedback` (`crates/spectre-graph/src/process_list.rs:54`). `schedule` (`crates/spectre-graph/src/topology.rs:93`) is a DFS reverse postorder that never fails; back-edges are detected at `topology.rs:136` and skipped, which orders the consumer ahead of its producer so it reads the previous block's buffer. There is no node, no descriptor, no error, and no declared latency. `topological_order` (`topology.rs:18`) does return `Err(cycle)`, but nothing calls it.

`PRODUCT_VISION.md` line 87 forbids exactly this: "The compiler does not silently convert arbitrary cycles into hidden one-block latency."

Removing the silent conversion is not in question — every option rejects undeclared cycles and requires a visible feedback-break element. The question was what a declared break element *costs*.

A one-block floor is 2.67 ms at 48 kHz / 128 frames. That is fine for delays, choruses, and feedback networks. It rules out anything where the loop length is the pitch: Karplus-Strong, waveguides, modal resonators, and physical modeling generally. Those need feedback measured in single samples.

The `SPEC.md` recommendation was a one-block floor now, with sub-block scheduling declared as a Milestone 12 compiler upgrade, on the grounds that the delay element declares frames either way so the persisted patch format survives the change.

## Decision

**Sub-block scheduling ships in Milestone 3, and `SCC_FRAME_FLOOR = 1`.**

Compilation and execution:

1. Nodes outside every strongly-connected component are block-processed exactly as before.
2. Nodes inside an SCC iterate at the declared frame floor — one sample.
3. The executor therefore has **two dispatch paths**, not one path with a parameter: block dispatch and per-sample dispatch.
4. Undeclared cycles are still a compile error naming every participating node and the offending domain. The floor changes what a break element costs, not whether one is required.
5. The plan records each component's membership, frame floor, and iteration count, so the UI can show where a loop closes and what it costs.

This is the option the spec recommended against, on both timing and floor. It was taken because the flagship modular instrument is a stated product goal, and a rack that cannot express a plucked string or a resonator is not the product. Capability was chosen over budget, deliberately.

## Why the alternatives lost

- **One block, with sub-block deferred to Milestone 12.** The spec's recommendation. Cheap, simple, and the upgrade path is real. Rejected because the deferral is not free in practice: Milestone 12 is where the modular instrument lands, so the instrument and the scheduler it needs would arrive together anyway, with ten milestones of graph code written against a floor assumption that then changes.
- **One block, permanently.** Same as above with the door closed. Rejected outright — short feedback would have to live inside a node forever, which means resonators can only be built by whoever writes DSP nodes, not by whoever patches the rack.

## Consequences

- **A second execution path in `process_list.rs`.** A node inside an SCC is called once per sample with single-frame slices. This is a real structural addition, not a parameter.
- **One question is still open and blocks slice T6a: control-value behavior inside a component.** At a one-sample floor, chunk boundaries no longer align to the 8-frame control grid, so one control point spans eight iterations. Interpolating per sample and holding per chunk both work and they sound different. This is settled when the scheduler is specified.
- **A second open question, decidable by measurement:** whether per-sample dispatch calls `AudioNode::process` with single-frame slices, or `AudioNode` gains a per-sample entry point. The first keeps one trait; the second avoids constructing a slice per sample.
- **This is what VCV Rack does, and it is why VCV is CPU-hungry** — the difference is that VCV runs its whole graph per sample, while here the cost is confined to declared components. A patch with no cycles pays nothing.
- **Cost compounds with two other decisions.** `CONTROL_PERIOD_FRAMES = 8` (decision 5) and `MAX_LANES = 32` (decision 15) were chosen independently, and all three spend from the same 64-frame stress budget. A large SCC carrying 32-lane cables at an 8-frame control grid is the worst case, and it is a plausible modular patch rather than a synthetic one.
- **The release fixture set gains a mandatory large-SCC patch.** Nothing in the existing bench set — graph compile/swap, osc, filter, reverb — would find that cliff.
- **Latency compensation is skipped inside a component and reported as skipped.** Unchanged from the one-block design; compensating a loop is meaningless.
- **The persisted patch format is unaffected.** The delay element declares its delay in frames under either floor, which is what makes this a scheduler decision rather than a format decision.

## Current wiring status

Not implemented, and the behavior it replaces is currently pinned as intended by a test: `crates/spectre-graph/src/process_list.rs:280`, `feedback_cycle_compiles_with_one_block_delay` — "The cycle no longer fails; the back-edge is scheduled as a one-block delay." Slice T6 inverts that test into a rejection test; slice T6a adds the scheduler.

`crates/spectre-graph/src/nodes/delay_node.rs` claims at `:4` and `:16` that topology compilation auto-inserts it to break feedback cycles. Both comments are false — nothing inserts it. Slice T0a corrects them before any of this work begins.
