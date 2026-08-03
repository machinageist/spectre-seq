<!--
Author: Jeff
Date: 2026-08-01
Description: Record the implemented graph-swap decision — an rtrb SPSC executor handoff, not ArcSwap.
Notes: Filename keeps the superseded ArcSwap name because ADR numbers are stable; the title is corrected.
-->

# ADR 002 — rtrb SPSC executor handoff for graph swap

- Status: **Accepted** (2026-08-01), recording behavior already implemented in `crates/geist-graph/src/swap.rs`
- Supersedes: `INITIAL_PLAN.md:29` (`Arc<ArcSwap<ProcessGraph>>`) and `PROPOSED_FILE_TREE.md:53` (`ArcSwap<ProcessList> double-buffer mechanism`)
- Filename: `002-arcswap-graph-swap.md` is retained. ADR numbers and paths are stable, so the slug is now historical rather than descriptive. Read the title, not the filename.

## Context

The original plan compiled graphs on the app thread and handed them to the audio
thread through `ArcSwap`. That plan predates the executor. It does not survive
contact with the implemented one.

`Executor` (`crates/geist-graph/src/process_list.rs:137`) is a mutable object,
not immutable data. It owns:

- `nodes: Vec<Box<dyn AudioNode>>` — every node's internal DSP state;
- `pool: Vec<f32>` — the preallocated channel buffer pool;
- `input_scratch: Vec<f32>` — the per-node input gather area.

`Executor::process_block` takes `&mut self` and writes all three every block.
`AudioNode::process` likewise takes `&mut self` (`crates/geist-graph/src/node.rs:17`).

ArcSwap only lends shared references. `ArcSwap<Executor>::load()` yields a guard
that derefs to `&Executor`, never `&mut Executor`. `Arc` grants unique access
only when the refcount is one, which is precisely what a swap primitive cannot
promise on the audio thread.

Reaching `&mut` through an ArcSwap would have required one of:

- interior mutability (`Mutex`/`RefCell`) around the executor — a lock on the
  callback, prohibited by `docs/realtime_rules.md`;
- `unsafe` aliasing of the buffer pool and node state — unjustifiable in a crate
  that is otherwise `#![deny(unsafe_code)]`;
- rebuilding node state each block — not realtime, and it destroys filter, delay,
  and voice continuity across a swap.

The executor must be *owned* by whichever thread runs it. Ownership must move,
not be shared. That rules ArcSwap out on type grounds, before any performance
argument is reached.

## Decision

Graph publication is a **transfer of ownership across two lock-free SPSC rings**,
implemented in `crates/geist-graph/src/swap.rs`:

- `graph_swap(initial: Option<Executor>)` builds a `GraphPublisher` (app thread)
  and an `ActiveGraph` (audio thread) sharing two `rtrb` rings of
  `SWAP_CAPACITY = 8`: app-to-audio for new executors, audio-to-app for retired ones.
- `GraphPublisher::publish(executor) -> bool` moves a freshly compiled executor
  toward the audio thread. `false` means the audio inbox was full and the
  publication did not happen.
- `ActiveGraph::poll_swap() -> bool` adopts a pending executor at a block
  boundary. It is wait-free: a ring pop and an `Option::replace`.
- `ActiveGraph::current_mut()` lends the running executor for one block.

`rtrb` moves values by ownership, so a compiled executor — node state and whole
preallocated buffer pool included — crosses the thread boundary intact, with no
refcount traffic and no shared borrow on the audio thread.

## Reclaim path

The audio thread must never run a destructor. `Executor` owns three heap
allocations plus a `Vec<Box<dyn AudioNode>>`; dropping it frees all of them, and
plugin-backed nodes may free far more.

`ActiveGraph::poll_swap` therefore refuses to adopt unless it can first guarantee
a home for the executor it is about to displace:

```rust
if self.to_app.slots() == 0 {
    return false;
}
```

Only after the return ring is known to have room does it pop the new executor and
push the old one back. The retired executor is never dropped on the callback — it
is shipped to the app thread and freed there by `GraphPublisher::reclaim()`,
which drains the return ring and reports how many executors it freed.

The ordering is the whole invariant: check capacity, then swap. A full return
ring degrades to "keep running the current graph one more block," which is always
safe. It never degrades to a deallocation on the callback.

## Consequences

- **`arc-swap` is not a workspace dependency.** It appears in no `Cargo.toml` and
  has no `Cargo.lock` entry. The name survives only in this ADR's filename and in
  the two superseded planning documents.
- **Strictly SPSC.** One publisher thread, one audio thread. A second publisher
  would corrupt the ring. Multi-writer publication needs its own design, not a
  second `GraphPublisher`.
- **Publication can fail, and the failure is data.** `publish` returns `bool`.
  Discarding it leaves the app believing a graph is live that the audio thread
  never received. The acknowledged-publication protocol in
  `docs/changes/project-document/SPEC.md` treats that divergence as a defect;
  this `bool` is the primitive that protocol has to consume.
- **The app thread owns reclamation.** If `reclaim()` is never called, up to
  `SWAP_CAPACITY` retired executors accumulate and `poll_swap` then stalls,
  pinning the audio thread to its current graph. Stalling is the intended failure
  mode; deallocating on the callback is not.
- **Swaps are block-aligned.** Adoption happens between blocks, so a graph change
  never splits a block. Node state does not carry across a swap; the new
  executor's nodes start from their own prepared state.
- **No `unsafe`.** `geist-graph` stays `#![deny(unsafe_code)]`.

## Current wiring status

This mechanism is implemented and tested, but it is **not on the running audio
path**. `graph_swap` has exactly two consumers today:

- `crates/geist-graph/src/swap.rs:107-136` — publish/adopt/reclaim unit tests;
- `crates/geist-graph/benches/graph_bench.rs:53` — the `publish_and_swap_128` bench.

The shipping engine does not use it. `app/geist-daw/src/engine.rs` imports only
`geist_graph::node::AudioNode` (line 18) and runs a hand-wired fixed three-track
path; it never builds a `Graph`, calls `compile`, or holds an `Executor`. See
`docs/architecture.md` for that gap in full.

Wiring the compiled graph onto the audio path is roadmap Milestone 3, delivered
through the render-generation protocol in `docs/changes/project-document/SPEC.md`
(slice D5). That protocol layers generation identity, acknowledgement, and a
shared reclaim queue for plugins and assets *on top of* this handoff. It does not
replace it.

## Alternatives considered

- **`ArcSwap<Executor>`** — rejected on type grounds above: shared refs cannot
  drive a `&mut self` block executor without a lock or `unsafe`.
- **`ArcSwap<ProcessPlan>` with node instances pinned on the audio thread** — the
  plan alone is immutable pure data, so this typechecks. Rejected because it
  splits one atomic change across two owners: adding a node requires the plan and
  the node instance to arrive together, and the audio thread would have to
  reconcile them. It also leaves node construction and destruction unsolved.
- **`Mutex<Executor>` with `try_lock` on the callback** — rejected. Priority
  inversion risk, plus a silent fallback path whenever the lock is contended.
- **Triple buffer of preallocated executors** — rejected as premature. It fixes
  the graph shape at startup, which defeats the point of a swap.
