<!--
Author: Jeff
Date: 2026-08-09
Description: The single active Spectre milestone
Notes: Exactly one milestone is active; the roadmap owns ordering
-->

# Current Milestone — R4 Credible Alpha

- **Status:** accepted
- **Last verified:** 2026-08-09
- **Scope:** track to master, MIDI clip, a small original synth/effect, minimal UI, save/reload, bounce
- **Decision authority:** Jeff
- **Upstream sources:** `rebuild-roadmap.md`, product seeds, decisions 8/13/14/22, CORE-004
- **Downstream dependents:** `../status/NEXT.md`, implementation slices
- **Supersedes:** the R3 live-shell milestone, exited 2026-08-09
- **Open decisions:** none new at intake; decisions 13 and 17 gate later milestones
- **Known gaps:** R4 opens with no implementation. `./spectre` still does not use `spectre-audio`, so nothing the user can launch makes sound

## Inherited debt

R4 carries four obligations from earlier milestones. None is optional and none should be rediscovered later:

1. **Linux device qualification (decision 23).** R3 exited on macOS hardware alone. No Linux audio device has ever been opened. Discharge with `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture` on real Linux hardware. Until then no Linux support claim is authorized, and decision 1's co-first-class commitment remains undischarged.
2. **Runtime parameter seam (decision 22).** The design is accepted in `../03-architecture/dsp-device-io.md`; the implementation lands here. The RT-002 parameter lane already exists and is tested but nothing consumes it, so the bridge counts edits as `parameters_pending`. A playable alpha needs this closed.
3. **CORE-004 atomic save.** The API design is accepted in `../03-architecture/project-persistence.md`; the filesystem implementation lands here, with crash qualification at R5.
4. **CORE-001 reorder evidence.** Explicitly gated on the first persisted collection, which R4 introduces.

## Wiring gap

R3 built a live shell that nothing launches. `spectre-audio` has a qualified backend, a control transport, a callback bridge, MIDI ingress, and health telemetry, all tested — but `./spectre` never constructs any of it, so Play still produces no sound. Connecting the app to the audio thread is the first thing that makes R4's other work observable, and it is the honest headline for this milestone.

## R3 exit record

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

Slice 6 accepted RT-003 and implemented it in `CompiledPlan::process`. Containment runs on each node's output before that output can reach any downstream device. Denormals flush to signed zero using a software FTZ-equivalent, chosen over CPU control-register manipulation because it is portable across the macOS and Linux targets and needs no platform-specific unsafe code. Any NaN or infinity silences that entire node — not just the offending sample — and records it as the last contaminated node, so a single poisoned sample cannot leak the rest of the block.

Containment accumulates as plain integers on the render path, keeping `spectre-graph` free of atomics, and the bridge republishes them into its telemetry atomics each block so the app thread can read them without a lock. Seven injection tests cover NaN, both infinities, both denormal signs, clean output, cross-node isolation, whole-node silencing from one bad sample, accumulation across quanta, and the fact that the four shipping devices trip nothing during ordinary rendering. The poisoning source and effect devices are test-only, because the shipping devices already contain non-finite values at their own boundary and cannot produce the input this requirement is about.

Slice 8 landed `spectre_audio::midi`, the timestamped ingress. Absolute sample timestamps become block-relative frame offsets; a message past the end of the block is refused so it can be delivered with its own block, and a message timestamped before the block is late rather than invalid, so it clamps to the block start and is counted instead of discarded. MIDI carries no note identity, so ingress allocates a nonzero ID per attack and matches the release from a fixed 16×128 table, which is also what makes all-notes-off coherent.

Ordering at equal timestamps reuses the accepted contract key rather than restating it: `NoteEventKind::rank` became public so producers sort by exactly the key block validation enforces, and a second definition cannot drift out of step. Sorting uses `sort_unstable_by`, which does not allocate and is deterministic here because the unique sequence number makes the key a total order. Releases precede attacks at one offset, and equal-rank events keep the order the driver delivered them. Fourteen tests cover this, and the one that binds hardest feeds the ingress output straight into the real plan, which rejects unsorted events — a test that only inspected the array could agree with itself while violating the contract.

Slice 9 landed the health telemetry. `BridgeTelemetry` publishes fractional headroom per block as `f32` bits in an atomic, tracks the worst case, and counts any block consuming its whole budget as an xrun. Worst-case headroom starts at infinity so the first block establishes the real minimum; a zero default would have looked indistinguishable from a saturated callback. `Instant::now` reads a monotonic clock through the vDSO/commpage, so measurement stays callback-safe, and the RT-001 guard covers the measured path.

Slice 7's lifecycle drill is implemented and its deterministic half passes against the null backend: three start/stop/restart cycles with rendering preserved across each gap, sample-rate changes reopening without losing the device, and device loss reported through `BackendError::Closed` on every subsequent operation with recovery on a fresh stream. That covers the state machine, not a driver. No live driver has been opened, so backend qualification and the hardware drill both remain open.

## Requirements in scope

Product seeds for track routing, MIDI clips, and bounce; decision 22's parameter seam; CORE-004's atomic save; CORE-001 reorder evidence; decision 8's egui shell. RT-001..003 remain standing workspace policy and must not regress.

## Non-goals

VST3 hosting, recording, automation and modulation, session/live slots, mixer sends and returns, latency compensation, and the flagship synth. Those are R5 and later.

## Exit evidence

- `./spectre` opens the qualified backend and produces sound through the existing compiled plan, with no second render path.
- Decision 22's parameter seam is implemented, so a UI edit changes live audio.
- A MIDI clip plays through a track into master.
- One small original synth and one original effect ship as the alpha's voice.
- Atomic save and reload round-trip a project containing tracks, clips, and device parameters (CORE-004 implementation, CORE-001 reorder evidence).
- Offline bounce renders the same project deterministically and matches the live path's computation.
- An end-to-end fixture plus a written manual QA protocol both pass.
- Linux device qualification runs, discharging decision 23's debt.
- `cargo fmt`, strict Clippy, and the full workspace suite stay green.
- Traceability and status match the implementation.

## R3 exit record

R3's ten rows, as closed on 2026-08-09:

All ten rows are closed, two of them on macOS only under decision 23. R3 exited 2026-08-09.

- ~~Audio backend selected, recorded as a decision-gates row, and qualified on hardware~~ — closed 2026-08-09 **on macOS only**, per decision 23. cpal opened an M-Audio AIR 192|6 through CoreAudio and rendered 173 real driver callbacks with 0 xruns, 0 plan errors, 0 contaminated nodes, and worst-case headroom 0.990. Linux device qualification did not run and is carried to R4 as debt.
- ~~The callback bridge drives the existing `CompiledPlan` with no second render path~~ — closed 2026-08-09; bridge output hashes identically to the offline render of the same fixture.
- ~~Allocation and lock guards wrap every callback-reachable path in CI (RT-001)~~ — closed 2026-08-09 with an RT-section allocator guard and a positive control.
- ~~Control↔render transfer uses a bounded wait-free structure with tested overflow policy and off-thread reclamation (RT-002)~~ — closed 2026-08-09.
- ~~Denormal flush and NaN/Inf containment pass per-node-type injection fixtures, isolating the offending node and emitting silence (RT-003)~~ — closed 2026-08-09.
- ~~A device lifecycle drill covers start, stop, device change, sample-rate change, and device loss without panic or audio-thread blocking~~ — closed 2026-08-09 **on macOS only**, per decision 23. The drill passes against the null backend and against real macOS hardware, where two start/stop cycles on a live CoreAudio stream produced 173 callbacks with no panic, no xrun, and no blocking. The Linux run is carried to R4 as debt.
- ~~MIDI events carry timestamps through to the plan with defined ordering at equal timestamps~~ — closed 2026-08-09; the plan itself validates the ordering the ingress produces.
- ~~Health telemetry reports xruns and callback headroom off the audio thread~~ — closed 2026-08-09.
- ~~`cargo fmt`, strict Clippy, and the full workspace suite stay green~~ — 230/230 tests pass with 1 ignored hardware drill, plus the smoke and offline self-tests.
- ~~Traceability and status match the implementation~~ — updated 2026-08-09, including correcting stale claims that R3 had no implementation.

## Hardware qualification record

| Platform | Date | Backend / device | Blocks | xruns | Worst headroom | Plan errors | Contaminated |
|---|---|---|---|---|---|---|---|
| macOS | 2026-08-09 | cpal / CoreAudio, M-Audio AIR 192\|6 | 173 | 0 | 0.990 | 0 | 0 |
| Linux | not run | cpal / ALSA (decision 20 baseline) | — | — | — | — | — |

Linux **build** qualification did run on 2026-08-09, in a Linux aarch64 container with `libasound2-dev`: the workspace compiles and links against ALSA and all 34 non-hardware `spectre-audio` tests pass, which is the first time the CI `libasound2-dev` step has been exercised. That is a build and portability result, **not** a device qualification. The same run confirmed the drill fails closed on a machine with no audio device, panicking with "no output device to qualify against" rather than reporting a false pass — so the Linux row cannot be satisfied by a container or a VM without real audio.

The macOS run is the first time a live driver has ever been opened in this project. It confirms the callback bridge executes the compiled plan under a real driver, that RT-001 holds there, and that the render consumes about 1% of its time budget on this fixture.

## Single-platform exit and the debt it creates

R3 exited on macOS qualification alone, decided by Jeff on 2026-08-09 and recorded as decision 23. This narrows decision 1, which makes macOS and Linux co-first-class, and the narrowing is deliberate and scoped to R3's exit rather than a change to decision 1 itself.

What is and is not established:

- **Established:** the backend, bridge, and RT-001..003 behavior hold under a real CoreAudio driver; the workspace builds and links against ALSA on Linux with all 34 non-hardware audio tests passing; and the drill fails closed where no device exists, so it cannot report a false pass.
- **Not established:** that cpal's ALSA backend opens, streams, and survives device lifecycle events on real Linux hardware. No Linux audio device has ever been opened.

The debt carries into R4 and must be discharged before any beta or release claim of Linux support:

```sh
cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture
```
