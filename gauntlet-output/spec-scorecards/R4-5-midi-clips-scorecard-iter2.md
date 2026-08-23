<!--
Author: Jeff
Date: 2026-08-23
Description: Blind verification scorecard for the R4-5 midi-clips spec, iteration 2 (remediation 1)
Notes: Independent of the iteration-1 scorecard; every claim re-checked against source, never
  against a document. Scope statement at the foot of this file.
-->

# Scorecard: MIDI Clips

**Feature ID:** `midi-clips`
**Spec file:** gauntlet-output/specs/R4-5-midi-clips.md
**Reviewer agent:** blind verification agent, R4-5 iteration 2
**Date:** 2026-08-23
**Spec iteration reviewed:** 2 (remediation 1)

---

## Verdict: PASS

**Summary:** The remediation is correct and goes further than the iteration-1 scorecard did —
it establishes that `spsc::bounded` clamps every argument from 0 through 3 onto the same
four-slot ring, so the scorecard's own prescribed fix (`bounded(SCHEDULE_LANE_CAPACITY - 1)`)
would not have worked, and sets the constant to 3 because 3 is the only argument that equals what
the primitive delivers. I recomputed that arithmetic from `crates/spectre-audio/src/spsc.rs:57-68`
and `:96-98` and it is exactly right; the constant moved at every site, and I-10 is now a
falsifiable test built on two patterns that exist verbatim. **The one remaining defect is the same
failure mode one lane over:** §4.7's "absolute ceiling is five copies per bus — 10 MiB" assumes at
most one retired-but-unreclaimed schedule per bus, but the reclaim lane is `bounded(32)`, which
delivers usable capacity **63**, and the spec's own I-9 stipulates five installs before `reclaim`
is called — so the stated memory ceiling is not a ceiling.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | Unchanged by this iteration and re-verified. §4.2/§4.3 mark `emit_block` and `install` callback-safe against the full RT-001 list; `install` is a pointer swap whose retired box must go to `ControlReceiver::retire`, which exists at `control.rs:314-324` with the comment at `:311-313` giving the same RT-001 reason the spec gives. §4.3 step 9's sort reuses ingress's shipped non-allocation argument (`midi.rs:6-8`, `:136-143` — `sort_unstable_by` on `(frame_offset, kind.rank(), sequence)`, verified verbatim) rather than inventing one. §4.3's revised step table names current line positions and all thirteen land: `:165`, `:166-173`, `:175`, `:176`/`:258`, `:181-186`, `:188-200`, `:202-207`. S-13/I-11 extend the real `rt_guard.rs`. | — |
| 1B. Control↔render communication | 2 | The iteration-1 defect is **fixed, and fixed better than it was prescribed.** `SCHEDULE_LANE_CAPACITY = 3` (§4.2 line 633) is now derived from the primitive: I recomputed `bounded(n)` for n = 0…4 and confirm `requested = n.max(2) + 1`, `slots_len = requested.next_power_of_two()`, `mask = slots_len - 1`, so n ∈ {0,1,2,3} all give 4 slots / capacity 3 and n = 4 gives 8 slots / capacity 7. `Producer::capacity()` returns the mask (`spsc.rs:96-98`). Three publishes succeed and the fourth returns `Err`, exactly as CLIP-006 states. §4.4's RT-002 disposition is exact at every cited line (`control.rs:248-258`, `:261-271`, `:310-324`). **The remaining defect:** §4.7 states "a bus can hold one installed schedule, three queued, and one retired-but-unreclaimed copy, so the absolute ceiling is five copies per bus — **10 MiB**." Nothing bounds the reclaim lane to one. `bounded::<RetiredState>(DEFAULT_RECLAIM_CAPACITY)` with `DEFAULT_RECLAIM_CAPACITY = 32` (`control.rs:19`, `:217`) yields 33 → 64 slots → **usable capacity 63**, and `ControlSender::reclaim` (`:274-281`) drains only when the app thread calls it — a cadence this spec never fixes. §5.2's own I-9 ("install 5 schedules across 5 blocks … call `ControlSender::reclaim`") requires four retired schedules to be simultaneously outstanding, contradicting §4.7 directly. | Restate §4.7 and CLIP-001's "×2 … = 4 MiB" as conditional on a stated invariant — the app thread calls `reclaim()` at least once per publish — and give the unconditional bound from the reclaim lane's real depth (`bounded(32)` → 63), or bound it by counting `schedules_held`/reclaim occupancy. State it once, in the same idiom CLIP-006 now uses. |
| 1C. Numerical containment | 3 | Unchanged and still correct. §6.4 3C narrows the obligation correctly because `ClipPlayer` adds no DSP node — the `AudioProcessor` trait is at `io.rs:163-172` and `ClipPlayer` does not implement it. Two injection tests remain named (S-12 at the bake boundary, I-6 at the plan boundary), and I-6's expected `ProcessError::InvalidEvent` is right: `is_valid` (`io.rs:122-146`) requires finite velocity in `[0,1]`, and `ProcessContext::new` returns `InvalidEvent` at `:93-95`. | — |
| 1D. Determinism | 3 | Untouched by the remediation and re-verified. The FNV-1a walk cited for I-5 is real (`spectre-offline/src/lib.rs:271-278`, seed `0xcbf2_9ce4_8422_2325` at `:271`). §4.3's equal-timestamp claim is exact: `ProcessContext::new` builds `(frame_offset, kind.rank(), sequence)` at `io.rs:96` and rejects `order <= previous` at `:97-99`, so equality is a rejection and the band split makes live-before-clip a deterministic consequence rather than an accident. | — |
| 1E. Graph and plan contract | 3 | §4.1 keeps GRAPH-001's split (`requirements-ledger.md:55`) and states `ClipSchedule` is the note-side analogue of `CompiledPlan` — app-built, render-executed, not a graph node, not part of compilation. Editing clip content never recompiles the plan, transferring decision 22's reasoning (`decision-gates.md:47`) rather than restating it. Untouched by this iteration. | — |
| 1F. Failure behavior | 3 | Ten counted error states, each failing closed. E-6/E-7's ordering claim is exactly right against source: the frame-capacity refusal at `bridge.rs:167-173` fills silence, counts, and **returns** before `apply_transport()` (`:175`), before `collect_notes()` (`:176`), and before `blocks_rendered` (`:204-206`), while the plan error at `:192-200` occurs after step 10's advance. E-5 correctly cites `spsc.rs:80` for the value being handed back. §4.3's new `schedules_held` counter is the right shape: a reclaim-lane refusal means the render thread keeps holding the box rather than dropping it, which is what `retire`'s `Err(rejected)` return at `:317-321` demands. | — |
| 1G. Test specification | 3 | **I-10 is now genuinely falsifiable and the `==` is justified.** The rewrite loops until `Err` with no arithmetic on the publish count, and assertion (a) — the loop accepted exactly `SCHEDULE_LANE_CAPACITY` — fails if anyone changes the constant off the `bounded` fixed point in either direction: at 2 the lane still delivers 3 (accepted 3 ≠ 2), at 4 it delivers 7 (accepted 7 ≠ 4). The sibling `>=` the spec contrasts it with exists verbatim at `control_channel.rs:174` (`assert!(accepted >= 4, "requested depth must be honored")`) and the spec's explanation of *why* that one is `>=` is correct — `control_channel(&targets(1), 4, 8)` yields capacity 7. Assertion (d) cites `spsc_queue.rs:43-46`, which is verbatim the comment "Draining one element makes room for exactly one more" followed by the pop/push/push-fails triple. Assertions (b) and (c) fail if the box is dropped or the counter is not incremented. Forty-one tests, all executable against harnesses that exist. | — |

**Lens average:** 2.857 (20/7)
**Lens pass:** Yes — avg ≥ 2.0, one 1-or-below? none; zero 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | Untouched. §3.2's primary flow is sketch → hear; Branch A keeps the transport running through an edit and re-bakes without recompiling the plan; Branch B handles the loop seam. §5.3 E-1 asserts the no-lost-context property as a test rather than describing it. | — |
| 2B. Linked lenses | 3 | Untouched. §3.3 states every field reads `MidiClip`/`ClipPlacement` directly with no per-lens copy, and identity is the project `IdGen`'s `ObjectId`. U-8 asserts ID survival across reorder and round-trip, which is CORE-001's gated reorder evidence. | — |
| 2C. Modulation visibility | 2 | Unchanged from iteration 1, and the iteration-1 Priority-2 item asking for a forward-compatibility sentence was **not** taken up. `PROD-002` appears twice in the spec — once in §7.1's ledger-line list (`requirements-ledger.md:63`, verified correct) and once in §6's deferral of automation to R9 — but §3.3's note list, which introduces the first per-note field surface in the product, still says nothing about inheriting the base/automation/modulation split. Correct by deferral, and still thin. | Add one line to §3.3: when a clip field becomes modulatable, its presentation is PROD-002's and inherits R9's split, so the R4 list is not the precedent that forecloses it. |
| 2D. Keyboard-first, calm UI | 3 | Untouched and still a model AF-5 boundary: §3.4 fixes context-scoped resolution, remappability, and searchability by name while refusing to fix a default map, citing the prohibition's source. §3.5 has one moving element with a reduced-motion alternative. Q9 records that someone must eventually choose keys, on evidence this project does not have. | — |
| 2E. Convergent-pattern grounding | 3 | The line slip is fixed and verified: `OBS-BW53-AUTO-005` is now cited to `bitwig-studio-observations.md:40`, and `:40` is in fact AUTO-005 ("Shift bypasses grid, Alt+drag curves a transition…"), with AUTO-006 at `:41`. The three convergences each still carry two independent products, and the one deliberate divergence (note chase, `OBS-AB12-ARR-010`) keeps its stated reason — `PulseInstrument` resets phase on every attack, `source.rs:186` — and stays routed to Q5. | — |
| 2F. Differentiation | 3 | Untouched. Appendix A names a checkable differentiator (one event path, one ordering key, enforced by the renderer) and immediately states its own limitation, refusing to claim competitors lack an ordering guarantee "because no record in the corpus describes one." | — |
| 2G. Benchmark evidence discipline | 3 | Untouched by the remediation. I re-verified the two OBS citations this iteration moved or leaned on (`OBS-BW53-AUTO-005` at `:40`, `OBS-AB12-ARR-010` at `ableton-live-observations.md:34`) and both are exact. The Logic Pro / Serum 2 gaps remain named rather than filled, and Serum 2's two records remain explicitly unused, which keeps D-R2's quarantine respected. | — |

**Lens average:** 2.857 (20/7)
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | Untouched. §6.4 3A produces R4's exit row and defers nine adjacent features each with an owning milestone. Nothing smuggled by this iteration — the only scope-shaped edit is a constant value. | — |
| 3B. Non-goal respect | 3 | Untouched, and improved by the citation fix: both recitations of the cloud non-goal now point at `docs/00-product/vision.md:57`, which reads "No cloud services, collaboration servers, or content stores" — verified verbatim. §6.2 keeps the clip format as Spectre's own JSON, not SMF, not `.als`. | — |
| 3C. Deliberately small first devices | 3 | Untouched. Adds no DSP device; `ClipPlayer` does not implement `AudioProcessor`. Decision 15 cited correctly. | — |
| 3D. Originality | 3 | Strengthened. All seven bounds are Spectre-derived with printed derivations, and I recomputed every one this iteration touched. CLIP-006 is now derived from Spectre's own primitive rather than from a stated preference, which is the strongest possible form of the decision-16 argument. CLIP-004's over-broad sufficiency claim is corrected and the monophonic/polyphonic distinction is now stated in both §4.2 and §7.2. CLIP-005's arithmetic table is correct at every entry (`frames × BPM ÷ 3,000`: 256→102.4/10.24, 1,024→409.6/40.96, 2,048→819.2/81.92). | — |
| 3E. Platform commitment | 3 | Untouched, and CLIP-005's rewrite improves it: the bound is now stated against `plan.max_frames()` with the driver-negotiated block size named as R4-1/decision 20's property, and the consequence for the implementer written out. No Linux claim is authorized; decision 23's debt is named as inherited. | — |
| 3F. Accessibility trajectory | 3 | Untouched. §3.7's label formats, named custom actions, focus order, and color-independent active state remain, and the spec does not claim decision 17's audit passes. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes
**Auto-fail triggered:** No

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | Both iteration-1 §7.1 defects are fixed and I confirmed the fixes at source. `order_rank` is now transcribed "transport 0 > note-off 1 > **note-on** 2 > control 3 > param 4," which matches `event.rs:24-28` exactly (`TransportSeek` 0, `NoteOff` 1, `NoteOn` 2, `Control` 3, `ParamChange` 4). The three `MidiIngress` pointers now read `:90`/`:91`/`:92` and `midi.rs:90` is `next_id: 1`, `:91` is `sequence: 0`, `:92` is `late_messages: 0`. §4.3's two sequence-counter anchors moved to `:91` correctly. I then re-opened roughly 60 further §7.1 citations across 13 files and every one landed: `io.rs` 24/33/49/52/77/107/122/146/148/158/163-172/194-196; `midi.rs` 14/65/74/87/88/102/108/111/115/135/153/162/173/188/199-203; `bridge.rs` 20/24-39/63-115 (eleven `Atomic` fields, eleven accessors — counted)/119-129/162-208/257-271/274-289; `control.rs` 17-19/192/248-258/310-324; `spsc.rs` 57/80/103/118; `time.rs` 9/12/22; `tempo.rs` 10-11/72-74/87/99; `transport.rs` 26/35/82/96/163-172; `spectre-graph/src/lib.rs` 15/344/456/465/487; `source.rs` 24/111/165/186/188-193/194/198; `spectre-project/src/lib.rs` 13/16/19/105-124/157; `command.rs` 37/72/114/134; `spectre-app/src/lib.rs` 37/205/338/405; `spectre-offline/src/lib.rs` 178/271-283/288; `midi_ingress.rs:287`. The absence claims hold in both directions: `crates/spectre-project/src/` contains exactly `command.rs` and `lib.rs`; `render` never calls `Transport::advance`; `io.rs:24`/`:32` derive only `Debug, Clone, Copy, PartialEq`. | — |
| 4B. Status vocabulary | 3 | Untouched and correct. The spec is `proposed`; R4-1/R4-2/R4-4 are "specified and not implemented"; §7.1 splits Implemented / Absent / Planned / Gated. The header's decision **not** to advance `Last verified` — with the reason given, that this pass re-verified the citations it touched and not all of §7.1 — is the correct application of the `implemented`/`verified` distinction to the spec's own metadata. | — |
| 4C. Traceability | 3 | Both iteration-1 defects are fixed and verified. `current-milestone.md:77` no longer appears; both cloud-non-goal recitations point at `vision.md:57`, verified verbatim. CLIP-004's claim is now scoped to monophonic density with the polyphonic case routed to E-8's counted refusal. PROD-003 is at `requirements-ledger.md:64` ("Every numeric limit in Spectre MUST have its own rationale recorded in this ledger; copied vendor limits are prohibited"), verified, and `requirements-ledger.md` remains in §7.2's modified-files list carrying all seven rows including CLIP-006 at its new value. | — |
| 4D. Honest gaps | 3 | Eleven open questions retained, and Q2 is materially improved: it now records that `spsc::bounded` cannot build a lane shallower than three, names that as **evidence for** the single-slot alternative rather than as a decision, and still routes the decision-21 shape change to Jeff. That is the correct handling of a fact that cuts against the spec's own proposal. | — |
| 4E. Evidence commands | 3 | Untouched. The workspace gate is quoted exactly as `docs/status/STATUS.md:46-48` has it, and the four targeted runs name files §7.2 creates. | — |
| 4F. No fake surfaces | 3 | Untouched. §1.2 states `./spectre` produces no sound and cites `STATUS.md:37`; §7.4 says a clip can be modeled and unit-tested but not heard without R4-1; §4.7 still refuses to assert a headroom threshold. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes

---

## Auto-Fail Roll-Call

| Rule | Result | Basis |
|---|---|---|
| **AF-1** — Contradicting accepted authority | **Pass** | The single shape change to an accepted row — decision 21's three lanes becoming four (`decision-gates.md:45`) — is still flagged in §4.3 and §4.4, proposed rather than asserted, given an alternative, and routed to §8 Q2 as Jeff's call. Q2 now adds a fact that argues *against* the spec's own proposal, which is the opposite of assertion. TIME-002's ordering is reinforced, not overridden. |
| **AF-2** — Unbacked implementation claims | **Pass** | I opened every source path §7.1 cites and checked the claim at the cited line, in both directions. Nothing is described as existing that does not; the four absence claims are correct. The two iteration-1 pointer defects are repaired. |
| **AF-3** — Realtime discipline violation | **Pass** | No allocation, deallocation, lock, I/O, logging, or panic on a callback-reachable path. The new `schedules_held` counter closes the one place a reclaim-lane refusal could have led to a render-thread drop. Every new bound has a counted overflow policy; reclamation is off-thread. The §4.7 memory finding is an understated ceiling, not a discipline violation — the lane is still bounded and still counts. |
| **AF-4** — Borrowed numeric limits | **Pass, and strengthened** | Seven bounds, seven Spectre-derived rationale rows, `requirements-ledger.md` in the modified-files list with "prose beside a constant does not discharge it." CLIP-006's value is now derived from Spectre's own primitive rather than from a design preference — the least borrowable form a constant can take. No vendor limit appears anywhere. |
| **AF-5** — Conclusions the evidence does not support | **Pass** | No default shortcut map, no gesture count, no time budget, no monitoring-latency threshold, no promoted archetype, no platform/backend order. §4.7 still refuses to extrapolate from the single macOS headroom measurement. Nothing this iteration added introduces a prohibited conclusion — the edits are a constant, a test, five citations, and two rationale rewrites. |
| **AF-6** — Optimistic language | **Pass** | The header's iteration-2 note describes the change without inflating it, states outright which claims were **not** re-verified, and declines to advance `Last verified`. No promotional description of state or difficulty found. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every type the spec builds on was opened at its cited location this iteration: `NoteEvent`/`NoteEventKind`, `MAX_NOTE_EVENTS_PER_BLOCK`, `ProcessContext::new`, public `rank`, `MidiIngress`, `RenderBridge`, `ControlSender`/`ControlReceiver`/`RetiredState`, `Transport`/`LoopRegion`, `TempoMap`, `BeatTicks`, `IdGen`, `EditHistory`, `ProjectDoc`, `AppModel`. |
| API/interface changes are feasible with current architecture | ✓ | `with_clip_player` is additive so existing `RenderBridge::new` callers compile untouched. `collect_notes`'s clear→append change is the single line at `bridge.rs:258`. The step-table insertions all fit between existing statements at the line positions given. The fourth lane is one more `bounded` pair alongside the three at `control.rs:214-217`. |
| Views/screens fit current navigation pattern | ✓ | Two panels inside the existing `Arrange` lens; selection stays `ObjectId`-keyed. Unchanged this iteration. |
| Dependencies are available and version-compatible | ✓ | No crate added; no new inter-crate edge from this spec. Unchanged this iteration. |
| Platform/renderer requirements are realistic | ✓ | No platform-conditional code; no Linux claim; CLIP-005's rewrite now names the driver-negotiated block size as the live assumption instead of hiding a fixed 256 inside a derivation. |
| Test strategy is executable with current infrastructure | ✓ | **The iteration-1 blocker is cleared.** I-10 is executable and falsifiable as rewritten, and both patterns it names exist verbatim: `control_channel.rs:160-175` (loop-until-`Err`, `assert!(accepted >= 4)`, `note_overflows() == 1`) and `spsc_queue.rs:34-48` (`producer.capacity()`, `Err(999)` returned, `:43-46`'s drain-one-make-room-for-one). All 41 tests run without a device. |
| Performance budget is realistic for target hardware | ✗ **one defect** | Note-memory arithmetic re-checked and correct: `ScheduledNote` 32 B (16+4+4+2+6); 4,096 × 32 B = 128 KiB; ×16 = 2 MiB; ×2 = 4 MiB; ×5 = 10 MiB; scratch growth 768 × 32 B = 24 KiB. CLIP-003 (1,658,880,000), CLIP-004 (2.5 / 102.4 / 204), CLIP-005's whole table, CLIP-006 (187.5), CLIP-007 (24.85 days), and the 2^63 sequence-band horizon all recompute correctly, as does the merged ceiling 512 + 512 = 1,024 against `io.rs:88`'s `>` comparison. **But the "five copies per bus" premise is wrong** — see 1B. The reclaim lane is `bounded(32)` → 64 slots → usable capacity 63, drained only when the app thread calls `reclaim()`, and I-9 requires four retired schedules outstanding at once. Both the 4 MiB and 10 MiB figures rest on an unstated and untested invariant. |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares all five, R4-1 and R4-4 as hard blockers. §1.3's primary success signal is stated as unreachable until R4-1. |

**Feasibility verdict:** Feasible with caveats
**Caveats:** (1) §4.7's memory ceiling is understated because the reclaim lane's real depth is 63, not 1 — see Priority 1.1. (2) CLIP-005's `MAX_BLOCK_SEGMENTS = 2` remains correct only up to a 1,024-frame granted block; the spec now says so explicitly and tells the implementer what to do above it, which converts an iteration-1 caveat into a declared precondition.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.857 | 35% | 1.000 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 3.000 | 20% | 0.600 |
| **Composite** | | | **2.914** |

**Pass conditions (from criteria.md, binding):**
- [x] Composite ≥ **2.30** — 2.914 (up from 2.798 at iteration 1)
- [x] Every lens average ≥ **2.00** — 2.857 / 2.857 / 3.000 / 3.000
- [x] No criterion scores **0** — none
- [x] At most **two** criteria score 1 — **zero** criteria scored 1
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear
- [x] Feasibility rule satisfied — I opened every source path the spec cites and checked the claim at the cited lines, in both directions. §7.1 is now accurate at every citation I checked, including the two the iteration-1 review found wrong. The remaining defect is a derivation error in §4.7, not a §7.1 misdescription.
- [ ] Reviewer personally executed every command claimed as passing — **N/A.** No command in §5 is executable: the files §7.2 creates do not exist. I verified instead that the gate commands match `STATUS.md:46-48`, that every harness the tests extend exists (`rt_guard.rs`, `bridge_plan.rs`, `app_model.rs`, `spsc_queue.rs`, `control_channel.rs`, `midi_ingress.rs`), and that the two test patterns I-10 now imitates are present verbatim at their cited lines.

**All conditions met:** Yes → **PASS**

---

## Statement on the iteration-1 findings

Required by this run's documented pattern of reviewers being wrong. Taking each iteration-1
Priority-1 and Priority-2 item in turn:

1. **The lane-depth defect was real, and the prescribed fix was wrong.** The scorecard's diagnosis
   (`bounded(2)` yields usable capacity 3) is correct. Its suggested remedy —
   "pass `SCHEDULE_LANE_CAPACITY - 1` to `bounded`" — would not have worked, because
   `capacity.max(MIN_CAPACITY)` clamps 1 up to 2 and the ring is identical. The spec says so and
   is right. **This is a fifth remediation overturning its own scorecard, and it is correct.**
2. **§7.1's `order_rank` variant name.** Iteration-1 finding correct; fixed; verified at
   `event.rs:26`.
3. **The three `midi.rs` off-by-one pointers.** Iteration-1 finding correct; fixed; verified at
   `midi.rs:90-92`.
4. **CLIP-004's over-broad sufficiency claim.** Iteration-1 finding correct; fixed in both §4.2
   and §7.2.
5. **CLIP-005 pinned to an assumed buffer size.** Iteration-1 finding correct; addressed by
   stating the assumption and its consequence rather than by changing the constant, which is the
   right call for a bound the driver negotiates.
6. **`current-milestone.md:77`.** Iteration-1 finding correct; both recitations moved to
   `vision.md:57`; verified.
8. **`OBS-BW53-AUTO-005` line number.** Iteration-1 finding correct; moved to `:40`; verified.

Items 7, 9, 10, and 11 were not taken up. Item 9 is still live and still minor — §3.6 E-10 writes
`spectre_project::validate` for what is a module-private `fn validate` at
`crates/spectre-project/src/lib.rs:105`. Item 11 is also still live and also harmless: the offline
FNV-1a walk is `:271-278`, with `:279-284` constructing the `RenderReport`; `:271-283` slightly
over-extends but the anchor is exact. Item 7 is carried forward below as Priority 2.

---

## Remediation Brief

This spec passes. The items below are corrections to make before implementation, not conditions of
the verdict.

### Priority 1 — Fix before an implementer touches this

1. **The reclaim lane's depth does not match §4.7's memory ceiling — the same defect class as
   iteration 1, one lane over.** §4.7 states "a bus can hold one installed schedule, three queued,
   and one retired-but-unreclaimed copy, so the absolute ceiling is five copies per bus — **10
   MiB**," and §7.2's CLIP-001 row states "×2 for one retired schedule per bus in flight on the
   reclaim lane = **4 MiB**." Neither holds. The reclaim lane is built as
   `bounded::<RetiredState>(DEFAULT_RECLAIM_CAPACITY)` with `DEFAULT_RECLAIM_CAPACITY = 32`
   (`crates/spectre-audio/src/control.rs:19`, `:217`), which computes `32.max(2) + 1 = 33`,
   `next_power_of_two = 64`, `mask = 63` — **usable capacity 63**, sixty-three outstanding
   `Box<dyn Send>` values, not one. It is drained only when the app thread calls
   `ControlSender::reclaim` (`:274-281`), and this spec fixes no cadence for that call. §5.2's own
   **I-9** stipulates "install 5 schedules across 5 blocks … call `ControlSender::reclaim`,"
   which requires four retired schedules outstanding simultaneously — a direct contradiction of
   §4.7's premise. Two changes: (a) state the invariant that makes 5 copies true (the app thread
   reclaims at least once per publish cycle) as a design requirement of §4.4 and assert it in a
   test, then present 4 MiB and 10 MiB as conditional on it; and (b) state the unconditional bound
   separately, derived from the reclaim lane's real capacity the way CLIP-006 is now derived from
   the schedule lane's. Do not repeat iteration 1's shape of error — read `bounded`'s output, do
   not assume the argument.

### Priority 2 — Should fix for quality

2. **Give 2C the forward-compatibility sentence iteration 1 asked for.** §3.3's note list is the
   first per-note field surface in the product. One line stating that any modulatable clip field
   inherits PROD-002's base/automation/modulation split at R9 (`requirements-ledger.md:63`) keeps
   the R4 list from becoming the precedent that forecloses it. Carried forward unaddressed.

3. **`schedules_held` has no test.** §4.3 adds the counter for retired schedules the reclaim lane
   refused — the path on which the render thread must keep holding the box rather than drop it,
   which is an RT-001 obligation (`control.rs:311-313`). §5.2 covers `schedule_overflows` (I-10)
   and app-thread reclamation (I-9) but nothing drives the reclaim lane to refusal. Add the
   symmetric test: fill the reclaim lane, install once more, assert `schedules_held() == 1` and
   that no drop occurred on the render thread. This is also the test that would pin Priority 1.1.

### Priority 3 — Consider for excellence

4. **§3.6 E-10 writes `spectre_project::validate` as though it were a public path.** It is
   module-private (`fn validate`, `crates/spectre-project/src/lib.rs:105`). §7.2's modified-files
   row cites it correctly at `:105-124`; only the E-10 prose implies an API. Carried forward from
   iteration 1 item 9, unaddressed.

5. **The offline FNV-1a citation `:271-283` over-extends by one line.** The walk is `:271-278`;
   `:279-284` constructs the `RenderReport`. Cited twice (§1.3, I-5). Carried forward from
   iteration 1 item 11, unaddressed.

6. **`CLIP_SEQUENCE_BAND` is the one constant in §7.2's new-file list with no ledger row.** The
   other seven have CLIP-001…007. Its derivation is present and correct in §4.3 (the 2^63 /
   292,000-year horizon), so this is a bookkeeping gap against PROD-003's "every numeric limit,"
   not a missing argument. Either add the row or state in Q3 why a band mask is not a limit.

7. **§3.1's bars-beats transport-header readout is still unargued as in-scope.** Carried forward
   from iteration 1 item 10. Defensible as necessary to place clips musically; say so, or move it
   to R4-4.

---

**Reviewer's scope statement.** This review is independent of the iteration-1 scorecard: I read
that scorecard to know what was claimed, then re-derived every finding from source. **Opened in
full:** the spec (1,599 lines), `gauntlet-output/criteria.md`, `templates/SCORECARD-TEMPLATE.md`,
`crates/spectre-audio/src/spsc.rs` (126 lines, read line by line and the `bounded` arithmetic
recomputed by hand for arguments 0–4 and 32). **Opened at every cited range with surrounding
context:** `crates/spectre-audio/src/control.rs` (lanes, `retire`, `reclaim`, `RetiredState`,
`control_channel`), `crates/spectre-audio/src/bridge.rs` (`:20`, `:24-39`, `:60-80`, `:115-132`,
`:160-210`, `:245-270`, `:274-289`), `crates/spectre-audio/src/midi.rs`,
`crates/spectre-dsp/src/io.rs`, `crates/spectre-core/src/event.rs`,
`crates/spectre-core/src/{time,tempo,transport}.rs`, `crates/spectre-graph/src/lib.rs` at its five
cited anchors, `crates/spectre-dsp/src/source.rs`, `crates/spectre-project/src/{lib,command}.rs`,
`crates/spectre-app/src/lib.rs` at `:37`/`:205`/`:338`/`:405`,
`crates/spectre-offline/src/lib.rs:178-290`, `crates/spectre-audio/tests/spsc_queue.rs:25-55`,
`crates/spectre-audio/tests/control_channel.rs:150-185`,
`crates/spectre-audio/tests/midi_ingress.rs:287`, `docs/00-product/vision.md:55-59`,
`docs/01-requirements/requirements-ledger.md:62-66`,
`docs/02-reference-research/bitwig-studio-observations.md:38-42`. I re-checked roughly 60 §7.1
citations by printing the exact cited line and comparing. **Sampled rather than fully read:** the
Ableton, Bitwig, and synth/modular observation files beyond the lines cited this iteration — I
re-verified only the two OBS records the remediation touched or leaned on, and took iteration 1's
verification of the other eighteen and of the corpus counts as standing, since the remediation did
not touch them. `docs/06-plans/current-milestone.md`, `decision-gates.md`, `STATUS.md`, and
`dsp-device-io.md` were spot-checked at the lines this iteration moved, not re-read in full.
**Not done:** I did not compile anything or run the workspace gate — no test named in §5 exists.
I could not diff iteration 1 against iteration 2, because the spec is untracked in git
(`git status` reports `?? gauntlet-output/specs/R4-5-midi-clips.md`), so the header's claim that
nothing else changed is **not** independently verified; I checked it only by re-verifying the
sections the header names plus §7.1 in full. I graded AF-5 against `criteria.md`'s enumeration
rather than opening the workflow field study directly.

---

**End of scorecard.**
