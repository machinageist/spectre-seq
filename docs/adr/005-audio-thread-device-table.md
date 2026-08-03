<!--
Author: Jeff
Date: 2026-08-03
Description: Record the device-ownership decision — an audio-thread DeviceTable, with generations carrying only the plan.
Notes: Decision 8 of docs/changes/typed-realtime-graph/SPEC.md, called there the most consequential decision in that spec.
-->

# ADR 005 — Audio-thread `DeviceTable` for device instance ownership

- Status: **Accepted** (2026-08-03)
- Source: `docs/changes/typed-realtime-graph/SPEC.md` decision 8, with decisions 9 and 12 as consequences
- Builds on: ADR 002, which established that graph publication moves ownership rather than sharing it
- Delivery: roadmap Milestone 3, slice T2 in `docs/changes/typed-realtime-graph/PLAN.md`, jointly with slice D5 in `docs/changes/project-document/PLAN.md`

## Context

ADR 002 settled *how* a compiled graph reaches the audio thread: an rtrb SPSC ownership handoff, because `Executor` is a mutable object and `ArcSwap` only lends shared references. It did not settle *what* crosses.

Today the whole executor crosses, node instances included. `Executor::new` (`crates/geist-graph/src/process_list.rs:150`) takes node instances out of the graph by move. So recompiling means reconstructing every node.

That is the defect. A graph edit today discards:

- every filter's internal state;
- every delay line's contents;
- every sounding voice.

Adding one effect to one track therefore glitches all twenty devices in the project. No DAW can ship that. It is also the fourth of Track D's audit findings against the current engine, and it is invisible in the code — nothing names it, it falls out of the ownership model.

The requirement is stated in `SPEC.md`: *node DSP state survives a graph edit*.

## Decision

**Device instances are owned by the audio thread in a fixed-capacity `DeviceTable`. A published generation carries only the plan.**

```
RenderGeneration {
    id: GenerationId,
    requires_control_sequence: u64,
    plan: RenderPlan,            // pure data, no node instances
    arenas: Arenas,              // preallocated, sized by the plan
    route_index: RouteIndex,     // sorted (DeviceId, ParamKey) -> param slot
    latency: PlanLatency,
}
```

`DeviceTable` is a `Vec<Option<Box<dyn AudioNode>>>` sized once. The plan addresses devices by slot index.

A structural edit is two steps, not one:

1. The app thread constructs the device instance and sends `InstallDevice { slot, instance }` through the control stream. Moving a `Box` into a slot is a pointer write — no allocation occurs on the callback. The audio thread acknowledges the sequence number.
2. The app thread publishes a plan that references the slot, stamped with `requires_control_sequence` equal to that install's sequence. The audio thread adopts the plan only once it has applied that sequence.

Removal reverses: publish a plan that no longer references the slot, then send `RemoveDevice { slot }`. The box rides the return ring and is dropped on the app thread. Arenas retire the same way when a plan resizes them.

## Why the alternatives lost

- **Generation owns the instances; rebuild on every recompile.** Simplest ownership and it is what exists. Rejected: it is precisely the defect above. Every graph edit resets every device.
- **Retire, reclaim, rebuild.** The app asks the audio thread to hand the generation back, moves surviving instances into a new plan, and republishes. Rejected on cost: it produces either a gap where the audio thread has no graph, or enough double-buffering machinery to exceed what the `DeviceTable` costs — and it makes every edit's latency depend on the swap round trip.

## Consequences

- **`swap.rs` changes shape but not kind.** `GraphPublisher` gains a `GenerationId` on the payload and an acknowledgement consumer. `ActiveGraph::poll_swap` gains the control-sequence precondition on top of its existing retire-slot precondition. `reclaim` already does the right thing and is untouched. ADR 002's check-capacity-then-swap invariant still holds.
- **The audio thread holds instances the current plan does not reference.** Between a removal's plan swap and its `RemoveDevice` message, and for any slot installed ahead of use. This is intended: the alternative is a plan referencing a slot that is not installed, which is unsound.
- **A plan can never reference an uninstalled slot.** `requires_control_sequence` is what guarantees it, and it is an acceptance criterion rather than a convention.
- **Control addressing must be durable** (decision 9). Compiled slot indices die with their generation; a knob turn in flight during an edit would be lost. Messages therefore carry `(DeviceId, ParamKey)` and resolve through the generation's sorted route index — a bounded binary search over a cache-resident array.
- **Note identity on the callback stays runtime** (decision 12). Durable `NoteId` never enters the audio thread; live MIDI input, arpeggiators, and chord devices have no durable identity and cannot mint one there.
- **The reclaim path now carries three kinds of payload**: retired plans and arenas, retired device instances, and retired media assets. All three drop on the app thread. This is what closes roadmap gap 14.
- **`DeviceTable` capacity is a bound that can be exhausted.** Exhaustion is an explicit rejection on the app thread, never a silent failure or a reallocation.

## Current wiring status

Not implemented. Nothing in the running app touches the compiled graph at all — `app/geist-daw/src/engine.rs` imports only `geist_graph::node::AudioNode` and runs a hand-wired fixed three-track chain. See `docs/architecture.md`.

Slice T2 implements this and is the first slice where the two specs meet. Its acceptance test is direct: add and remove an unrelated device, and assert an existing device's filter state, delay line, and sounding voices are untouched.
