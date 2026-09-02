<!--
Author: Jeff
Date: 2026-08-09
Description: Active Spectre engineering milestone and retained predecessor exit debt
Notes: The roadmap owns ordering and the concurrent-milestone rule
-->

# Current Milestones — R5 Project Safety; R4 Exit-Pending

- **Status:** accepted
- **Last verified:** 2026-08-31
- **Scope:** active R5 project-safety engineering plus R4's retained operator-evidence obligation
- **Decision authority:** Jeff
- **Upstream sources:** `rebuild-roadmap.md`, product seeds, decisions 8/13/14/22, CORE-004, resolved D-R6
- **Downstream dependents:** `../status/NEXT.md`, implementation slices
- **Supersedes:** R4 as the sole engineering milestone; R4 remains exit-pending rather than exited
- **Open decisions:** none blocking R5 engineering; decision 13 is gated at R5 exit and decision 17 before beta
- **Known gaps:** R4 has one retained activity: its operator protocol has never passed. R5's crash-save, sidecar-autosave, and explicit-recovery seams exist, but none of the autosave/recovery behavior is reachable from `./spectre`; deeper forward fields on track/clip/view types are not represented by the current codec

## Milestone state

Jeff accepted the roadmap's concurrent-milestone rule on 2026-08-31, resolving D-R6. R5 is the
active engineering milestone. R4 is `exit-pending`, not exited: no operator has completed
`../05-quality/r4-qa-protocol.md`, so no document may claim an aggregate R4 PASS.

The rule does not waive dependencies. R5 may proceed because R4's remaining work is operator
evidence, while R5's safety work does not require that evidence to be implemented. Any R5 shell
change covered by the R4 protocol keeps the affected manual rows pending and must be exercised by
the eventual operator run.

## R5 Project Safety — active queue

Outcome: atomic save, journaled autosave, explicit recovery, migrations, bounded undo/redo, and
missing-media diagnostics. Exit requires crash and recovery drills over the product path.

1. ~~Crash-qualify atomic replacement against real process death.~~ Landed in `0883929`; the
   current follow-up replaces load-dependent sleeps with a completed-save handshake and retains
   the torn-write negative control.
2. ~~Write autosaves to an atomic sidecar without touching the saved project.~~ Landed in
   `f58c06b` as `spectre-project::journal`.
3. ~~Inspect, describe, accept, or decline recovery without applying it silently.~~ Landed with
   `64bb8c8` as `spectre-project::recovery` and its drill.
4. ~~Preserve feasible unknown fields through the shell's open → edit → save boundary before that
   boundary is reused by autosave.~~ Landed 2026-08-31 for envelope, project, device, and parameter
   maps, carried by stable identity without accepting unknown devices or parameters.
5. ~~Implement the first explicit schema migration with CORE-001 identity evidence and CORE-003
   forward-field preservation.~~ Landed 2026-08-31: schema 1 → 2 is an explicit product adoption
   step, preserves project identity and represented unknown fields, resumes the ID generator, and
   refuses a schema-1 label carrying schema-2 state.
6. Wire autosave and an explicit recovery offer into `./spectre`; use content change rather than an
   invented timer. Recover loads the autosaved state into memory as unsaved and retains both disk
   versions until a later successful manual Save, per Jeff's 2026-08-31 decision.
7. Bind the existing bounded command history to product mutations and verify grouped undo/redo
   across save, reload, and recovery.
8. Add missing-media diagnostics once persisted media references exist; until then the slice is
   gated rather than represented by a fake fixture.
9. Run product-path crash/recovery drills, the full workspace gate, and the R5 exit review.

## R4 Credible Alpha — exit-pending record

## R4 inherited-debt record — all four discharged

R4 carried four obligations from earlier milestones. All four are discharged; they remain here as
historical evidence and must not be rediscovered as current work:

1. ~~**Linux device qualification (decision 23).**~~ **Discharged 2026-08-28.** R3 exited on macOS hardware alone; the drill has now run on Arch Linux across three rows, one on the raw ALSA path with no sound server. It was not the single command the debt described — it first exposed a real `find_device` defect on ALSA. Decision 1's co-first-class commitment is discharged for the device seam and still open for the shell, which has no operator pass on any platform.
2. ~~**Runtime parameter seam (decision 22).**~~ **Discharged 2026-08-24 by slice 2.** The RT-002 parameter lane is consumed end to end. Slice 2 left the accepted `Gain` smoothing clause unresolved; **D-R3 was later closed by implementation on 2026-08-28**, using the existing block boundary and preserving live/offline equality.
3. ~~**CORE-004 atomic save.**~~ **Discharged 2026-08-28 by slice 7.** `crates/spectre-project/src/fs.rs` implements the accepted ordered replacement contract. **R5 process-death crash qualification landed 2026-08-29**, with its per-child completed-save synchronization repaired 2026-08-31; this paragraph remains the inherited R4 record, not a current gap.
4. ~~**CORE-001 reorder evidence.**~~ **Discharged 2026-08-28 by slice 7.** Persisted reorder and undo preserve identity. **The first migration evidence landed at R5 on 2026-08-31** against the checked-in schema-1 fixture and product adoption path.

## Wiring gap — closed 2026-08-24

R3 built a live shell that nothing launched. `spectre-audio` had a qualified backend, a control transport, a callback bridge, MIDI ingress, and health telemetry, all tested — and `./spectre` never constructed any of it, so Play produced no sound.

Slice 1 closed that. `crates/spectre-app/src/engine.rs` compiles a plan from the app's own validated snapshot, opens the default device through the seam at its native rate, and moves the existing `RenderBridge` into the render closure. The seam gained exactly two app-thread methods — `AudioBackend::default_sample_rate` and `AudioStream::stream_errors` — because `DeviceInfo` reported no format and `error_count` was inherent to `CpalStream` and unreachable through `Box<dyn AudioStream>`. The first of those earned itself immediately: the qualification interface's default rate is **88 200 Hz**, so a hardcoded 48 000 would have opened at the wrong rate or not at all.

No second render path was created: the app's live output hashes identically to `render_app_snapshot` over the same snapshot.

**What slice 1 does not establish.** No one has confirmed by ear that sound leaves the speakers, and the manual protocol in the slice's spec §5.4 — four resting engine states, the `Retry engine` control, text-size and window-size extremes, and mid-playback device removal — has not been run. Status is `implemented`, not `verified`.

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

Slice 7's lifecycle drill is implemented and its deterministic half passes against the null backend:
three start/stop/restart cycles with rendering preserved across each gap, sample-rate changes
reopening without losing the device, and device loss reported through `BackendError::Closed` on
every subsequent operation with recovery on a fresh stream. Hardware qualification later passed
on both macOS and Linux; the exact rows remain in the qualification record below.

## Requirements in scope

Product seeds for track routing, MIDI clips, and bounce; decision 22's parameter seam; CORE-004's atomic save; CORE-001 reorder evidence; decision 8's egui shell. RT-001..003 remain standing workspace policy and must not regress.

## Non-goals

VST3 hosting, recording, automation and modulation, session/live slots, mixer sends and returns, latency compensation, and the flagship synth. Those are R5 and later.

## Exit evidence

- `./spectre` opens the qualified backend and produces sound through the existing compiled plan,
  with no second render path — **OPEN. Not struck**, because "produces sound" is not established
  until someone hears it. Partially closed 2026-08-24, strengthened twice on 2026-08-28.
  The open path, the render, and the no-second-path claim are all evidenced. Two gaps in that
  evidence closed the same day. First, the 2026-08-24 macOS drill opened through `open_default`,
  which the binary does not call; `the_engine_main_actually_opens_reaches_a_real_device_and_renders`
  now drills `open_track_engine` with the same track list, tempo map, seed, and selected track
  `main.rs` uses. Second, **no drill asserted the render carried any signal** — a clean render of
  pure silence satisfied every counter, which `r4-qa-protocol.md` names as the highest-value
  failure it exists to catch. `RenderBridge` now folds a peak in the interleave loop it already
  walks and publishes it off thread, and both hardware drills assert it. On Linux the product's
  own path renders 90 driver blocks at `session_peak=0.16` with 0 xruns, 0 plan errors, and 0
  stream errors.
  **Still open, and irreducibly so:** no operator has confirmed the signal leaves the speakers.
  The row is `implemented`, not closed.
- ~~Decision 22's parameter seam is implemented, so a UI edit changes live audio~~ — **reopened and
  re-closed 2026-08-28.** It was first closed on 2026-08-24 against `build_engine_parts`, whose
  lane targets are derived from the model's own device snapshot, so the edit addressed a target
  that existed by construction. `./spectre` opens `open_track_engine`, whose targets are ObjectIds
  the graph builder allocates from `APP_GRAPH_SEED`; the two ID spaces are disjoint, so **no Shape
  edit reached live audio in the product** and the app reported "stored but did not reach live
  audio" on every one.
  `apply_parameter_edit` now resolves the destination **by role on the selected track** — the flat
  list's instrument row drives that track's instrument, its gloam row that track's insert, its
  gain row that track's gain — using the target-index helpers `TrackList` already owns.
  `a_shape_edit_changes_live_audio_through_the_engine_the_app_opens` asserts the rendered output
  changes with `parameters_pending == 0`, and `an_edit_for_a_device_no_track_hosts_stays_model_only`
  is its control: `saturator` is on no track, so its edit stays model-only and says so rather than
  being misrouted onto some other node.
  **This is not R4-4 §8 Q2's device-browser re-parenting, which stays R6 work.** Build and Shape
  still present the same flat list and no surface moved; only an edit's destination is resolved.
  The visible seam Q2 names — a device list that belongs to no track — is still there.
- ~~A MIDI clip plays through a track into master~~ — **partially closed 2026-08-28.** The clip model, the baked schedule, and the merge rule are evidenced; `crates/spectre-offline/tests/e2e_alpha.rs` renders three tracks' clips into master offline. The live path composes too: `the_live_bridge_plays_every_instrument_track` drives all three tracks' clips through `RenderBridge` and matches the offline hash exactly, at zero callback allocations. **Closed in the product too:** `./spectre` bakes each track's clips before the stream opens and plays them; Play sends the transport command alone, so the project's material replaces the audition note rather than merging with it. A project with no clips still auditions, so R4-1's evidence is unchanged. What remains open on this row is the manual protocol — nobody has confirmed by ear that a clip is what they hear.
- ~~One small original synth and one original effect ship as the alpha's voice~~ — **closed 2026-08-28, corrected the same day.** `Filament` and `Gloam` landed at R4-6 with thirteen DEV ledger rows. The row was first marked closed when R4-9 added `TrackInstrument::Filament`, which made only the **synth** reachable; `Gloam` was still constructed nowhere, so "one original effect ships as the alpha's voice" was not true of any render. The track insert slot closes it properly: every alpha track is `Filament → Gloam`, and `the_tracks_gloam_insert_is_in_the_signal_path` proves the effect is audible rather than merely stored.
- ~~Atomic save and reload round-trip a project containing tracks, clips, and device parameters (CORE-004 implementation, CORE-001 reorder evidence)~~ — **closed 2026-08-28.** Schema 2 persists tracks, clips, device parameters, the view context, and the ID generator's position; `save_reload_render_produces_the_same_hash` proves a round trip does not change what the composed project computes, and `track_ids_survive_reorder_across_a_save` closes CORE-001's persisted half. Crash durability is R5's and is not claimed.
- ~~Offline bounce renders the same project deterministically and matches the live path's computation~~ — **closed 2026-08-28.** `a_multi_block_bounce_matches_the_live_path_block_for_block` renders sixteen blocks each way to one hash, and the `--bounce` CLI reproduces that hash from a separate process. **Mechanism corrected 2026-08-28:** both assertions originally compared against a `LIVE_PATH_HASH` literal produced on macOS, which failed on Linux while every rendered sample still agreed — the row's own non-claim about libm predicted exactly that. The expected hash is now derived in-process, and the round trip holds as CLI == bounce (`bounce_cli`) and bounce == live (`bounce_equivalence`). Scoped to one process and one build: cross-machine bit-reproducibility is still not claimed.
- An end-to-end fixture plus a written manual QA protocol both pass — **half closed 2026-08-28.** The fixture exists and passes: `crates/spectre-offline/tests/fixtures/r4-alpha.json` with 15 assertions in `e2e_alpha.rs`. The protocol exists at `docs/05-quality/r4-qa-protocol.md`, and its standing constraints were corrected on 2026-08-28 after two of them were lifted by code before the protocol had ever run. `docs/05-quality/r4-qa-records.md` now carries block Q-1: the automated half on Arch Linux, Flow A passing 16/16, a clean workspace gate, and thirteen manual rows recorded `NOT RUN`, outcome `INCONCLUSIVE`. **No operator pass exists on any platform**, so this row stays open.

  **These two rows are the whole of R4's remaining exit, and they are one activity.** Eight of ten
  are closed. Row 1's residue — that signal reaching the driver reaches the speakers — is manual
  check row 1 of this very protocol. An operator working the protocol once discharges both; there
  is no second piece of work hiding behind the count.
- ~~Linux device qualification runs, discharging decision 23's debt~~ — **closed 2026-08-28** across three rows in the qualification record above, one of them on the raw ALSA path with no sound server. The drill did not run unchanged: it first exposed a real `find_device` defect on ALSA that had to be fixed before any device would open.
- ~~`cargo fmt`, strict Clippy, and the full workspace suite stay green~~ — **closed 2026-08-28 on Linux**: formatting clean, strict Clippy clean, 443 passed / 0 failed / 2 ignored, offline self-test exit 0. This is the first green full gate this project has ever produced on Linux; the workspace did not compile there until the same day. Two failures were cleared to reach it, both the R4-8 `LIVE_PATH_HASH` literal.
- ~~Traceability and status match the implementation~~ — **re-verified 2026-08-28** against all nine slices after four claims were found overstated: R4-6's effect was constructed nowhere, R4-2's seam was verified on an engine the app does not open, R4-5's five specified surfaces were undrawn, and the drill cited as proof `./spectre` reaches a driver exercised a function the binary never calls. All four are corrected in code or in the record. Rechecked by the QA protocol's manual row 15 at each run.

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

| Platform | Date | Backend / device | Blocks | xruns | Worst headroom | Plan errors | Contaminated | Frame rejections |
|---|---|---|---|---|---|---|---|---|
| macOS | 2026-08-09 | cpal / CoreAudio, M-Audio AIR 192\|6 | 173 | 0 | 0.990 | 0 | 0 | not recorded |
| macOS | 2026-08-24 | cpal / CoreAudio, M-Audio AIR 192\|6 (audio-crate drill, re-run) | 175 | 0 | 0.974 | 0 | 0 | 0 |
| macOS | 2026-08-24 | cpal / CoreAudio, M-Audio AIR 192\|6 (**app engine drill**, 88 200 Hz) | 88 | 0 | 0.970 | 0 | 0 | 0 |
| Linux | 2026-08-28 | cpal / ALSA via **pipewire-alsa**, C-Media USB Audio (card 2) | 363 | 0 | 0.844 | 0 | 0 | 0 |
| Linux | 2026-08-28 | same host and device, re-run | 369 | 0 | 0.826 | 0 | 0 | 0 |
| Linux | 2026-08-28 | cpal / **raw ALSA** `plug` → `hw:1,0`, onboard ALC285, **no sound server** | 198 | 0 | 0.822 | 0 | 0 | 0 |
| Linux | 2026-08-28 | cpal / pipewire-alsa (**app engine drill**, `open_default`, 44 100 Hz) | 94 | 0 | 0.837 | 0 | 0 | 0 |
| Linux | 2026-08-28 | cpal / pipewire-alsa (**the engine `main.rs` opens**, `open_track_engine`, 44 100 Hz) | 90 | 0 | 0.442 – 0.897 over 5 runs | 0 | 0 | 0 |

The 2026-08-24 rows add the frame-capacity rejection count, which the original record did not
carry. Both read 0, so on this host cpal's fixed buffer request is honored rather than merely
unproven — a larger-than-requested block is now known-absent here instead of unproven-absent.
The third row is the first qualification through the **application's** own open path rather than
a test's, and it is the row that establishes `./spectre` reaches a real driver.

The three Linux rows are the first Linux audio devices ever opened by this project, on
`mg-arch` (Arch Linux, kernel 7.1.9-arch1-2, x86_64, ALSA k7.1.9-arch1-2, PipeWire 1.6.8),
`rustc 1.98.0`, at commit `a157846`. Two things must be read with them, because both change what
the rows authorize.

**The drill did not run unchanged, contrary to decision 23's stated confidence.** It failed
`UnknownDevice(DeviceId("default"))` before opening anything: `CpalBackend::find_device` resolved
ids only against `host.output_devices()`, and on ALSA the default PCM is named `default` but is
absent from that enumeration, so the id `default_output_device()` reports resolved against
nothing. CoreAudio lists its default, which is why the round trip held on macOS for three prior
qualification rows and broke the first time the drill met ALSA. `find_device` now checks the
default before the enumeration. **This was a real seam defect on the app's own open path**, not a
test artifact, and no macOS run could have found it.

**The host supplied no usable ALSA default, so one was supplied for the run and must be read as
part of the configuration.** `mg-arch` has no `pcm.!default` at all — `pipewire-alsa`'s
`99-pipewire-default.conf` is not installed — so ALSA fell back to `defaults.pcm.card 0`, which
is an HDMI-only NVidia card with no device 0, failing `snd_pcm_open` with ENOENT. Each row above
ran with `ALSA_CONFIG_PATH` pointing at a temporary file that includes the system `alsa.conf` and
adds the missing default; nothing on the host was installed or modified. The first two rows route
through PipeWire's ALSA plugin, which is the ordinary Linux desktop path; the third bypasses every
sound server and drives `hw:1,0` directly, which is decision 20's raw ALSA baseline. A fourth
attempt at raw `hw:2,0` failed `device is no longer available` because PipeWire holds that
interface as its default sink — an exclusive-access fact about the host, not a defect.

Worst-case headroom on Linux (0.82–0.84) is materially tighter than macOS's (0.97–0.99). These
are different devices at different period sizes and the two columns are not comparable as a
platform ranking.

**Every number above, on both platforms, was measured on the three-node Pulse → Gain → Saturator
chain in `lifecycle_health.rs` — not on the plan the product runs.** That gap is now measured
rather than suspected. `crates/spectre-offline/tests/alpha_hardware.rs` drives the composed alpha
through the same device, the same lifecycle, and the same telemetry:

| Plan | Nodes | Worst headroom, 6 runs | Median | Median cost |
|---|---|---|---|---|
| Pulse → Gain → Saturator | 3 | 0.707 – 0.892 | 0.849 | ~15% of budget |
| **Alpha as specified** | **11** | **−0.685 – 0.758** | **0.634** | **~37% of budget** |

The product's real plan costs about **2.4× the callback budget** the qualification chain does.
**One of six alpha samples was negative (−0.685), meaning that block overran its budget** — on an
otherwise idle machine, measured alone. No run recorded a plan error, a contaminated node, a
frame-capacity rejection, or a refused clip event, and `session_peak` confirms every run carried
signal rather than passing on silence. But an overrun is an audible dropout, and this host
produced one in six runs of the plan the alpha actually is.

**These numbers replace an earlier set that was measured wrong, and the correction is instructive.**
The first pass reported the chain at 0.834 and the alpha at 0.404, sampled in back-to-back batches
while cargo was still compiling other targets. Run alone and sequentially, both figures rise and
the alpha's spread widens. `worst_headroom` is a worst case over blocks and is dominated by
machine load, not by plan size: **read every absolute headroom figure in this document as a
property of the host at that moment.** What survived the correction is the ratio — the alpha cost
about 2.4× the chain under both methodologies — which is the comparison the row is actually for.

**The drill that cited itself as proof `./spectre` reaches a real driver was on the wrong path,
and that is the third instance of this shape in one run.** `app_engine_opens_a_real_device_and_renders`
opens through `open_default`, which is called from the test file and nowhere else; `main.rs` calls
`open_track_engine`. The two build different plans, register different lane targets, and take
different code paths into the backend. `the_engine_main_actually_opens_reaches_a_real_device_and_renders`
now drills the binary's own path with the same track list, tempo map, seed, and selected track,
and `APP_GRAPH_SEED` moved out of `main.rs` into the library so the drill and the binary cannot
open with different graphs. Both drills pass on Linux; the fifth and sixth rows above are theirs.

**The alpha contained no effect until 2026-08-28, and R4-9's spec required one.** R4-6 shipped
`Gloam`; R4-9 §"Devices per track" specifies *"one `Filament` instrument and one `Gloam` insert"*
and §Traceability asserts *"Flow A renders through `Filament → Gloam`"*. `build_track_graph` wired
instrument → track gain → sum → master and constructed no effect at all, so the fixture's stored
`gloam` device reached no render, live or offline. Two smaller defects hid inside that one:
`alpha_fixture.rs` declared `GLOAM_DEPTH: usize = 0`, but index 0 is `damp_hz`, so the fixture
wrote `0.44` under a key whose range is 20..=20000 — an out-of-range value no constructor ever
saw — and `e2e_alpha.rs`'s assertion compared it against the wrong descriptor's default and
passed. `spectre-app`'s `parameter_route_nodes` also computed target indices as `index * 2`,
which addressed the wrong parameter the moment a track's stride stopped being two.

All of it is closed. `Track` carries an optional `TrackInsert`, `build_track_graph` wires
instrument → insert → gain where a track declares one, the parameter lane routes the insert's
depth so `r4-qa-protocol.md` row 3 has a live control to drag, and the target-index layout is
defined once on `TrackList` beside the ordering it indexes.
`the_target_index_helpers_agree_with_the_emitted_ordering` fails against the old `index * 2`
formula, and `a_list_without_inserts_keeps_the_node_identities_it_had_before_the_slot_existed`
holds the compatibility line: a track with no insert allocates nothing extra, so a project written
before the slot existed serializes byte-identically (CORE-003) and rebuilds the same node IDs.

## Single-platform exit and the debt it creates

R3 exited on macOS qualification alone, decided by Jeff on 2026-08-09 and recorded as decision 23. This narrows decision 1, which makes macOS and Linux co-first-class, and the narrowing is deliberate and scoped to R3's exit rather than a change to decision 1 itself.

What is and is not established:

- **Established:** the backend, bridge, and RT-001..003 behavior hold under a real CoreAudio driver; `spectre-audio` builds and links against ALSA on Linux with all 34 non-hardware audio tests passing; and the drill fails closed where no device exists, so it cannot report a false pass. **Superseded 2026-08-28:** the Linux device drill has now run — see the qualification record above.
- ~~**Not established:** that cpal's ALSA backend opens, streams, and survives device lifecycle events on real Linux hardware.~~ **Established 2026-08-28** on `mg-arch` across three rows, one of them on the raw ALSA path with no sound server, after fixing the `find_device` defect the run exposed.

The debt carries into R4 and must be discharged before any beta or release claim of Linux support:

```sh
cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture
```
