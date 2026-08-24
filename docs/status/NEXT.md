<!--
Author: Jeff
Date: 2026-08-09
Description: Immediate Spectre work queue
Notes: Only active-milestone slices belong here
-->

# Next

- **Status:** accepted
- **Last verified:** 2026-08-24
- **Scope:** immediately actionable R4 slices; closed R2 and R3 queues retained as exit records
- **Decision authority:** Jeff
- **Upstream sources:** `STATUS.md`, `../06-plans/current-milestone.md`
- **Downstream dependents:** implementation sessions
- **Supersedes:** the R3 slice queue, closed 2026-08-09
- **Superseded by:** none
- **Open decisions:** none blocking R4 start; decision 23 leaves Linux device qualification as debt to discharge here
- **Known gaps:** neither slice 1 nor slice 2 has run its manual protocol; later milestones are intentionally not decomposed here

## Next slices

1. ~~Wire `./spectre` to the qualified backend so Play produces sound through the existing compiled plan~~ — landed 2026-08-24 as `crates/spectre-app/src/engine.rs`. The app compiles a plan from its own validated four-parameter snapshot, queries the device's native rate through two new app-thread seam methods (`AudioBackend::default_sample_rate`, `AudioStream::stream_errors`), opens at 256 frames with the plan reserving twice that, and moves the existing `RenderBridge` into the render closure. **No new render path and no new DSP:** the app's live output hashes identically to `render_app_snapshot` over the same snapshot, using the FNV-1a walk both the offline harness and the bridge test already use. The transport button and `Space` both send before mutating, so a refused transport send leaves the UI unchanged rather than diverging from the render thread. 14 new tests; the app's own hardware drill opened an M-Audio AIR 192|6 at its native 88 200 Hz for 88 blocks with 0 xruns and 0 stream errors. **Two limits recorded rather than hidden:** the manual protocol has not run and no one has confirmed audible output by ear; and the held audition note is scaffolding that slice 5 replaces with clip playback.
2. ~~Implement decision 22's runtime parameter seam on `AudioProcessor`, then connect the RT-002 parameter lane the bridge already drains~~ — landed 2026-08-24. `AudioProcessor` gained one **required** `set_parameter`, so a device added later must answer the seam rather than silently ignore every edit made to it; all four shipping devices and the three test doubles implement it. `CompiledPlan::set_parameter` addresses one node's processor by the same bounded linear scan `process` already performs; `spectre_audio::route::ParameterRoutes` resolves an RT-002 target to a node and key by binary search over a frozen boxed slice, and is **added to `rt_guard`'s scanned module set** rather than left outside it. The bridge applies the drained lane once per block before `process`, so a block sees one coherent parameter set. `a_shape_edit_changes_live_audio_and_nothing_stays_pending` asserts a UI edit changes the rendered hash with `parameters_pending == 0`; a 500-write sweep coalesces to one application. 8 new tests. **`Gain` still does not smooth** — see D-R3; R4-2 deliberately did not add smoothing, because it would break the bit-exact live/offline hash equality and needs its own bound with a rationale row.
3. Discharge decision 23's Linux debt: run `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture` on real Linux hardware and record it in the milestone's qualification table. One command; it only needs the box.
4. Introduce the track model with a track-to-master signal path, keeping the graph compilation contract intact.
5. Add MIDI clips that play through a track, reusing the existing bounded event-ordering contract and MIDI ingress rather than a second event path.
6. Ship one small original synth and one original effect as the alpha's voice, per decision 15's deliberately-small scope.
7. Implement CORE-004's atomic save and reload over the accepted persistence contract, and land CORE-001's reorder evidence on the first persisted collection.
8. Add offline bounce and prove it matches the live path's computation, extending the hash-equivalence approach the callback bridge already uses.
9. Write and run the end-to-end fixture and the manual QA protocol that R4's exit requires.

## Closed R3 queue

1. ~~Add the audio-backend decision-gates row and ratify it~~ — closed 2026-08-09 as decision rows 19 (cpal behind an `AudioBackend` trait seam, with a standing re-open trigger on RT-001 violations), 20 (ALSA baseline, JACK behind a feature), and 21 (split-lane RT-002 overflow policy). Jeff confirmed macOS and Linux qualification hardware, so the two-platform exit row stays closable.
2. ~~Stand up cpal behind the `AudioBackend` trait seam with an explicit null implementation~~ — landed 2026-08-09 as `spectre-audio`. `AudioBackend`/`AudioStream` traits, validated `StreamConfig`, `NullBackend` with a deterministic pump, and `CpalBackend` behind a default-on `cpal-backend` feature. 15 tests cover enumeration, config bounds, fail-closed lifecycle transitions, exact block geometry, off-thread rendering, and an allocation-free pump. CI now installs `libasound2-dev`.
3. ~~Define the RT-002 control→render channel per decision 21~~ — landed 2026-08-09 as `spectre_audio::{spsc, control}`. A bounded wait-free SPSC ring returns rejected values instead of dropping them, so a full lane never runs a destructor on the audio thread. Parameters use one `AtomicU32` latest-wins slot per target with a version counter; notes and transport are strict FIFO with counted overflow; retired state travels back to the app thread on a reclaim lane. 19 tests, including a 5,000-write sweep coalescing to one application, bit-exact signed-zero/subnormal/NaN delivery, cross-thread FIFO over 100,000 items, teardown that drops elements straddling the ring wrap, and allocation-free send/drain.
4. ~~Bridge the callback to the existing `CompiledPlan` for a fixed fixture~~ — landed 2026-08-09 as `spectre_audio::bridge::RenderBridge`. It drains transport and notes, executes the existing immutable plan, and interleaves the result; there is no second render path. The equivalence test rebuilds the offline fixture exactly and hashes the bridge's interleaved output with the same FNV-1a walk the offline harness uses, and the hashes match on an audible (nonzero-peak) render. Also covers oversized-block refusal into silence, transport ordering, note deferral without loss, and an allocation-free render. **Surfaced a blocker:** the accepted `AudioProcessor` contract has no runtime parameter seam, so live parameter edits cannot reach processors; the bridge counts them as `parameters_pending` rather than discarding them. See decision row 22.
5. ~~Wrap every callback-reachable path in allocation and lock guards in CI (RT-001)~~ — landed 2026-08-09 as `rt_guard.rs`. A thread-local RT-section flag makes the global allocator attribute violations to the exact call rather than inferring from totals, and a positive control proves the guard actually fires. Covers bridge render, both bridge refusal paths, control drain, retire, and the plan driven through a real backend callback. Lock-freedom is enforced structurally by scanning the RT modules for blocking primitives, labeled as the weaker guarantee it is. **Found and fixed a blocker:** `AudioProcessor` had no `Send` bound, so `CompiledPlan` was not `Send` and the bridge could not move to an audio thread at all. A compile-time assertion now pins it.
6. ~~Accept RT-003 and land denormal flush plus NaN/Inf containment~~ — RT-003 accepted 2026-08-09 and implemented in `CompiledPlan::process`. Containment runs on each node's output before it can reach any downstream device: denormals flush to signed zero (software FTZ-equivalent, portable across the macOS/Linux targets), and any NaN or infinity silences that whole node and records it. Stats accumulate as plain integers on the render path and the bridge republishes them into atomics for off-thread reading. 7 injection tests using test-only poisoning source and effect devices, since the shipping devices contain non-finite values at their own boundary.
7. **Implemented, awaiting hardware.** The device lifecycle drill landed 2026-08-09 as `lifecycle_health.rs`. The deterministic half runs against the null backend and passes: three start/stop/restart cycles with rendering preserved across each gap, sample-rate changes reopening without losing the device, and device loss reported through `BackendError::Closed` on every operation with recovery on a fresh stream. The hardware half is `hardware_lifecycle_drill`, marked `#[ignore]` because CI and most dev machines have no usable output device. **macOS qualification passed 2026-08-09** on cpal/CoreAudio with an M-Audio AIR 192|6: 173 real driver callbacks, 0 xruns, 0 plan errors, 0 contaminated nodes, worst-case headroom 0.990. **Linux is the only thing left:** run `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture` on a Linux host with a real ALSA device and record it in the milestone's qualification table.
8. ~~Carry MIDI timestamps into the plan with defined ordering at equal timestamps~~ — landed 2026-08-09 as `spectre_audio::midi`. Absolute sample timestamps become block-relative offsets; messages past the block are refused for a later block and late messages clamp to the block start and are counted rather than discarded. Ordering reuses the accepted contract key exactly by making `NoteEventKind::rank` public instead of restating it, and sorting uses `sort_unstable_by`, which does not allocate and is deterministic because sequence makes the key a total order. MIDI carries no note identity, so ingress allocates a nonzero ID per attack and matches the release from a fixed table. 14 tests, including one that feeds the result through the real plan, which is what actually enforces the ordering contract.
9. ~~Report xrun counts and callback headroom through off-thread health telemetry~~ — landed 2026-08-09. `BridgeTelemetry` publishes fractional headroom per block as `f32` bits in an atomic, tracks the worst case, and counts a block that consumes its entire budget as an xrun. Worst-case headroom starts at infinity so the first block establishes the real minimum rather than a zero default looking like a saturated callback. `Instant::now` reads a monotonic clock through the vDSO/commpage, so it stays callback-safe, and the RT-001 guard covers the measured path.

## Closed R2 queue

1. ~~Close the accepted JSON project-codec decision~~ — verified 2026-07-16 with a checked-in canonical fixture, exact rewrite/round trip, schema and semantic rejection, and unknown-field preservation.
2. ~~Close the accepted fixed-point `BeatTicks` decision~~ — verified 2026-07-16 at 960 PPQ with checked overflow and transparent signed-integer serialization evidence.
3. ~~Complete the R1 exit disposition for CORE-001 reorder/migration scope and CORE-004 atomic-save API design~~ — dispositioned 2026-07-17: CORE-004 design accepted via the persistence contract; CORE-001 stays implemented with reorder evidence gated to R4 and migration evidence to R5.
4. ~~Implement the R2 editable-graph/compiled-plan split around the accepted DSP I/O contract~~ — landed 2026-07-17 as `spectre-graph` (EditableGraph/CompiledPlan, validated compilation, allocation-free execution, seven behavioral tests, graph-compilation contract).
5. ~~Move the existing Pulse → Gain → Saturator fixture onto the compiled plan and add silence/impulse/hash gates~~ — landed 2026-07-17: fixture renders through the plan bit-identically to the hand-wired chain; silence, impulse, allocation, and hash gates pass.
6. ~~Transfer device parameter snapshots from the app model to the offline plan before live audio work~~ — completed 2026-08-06: the renderer-neutral DTO lives in `spectre-dsp` with private fields, getters, and canonical clamping; the app emits exactly the four canonical fixture identities; offline rendering accepts a complete order-independent set and rejects incomplete, duplicate, unknown, mismatched, or non-canonical input before plan construction.
7. ~~Choose the next narrow, testable interaction slice without implying live audio, broad R3, or automation capability~~ — completed 2026-08-06 via an explicit blind code-level fallback because no copied `./spectre` user-feedback artifact was available. Device Focus Drill-In adds stable app-thread device selection, atomic Build → Shape focus, selected-only descriptor-backed Shape controls, selection-aware feedback/smoke output, and no live or persisted control path.

## Parallel research

- Obtain visible-session Bitwig Studio evidence.
- Obtain visible-session Ableton Live evidence to corroborate interview self-report.
- Inventory the REAPER guide table of contents.
- Keep the four action-sequence observations separate from the two thematic self-reports.

## Standing constraints

- No frequency or priority claims from the current workflow corpus.
- Every admitted workflow record updates all linked research artifacts in the same slice.
- Callback-reachable code remains allocation-free, lock-free, bounded, and free of I/O and panics.
