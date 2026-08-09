<!--
Author: Jeff
Date: 2026-08-09
Description: The single active Spectre milestone
Notes: Exactly one milestone is active; the roadmap owns ordering
-->

# Current Milestone — R3 Live Shell

- **Status:** accepted
- **Last verified:** 2026-08-09
- **Scope:** qualified audio backend, callback bridge over the existing compiled plan, MIDI timestamps, health telemetry
- **Decision authority:** Jeff
- **Upstream sources:** `rebuild-roadmap.md`, RT-001..003, `../03-architecture/graph-compilation.md`
- **Downstream dependents:** `../status/NEXT.md`, implementation slices
- **Supersedes:** the R2 offline-graph milestone, exited 2026-08-09
- **Open decisions:** RT-003 acceptance; decision rows 19-22 all ratified 2026-08-09, with row 22's parameter seam accepted as design and implemented at R4
- **Known gaps:** no audio backend, callback bridge, or MIDI ingress exists; all ten R3 exit rows remain open

## R0/R1 exit record

R0/R1 exited 2026-07-17 with all exit evidence passing: formatting, strict Clippy, and the full workspace test suite; tempo/time/transport/event/ID/persistence property coverage; the deterministic offline harness; and traceability matching implementation. CORE-004's atomic-save design is accepted with implementation at R4; CORE-001's reorder/migration evidence is explicitly gated to R4/R5.

## R2 exit record

R2 exited 2026-08-09 against its stated roadmap exit gate. `spectre-graph` implements the GRAPH-001 split: an app-thread `EditableGraph` with stereo-bus semantics and single-feed inputs, and an immutable `CompiledPlan` built through validated compilation (ancestor inclusion, missing-input rejection, implicit-cycle diagnostics, factory layout verification, deterministic ordering, preallocated planar buffers). Plan execution is allocation-free and lock-free with take/restore buffer handoff.

The offline Pulse → Gain → Saturator fixture renders through the compiled plan, bit-identical to a hand-wired chain, with the silence, impulse, allocation, and deterministic-hash gates passing on that path. The renderer-neutral `DeviceParameterSnapshot` DTO carries app-model parameter edits into offline rendering, and `render_app_snapshot` rejects empty, partial, duplicate, aliased, unknown, mismatched, non-finite, out-of-range, and non-canonical input before constructing processors. Device Focus Drill-In added stable app-thread device selection and fail-closed Build → Shape focus without graph mutation, persistence, or live audio.

GRAPH-002 did **not** close at R2 and is not claimed. Only implicit-cycle rejection exists; explicit one-quantum-priced feedback edges remain unimplemented and stay gated at decision row 7 before the R11 modular surface. R2 exited on its four render gates, not on full GRAPH-002 satisfaction.

## Current evidence

Slice 2 landed `spectre-audio`, the decision-19 trait seam. `AudioBackend` owns device enumeration and stream construction; `AudioStream` owns the Stopped/Running/Closed lifecycle. Both are app-thread surfaces that may allocate. `StreamConfig` validates sample rate, channel count, and buffer size against explicit inclusive bounds before any driver call, so an invalid open fails before it reaches cpal.

`NullBackend` drives its callback through an explicit `pump` rather than a timer, so tests observe exact block counts with no sleeps, no races, and no hardware; it serves CI and will serve offline rendering. `CpalBackend` is the only file where cpal types appear, sits behind a default-on `cpal-backend` feature, and counts stream errors into an atomic instead of formatting or logging on the audio thread.

15 tests cover enumeration, inclusive config bounds, fail-closed lifecycle transitions including use-after-close, exact block geometry, rendering off the enumerating thread, and an allocation-free steady-state pump. cpal enumeration is asserted to be well-formed with or without hardware, so headless CI passes without pretending a device exists.

Slice 3 landed the RT-002 transport as `spectre_audio::{spsc, control}`. The SPSC ring is bounded and wait-free in both directions: push and pop each perform a fixed number of steps with no loops, no locks, and no allocation after construction. `push` returns the rejected value instead of consuming it, which is what keeps a full lane from running a destructor on the audio thread.

The three lanes implement decision 21 directly. Parameters use one `AtomicU32` latest-wins slot per registered target plus a version counter, so a 5,000-write sweep coalesces to a single application and cannot starve anything. Notes and transport are strict FIFO and are never dropped to make room; overflow returns an error and increments a counter read off-thread. Retired render state goes back to the app thread on a reclaim lane, and when that lane is full the value is handed back rather than dropped, because dropping it on the audio thread would deallocate.

19 tests cover the transport, including bit-exact signed-zero, subnormal, and NaN delivery; cross-thread FIFO over 100,000 items; repeated mask wrap without corruption; teardown that drops elements straddling the wrap; a drop-witness proving retired state is released on the app thread and never the render thread; and allocation-free send and drain.

Slice 4 landed `spectre_audio::bridge::RenderBridge`, the callback bridge. It drains transport commands and notes, executes the existing immutable `CompiledPlan`, and interleaves the plan's stereo output into the driver buffer. There is no second render path.

The equivalence evidence is the point of the slice: the test rebuilds the offline fixture exactly — same ID seed, same device values, same events — drives it through the bridge, and hashes the interleaved output with the same FNV-1a walk the offline harness applies to its planar output. The hashes match, and the fixture's peak is nonzero, so the match is not two silent buffers agreeing. Repeated renders are deterministic. Oversized blocks are refused into exact silence rather than stale audio or noise, notes beyond the block scratch stay queued and arrive on later blocks, and the render path is allocation-free in steady state.

Slice 4 surfaced a blocker that decision 22 now tracks. The accepted `AudioProcessor` contract bakes parameters in at construction and offers no callback-safe way to change them on a live plan, so the RT-002 parameter lane is implemented and tested but nothing can consume it. The bridge counts observed changes as `parameters_pending` rather than silently discarding them, keeping the gap visible. Recompiling a plan per change is not an option: compilation allocates.

Slice 5 landed the RT-001 guards as `rt_guard.rs`. A thread-local flag marks an RT section and the global allocator records any allocation or deallocation inside one, so a violation is attributed to the exact call instead of inferred from before/after totals. A positive control deliberately allocates inside a guarded section and asserts the guard catches it, which is what makes every passing result meaningful rather than a broken probe reporting success. Guarded paths: bridge render, the frame-capacity and empty-block refusals, control drain, retire, and the plan driven through a real backend callback.

Lock-freedom is enforced structurally, by scanning the RT modules for `Mutex`, `RwLock`, `Condvar`, `thread::sleep`, and logging macros. This is deliberately labeled the weaker of the two guarantees: it proves those modules never name a blocking primitive, not that no future call could reach one indirectly.

Slice 5 also found and fixed a blocker that would have stopped the live shell outright. `AudioProcessor` carried no `Send` bound, so `CompiledPlan` held `Box<dyn AudioProcessor>` and was not `Send`, meaning the bridge could never be moved onto an audio thread. The bound is now on the trait, all four v1 devices satisfy it unchanged, and a compile-time assertion pins `CompiledPlan`, the bridge, and both control halves as `Send` so it cannot regress silently. The DSP device I/O contract records the change.

No live driver has been opened yet; that is the slice 7 lifecycle drill on qualification hardware.

## Requirements in scope

RT-001 (no allocation, deallocation, blocking locks, I/O, logging, or panics across the callback boundary), RT-002 (bounded wait-free control↔render communication with defined overflow policy and off-thread reclamation), RT-003 (denormal flush and NaN/Inf containment with node isolation, emitting silence rather than noise).

RT-001..003 are currently traced as workspace policy with no implementation. RT-003 remains `proposed` in the requirements ledger and needs acceptance during this milestone.

## Decisions ratified at intake

Ratified 2026-08-09 as decision-gates rows 19-21:

1. **Audio backend.** `cpal` behind an `AudioBackend` trait seam with a null implementation for CI and offline use. Chosen to get the callback bridge built, with the trait seam making the backend replaceable. Row 19 carries a standing re-open trigger if the lifecycle drill or RT-001 guards catch cpal allocating, locking, or blocking on a callback-reachable path.
2. **Linux baseline.** ALSA for qualification, since PipeWire exposes an ALSA compatibility layer; JACK behind a cargo feature and never a hard dependency.
3. **RT-002 overflow policy.** Split lanes: parameters are latest-wins per `(device, parameter)` target; notes and transport are strict FIFO and never dropped, with overflow on those lanes counted and surfaced off-thread as a defect.

Still open:

- **RT-003 acceptance.** The requirement is still `proposed`; its per-node-type injection fixtures define what containment means. Slice 6 closes it.

## Qualification hardware

Jeff confirmed both macOS and Linux hardware are available, so the two-platform exit row is closable rather than partially blocked. The Linux pass runs at slice 7.

## Non-goals

VST3 hosting, recording, arrangement editing, piano roll, automation and modulation, latency compensation beyond health telemetry, mixer routing, and UI build-out beyond what makes the callback bridge observable.

## Exit evidence

- Audio backend selected, recorded as a decision-gates row, and qualified on at least one macOS and one Linux device.
- The callback bridge drives the existing `CompiledPlan` with no second render path.
- Allocation and lock guards wrap every callback-reachable path in CI (RT-001).
- Control↔render transfer uses a bounded wait-free structure with tested overflow policy and off-thread reclamation (RT-002).
- Denormal flush and NaN/Inf containment pass per-node-type injection fixtures, isolating the offending node and emitting silence (RT-003).
- A device lifecycle drill covers start, stop, device change, sample-rate change, and device loss without panic or audio-thread blocking.
- MIDI events carry timestamps through to the plan with defined ordering at equal timestamps.
- Health telemetry reports xruns and callback headroom off the audio thread.
- `cargo fmt`, strict Clippy, and the full workspace suite stay green.
- Traceability and status match the implementation.
