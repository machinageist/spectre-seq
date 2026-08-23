<!--
Author: Jeff
Date: 2026-08-21
Description: Blind verification scorecard for the R4-5 midi-clips spec, iteration 1
Notes: Opened in full — io.rs (198 lines), midi.rs (223 lines), bridge.rs, event.rs, the spec
  (1,518 lines), criteria.md, the scorecard template, requirements-ledger.md :25-70. Opened at
  every cited range plus surrounding context — control.rs, spsc.rs, transport.rs, tempo.rs,
  time.rs, meter.rs, id.rs, source.rs, spectre-graph/src/lib.rs, spectre-project/src/{lib,command}.rs,
  spectre-app/src/lib.rs, spectre-offline/src/lib.rs, bridge_plan.rs, rt_guard.rs, spsc_queue.rs,
  control_channel.rs, midi_ingress.rs, all three Cargo.toml dependency blocks, decision-gates.md,
  NEXT.md, STATUS.md, current-milestone.md, dsp-device-io.md. Opened all 20 distinct OBS- IDs the
  spec cites, individually, at their lines; recomputed corpus counts. Recomputed all seven CLIP-00n
  derivations and every arithmetic claim in §4.7. Sampled rather than fully read — spectre-graph
  and spectre-app source beyond the cited ranges, and the 85-record Ableton file beyond its 17
  cited lines. Did not compile or run the workspace gate; no test in §5 exists yet.
-->

# Scorecard: MIDI Clips

**Feature ID:** `midi-clips`
**Spec file:** gauntlet-output/specs/R4-5-midi-clips.md
**Reviewer agent:** blind verification agent, R4-5
**Date:** 2026-08-21
**Spec iteration reviewed:** 1

---

## Verdict: PASS

**Summary:** The spec does the one thing the slice was commissioned to do and does it precisely —
`R4-5-MERGE` (§4.3) reuses the shipped ordering key `(frame_offset, NoteEventKind::rank(), sequence)`
verbatim from `crates/spectre-dsp/src/io.rs:96`, calls the public `rank` rather than restating it,
and explicitly forbids reaching for `spectre_core::EventKind::order_rank`, which I confirmed is the
trap sitting next door and used by no production path. The most critical gap is a design constant
that does not survive the primitive it is built on: `SCHEDULE_LANE_CAPACITY = 2` fed to
`spsc::bounded` yields a **usable capacity of 3** (`spsc.rs:57-68`), so CLIP-006's "one in flight
plus one publishing" is not the lane the spec builds, and integration test I-10 asserts an overflow
at three publishes that cannot occur. That is a real defect in a bound and its test, but it sits in
§4.2/§5.2 rather than §7.1, it fails safe, and it does not outweigh a spec whose ~60 source-level
claims I checked line by line and found substantively correct in both directions.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | §4.2 marks `emit_block` and `install` callback-safe with the full RT-001 list; `install` is a pointer swap whose returned box **must** go to `ControlReceiver::retire` (verified `control.rs:310-324`, whose own comment at `:311-313` gives the same reason). §4.3 step 9's sort is `sort_unstable_by`, and the spec reuses ingress's existing non-allocation argument (`midi.rs:6-8`, verified) rather than inventing one. S-13/I-11 extend the real `rt_guard.rs` harness, which exists with a thread-local RT-section allocator. | — |
| 1B. Control↔render communication | 2 | Correct in shape: three existing lanes unchanged (`control.rs:248-258`, `:261-271`, `:310-324` all verified), retired schedules on the reclaim lane, and the fourth-lane shape change to decision 21 (`decision-gates.md:45`, verified to name three lanes and two policies) is **routed to §8 Q2, not asserted** — which is what keeps AF-1 clear. But the stated depth does not survive the primitive. `spsc::bounded(2)` computes `requested = 2.max(2) + 1 = 3`, `slots_len = 4`, `mask = 3`, and `capacity()` returns `mask`; one slot stays empty by design (`spsc.rs:57-68`, `:96-98`). The lane admits **three** schedules. CLIP-006's "a third queued schedule means the app thread is outrunning the render thread" describes a lane the spec does not build. | Either pass `SCHEDULE_LANE_CAPACITY - 1` to `bounded`, or restate CLIP-006 in terms of `Producer::capacity()` and drop the exact-count claim. Fix I-10 with it. |
| 1C. Numerical containment | 3 | §6.4 3C establishes this feature adds **no** DSP node — `ClipPlayer` does not implement `AudioProcessor` (verified, trait at `io.rs:163-172`) — so the containment obligation is correctly narrowed to "a non-finite velocity cannot reach a device," and two injection tests are named anyway: S-12 at the app-thread bake boundary and I-6 at the plan boundary. I-6's expected `ProcessError::InvalidEvent` is correct: `is_valid` requires `velocity.is_finite()` (`io.rs:123-146`). RT-003 and `OBS-VCV-VOLT-006` cited accurately. | — |
| 1D. Determinism | 3 | §1.3 and I-4/I-5 use the **existing** FNV-1a walks, both verified where cited (`spectre-offline/src/lib.rs:271-283`; `hash_interleaved` at `bridge_plan.rs:80`), with a nonzero-peak guard so two silent buffers cannot agree. The equal-timestamp tie-break is the shipped code's own: §4.3 correctly states `ProcessContext::new` rejects `order <= previous` so equality is a rejection, not a tie (verified `io.rs:97`). §4.3 then states the consequence rather than leaving it accidental — at identical frame and rank, a live event resolves before a clip event, by sequence band. | — |
| 1E. Graph and plan contract | 3 | §4.1: `ClipSchedule` is "the note-side analogue of `CompiledPlan`" — immutable, app-thread-built, render-executed — explicitly not a graph node and not participating in compilation. Editing clip content never recompiles the plan; the decision-22 reasoning (`decision-gates.md:47`, verified) is transferred with its rationale. GRAPH-001 cited correctly at `requirements-ledger.md:55`. | — |
| 1F. Failure behavior | 3 | Ten error states in §3.6, each counted, each failing closed to exact silence or `AllNotesOff`. The E-6/E-7 distinction is exactly right against source: `render` returns at `bridge.rs:166-173` **before** `apply_transport()` (`:175`), before `collect_notes()` (`:176`), and before `blocks_rendered` increments (`:204-206`), while the plan error at `:192-200` occurs after. E-8/E-9 emit `AllNotesOff` rather than a partial event set, which is the correct choice — a half-emitted block strands attacks. E-5 correctly cites that `Producer::push` hands the value back (`spsc.rs:80`). | — |
| 1G. Test specification | 2 | 41 tests, most genuinely falsifiable and pinned to line-level behavior: I-3 pins the clear→append edit at `bridge.rs:258`; I-7 pins that `blocks_rendered` does **not** increment on the refusal path; S-4 pins the sequence-collision mode; S-13/I-11 use the real `rt_guard.rs`. Against that, **I-10's assertion is wrong**: with a lane of usable capacity 3, `SCHEDULE_LANE_CAPACITY + 1 = 3` publishes all succeed, so the test can never pass and never exercises the counted-overflow path it names. One wrong constant among 41 assertions, and it surfaces as a loud CI failure rather than a false green. | Rewrite I-10 to push until `send_schedule` returns `Err`, asserting the box comes back intact and `schedule_overflows() == 1` — the pattern `control_channel.rs:160-175` and `spsc_queue.rs:34-48` already use against `producer.capacity()`. |

**Lens average:** 2.714 (19/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | §3.2's primary flow is sketch → hear in five steps; Branch A keeps the transport running through an edit and re-bakes without recompiling the plan; Branch B handles the loop seam at the wrap frame. The "no lost context" property is **asserted as a test** (§5.3 E-1: creating a clip leaves `selected_track_id()` unchanged) rather than described. §3.5 keeps scroll/zoom context. | — |
| 2B. Linked lenses | 3 | §3.3: "every field reads `MidiClip` / `ClipPlacement` directly. There is no per-lens copy of clip state and no view model that owns a second version of a note — criterion 2B is satisfied structurally rather than by convention." Identity is `ObjectId` from the project `IdGen` (verified `id.rs:45-58`), and U-8 asserts ID survival across reorder and round-trip — which is CORE-001's explicitly gated reorder evidence (`requirements-ledger.md:46`, verified to say it is gated on the first persisted collection at R4 intake). | — |
| 2C. Modulation visibility | 2 | PROD-002 cited at `requirements-ledger.md:63` (verified) and automation correctly deferred to R9 with `OBS-AB12-AUTO-002` (verified at `:125`). But no parameter surface is introduced here, so the criterion is addressed only by deferral. Correct, and thin. | Note in §3.3 that when a clip gains any modulatable field, the base/automation/modulation split is PROD-002's and inherits R9's presentation — so the note list does not become the precedent that forecloses it. |
| 2D. Keyboard-first, calm UI | 3 | §3.4 is a model answer to the AF-5 boundary: it fixes what criterion 2D permits (context-scoped resolution, remappable, searchable by name, no binding hard-coded into a handler) and refuses what AF-5 prohibits, citing the source. §3.5 has exactly one moving element with a reduced-motion alternative. §3.3's note list is a table, not spreadsheet density. Q9 records that someone must eventually choose keys, on evidence this project does not have. | — |
| 2E. Convergent-pattern grounding | 3 | Three convergences, each with two independent products: clips as named/positioned/toggleable containers (`OBS-AB12-CLIP-001` + `OBS-BW53-CON-002`); grid snap with modifier bypass (`OBS-AB12-ARR-004` + `OBS-BW53-AUTO-005`); timeline-vs-launcher authority (`OBS-AB12-SES-005` + `OBS-BW53-LAUNCH-001`). All verified to exist and to say what is claimed. One deliberate divergence — note chase, `OBS-AB12-ARR-010` — with its reason stated (`PulseInstrument` resets phase on every attack, verified `source.rs:186`) and routed to Q5 rather than settled. One line slip: `OBS-BW53-AUTO-005` is at `bitwig-studio-observations.md:40`, not `:41` (`:41` is `AUTO-006`). | Correct the AUTO-005 line reference to `:40`. |
| 2F. Differentiation | 3 | §Appendix A names a specific, checkable differentiator — one event path, one ordering key, enforced by the renderer itself, so a merge defect is a refused block and a nonzero counter rather than a listening bug found later — and then states its own limitation without prompting: it is an engineering differentiator a musician experiences only as "it always plays the same," and the benchmark set is far ahead on everything a musician would name. It explicitly refuses to claim competitors lack an ordering guarantee, "because no record in the corpus describes one." | — |
| 2G. Benchmark evidence discipline | 3 | I opened all 20 distinct `OBS-` IDs individually. Every one exists exactly once in the corpus and says what the spec claims — including the three load-bearing ones: ARR-004's "grid snapping is bypassed with a held modifier," CLIP-001's "deactivated clips do not play when launched or during arrangement playback," and ARR-010's "playback chases MIDI notes by default." Counts recomputed and match criteria.md exactly: AB12 85, PP 11, VCV 6, SR2 **2**, Logic **0**. The gaps are named, not filled: Logic Pro's `draft`/`inventory-only` status quoted verbatim from `logic-pro.md:10-11`; Serum 2's two records named by ID with `blocked-source-gap` quoted from `serum-2.md:10-11` and **explicitly not used** (D-R2's quarantine is therefore respected); and the largest gap — no record anywhere describes how a clip becomes a per-block event stream — named with its reason (the corpus is extracted from user manuals, which do not document engines). | — |

**Lens average:** 2.857 (20/7)
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | §6.4 3A produces exit row `current-milestone.md:83` — "A MIDI clip plays through a track into master" — verified verbatim, and quotes the R4 scope line at `:12` correctly. Nine deferrals each named with the owning milestone and an OBS ID: session grid (R10), automation (R9), recording/step-record (R7), MIDI time tools, program change, per-clip signature, grooves, follow actions, piano roll. Nothing smuggled. | — |
| 3B. Non-goal respect | 3 | No hosting, no plugin-format authoring, no cross-DAW compatibility, no cloud, no video. §6.2 identifies the clip format as the specific place a compatibility non-goal is easiest to violate and commits to Spectre's own JSON under decision 3's envelope — not SMF, not `.als`. | — |
| 3C. Deliberately small first devices | 3 | Adds no DSP device at all. `ClipPlayer` explicitly does not implement `AudioProcessor` (verified: trait at `io.rs:163-172`). `PulseInstrument` is unchanged, and decision 15 is cited correctly at `decision-gates.md:39`. | — |
| 3D. Originality | 3 | All seven bounds derive from Spectre's own arithmetic with derivations printed beside each constant and re-checked in §7.2. I recomputed every one and the arithmetic is correct (see Feasibility below). §Appendix A item 5 makes the sharper point: no product in the corpus documents a notes-per-clip or clips-per-track maximum, so there is nothing to copy — which is the condition decision 16 assumes. | — |
| 3E. Platform commitment | 3 | §4.6 is precise: no `#[cfg(target_os)]`, no syscall, no driver interaction; **no Linux claim authorized**; decision 23's debt named as inherited and undischarged with the empty Linux qualification row cited (`current-milestone.md:114`, verified empty). The one platform-shaped hazard — a backend granting a block larger than `plan.max_frames()` — is named as E-6 with its R4-1 dependency. Every test in §5 runs on both platforms without a device. | — |
| 3F. Accessibility trajectory | 3 | §3.7 is unusually concrete: label formats for clips and notes with pitch announced by name, **named custom actions so a screen-reader user is never required to drag**, text scaling that keeps the rectangle's time geometry, color-independent active state, explicit focus order that is never trapped. The list-over-canvas choice in §3.1 is argued as the concrete non-foreclosure, and the spec explicitly does not claim decision 17's audit (`decision-gates.md:41`, verified) passes. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes
**Auto-fail triggered:** No

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 2 | §7.1 is exceptionally thorough and mostly exact — I checked roughly 60 line-level claims across 16 files and the large majority land on the exact line, including hard ones (`rank`'s doc comment at `io.rs:149-151`, `select_track` at `lib.rs:297-301`, `render_vertical_slice` at `:288`, `ingress_output_is_accepted_by_the_real_plan` at `midi_ingress.rs:287`, `signature_at` at `meter.rs:173`). The **"Absent — verified absent"** block is correct in both directions: `spectre-project/src` contains exactly `command.rs` and `lib.rs`; `render` never calls `Transport::advance`; no `Ord`/`PartialOrd` on `NoteEvent`/`NoteEventKind`; `sort_events`/`TimedEvent` appear only in `event.rs`, `lib.rs:14`, and two `spectre-core` test files. Three defects: (a) `EventKind::order_rank` is transcribed as "transport 0 > note-off 1 > **note-off** 2 > control 3 > param 4" — the source is `NoteOff` 1, **`NoteOn`** 2 (`event.rs:25-26`); (b) three `midi.rs` field citations are off by one — `next_id` is `:90`, `sequence` `:91`, `late_messages` `:92`, cited as `:91`/`:92`/`:93`; (c) §4.3 anchors "its counter starts at 0" to `midi.rs:92`, which is `late_messages: 0`. The claims are true; the pointers are not. | Fix (a) to `note-on 2`; renumber (b) and (c) to `:90`/`:91`/`:92`. |
| 4B. Status vocabulary | 3 | Vocabulary used correctly throughout: the spec is `proposed`; R4-1/R4-2/R4-4 are "specified and **not implemented**"; §7.1 splits Implemented / Absent / Planned / Gated exactly as AF-2 requires; ledger rows are "proposed until Jeff accepts." The `implemented`/`verified` distinction is respected — the spec does not claim ingress is proven in production, it says nothing constructs `MidiIngress` outside its test file, which I confirmed by grep. | — |
| 4C. Traceability | 2 | Nearly every normative claim carries an ID, decision row, OBS ID, or source path, and every requirement and decision line number I checked is exact: RT-001/002/003 at `:28-30`, TIME-001…005 at `:36-40`, CORE-001/002/003 at `:46-48`, GRAPH-001 `:55`, PROD-002 `:63`, PROD-003 `:64`; decisions 1/5/6/15/16/17/21/22/23 at `:25`/`:29`/`:30`/`:39`/`:40`/`:41`/`:45`/`:47`/`:49`. `dsp-device-io.md` §Events at `:43-52` and §Realtime contract at `:75-79` are exact, and `:48` independently states the accepted `(frame_offset, semantic_rank, sequence)` order the spec reuses. Two defects: (a) `current-milestone.md:77` is cited **twice** (§4.4, §6.1) for "cloud services are a named non-goal" — `:77` is R4's non-goals list (VST3 hosting, recording, automation, session slots, sends, latency compensation, flagship synth) and does not mention cloud; the true home is `vision.md:57`. (b) CLIP-004's rationale claims 512 "clears the densest block Spectre's own tempo and tick bounds admit" — true only at one note per tick. `MidiClip::insert_note` refuses only over-capacity and out-of-range notes, so stacked notes at a shared start tick are representable and tempo/tick bounds do not bound event density. | (a) Recite to `docs/00-product/vision.md:57`. (b) Restate CLIP-004 as bounding the densest **monophonic** block and note that polyphonic density is handled by E-8's counted refusal, not by the bound. |
| 4D. Honest gaps | 3 | Eleven open questions, three flagged as needing Jeff before implementation, each naming what it blocks. Q11 is the standout: it raises the pre-existing `sort_events` trap, declines to propose deleting it as out of surgical scope, and says it is "raised because it is the trap this feature is most likely to fall into." §7.4 lists five dependencies with honest states including two "spec exists; not implemented." The two unmeasured numbers (JSON size, bake cost) are labeled estimates **in place** and routed to Q7/Q10. | — |
| 4E. Evidence commands | 3 | The workspace gate is quoted exactly as `docs/status/STATUS.md:46-48` has it — `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` — verified verbatim. The four targeted runs are well-formed `cargo test -p <crate> --test <file>` invocations naming files §7.2 creates. §5's honesty statement correctly notes every test runs without a device, which matches the existing `bridge_plan.rs` pattern. | — |
| 4F. No fake surfaces | 3 | §1.2 states outright that `./spectre` produces no sound today and cites `STATUS.md:37`, which I verified says exactly that ("nothing in `./spectre` uses them… Play changes the model's transport state without producing sound"). §3.3's empty states state the fact rather than advertise, with the reason given. §7.4 says a clip "can be modeled and unit-tested but cannot be *heard*" without R4-1. §4.7 refuses to assert a headroom threshold and §5.4 records rather than grades `xruns`/`worst_headroom`. | — |

**Lens average:** 2.667 (16/6)
**Lens pass:** Yes

---

## Auto-Fail Roll-Call

| Rule | Result | Basis |
|---|---|---|
| **AF-1** — Contradicting accepted authority | **Pass** | No Accepted row contradicted. TIME-002's ordering (`requirements-ledger.md:37`) is *reinforced*, not overridden — `rank` puts Off/AllNotesOff before On. The one shape change to an accepted row (decision 21's three lanes → four) is flagged in §4.3 and §4.4, proposed rather than asserted, given an alternative, and routed to §8 Q2 as "Jeff's, not an implementer's." |
| **AF-2** — Unbacked implementation claims | **Pass** | Every existence claim opened and checked. §7.1 distinguishes implemented / absent / planned / gated, and the absence claims are correct in both directions — no clip type exists, nothing advances the transport on the render thread, no `Ord` on the event types. Nothing is described as existing that does not. |
| **AF-3** — Realtime discipline violation | **Pass** | No allocation, deallocation, lock, I/O, logging, or panic proposed on a callback-reachable path. `install` is a swap and the spec explicitly warns that dropping the returned box on the render thread would violate RT-001. Every new bound has a defined, counted overflow policy; reclamation is off-thread on the existing reclaim lane. |
| **AF-4** — Borrowed numeric limits | **Pass** | Seven bounds, seven Spectre-derived rationale rows (CLIP-001…007), and `docs/01-requirements/requirements-ledger.md` **appears in §7.2's modified-files list** with the explicit statement that "prose beside a constant does not discharge it." No vendor limit copied. |
| **AF-5** — Conclusions the evidence does not support | **Pass, and notably so** | No default shortcut map (§3.4 refuses one by name and cites the source document); no gesture count; no time budget (§4.7 refuses to extrapolate from the single macOS measurement at `current-milestone.md:113`); no monitoring-latency threshold (§5.4 records rather than grades); no promoted workflow archetype (Q8 declines to claim list-first entry is what users want). Clip launching — the adjacent prohibited territory — is deferred to R10 in §3.1 and §6.4. |
| **AF-6** — Optimistic language | **Pass** | §6.3's self-audit checks out; I found no promotional description of state or difficulty. Uncertainty is carried in §8, unmeasured numbers are labeled in place. |

---

## Feasibility Check

Read the actual source files referenced in the spec before filling this table.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every type the spec builds on was opened at its cited location: `NoteEvent`/`NoteEventKind` (`io.rs:25`, `:33`), `MAX_NOTE_EVENTS_PER_BLOCK` (`:52`), `ProcessContext::new` (`:77-107`), `rank` public (`:152-157`), `MidiIngress` (`midi.rs:65-74`), `RenderBridge` (`bridge.rs:119-129`), `Transport`/`LoopRegion`, `TempoMap`, `BeatTicks`, `IdGen`, `EditHistory`, `ProjectDoc`, `AppModel`. New types are fully specified with private fields and accessors. |
| API/interface changes are feasible with current architecture | ✓ | `with_clip_player` is additive, so `RenderBridge::new` callers compile untouched. `collect_notes`'s clear→append change is one line (`bridge.rs:258`). The §4.3 step table's insertions all fit between existing statements. |
| Views/screens fit current navigation pattern | ✓ | Two panels inside the existing `Arrange` lens; selection stays `ObjectId`-keyed via the existing `select_track` pattern (`lib.rs:297-301`); empty-state wording mirrors `SHAPE_EMPTY_MESSAGE` (`:92`). |
| Dependencies are available and version-compatible | ✓ | No crate added. Verified all three `Cargo.toml` dependency blocks match §4.1 exactly: `spectre-audio` has no `spectre-project` edge and `spectre-app` has no audio crate — both stated correctly, and the missing `spectre-app → spectre-audio` edge is attributed to R4-1 rather than claimed. `eframe 0.32.3` with `default_fonts` confirmed. |
| Platform/renderer requirements are realistic | ✓ | No platform-conditional code; no Linux claim; the buffer-geometry hazard is named as inherited from R4-1/decision 23. |
| Test strategy is executable with current infrastructure | ✓ with one defect | 40 of 41 tests are executable against existing harnesses (`bridge_plan.rs`, `rt_guard.rs`, `app_model.rs`, `smoke_cli.rs` all verified present; `app_model.rs` has the 27 tests the spec claims). **I-10 is not executable as written** — see Priority 1.1. |
| Performance budget is realistic for target hardware | ✓ | All arithmetic recomputed and correct: `ScheduledNote` 32 B (16+4+4+2+6); 4,096 × 32 B = 128 KiB; ×16 = 2 MiB; ×2 = 4 MiB; 4,096 × 240 ticks = 983,040 ticks = 1,024 beats = 256 bars. CLIP-003: 86,400 s × 1,200 ÷ 60 = 1,728,000 beats × 960 = **1,658,880,000** ✓. CLIP-004: 48,000 × 60 ÷ (1,200 × 960) = **2.5**; 256 ÷ 2.5 = **102.4**; 102 + 102 = **204**; 512 ÷ 204 = 2.51× ✓. CLIP-005: a 32nd = 120 ticks > 102.4 ✓. CLIP-006: 48,000 ÷ 256 = **187.5** ✓. CLIP-007: 2,147,483,648 ÷ 1,000 ÷ 86,400 = **24.85 days** ✓. Sequence band: 2^63 ÷ 10^6 ÷ 3.156×10^7 ≈ **292,000 years** ✓. `NoteEvent` 32 B → 768 × 32 = **24 KiB** ✓. **Critically, the merged-array ceiling holds:** 512 clip + 512 lane = 1,024 = `MAX_NOTE_EVENTS_PER_BLOCK`, and `ProcessContext::new` rejects on `>` not `>=` (`io.rs:88`), so a maximal legal block passes. |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares all five, with R4-1 and R4-4 correctly identified as hard blockers and R4-2/R4-6 as soft. §1.3's primary success signal is explicitly stated as unreachable from the application until R4-1 lands. |

**Feasibility verdict:** Feasible with caveats
**Caveats:** (1) `SCHEDULE_LANE_CAPACITY = 2` passed to `spsc::bounded` produces a lane of usable capacity 3, so CLIP-006's rationale and I-10's assertion both describe a lane the spec does not build. (2) CLIP-005's `MAX_BLOCK_SEGMENTS = 2` rationale is pinned to a 256-frame block, but block size is a driver property negotiated by R4-1/R4-3, not a Spectre constant — at 1,024 frames the block spans 409.6 ticks (~0.43 beat) and at 2,048 frames ~0.85 beat, so the refusal the spec characterizes as biting only below a 32nd note would, at larger buffers, silence musically plausible short loops with `AllNotesOff`.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.714 | 35% | 0.950 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.667 | 20% | 0.533 |
| **Composite** | | | **2.798** |

**Pass conditions (from criteria.md, binding):**
- [x] Composite ≥ **2.30** — 2.798
- [x] Every lens average ≥ **2.00** — 2.714 / 2.857 / 3.000 / 2.667
- [x] No criterion scores **0** — none
- [x] At most **two** criteria score 1 — **zero** criteria scored 1
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear
- [x] Feasibility rule satisfied — I opened every source path the spec cites and checked the claim at the cited lines. §7.1's substantive claims hold in both directions; its defects are one transcription typo (`note-off 2` for `note-on 2`) and three off-by-one pointers within the same struct literal, none of which asserts code that does not exist or denies code that does. The `bounded(2)` defect is a design-constant error in §4.2/§5.2, not a §7.1 misdescription — §7.1's own `spsc` claims (`bounded` `:57`, `push` `:80` returning `Result<(), T>`, `pop` `:103`, `is_empty` `:118`) are all correct.
- [ ] Reviewer personally executed every command claimed as passing — **N/A.** No command in §5 is executable: the files §7.2 creates do not exist. I verified instead that the gate commands match `STATUS.md:46-48` verbatim, that every named crate and test-file convention exists, and that every harness the tests extend (`rt_guard.rs`, `bridge_plan.rs`'s `hash_interleaved`, `app_model.rs`, `smoke_cli.rs`) is present.

**All conditions met:** Yes → **PASS**

---

## Remediation Brief

This spec passes. The items below are corrections to make before implementation, not conditions of
the verdict.

### Priority 1 — Fix before an implementer touches this

1. **The schedule lane's depth does not match the primitive.** §4.2's `SCHEDULE_LANE_CAPACITY = 2`,
   §7.2's CLIP-006 row, and §5.2's I-10 all assume the lane holds exactly two. `spsc::bounded`
   computes `requested = capacity.max(MIN_CAPACITY) + 1`, rounds to `next_power_of_two`, and returns
   `capacity() == mask == slots_len - 1` (`crates/spectre-audio/src/spsc.rs:57-68`, `:96-98`;
   `MIN_CAPACITY = 2` at `:16`). `bounded(2)` therefore allocates 4 slots and yields **usable
   capacity 3**. Two changes: (a) restate CLIP-006 either as `bounded(SCHEDULE_LANE_CAPACITY - 1)`
   or in terms of `Producer::capacity()`, dropping the "a third queued schedule means the app thread
   is outrunning the render thread" claim; (b) rewrite **I-10** to push until `send_schedule` returns
   `Err`, then assert the box comes back intact and `schedule_overflows() == 1` — the pattern
   `crates/spectre-audio/tests/spsc_queue.rs:34-48` and `control_channel.rs:160-175` already use.
   As written, I-10 asserts an overflow that cannot occur at that publish count and can never pass.

2. **§7.1 mislabels an enum variant.** The "Absent but easy to mistake for present" block transcribes
   `EventKind::order_rank` as "transport `0` > note-off `1` > **note-off** `2` > control `3` >
   param `4`." The source (`crates/spectre-core/src/event.rs:24-28`) is `TransportSeek` 0, `NoteOff`
   1, **`NoteOn`** 2, `Control` 3, `ParamChange` 4. The rank values and the block's conclusion are
   correct; the variant name is not. Change the second "note-off" to "note-on."

3. **Three `midi.rs` line references are off by one.** In `MidiIngress::new`, `next_id: 1` is at
   `:90`, `sequence: 0` at `:91`, and `late_messages: 0` at `:92`. §7.1 cites `:91`/`:92`/`:93`, and
   §4.3's "its counter starts at `0` (`midi.rs:92`)" points at `late_messages`, not `sequence`. The
   claims are true — correct the pointers so a reader checking them lands on the right line.

### Priority 2 — Should fix for quality

4. **CLIP-004's sufficiency claim is over-broad.** "512 clears the densest block Spectre's own tempo
   and tick bounds admit" holds only at one note per tick. `MidiClip::insert_note` (§4.2) refuses
   only over-capacity and out-of-`[0, length)` notes, so a chord at a shared start tick is
   representable and tempo/tick bounds do not bound polyphony — a 10-note chord per tick would emit
   ~1,020 attacks in one 256-frame block at `MAX_BPM`. The bound is still safe, because E-8 fails
   closed to `AllNotesOff` and counts. Restate the row as bounding the densest **monophonic** block
   and point the polyphonic case at E-8 explicitly.

5. **CLIP-005 is pinned to an assumed buffer size.** The `MAX_BLOCK_SEGMENTS = 2` rationale computes
   one block as 102.4 ticks from 48 kHz / 256 frames and concludes the refusal only bites below a
   32nd note. Block size is negotiated by the driver (R4-1, decision 20's ALSA baseline), not fixed
   by Spectre. At 1,024 frames one block is 409.6 ticks (~0.43 beat) and at 2,048 frames ~0.85 beat,
   so short loop regions that are musically ordinary would be refused with `AllNotesOff`. Either
   state the buffer-size assumption as a precondition and route it to §8, or derive the bound from
   `plan.max_frames()` rather than a fixed 256.

6. **`current-milestone.md:77` does not say what §4.4 and §6.1 cite it for.** Line 77 is R4's
   non-goals list — VST3 hosting, recording, automation and modulation, session/live slots, mixer
   sends and returns, latency compensation, and the flagship synth. It does not mention cloud
   services or servers. The claim is true; recite both occurrences to `docs/00-product/vision.md:57`
   ("No cloud services, collaboration servers, or content stores").

7. **Give 2C a forward-compatibility sentence.** §3.3's note list introduces the first per-note
   field surface. Add a line stating that any modulatable clip field inherits PROD-002's
   base/automation/modulation split at R9, so the R4 list does not become the precedent that
   forecloses it.

### Priority 3 — Consider for excellence

8. `OBS-BW53-AUTO-005` is at `docs/02-reference-research/bitwig-studio-observations.md:40`, not
   `:41`; `:41` is `AUTO-006` (absolute vs. relative automation). The ID and the characterization
   are correct — only the line is off.

9. §3.6 E-10 refers to `spectre_project::validate` as though it were a public path. It is
   module-private (`fn validate`, `crates/spectre-project/src/lib.rs:105`). Extending it is fine;
   the name should not imply a public API.

10. §3.1 adds a bars-beats readout to the existing transport header — a modification outside the
    clip surfaces proper. It is defensible as necessary to place clips musically, but it is the one
    edit in §3.1 that R4's "minimal UI" scope does not obviously demand. Say why it is in scope, or
    move it to R4-4's track/transport work.

11. §7.1's citation of the offline FNV-1a walk as `:271-283` slightly over-extends — the hash loop
    is `:271-278` and `:279-284` constructs the `RenderReport`. Harmless, and the anchor line is
    exact.

---

**Reviewer's scope statement.** Opened in full: the spec (1,518 lines), `criteria.md`, the scorecard
template, `crates/spectre-dsp/src/io.rs`, `crates/spectre-audio/src/midi.rs`,
`crates/spectre-audio/src/bridge.rs`, `crates/spectre-core/src/event.rs`, and
`requirements-ledger.md` `:25-70`. Opened at every cited range with surrounding context:
`control.rs`, `spsc.rs`, `transport.rs`, `tempo.rs`, `time.rs`, `meter.rs`, `id.rs`, `source.rs`,
`spectre-graph/src/lib.rs`, `spectre-project/src/{lib,command}.rs`, `spectre-app/src/lib.rs`,
`spectre-offline/src/lib.rs`, `bridge_plan.rs`, `rt_guard.rs`, `spsc_queue.rs`, `control_channel.rs`,
`midi_ingress.rs`, all three `Cargo.toml` dependency blocks, `decision-gates.md`, `NEXT.md`,
`STATUS.md`, `current-milestone.md`, `dsp-device-io.md`, `README.md`'s precedence section. Opened all
20 distinct `OBS-` IDs individually at their cited lines and recomputed corpus counts. Recomputed all
seven CLIP-00n derivations and every arithmetic claim in §4.7 and §4.2. **Sampled rather than fully
read:** `spectre-graph/src/lib.rs` and `spectre-app/src/lib.rs` beyond the cited ranges, and
`ableton-live-observations.md` beyond its 17 cited lines plus a full-file ID count. **Not done:** I
did not compile anything or run the workspace gate — no test named in §5 exists yet — and I graded
AF-5 against `criteria.md`'s enumeration rather than opening the workflow field study directly.

---

**End of scorecard.**
