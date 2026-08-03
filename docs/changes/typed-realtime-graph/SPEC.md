<!--
Author: Jeff
Date: 2026-08-01
Description: Product and architecture contract for the typed multi-rate realtime render graph.
Notes: Accepted 2026-08-03; all seventeen decisions are settled in the decision record at the end.
-->

# Typed Multi-Rate Realtime Graph

## Status

**Accepted 2026-08-03.** All seventeen questions are settled; `## Decision record` holds each one with its rejected alternatives and the reason it lost.

Three decisions were settled against this document's own recommendation, and their consequences are folded into the contract below rather than left in the record: `CONTROL_PERIOD_FRAMES` is 8 rather than 16, `MAX_LANES` is 32 rather than 16, and sub-block cycle scheduling with a one-sample floor lands in this milestone rather than at Milestone 12. The third is a scope increase to Milestone 3; see `## Cycle, delay, and latency contract` and ADR 006.

This spec targets roadmap Milestone 3 and immediate work order item 4. It is the content half of a contract whose protocol half is already accepted:

- `docs/changes/project-document/SPEC.md` fixes **how** realtime state is published — versioned render generations, monotonic acknowledged `GenerationId`, a bounded sequenced control stream with sample offsets, explicit saturation reconciliation, and an off-callback reclaim queue.
- This spec fixes **what a generation contains** and **what a control message addresses**.

It also unblocks `docs/changes/project-document/PLAN.md` Slice D5, which is explicitly gated on "the render-generation *content* boundary must be settled against Milestone 3 first."

## Problem

### What the code does today

Port typing is metadata only.

- `crates/spectre-core/src/port.rs:19` declares seven `PortType` variants: `Audio`, `Cv`, `Gate`, `Note`, `Midi`, `Parameter`, `Meter`.
- `crates/spectre-core/src/signal.rs:23` classifies them into three `SignalRate` values.
- Nothing consumes either classification. `SignalRate` has no reader outside its own unit tests. `PortType` is read only by `can_connect` (`crates/spectre-core/src/port.rs:60`) for equality checking.
- No node anywhere in the workspace declares a `Cv`, `Gate`, `Note`, `Midi`, `Parameter`, or `Meter` port. Every `PortSpec::new` call site in `crates/` and `plugins/` uses `PortType::Audio`, except two negative tests.

The executor routes everything through `f32` sample buffers.

- `ProcessPlan` (`crates/spectre-graph/src/process_list.rs:43`) carries `steps`, a flat `buffer_count`, and `frames`. There is one buffer pool and one buffer kind.
- `Executor::pool` (`crates/spectre-graph/src/process_list.rs:141`) is `Vec<f32>`, sliced into `frames`-sized channel regions. `ChannelSource` is `Buffer(usize) | Silent`.
- A `Note` edge and an `Audio` edge compile to the same thing: a `frames`-length `f32` region. A note stream would be silently reinterpreted as samples.

Notes and parameters are global broadcasts, not routes.

- `ProcessContext` (`crates/spectre-core/src/context.rs:20`) holds `notes: &'a [NoteEvent]` and `params: &'a [ParameterChange]`.
- `Executor::process_block` (`crates/spectre-graph/src/process_list.rs:172`) takes one `notes` slice and one `params` slice and passes the identical borrow to every node in the plan (`:205`).
- The only consumer is `plugins/spectre-synth/src/daw_node.rs:120`, which iterates `ctx.notes()`. Two synths in one graph would both sound every note in the block. There is no addressing, no filtering, and no per-port delivery.
- `ParameterChange` carries a bare `ParamId` (`crates/spectre-core/src/events.rs:126`) with no owning device, so parameter identity is globally flat.

Cycles are converted to hidden one-block delay. **This contradicts the product vision.**

- `PRODUCT_VISION.md` line 87: "Every feedback cycle contains an explicit visible delay or feedback element. The compiler does not silently convert arbitrary cycles into hidden one-block latency."
- `compile` calls `schedule(graph)` and keeps only `.order`, discarding `.feedback` (`crates/spectre-graph/src/process_list.rs:54`).
- `schedule` (`crates/spectre-graph/src/topology.rs:93`) is a DFS reverse postorder that never fails. Back-edges are detected at `crates/spectre-graph/src/topology.rs:136` and simply skipped, which orders the consumer ahead of its producer. The consumer therefore reads the producer's pool buffer still holding the previous block. That is a one-block delay, applied automatically, with no node, no descriptor, no error, and no declared latency.
- `topological_order` (`crates/spectre-graph/src/topology.rs:18`) does return `Err(cycle)`, but no caller uses it. It is dead except in tests.
- `crates/spectre-graph/src/process_list.rs:280`, `feedback_cycle_compiles_with_one_block_delay`, pins this behavior as intended: "The cycle no longer fails; the back-edge is scheduled as a one-block delay."
- The delay is invisible in the compiled plan. `ProcessPlan` records nothing about which edges were deferred, so no UI, latency calculation, or persistence layer can see it.
- **`DelayNode` is never inserted by anything.** `crates/spectre-graph/src/nodes/delay_node.rs:4` claims "auto-inserted one-block delay for feedback loops" and `:16` claims "Topology compilation inserts this to break feedback cycles". Both comments are false; the only non-test references are module re-exports. The comment contract in `.claude/skills/geist-realtime-rust.md` requires comments to state actual behavior.

Fan-in is rejected, not ordered.

- `Graph::connect` (`crates/spectre-graph/src/graph.rs:106`) rejects a second edge into an already-connected input with `GeistError::Internal("input port already connected")`. `Internal` is documented at `crates/spectre-core/src/errors.rs:32` as "indicates a bug", so a legal user action reports as an engine defect.
- Independently, `compile` builds `feed: BTreeMap<PortId, PortId>` keyed on destination (`crates/spectre-graph/src/process_list.rs:68`). If fan-in were ever permitted at the graph layer, the last-inserted edge would silently win. Deterministic fan-in requires changes in both places.

Channels are a bare count with no layout.

- `PortDescriptor.channels: u16` (`crates/spectre-core/src/port.rs:52`) has no layout meaning. `can_connect` (`crates/spectre-core/src/port.rs:70`) requires exact equality, so mono cannot feed stereo. That happens to satisfy "no implicit upmix", but by accident: there is no bus descriptor, no layout vocabulary, and no adapter to make the conversion explicitly.

Nothing declares latency or polyphony.

- `AudioNode` (`crates/spectre-graph/src/node.rs:15`) has `process`, `prepare`, `reset`. No latency report, no tail, no bus negotiation, no lane declaration.
- Nothing in the plan or the graph models polyphonic lanes.

The whole compiled path has no production consumer.

- `Graph::add_node` is called only from unit tests and `crates/spectre-graph/benches/graph_bench.rs`. The running application uses `SynthProcessor` (`app/geist-daw/src/engine.rs:575`), a fixed track array that bypasses the graph entirely. This is roadmap gap 12.
- Consequence for planning: rewriting the graph engine breaks no shipping behavior, and also delivers no user-visible behavior until Milestone 4 attaches the hybrid track to it.

What is already correct and must be reused.

- `crates/spectre-graph/src/swap.rs` is the lock-free handoff **and** the reclaim path. `GraphPublisher::publish` (`crates/spectre-graph/src/swap.rs:49`) is a non-blocking push; `ActiveGraph::poll_swap` (`crates/spectre-graph/src/swap.rs:67`) refuses to adopt unless a retire slot is free, so the audio thread never drops; `GraphPublisher::reclaim` (`crates/spectre-graph/src/swap.rs:54`) drops retired payloads on the app thread. This satisfies project-document invariant 3 today. Do not replace it.
- `Executor::process_block` genuinely does not allocate: the pool and the input scratch are sized in `Executor::new` (`crates/spectre-graph/src/process_list.rs:161`).
- `AtomicTransport` (`crates/spectre-core/src/transport.rs:114`) is a working seqlock with no allocation and no unsafe.
- `MeterCell` (`crates/spectre-graph/src/nodes/monitor.rs:20`) is a working audio-to-UI atomic publication cell.
- `spectre-graph` carries `#![deny(unsafe_code)]`. Every design below must hold without `unsafe`.

### What must be true instead

1. A compiled route carries the domain it was declared with, and a domain mismatch is impossible at runtime because it was impossible at compile time.
2. An event reaches exactly the destinations its edges name, at its sample offset, with stable identity.
3. Control-rate signals cost control-rate work, and every rate crossing is a visible node with declared behavior.
4. Fan-in produces the same samples every time the same project is loaded.
5. Every cycle contains a delay element the user placed and can see, and its latency is a number the rest of the system can read.
6. Node DSP state survives a graph edit.
7. None of it allocates, deallocates, locks, logs, or blocks on the callback.

## Scope boundary with the publication protocol

The accepted protocol is not reopened. This spec places its content inside it.

| Fixed by project-document SPEC | Filled in by this spec |
| --- | --- |
| `RenderGeneration` is immutable, app-thread-built, `GenerationId`-identified | The generation payload is a `RenderPlan` plus its typed arenas |
| Publication is a swap the audio thread cannot block on | The swap carries the plan, not the device instances (decision 8) |
| Audio thread acknowledges the executing `GenerationId` | Adoption additionally waits on a control-sequence precondition |
| Bounded sequenced control stream with sample offsets | Message *types* per domain and how each addresses a compiled destination |
| Saturation and rejection reconcile explicitly | Which per-domain counters the audio thread publishes so reconciliation can be specific |
| Retired payloads drop off the callback | Retired device instances and arenas also travel the reclaim path |

One protocol addition is proposed, not a redesign: a generation declares `requires_control_sequence: u64`, and the audio thread adopts it only after it has applied that sequence. This is what makes a two-step structural edit — install a device instance, then reference it from a new plan — safe without a lock. It composes with the existing acknowledgement rather than replacing it.

## Design principle: the compiler never converts

Every conversion between domains, rates, channel layouts, or lane counts is a node in the graph with a durable identity, a declared behavior, and a declared latency. The compiler's only response to a mismatch is a typed rejection naming both ports.

Usability is preserved one layer up: **the editor may offer to insert a visible adapter** when a user drags an illegal cable, so the common case is still one gesture. The inserted adapter is a real node the user can see, select, configure, bypass, and persist.

This resolves the tension between the vision's explicitness requirement and normal patching ergonomics without giving the compiler any implicit behavior. It applies uniformly to rate crossings, mono/stereo, note/MIDI, gate/note, and polyphonic lanes.

## Port and bus descriptor contract

`PortDescriptor` is replaced. `channels: u16` is not sufficient.

```
PortDescriptor {
    id: PortId,
    node: NodeId,
    direction: PortDirection,
    domain: SignalDomain,
    rate: SignalRate,            // declared, not derived from domain
    bus: BusLayout,              // stream domains only
    lanes: LaneSpec,             // stream and note domains
    fan_in: FanInPolicy,         // input ports only
    capacity: EventCapacity,     // event domains only
    param: Option<ParamKey>,     // parameter destinations only
    name: &'static str,
}
```

`SignalDomain` replaces `PortType`:

```
SignalDomain = Audio | Cv | Gate | Note | Midi | Parameter
```

`Meter` leaves the connectable set (decision 4). Analysis that feeds back *into* the graph is `Cv`. Analysis that feeds the UI is a node-declared outlet, not an edge — a `MeterCell`-backed atomic or a bounded ring, never a compiled route. `PRODUCT_VISION.md` line 83 still lists meters and analysis feedback as a signal domain; it is one, but it is not a routable one. That clarification is recorded in the vision.

`BusLayout` is extensible from day one but only two variants compile in v1:

```
BusLayout =
    Mono
  | Stereo
  | Declared { channels: u16, layout: ChannelLayoutId }
```

`Declared` parses, validates, persists, and round-trips. The v1 compiler rejects it with a distinct `UnsupportedLayout` error. Surround and complex plugin I/O become a compiler upgrade, never a descriptor migration. There is no implicit conversion between any two layouts, including `Mono` and `Stereo`.

Connection validity, checked entirely on the app thread:

1. Directions oppose.
2. Domains are equal. No cross-domain edge exists; a cross-domain adapter node exists instead.
3. Rates are equal.
4. Bus layouts are equal.
5. Lane counts are compatible under the lane contract.
6. The destination's `FanInPolicy` admits the new edge.

Each failure is a distinct typed error naming both port ids and both descriptors. None of them is `GeistError::Internal`.

## Domain and rate contract

| Domain | Rate | Storage | Fan-in default |
| --- | --- | --- | --- |
| `Audio` | Audio | `f32` per frame per channel per lane | `Sum` |
| `Cv` | Audio or Control, declared per port | `f32` per point per channel per lane | `Sum` |
| `Gate` | Audio or Control, declared per port | `f32` per point per lane | `Max` |
| `Note` | Event | bounded timestamped `NoteEventOut` run | `Merge` |
| `Midi` | Event | bounded timestamped `MidiEvent` run | `Merge` |
| `Parameter` | Control | `f32` per point, over one base value | `Sum` of modulation over exactly one base |

`Cv` and `Gate` carry a declared rate rather than a fixed one (decision 1). A control-rate `Cv` port and an audio-rate `Cv` port are the same domain and cannot connect to each other without a visible rate adapter. This keeps one cable colour per domain while making the cost and the conversion visible, and it satisfies the vision requirement that audio-rate modulation exist only where a destination declares it.

CV values are normalized (decision 2): bipolar `-1..1`, unipolar `0..1`, sharing the number space of `ParamRange` and automation. Ports that declare themselves pitch CV carry one additional fixed relationship — `1.0` equals one octave — so the modular instrument has a pitch convention without every mixer and effect destination speaking volts. This is a persisted convention; changing it after patches exist is a migration.

Control-rate storage is a reduced-resolution buffer, not one scalar per block. A control buffer holds `ceil(frames / CONTROL_PERIOD_FRAMES)` points; the final point may cover fewer frames.

```
CONTROL_PERIOD_FRAMES: usize = 8
```

That is 6 kHz at 48 kHz: 16 points per 128-frame block, 8 per 64-frame block (decision 5). It is a compile-time constant and never a project setting, so the control rate in hertz is independent of block size — changing the buffer from 128 to 64 frames must not change how modulation sounds, and a project must render identically on two machines. A project-configurable value would make renders non-reproducible and performance fixtures non-comparable.

Every control-rate consumer declares how it reads a control buffer: `Hold` (step at each point) or `Ramp` (linear between points). `Ramp` is the default for parameters and CV; `Hold` is the default for gates.

Inside a strongly-connected component the control grid no longer aligns to chunk boundaries; see `## Cycle, delay, and latency contract`.

## Buffer and arena contract

One arena per domain family, all sized at compile time on the app thread, all owned by the generation.

```
Arenas {
    audio:   Vec<f32>,          // audio, audio-rate cv, audio-rate gate
    control: Vec<f32>,          // control-rate cv, gate, parameter
    notes:   Vec<NoteSlot>,
    midi:    Vec<MidiEvent>,
}
```

Slot assignment rules, all resolved at compile time:

- Every output port owns a contiguous ascending run in its arena. This is the existing `resolve_outputs` invariant (`crates/spectre-graph/src/process_list.rs:119`) generalized per domain, and it is what lets the executor hand a node exactly one `&mut` slice per domain with no `unsafe`.
- Every node's output ports within one domain are themselves contiguous, so `ProcessContext` holds one mutable borrow per domain and sub-slices it by a per-port range table.
- Unconnected inputs bind to a shared per-domain silent region that no node may write.
- A node with no outputs in a domain gets an empty slice.

Input binding replaces `ChannelSource`:

```
InputBinding {
    slot: SlotRef,              // arena offset, or Silent
    op:   Copy | Add,           // first contributor copies, the rest accumulate
}
```

The executor gathers into the existing preallocated scratch. Summing fan-in adds one `+=` loop and no allocation.

`ProcessContext` is rebuilt around per-port access and loses its global event slices:

```
ProcessContext<'a> {
    frames, sample_rate_hz, transport,
    audio_in(port)   -> &[f32]         // by declared port index
    audio_out(port)  -> &mut [f32]
    control_in(port) -> &[f32]
    control_out(port)-> &mut [f32]
    notes_in(port)   -> &[NoteEventOut]
    notes_out(port)  -> NoteWriter<'_>
    midi_in(port) / midi_out(port)
    param(key)       -> ParamView<'_>  // this node's parameters only
    meter(outlet)    -> &MeterSink
}
```

`ctx.notes()` and `ctx.param_changes()` are deleted. A node can no longer see another node's events.

## Event contract

Event routes are per-block arena runs, not rings. A producer writes its run during its step; the consumer reads it later in the same block. No cross-thread machinery is involved and nothing is retained between blocks.

Capacity. Each event output port declares `max_events_per_block`. The compiler sums declarations into the arena size and records per-port ranges. A producer that would exceed its declared capacity has its excess event dropped and increments a per-port overflow counter published to the app thread.

Overflow policy is per domain (decision 13):

- **`Note` ports reserve headroom.** A declared tail of each note run is writable only by note-off and choke events. An ordinary note-on that would exceed the unreserved capacity is dropped and counted; a note-off never is. Overflow therefore cannot produce a stuck note, which is the worst failure mode in the system and worth the more complex writer.
- **`Midi` ports drop newest and count.** MIDI overflow degrades gracefully and does not justify the reserve machinery.

Proving capacity at compile time was rejected: a producer whose output depends on a randomization stage — an arpeggiator, a chance device — cannot bound its output honestly, so the guarantee would be fiction.

Ordering. Every event run is sorted by `sample_offset` ascending. Within one offset, order is production order for a single source. For fan-in, the merge key is `(sample_offset, route_ordinal, source_position)`, giving a total order that is stable across runs and across sessions. The merge is a bounded k-way merge over already-sorted runs; it allocates nothing.

Note identity. The realtime note event carries `NoteInstanceId: NonZeroU32`, unique among sounding notes within one generation lifetime, minted by whatever originates the note — the clip scheduler, live MIDI input, or a note device such as an arpeggiator that creates notes of its own. Every note-off and every expression event references the instance id of its note-on. Durable `NoteId` from `ProjectDocument` does not enter the audio thread; the scheduler owns the mapping in both directions (decision 12). Live MIDI input, arpeggiators, and chord devices have no durable document identity and cannot mint one on the callback; vision invariant 9 is about editing, undo, expression editing, and persistence, which are all document concerns. The current `note_id: i32` with the `-1` sentinel (`crates/spectre-core/src/events.rs:26`) is retained only inside the VST3 and MIDI interop adapters, where the sentinel convention is required.

Note expression is in the type from the first slice, even before any producer emits it, so devices are not rewritten when MPE lands:

```
NoteEventOut =
    On         { offset, instance, channel, key, velocity, tuning: Option<f32> }
  | Off        { offset, instance, velocity }
  | Choke      { offset, instance }
  | Expression { offset, instance, kind: ExpressionKind, value: f32 }
```

`ExpressionKind` covers pitch, pressure, timbre, brightness, volume, and pan. `tuning` carries a per-note cents offset so twelve-tone equal temperament is a default rather than a schema limit, matching the vision.

MIDI is a separate domain and a separate arena. Note and MIDI never share a buffer, because every automatic conversion between them is lossy in one direction or the other. `NoteToMidi` and `MidiToNote` are explicit adapters that declare exactly what they drop.

## Parameter and control contract

A parameter destination is addressed by `(DeviceId, ParamKey)`, both durable identities owned by `ProjectDocument`. The flat `ParamId` in `ParameterChange` is replaced.

Three layers reach one parameter, and they combine in a fixed order:

1. **Base** — the document's current value, or arrangement/clip automation evaluated for this block. Delivered through the control stream as a timestamped value with a sample offset.
2. **Modulation** — the sum of every `Parameter`-domain edge into the port, evaluated at control rate.
3. **Clamp and smooth** — clamped to the declared `ParamRange`, then smoothed by the destination's declared smoothing time.

The result is a control-rate buffer the node reads through `ctx.param(key)`. Evaluation precedence between arrangement automation, clip automation, and realtime modulation stays deferred to the parameter-control specification named in the vision; this spec fixes only the storage and the routing.

Audio-rate modulation exists only where a parameter destination declares `accepts_audio_rate: true`. Everywhere else, an audio-rate source into a parameter requires a visible downsampling adapter.

Control-stream addressing. Messages carry the durable `(DeviceId, ParamKey)` pair. Each generation carries a sorted route index built on the app thread; the audio thread resolves a message with a bounded binary search over it. A message whose target is absent from the current generation increments an unresolved counter and is dropped, never guessed (decision 9). The consequence that matters: a knob turn in flight during a graph edit is still applied after the swap, because the address does not depend on the generation.

## Meter and analysis contract

Meters flow out of the realtime graph toward the UI. They are not edges (decision 4).

A node declares meter outlets in its descriptor. Each outlet is backed by an app-thread-allocated publication object shared by `Arc`:

- scalar outlets use an atomic cell, the existing `MeterCell` pattern;
- waveform and spectrum outlets use a bounded SPSC ring, written non-blocking, dropping on full with a counter.

Outlets are created and destroyed on the app thread and travel the reclaim path with their generation. `Arc` clones and drops never occur on the callback.

Anything a *device* consumes — envelope followers, sidechain detectors, pitch trackers — is `Cv`, not `Meter`, and routes through the normal typed graph.

## Fan-in contract

Every input port declares a policy:

```
FanInPolicy = Single | Sum | Max | Merge | Reject
```

`Single` rejects a second edge with a typed error, not `Internal`. `Sum` and `Max` apply to stream domains. `Merge` applies to event domains.

Determinism requires an order. Each input port owns an **ordered list of incoming route ordinals**, persisted by the document (decision 6). The compiler emits contributors in that order; the executor accumulates in that order. Float addition is not associative, so without a persisted order the same project can render different samples after a reload. The ordinal is also what a user reorders when the summing order is musically meaningful.

## Adapter contract

Adapters are ordinary nodes with durable identity, declared latency, and inspectable parameters. The v1 set:

| Adapter | Crossing | Declared behavior |
| --- | --- | --- |
| `CvUpsample` | control rate to audio rate | `Hold`, `Ramp`, or one-pole smoothing with a declared time constant |
| `CvDownsample` | audio rate to control rate | `Pick`, `Mean`, or `Peak` per control point |
| `MonoToStereo` | `Mono` to `Stereo` | gain law, default `Copy` (decision 16) |
| `StereoToMono` | `Stereo` to `Mono` | `Sum`, `Mean`, or `LeftOnly` |
| `NoteToMidi` / `MidiToNote` | note and MIDI domains | declared lossy fields, channel mapping |
| `GateToNote` / `NoteToGate` | gate and note domains | voice allocation policy, fixed key or key source |
| `CvToParam` | CV into a parameter destination | depth, polarity, curve |
| `ParamToCv` | parameter value out as CV | range mapping |
| `LaneSplit` / `LaneMerge` / `LaneReduce` / `LaneBroadcast` | lane count changes | reduction function, lane mapping |
| `AudioFeedbackDelay` | cycle break, stream domains | delay in frames, minimum enforced |
| `EventFeedbackDelay` | cycle break, event domains | delay in blocks |

Every adapter reports its latency in frames so compensation can account for it.

## Polyphonic lane contract

A stream or note port declares a lane specification:

```
LaneSpec = Fixed(u16) | Inherit | Reduce(ReduceOp) | Broadcast
```

- `Fixed(n)` — the port always carries `n` lanes. `Fixed(1)` is a mono-lane cable.
- `Inherit` — the output lane count equals the resolved input lane count. This is how a filter or a VCA participates in polyphony without knowing about it.
- `Reduce(op)` — an input that collapses `n` lanes to one by `Sum`, `Mean`, `Max`, or `First`. This is the explicit voice-domain exit the vision requires.
- `Broadcast` — an input that accepts one lane and presents `n`.

Lane resolution runs at compile time by propagation from sources through `Inherit` ports, then validation. An unresolvable or contradictory lane count is a typed rejection naming the ports.

There is no implicit lane broadcast, not even 1-to-N (decision 14). A lane-count mismatch on an edge is a compile rejection, and the editor auto-inserts a visible `LaneBroadcast` or `LaneReduce` on the offending drag — one user gesture, and the persisted patch still shows exactly what happens to the signal. Vision line 93 requires an explicit adapter for voice-domain crossings, and the same editor-assisted pattern covers mono-to-stereo, so there is one rule rather than two.

Buffer layout is lane-major: a poly stereo audio cable occupies `lanes × channels × frames`, so one lane's channels are contiguous and per-lane processing is a slice walk.

```
MAX_LANES: u16 = 32
```

32 lanes (decision 15) rather than the VCV-conventional 16. A poly stereo audio cable at 128 frames therefore costs 32 KB, and that arena lives on the audio thread beside the `DeviceTable`. Raising or lowering it later is a constant change, not a format change. A per-project value was rejected: it makes arena sizing and reproducibility project-dependent and stops performance fixtures being comparable.

Per-voice modulation stays inside its lane domain. A global modulator reaching a polyphonic destination passes through a `LaneBroadcast`; a per-voice signal leaving the instrument passes through a `LaneReduce`. There is no implicit crossing in either direction.

## Cycle, delay, and latency contract

**Hidden one-block conversion is removed from the production contract.** `schedule` in its current form stops being the compiler's entry point.

Compilation:

1. Build the node-level dependency graph, **excluding edges that terminate at a declared feedback-break input**.
2. Topologically sort the remainder. This is `topological_order` (`crates/spectre-graph/src/topology.rs:18`), which already returns `Err(cycle)` and is currently dead code.
3. Any remaining cycle is a compile error naming every participating node and the domain of the offending edges. The graph does not compile. The previously published generation stays live.
4. A feedback-break node declares its domain and its delay. `AudioFeedbackDelay` declares delay in frames; `EventFeedbackDelay` declares delay in blocks. The delay is a parameter the user sees and sets.
5. The plan records every break, its node, its domain, and its delay, so the UI can show where a loop closes and what it costs.

Minimum delay. **Graph-level feedback resolves to one sample, and the sub-block scheduler ships in this milestone** (decision 10). This is a deliberate scope increase: both this document's recommendation and the roadmap placed sub-block scheduling at Milestone 12, with a one-block minimum until then. The owner chose capability over budget, because a one-block floor means no graph-level Karplus-Strong, resonator, or physical modeling — the flagship modular instrument would ship unable to express its defining patches.

What the floor requires:

1. **Strongly-connected components are scheduled as chunks.** Nodes outside every SCC are block-processed exactly as described above. Nodes inside an SCC iterate at the declared frame floor.
2. **`SCC_FRAME_FLOOR = 1`.** One sample. Only SCC members pay per-sample dispatch; the rest of the graph is untouched. This is what VCV Rack does globally, and it is why VCV is CPU-hungry — here the cost is confined to declared cycles.
3. **The executor gains a second execution path.** Block dispatch and per-sample dispatch are two modes in `process_list.rs`, not one mode with a parameter. A node inside an SCC is called once per sample with single-frame slices.
4. **Compilation still rejects undeclared cycles.** An SCC without a declared feedback-break element is a compile error, exactly as above. The floor changes what a break element *costs*, not whether one is required. Vision line 87 is satisfied: the delay is a node the user placed and can see.
5. **The plan records SCC membership**, its frame floor, and its iteration count, so the UI can show where a loop closes and what it costs.

Open at the scheduler slice, and blocking it: **how control-rate values behave inside an SCC.** At a one-sample floor, chunk boundaries no longer align to the 8-frame control grid, so a control buffer point spans eight iterations. Interpolating per sample and holding per chunk are both defensible and they sound different. This is settled when the scheduler is specified, not guessed at during implementation.

Performance consequence to own: `CONTROL_PERIOD_FRAMES = 8`, `MAX_LANES = 32`, and this floor were each chosen independently and each spend from the same 64-frame budget. The release fixture set needs a large-SCC patch specifically, because no existing bench would find that cliff.

Latency. Two kinds of delay must not be confused:

- **Semantic delay** changes what the user hears in a loop. It is never inserted by the compiler.
- **Compensating delay** aligns parallel paths the user already expects to be aligned. The compiler inserts it and reports every insertion (decision 11).

Vision line 87 does not forbid the second. It is scoped entirely to feedback cycles — "the compiler does not silently convert arbitrary cycles into hidden one-block latency" — and line 89 requires latency behavior be "deterministic and visible," not absent. Reported plugin delay compensation satisfies both. The alternative, hand-aligning every send, parallel chain, and latent plugin, is unusable.

Each node descriptor declares `latency_frames` on the app thread, so the compiler computes path latency without touching node instances. The compiler computes arrival latency at every summing input and every bus boundary, inserts compensating delay on the shorter paths, and records every insertion in the plan so the total and per-path figures are readable by the UI and by the recording path. Compensation is skipped inside a strongly-connected component and reported as skipped. A latency change is a structural change and produces a new generation.

## Generation content and device ownership

A `RenderGeneration` contains:

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

Device instances are **not** in the generation (decision 8). They live in an audio-thread-owned `DeviceTable` — a fixed-capacity `Vec<Option<Box<dyn AudioNode>>>` — and the plan addresses them by slot. This is the most consequential decision in the spec; ADR 005 records it and its consequences for `swap.rs`.

The reason is state continuity. Under today's design, `Executor::new` (`crates/spectre-graph/src/process_list.rs:150`) takes node instances out of the graph by move. Recompiling therefore means reconstructing every node, which discards filter state, delay lines, and sounding voices on every graph edit. A DAW cannot glitch every device when a user adds an effect.

The two-step structural edit:

1. The app thread constructs a device instance and sends it through the control stream as `InstallDevice { slot, instance }`. Moving a `Box` into a slot is a pointer write; no allocation occurs on the callback. The audio thread acknowledges the sequence.
2. The app thread publishes a plan that references the slot, stamped with `requires_control_sequence` equal to that install's sequence. The audio thread adopts the plan only once it has applied that sequence.

Removal reverses: publish a plan that no longer references the slot, then send `RemoveDevice { slot }`, whose box travels the reclaim ring and is dropped on the app thread. Arenas retire the same way when the plan resizes them.

The existing `swap.rs` mechanism is unchanged in kind. `GraphPublisher` gains a `GenerationId` on the payload and an acknowledgement consumer; `ActiveGraph::poll_swap` gains the control-sequence precondition. `reclaim` already does the right thing.

## Realtime constraints

Non-negotiable on the callback: no allocation, no deallocation, no locking, no logging, no file or device I/O, no formatting, no mutable document traversal, no panic across an FFI boundary.

Specific hazards this design must avoid, each with its mitigation:

| Hazard | Mitigation |
| --- | --- |
| Arena resize on plan change | Arenas are built with the plan on the app thread and swapped whole |
| `Box<dyn AudioNode>` drop on the callback | Removal moves the box into the reclaim ring; drop happens on the app thread |
| `Arc` clone or drop of a meter or asset on the callback | Publication objects are created and released on the app thread only |
| Unbounded event growth | Every event port declares a per-block capacity; the arena is sized from declarations |
| Route lookup cost | Sorted index, bounded binary search, no hashing and no allocation |
| Unresolved control message churn | Counted and published; never retried on the callback |

Enforcement, required in the same slices as the features:

- a debug allocator guard that fails a test if the callback allocates or frees;
- a graph-swap stress test that publishes continuously while audio runs;
- an overload test that saturates every bounded queue and asserts explicit reconciliation, never silent divergence;
- callback benchmarks at 48 kHz / 128 frames on the documented mainstream eight-core reference machine, plus the 64-frame live stress mode, on reproducible project fixtures covering graph size, track count, voice count, device and modulation density, recording, graph swaps, and plugin teardown.

## Required code changes

This is a specification task; no Rust was touched. These are the changes the spec implies, recorded here rather than performed:

1. `crates/spectre-core/src/port.rs` — `PortType` becomes `SignalDomain`; `channels: u16` becomes `BusLayout`; the descriptor gains rate, lanes, fan-in policy, event capacity, and `ParamKey`.
2. `crates/spectre-core/src/signal.rs` — `SignalRate` stops being derived from the domain and becomes a declared field.
3. `crates/spectre-core/src/context.rs` — the `notes` and `params` global slices are deleted and replaced by per-port accessors.
4. `crates/spectre-core/src/events.rs` — `NoteEvent` gains `NoteInstanceId` and expression variants; `ParameterChange` gains `DeviceId`.
5. `crates/spectre-core/src/errors.rs` — new typed connection and compilation errors; `Internal("input port already connected")` at `crates/spectre-graph/src/graph.rs:107` is replaced by a real error.
6. `crates/spectre-graph/src/process_list.rs` — plan and executor rewritten around per-domain arenas, ordered fan-in, and a device table.
7. `crates/spectre-graph/src/topology.rs` — `schedule`'s automatic back-edge deferral is removed from the compile path; `topological_order` becomes the compiler's sort; feedback-break edges are excluded before the sort.
8. `crates/spectre-graph/src/nodes/delay_node.rs` — the false "auto-inserted" comments at lines 4 and 16 are corrected; the node becomes the user-placed `AudioFeedbackDelay`.
9. `crates/spectre-graph/src/process_list.rs:280` — the test `feedback_cycle_compiles_with_one_block_delay` is inverted into a rejection test.
10. `crates/spectre-graph/src/swap.rs` — payload gains `GenerationId` and `requires_control_sequence`; an acknowledgement ring is added. The mechanism is kept.
11. `crates/spectre-graph/src/node.rs` — descriptor-side latency, bus, lane, and meter-outlet declarations.
12. `crates/spectre-graph/src/tests/{cycle,routing,topology}_tests.rs` — currently pseudocode scaffolds; they become the real suites.
13. `plugins/spectre-synth/src/daw_node.rs:120` — reads its own note input port instead of the global slice.
14. `docs/adr/002-arcswap-graph-swap.md` — still a scaffold; it should be rewritten to record the implemented rtrb handoff and the device-table decision.

## Non-goals

Deferred to their own specs and interviews:

- Sample-accurate transport and tempo slicing. Named in Milestone 3 but specified separately; this spec only requires that `ProcessContext` not preclude a per-block slice list.
- Evaluation precedence between arrangement automation, clip automation, and realtime modulation. The vision defers it to a dedicated parameter-control specification; this spec fixes only storage and routing.
- The hybrid track aggregate, device chain projection, and chain/graph round-trip. Milestone 4.
- Durable device, module, and rack identity beyond the `DeviceId` and `ParamKey` this spec consumes. Milestone 1 and Milestone 12.
- ~~Sub-block feedback scheduling for strongly-connected components.~~ **No longer a non-goal.** Decision 10 moved it into this milestone at a one-sample floor.
- Declared multichannel layouts beyond mono and stereo. Descriptors are extensible; the compiler is not.
- Voice allocation policy inside the flagship instruments.
- VST3 bus negotiation and note-expression mapping. Milestone 13, though the note and bus descriptors here are what it will map onto.
- Replacing `app::engine::SynthProcessor` with the compiled graph.

## Acceptance criteria

1. Every compiled route carries its declared domain, rate, bus layout, and lane count; no two domains share an arena.
2. A connection whose domain, rate, bus layout, lane count, or fan-in policy is incompatible is rejected on the app thread with a typed error naming both ports; no such rejection uses `GeistError::Internal`.
3. An event reaches exactly the destinations named by its compiled edges, at its declared sample offset, and no node can observe events on a port it does not own.
4. A note-off, choke, or expression event resolves to the same `NoteInstanceId` as its note-on, through every intermediate note device.
5. Control-rate signals are stored and evaluated at `CONTROL_PERIOD_FRAMES` resolution, and the control rate in hertz does not change when the block size changes.
6. A rate crossing, domain crossing, bus-layout crossing, or lane-count crossing exists in a compiled plan only as a declared adapter node; the compiler inserts none of them.
7. Fan-in produces bit-identical output for the same project across processes, reloads, and machines of the same architecture, because contributor order is persisted rather than derived.
8. A cycle that does not pass through a declared feedback-break node fails compilation and names every participating node and the offending domain; the previously published generation stays live.
9. Every feedback-break node's delay appears in the compiled plan and is readable by the UI.
10. Declared node latency is included in path latency; every compiler-inserted compensating delay is recorded in the plan and reported; compensation inside a strongly-connected component is reported as skipped.
11. Device DSP state survives a graph edit: adding, removing, or reordering an unrelated device does not reset a device's filter state, delay line, or sounding voices.
12. A generation is adopted only after its `requires_control_sequence` has been applied, so a plan never references an uninstalled device slot.
13. A control message addressed to a target absent from the current generation is counted and dropped, never applied to a different target.
14. Nothing allocates, deallocates, locks, logs, formats, or performs I/O on the callback, proven by a debug allocator guard under graph-swap stress.
15. Retired plans, arenas, device instances, and meter publication objects are dropped on the app thread.
16. Every bounded queue publishes a saturation or overflow counter, and the app reconciles explicitly rather than advancing mirrors.
17. Callback benchmarks meet the 48 kHz / 128-frame baseline and the 64-frame stress mode on reproducible fixtures, including a fixture whose graph contains a large strongly-connected component.
18. `BusLayout::Declared` round-trips through validation and persistence while the v1 compiler rejects it with a distinct, actionable error.
19. A node inside a strongly-connected component iterates at `SCC_FRAME_FLOOR`; a node outside every component is block-processed, and the two dispatch paths produce identical output for a graph with no cycles.
20. Every strongly-connected component's membership, frame floor, and iteration count appear in the compiled plan.

## Slice boundaries

All gating decisions are settled; see `## Decision record` and `PLAN.md`.

- **T1 — Descriptors and validation.** `SignalDomain`, declared `SignalRate`, `BusLayout`, `LaneSpec`, `FanInPolicy`, `EventCapacity`, typed connection errors. `spectre-core` only. No executor change.
- **T2 — Ownership skeleton.** Device table, plan-and-arena generation payload, `GenerationId` and `requires_control_sequence` on the swap, acknowledgement ring, allocator guard. Audio domain only; behavior otherwise preserved.
- **T3 — Streams.** Per-domain arenas for audio, CV, and gate; control-rate buffers at `CONTROL_PERIOD_FRAMES = 8`; ordered summing fan-in; `CvUpsample` and `CvDownsample`.
- **T4 — Events.** Note and MIDI arenas, per-route delivery, deterministic merge, `NoteInstanceId`, expression variants, reserved note-off headroom, capacity and overflow counters. Removes the global slices from `ProcessContext`.
- **T5 — Parameters.** `(DeviceId, ParamKey)` addressing, route index, control-stream resolution, base/modulation/clamp/smooth layering, `accepts_audio_rate` declarations.
- **T5a — Single-track spike.** One instrument, one effect, one meter, driven end to end by the compiled graph, proving the typed contract against real audio before lanes, buses, and cycles land on top of it.
- **T6 — Cycles and latency.** Feedback-break declaration, cycle rejection, `AudioFeedbackDelay` and `EventFeedbackDelay`, declared latency, compensation. Inverts the current feedback test.
- **T6a — Sub-block SCC scheduler.** Component detection, chunked iteration at `SCC_FRAME_FLOOR = 1`, the per-sample dispatch path, and plan recording of component membership. Carries a blocking checkpoint on control-value behavior inside a component.
- **T7 — Buses.** Mono and stereo as first-class layouts, `MonoToStereo` and `StereoToMono`, `Declared` rejection path.
- **T8 — Lanes.** Lane propagation and validation, lane-major layout at `MAX_LANES = 32`, `LaneSplit`, `LaneMerge`, `LaneReduce`, `LaneBroadcast`.
- **T9 — Meters and analysis.** Node-declared outlets, atomic and ring publication, reclaim.
- **T10 — Fixtures and gates.** Reproducible performance fixtures including a large-SCC patch, overload tests, graph-swap stress, callback benchmarks at both block sizes.

## Stop conditions

Stop before a slice when:

- the control-value behavior checkpoint on T6a is unresolved;
- the design would require the compiler to insert a semantic conversion;
- a graph edit would reset device DSP state;
- a plan could reference a device slot that is not installed;
- the callback would allocate, deallocate, lock, or drop;
- fan-in order would not be reproducible across a reload;
- a bounded queue would saturate without a counter and an explicit reconciliation path;
- independent review has unresolved findings.

## Decision record

All seventeen were settled 2026-08-03. Each entry keeps its rejected alternatives and the reason each lost, because that reasoning is why the contract above has its shape. Three were settled against the recommendation and are marked.

| # | Accepted |
| --- | --- |
| 1 | One `Cv` domain, rate declared per port |
| 2 | Normalized, with `1.0` = one octave on pitch CV ports |
| 3 | `Gate` is a stream |
| 4 | `Meter` is a node-declared outlet, not connectable |
| 5 | `CONTROL_PERIOD_FRAMES = 8` **(against recommendation)** |
| 6 | Fan-in order is persisted route ordinals |
| 7 | `Audio`/`Cv` sum, `Gate` max, `Note`/`Midi` merge, one base per parameter |
| 8 | Audio-thread `DeviceTable`; generations carry only the plan |
| 9 | Durable `(DeviceId, ParamKey)` addressing |
| 10 | Sub-block SCC scheduling in this milestone, one-sample floor **(against recommendation, twice)** |
| 11 | Compensating delay permitted and reported |
| 12 | Runtime `NoteInstanceId` on the callback |
| 13 | Reserved note-off headroom; MIDI drops newest |
| 14 | No implicit lane broadcast; editor-assisted adapter |
| 15 | `MAX_LANES = 32` **(against recommendation)** |
| 16 | Upmix by copy, downmix by mean |
| 17 | `spectre-document` owns the editor; T5a spike lands mid-sequence |

### 1. Is `Cv` one domain with a declared rate, or two domains?

- **A. One `Cv` domain, rate declared per port.** One cable colour. A control-rate CV output cannot reach an audio-rate CV input without a visible `CvUpsample`. Cost: rate becomes a second thing the user must read on a port.
- **B. Two domains, `CvControl` and `CvAudio`.** The type system alone prevents mismatches and the UI can colour them differently. Cost: doubles the domain count and every adapter table, and modules that legitimately work at either rate must declare two variants.
- **C. CV is always audio-rate, as in VCV Rack.** Simplest and most flexible. Cost: a project with a few hundred modulation routes pays audio-rate cost for LFOs and envelopes that do not need it, which directly threatens the 64-frame stress target.

**Accepted: A.** It preserves the vision's requirement that audio-rate modulation exist only where declared, without doubling the type surface. C is the wrong default for a DAW even though it is right for the rack; A lets the rack declare audio-rate ports everywhere while the mixer does not.

### 2. What is the numeric convention for CV?

- **A. Normalized.** Bipolar `-1..1`, unipolar `0..1`. Matches DAW modulation conventions and the existing `ParamRange` normalized space (`crates/spectre-core/src/params.rs`).
- **B. Volt-style.** `±5 V` range, `1 V/oct` pitch. Matches VCV Rack, Eurorack literature, and every module concept a modular user already knows.
- **C. Normalized with a fixed octave convention.** `-1..1` generally, and `1.0 = 1 octave` on ports declared as pitch CV.

**Accepted: C.** It keeps one number space shared with parameters and automation while giving the flagship modular instrument the pitch relationship it needs. B would make every mixer and effect modulation destination speak volts for no benefit. This choice is effectively permanent once patches are persisted.

### 3. Is `Gate` a stream or an event domain?

- **A. Stream, same storage as CV.** Sample-accurate rising edges, trivially derived from any CV, matches modular practice, and supports comparator, slew, and analog-style envelope behavior.
- **B. Event, timestamped triggers.** Much cheaper — a bar of silence costs zero — and integrates naturally with note scheduling and MIDI clock.
- **C. Both, as two domains.**

**Accepted: A.** Gate as a stream is what makes the modular instrument behave correctly, and the vision names gates and triggers alongside CV rather than alongside notes. B's cost saving is real but it makes comparators, slew, and analog-style envelopes awkward. C is not worth the surface area in v1.

### 4. Is `Meter` a routable port domain, or a node-declared outlet?

- **A. Node-declared outlet, not connectable.** Meters are a graph-to-UI side channel backed by atomic cells and bounded rings. Analysis consumed by devices is `Cv`. Matches how `MeterCell` already works.
- **B. Keep `Meter` connectable.** A meter edge could feed an analysis device or a UI node placed in the graph.

**Accepted: A.** A meter edge is not a signal edge — it has no consumer inside the graph, no ordering constraints, and no latency. Keeping it in the connectable set invites a domain with no defined semantics. This removes a variant the vision lists as a signal domain; "meters and analysis feedback" is still satisfied, just not as an edge. The clarification is recorded in `PRODUCT_VISION.md`.

### 5. What is `CONTROL_PERIOD_FRAMES`?

- **A. 8 frames.** 6 kHz at 48 kHz. 16 points at 128 frames, 8 at 64. Smoothest modulation, highest overhead.
- **B. 16 frames.** 3 kHz at 48 kHz. 8 points at 128 frames, 4 at 64.
- **C. 32 frames.** 1.5 kHz at 48 kHz. 4 points at 128 frames, 2 at 64. Cheapest, audibly steppy for fast envelopes.
- **D. One value per block.** Simplest, but modulation resolution then changes with buffer size, so a project sounds different at 64 frames than at 128.

**Accepted: A, against the recommendation of B.** 8 frames, fixed as a constant rather than a project setting. The recommendation was 16 on cost grounds; the owner chose the smoother grid. D is disqualified by the requirement that buffer size not change the sound, and a project-configurable value would make renders non-reproducible across machines and performance fixtures non-comparable. This doubles control-buffer work against the 64-frame stress target, and it is one of three answers that spend from that same budget — see `## Cycle, delay, and latency contract`.

### 6. Is fan-in order persisted, or derived?

- **A. Persisted per-input route ordinals.** Bit-reproducible summing across reloads and machines. The user can reorder contributions. Cost: another durable ordering to allocate, persist, and restore exactly through undo.
- **B. Derived from source `NodeId` order.** Free, deterministic within a session. Cost: a reload that reallocates node ids can change summing order, so a bounce is not bit-identical to the previous bounce.

**Accepted: A.** Bit-reproducible renders are a professional expectation and float addition is not associative. The identity cost is one ordinal per edge and Milestone 1 is already defining `RouteId`.

### 7. What is the default fan-in policy per domain?

Defaults: `Audio` and `Cv` sum; `Gate` takes the maximum; `Note` and `Midi` merge; `Parameter` sums modulation over a single base. Three sub-questions:

- Should `Gate` fan-in be `Max` (OR-like, matches modular expectation) or `Sum` (can exceed the gate ceiling)?
- Should a parameter destination accept multiple *base* sources at all, or exactly one base plus any number of modulations?
- Should `Audio` inputs default to `Sum` or to `Single`, forcing an explicit mixer node? `Single` is more explicit and matches today's behavior; `Sum` is what every DAW user expects when dropping a second cable on an input.

**Accepted:** `Max` for gates, exactly one base per parameter, `Sum` for audio. Audio `Sum` is the one place implicitness is worth it, because summing is the universally understood meaning of two cables into one input and no information is lost.

### 8. Who owns device instances?

- **A. The generation owns them; rebuild on every recompile.** Simplest ownership, matches today's `Executor::new`. Cost: every graph edit resets filter state, delay lines, and sounding voices. Unshippable for a DAW.
- **B. Retire, reclaim, rebuild.** The app asks the audio thread to hand the generation back, moves surviving instances into a new plan, republishes. Cost: a gap where the audio thread has no graph, or a large amount of double-buffering machinery.
- **C. Audio-thread-owned `DeviceTable`; generations carry only the plan.** Instances are installed and removed by control messages that move a `Box`. State survives by construction. Cost: the audio thread holds instances not referenced by the current plan until an explicit removal, and adoption must wait on a control sequence.

**Accepted: C.** It is the only option that satisfies "adding an effect must not glitch the other twenty devices." The cost is one extra field on the generation and one precondition in `poll_swap`. This is the most consequential decision in the spec: T2 and the shape of `swap.rs` both depend on it.

### 9. How does a control message address its destination?

- **A. Compiled slot index plus a `GenerationId` stamp.** Fastest. Cost: every message in flight during a swap is stale and must be dropped or republished, so a knob turn during an edit can be lost.
- **B. Durable `(DeviceId, ParamKey)` plus a per-generation sorted index.** Survives swaps. Cost: a bounded binary search per message.
- **C. Both — durable key plus a slot hint validated against the generation stamp.** Fast path plus correctness. Cost: larger messages and two code paths.

**Accepted: B.** The lookup is a handful of comparisons against a cache-resident sorted array, and the app thread never needs to know which generation is live in order to send control. C is a later optimization if profiling ever justifies it.

### 10. What is the minimum graph feedback delay?

- **A. One block.** 2.67 ms at 48 kHz / 128 frames. Cheap, simple, explicit. Cost: no graph-level resonators, Karplus-Strong, or physical modeling; short feedback must live inside a node.
- **B. Sub-block, via strongly-connected-component chunk scheduling with a declared frame floor.** Enables short feedback in the patchable rack. Cost: a real scheduler, per-chunk dispatch overhead, and a hard performance cliff if the floor is set too low.
- **C. Reject sub-block feedback permanently.** Same as A with no upgrade path declared.

**Accepted: B, in this milestone, with a one-sample frame floor. This overrides the recommendation twice over** — both on timing (now, not Milestone 12) and on floor (one sample, not one block). The reason is that a one-block floor means no graph-level Karplus-Strong, resonator, or physical modeling, so the flagship modular instrument would ship unable to express its defining patches. The delay element declares frames either way, so the persisted patch format is unaffected. What this costs is a second dispatch path in the executor and an open question about control values inside a component; both are recorded in `## Cycle, delay, and latency contract` and ADR 006.

### 11. May the compiler insert compensating delay?

- **A. Yes, and it must report every insertion.** Standard DAW plugin delay compensation; parallel paths align as users expect. Cost: it is technically an insertion the user did not ask for, which sits close to the vision's "no hidden conversion" line.
- **B. No. The user places every delay, including compensation.** Absolutely explicit. Cost: unusable — every send, every parallel chain, and every latent plugin would require manual alignment.

**Accepted: A**, with the distinction written into the vision: *semantic* delay is never inserted, *corrective* delay may be, and every corrective insertion is recorded in the plan and visible in the UI. The draft's worry that a literal reading of line 87 forbids all automatic insertion does not survive checking: line 87 is scoped entirely to feedback cycles, and line 89 requires latency behavior be "deterministic and visible," not absent.

### 12. What identity does a note carry on the audio thread?

- **A. Runtime `NoteInstanceId: NonZeroU32`,** minted by whatever originates the note; durable `NoteId` stays in the document with the scheduler holding the mapping.
- **B. Durable `NoteId` from the document**, carried all the way through the graph.
- **C. Both fields on every note event.**

**Accepted: A.** Notes originating from live MIDI input, an arpeggiator, or a chord device have no durable document identity and cannot mint one on the callback. Vision invariant 9 is about editing, undo, expression editing, and persistence — all document concerns. B would force every note-creating device to fabricate durable ids in realtime. C doubles the event size for a field most consumers never read.

### 13. What is the event overflow policy?

- **A. Drop newest, count, and rely on a stuck-note guard.** Simple. Cost: a dropped note-off is a stuck note until the guard fires.
- **B. Reserve headroom in every event queue that only note-off and choke may use.** No stuck notes from overflow. Cost: a more complex writer, and the reserve is idle in the common case.
- **C. Prove capacity at compile time and treat overflow as a bug.** Cleanest contract. Cost: requires every producer to bound its output, which an arpeggiator with a randomization stage cannot always do honestly.

**Accepted: B for the note domain, A for MIDI.** A stuck note is the worst failure mode in the system and it is worth a few reserved slots. MIDI overflow degrades gracefully and does not need the machinery.

### 14. Is implicit 1-to-N lane broadcast allowed?

- **A. No. Every lane-count change is an explicit adapter,** and the editor auto-inserts a visible `LaneBroadcast` on the offending drag. Matches vision line 95 literally, which lists broadcast alongside reduction, split, and merge.
- **B. Yes for 1-to-N only.** Broadcast is lossless and unambiguous, so it arguably is not the "hidden conversion" the vision forbids. Matches VCV Rack ergonomics exactly.

**Accepted: A with editor assistance.** The graph stays literally explicit, the persisted patch shows exactly what happens, and the user still performs one gesture. The same pattern then covers mono-to-stereo, so there is one rule instead of two. Editor-assisted adapter insertion is confirmed as product design; it is the thing that makes A tolerable.

### 15. What is `MAX_LANES`?

- **A. 16.** Matches VCV Rack. A poly stereo audio cable at 128 frames costs 16 KB.
- **B. 32.** More headroom for dense polyphonic patches. 32 KB per poly stereo cable.
- **C. Per-project configurable.** Cost: arena sizing and reproducibility both become project-dependent.

**Accepted: B, against the recommendation of A.** 32 lanes rather than the VCV-conventional 16. The recommendation was 16 on arena-growth grounds; the owner chose headroom for dense polyphonic patches. Raising or lowering it later is a constant change rather than a format change either way. C is rejected outright because it makes performance fixtures non-comparable. This doubles worst-case poly arena size, and that arena now lives on the audio thread beside the `DeviceTable`.

### 16. What are the mono/stereo conversion laws?

- **Upmix.** Copy to both channels (correlated, `+3 dB` perceived) versus scale by `0.707` (constant power, level-preserving).
- **Downmix.** `Sum` (can clip), `Mean` (level-preserving, `-6 dB` on correlated material), or `LeftOnly`.

**Accepted:** upmix by copy, downmix by mean, both exposed as a parameter on the adapter with those as defaults. Copy-on-upmix is what every DAW does when a mono track hits a stereo bus, and mean-on-downmix is what avoids clipping. Changing these later changes the level of existing projects, so they are fixed here.

### 17. Where does the typed graph live, and does it ship without a consumer?

Two related boundary questions:

- **Crate boundary.** Does `spectre-graph` keep the mutable app-thread `Graph` editor, or does `spectre_document::graph` become the editor — the Milestone 4 aggregate the project-document spec already names as owning devices, chains, routing, sends, and returns — with `spectre-graph` reduced to compile-and-execute?
- **Sequencing.** Milestone 3 delivers no user-visible behavior until Milestone 4 attaches the hybrid track, because `app::engine::SynthProcessor` bypasses the graph entirely. Is it acceptable to build ten slices of engine with no production consumer, or should a thin Milestone 4 spike land mid-sequence to prove the contract against real audio?

**Accepted:** `spectre-graph` becomes compile-and-execute only, with the durable graph model owned by `spectre-document`, matching the aggregate table already accepted. And land the minimal T5a spike — one instrument, one effect, one meter, driven by the compiled graph — so the typed contract is proven against real audio before lanes, buses, and cycles are built on top of it. Ten slices validated only by unit tests is a large uncontrolled bet.
