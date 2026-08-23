<!--
Author: Jeff
Date: 2026-08-23
Description: Blind verification scorecard for the R4-4 track-model spec, iteration 2 (remediation 1)
Notes: Independent re-verification against source, not against the iteration-1 scorecard. Read in
  full — criteria.md, the 1,817-line spec, and every source range the spec cites in §1 through
  Appendix A. All five grep transcripts re-executed. The protected §7.1 passage byte-compared
  against HEAD. Two iteration-1 pointers the remediation refused were checked and the remediation
  is right on both.
-->

# Scorecard: Track Model

**Feature ID:** `R4-4` (`track-model`)
**Spec file:** `gauntlet-output/specs/R4-4-track-model.md`
**Reviewer agent:** blind verification agent, R4-4 iteration 2
**Date:** 2026-08-23
**Spec iteration reviewed:** 2 (remediation 1)

---

## Verdict: PASS

**Summary:** Every iteration-1 finding is closed and each closure verifies against source rather
than against the scorecard that prescribed it — the `grep -rn track crates --include='*.rs'`
transcript now reproduces byte-exactly as written (five files, `spectre-project/src/lib.rs:162`
and `:170`, both `future_track_kind`), all `manifest.md` line citations are gone and what replaced
them is checkable (`bridge.rs:181-186` still drains into a discarding closure; `ProjectDoc` at
`lib.rs:29-38` carries `id`, `name`, `tempo_map`, `transport`, `unknown` and nothing else), and
the protected §7.1 refusal is byte-identical to HEAD and still accurate at all four cited
locations. The remediation additionally found and repaired a defect no reviewer caught: §7.1's
R4-1 bullet had *quoted* `manifest.md:19` with text that reproduces at no commit — I confirmed
against `git show 2e005e5:gauntlet-output/manifest.md` that line 19 read something else entirely
and that line 44 recorded R4-1 as **FAIL (AF-2) at 2.750**, not a pass. The most important
remaining finding is not a defect but a scope fact the spec states honestly and cannot fix from
inside the slice: mute, solo, and level are still inaudible until R4-2, so the audition half of
criterion 2A is deferred by construction.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | §4.1 names exactly one new callback-reachable item, `SumBus::process`, and places model, commands, builder, compilation, and parameter publication on the app thread, correctly noting `RenderBridge::render` calls only `CompiledPlan::process` and private helpers — verified `bridge.rs:162–208` (`render` opens at `:162`, closes at `:208`). It names the covering RT-001 guard: the counting global allocator at `plan_alloc.rs:22–37` asserting on `plan_process_is_allocation_free` (`:45`), and extends that fixture rather than inventing a guard (§5.1 test 14). The `:22–39` → `:22–37` correction is right — `:38` is blank and `:39` opens the next comment. It then **limits** the claim: the `rt_guard.rs` structural lock scan "reads `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, and `src/null.rs` … it does not scan `spectre-dsp` or `spectre-graph` at all," verified verbatim at `rt_guard.rs:294–297`, and refuses to draw a `SumBus` conclusion from it. Internal cross-reference now points at §5.1 test 14, which is the allocation test. | — |
| 1B. Control↔render communication | 3 | The iteration-1 gap is closed and closed correctly. §3.2, §4.3, and §4.4 now state normatively that **a level or mute edit publishes one target and a solo edit publishes one per track**, with the reason given from the formula (`effective_gain` reads `any_soloed()`), the return contract made checkable (`set_track_soloed` returns one `ObjectId` per track in list order, in **both** directions), and §5.1 test 16 asserting the count. The no-overflow argument is a property of the accepted transport rather than an assumption: `control_channel` preallocates one `ParameterSlot` per target at construction — verified `control.rs:205–213` — so N publications are N atomic stores into N distinct slots. The spec then discloses rather than glosses the one real cost: the N stores are not atomic as a set, so a drain landing mid-publication applies some tracks this block and the rest next, verified against `ParameterReader::drain` at `control.rs:125–139` and the version-before-bits ordering comment at `:128`. Lane, overflow behavior, and off-thread reclamation are all named (`RetiredState` verified `control.rs:192`, `retire` at `:314–324`). | — |
| 1C. Numerical containment | 3 | §4.3 gives `SumBus` boundary containment matching shipped precedent — non-finite input treated as `0.0`, the posture verified in `Gain` (`effect.rs:65–69`) and `Saturator` (`effect.rs:122–126`) — and names the second guard accurately: `CompiledPlan::process` runs `contain_channel` on every node's output before a downstream node can read it (verified `spectre-graph/src/lib.rs:512–522`), counting into `ContainmentStats` (`:389–397`), denormals flushing to signed zero (`:401–414`). Injection tests named specifically: test 10 (NaN and infinity buses on the device) and test 12 (a poisoned track silenced *before* the sum). Test 12's rewrite is materially better than the fix the iteration-1 scorecard prescribed: rather than adding a `ToneSource` import, it uses `containment.rs`'s own `Poison::Clean` variant, which writes `0.5` — verified at `containment.rs:38`, with `PoisonSource` at `:44` and the import list at `:9–12` confirmed to lack `ToneSource`. | — |
| 1D. Determinism | 3 | §1.3's success signal uses the **existing** FNV-1a walk with both constants quoted verbatim and verified at `spectre-offline/src/lib.rs:271` (offset basis `0xcbf2_9ce4_8422_2325`) and `:276` (prime `0x0000_0100_0000_01b3`), with an explicit refusal to add a second comparison method, restated in §4.3 and §5.2. §5.2 assertion 1 carries the anti-vacuity guard (`peak > 0.0` on both renders). Test 7 pins node-ID determinism against a fixed `IdGen` seed against the fixed allocation order §4.3 declares. Test 8 asserts bit equality only where the arithmetic guarantees it and permits one f32 ULP for the genuinely reordered sum — a test that can still fail, at 2 ULP. Bus order equals track order is grounded, not asserted: `compile` flattens input buses in bus order, verified `spectre-graph/src/lib.rs:298–308`. | — |
| 1E. Graph and plan contract | 3 | Both failure modes are named and refused. §4.4 consequence 1: "The compiled plan is never mutated," citing `CompiledPlan` "exposes no node or edge mutation API by design" — verified verbatim at `spectre-graph/src/lib.rs:416–417`, with the struct at `:416–427`; structural change produces a *new* plan on the app thread. §4.4 consequence 2: "Recompilation is never proposed per parameter change," citing decision 22, verified verbatim at `decision-gates.md` row 22: "(b) is rejected on its face: plan compilation allocates and cannot run per knob turn." The `structure_revision` split is the mechanism and §5.1 test 5 asserts it in both directions. The `MAX_FLAT_INPUTS` change is a pure widening: verified that `add_node`'s other two checks at `:138–143` and every rule at `:171–180`, `:227–240`, `:242–273`, `:275–281` are untouched, and that `grep -rn "MAX_FLAT\|InvalidLayout" crates/*/tests/` still returns nothing. The `:16` → `:15` correction is right — `MAX_FLAT_INPUTS` is at `:15`, `FLAT_OUTPUTS` at `:16`, `CHANNELS_PER_BUS` at `:12`, and Appendix A cites the latter two correctly. | — |
| 1F. Failure behavior | 3 | §3.6's eight rows are fail-closed throughout, every one "Data loss: no", with atomic non-mutation on E3/E4 following the `open_device_in_shape` precedent (verified `lib.rs:338–345`). Both binding rules hold: no error string is formatted on the audio thread, and **no threshold-derived alarm is specified** because Spectre owns one headroom measurement (verified `current-milestone.md:113` — macOS, 0.990, and `:114` reads "not run" for Linux). E8 classifies all-muted as exact silence rather than error. E6 states a `TooManyTracks` build failure "is a defect, and the message says so rather than implying user error." E7's `GraphError` `Display` is real and actionable — verified `spectre-graph/src/lib.rs:64–108`. | — |
| 1G. Test specification | 3 | Sixteen unit tests plus three integration assertions, each with setup, assertion, and a named edge case, each able to fail on a stated regression. Every proposed test compiles against a declared API: the five `AppModel` methods test 16 drives (`remove_track`, `reorder_track`, `set_track_level/muted/soloed`, `track_structure_revision`) are declared in §7.2's `spectre-app/src/lib.rs` entry, and `step_count` is real (verified `spectre-graph/src/lib.rs:445–447` — the spec's range is tighter and more correct than the iteration-1 scorecard's `:446`). Test 13's boundary arithmetic is right: `(MAX_SUM_BUSES + 1) * 2 = 34 > 32` refuses, `MAX_SUM_BUSES * 2 = 32` is accepted at the bound. Test 9's zero-bus path is feasible — `0.is_multiple_of(2)` passes `add_node`, the `MissingInput` loop at `:227–240` iterates an empty range, and `validate_buffers` accepts a zero-input effect (verified `io.rs:181–196`). Both iteration-1 nits are closed: §7.2 now states the `Eq` drop, and test 12 adds exactly one import. | See Priority 3 items 1 and 2. |

**Lens average:** 3.000 (21/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s
**Auto-fail triggered:** No — AF-3 does not trigger. The only new callback-path code is
`SumBus::process`, an accumulate-and-store loop over borrowed planar buffers with no allocation,
lock, I/O, logging, or panic. Overflow policy is now stated as a structural property of the
preallocated slot array rather than assumed, and off-thread reclamation is named.

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 2 | Half the iteration-1 deduction is closed and closed well. §3.2 step 4 now states **normatively** that transport position and transport state survive the engine rebuild, and grounds it rather than asserting it: `TransportCommand::Stop` sets `state` only (verified `spectre-core/src/transport.rs:86`), `position` moves only on `Seek` (`:87`) or `advance` (`:96`), and `Transport` lives on `AppModel` (`lib.rs:206`). It goes further than asked, naming a playhead reset a defect rather than a permitted variation. The remaining deduction is real and unfixable inside the slice: mute/solo/level are **not audible** until R4-2 (§3.2), and adding a track requires an explicit rebuild that stops and restarts the stream with an audible gap (§3.2 step 4, §4.4 consequence 3). The criterion grades whether the feature supports audition; here it substantially defers it. The honesty is exemplary and correctly routed to §8 Q3, but 3 means "no meaningful gap" and this gap is meaningful. | Not remediable in R4-4; see §8 Q3. |
| 2B. Linked lenses | 3 | Sidebar, inspector CONTEXT block, and Mix lens all read one `TrackList` on `AppModel`; §4.6 deletes `TrackView` outright rather than shipping beside it, with the reason stated against this criterion. Stable identity is carried by `ObjectId` from the project `IdGen` and preserved across reorder (test 1) and across undo via the whole-`Track` inverse (§4.3), which is CORE-001's requirement (verified `requirements-ledger.md:46`). Selection survives every flow in §3.2. | — |
| 2C. Modulation visibility | 3 | The iteration-1 gap is closed with a full paragraph, not a token sentence. §3.3's "The fader and PROD-002" quotes PROD-002 exactly — verified verbatim at `requirements-ledger.md:63`, acceptance evidence "UI-model tests at R9" — states that R4-4 ships none of it and claims none, and then argues the non-foreclosure structurally: `Track::level()` is a single stored base value no other subsystem writes, and `effective_gain` composes mute and solo *outside* it, so a later automation or modulation layer is further composition rather than a redefinition or a persisted-value migration. It also states correctly that no restore action is specified because there is nothing yet to restore to. The parameter-honesty half remains strong: the hard-coded `0.0..=1.0` slider (verified `main.rs:181`) is replaced by `GAIN_PARAMETERS[0]` (verified `effect.rs:19–20`, `0.0..=2.0`, default `1.0`) per `dsp-device-io.md:60`, and the Mix bar is normalized through `to_normalized` (verified `parameter.rs:53–56`). | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 is the correct answer to the AF-5 hazard: "**No keyboard shortcut is assigned to any of these commands, and none is proposed**," citing `product-implications.md` §"Prohibited conclusions at current evidence level" lines 90–102 — verified exactly: the header is at line 90, "a default shortcut map" at 96, "promotion of any workflow archetype" at 102. It commits only to the allowed shape (named, context-scoped, remappable commands with stable identifiers) and explicitly disowns the prototype's pre-existing `Space`/`1`–`4` handlers (verified `main.rs:425–437`). Reorder is two ordinary buttons, not drag-only. The action cluster appears on the selected row only, citing `vision.md`'s "no spreadsheet density" (verified `vision.md:41`). | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A identifies a genuine **divergence** between two researched systems on summing — `OBS-PP-ARCH-002` ("each mixing onto the signal from above") versus `OBS-BW53-GRID-002` ("an in port accepts exactly one cable … unconnected in ports read zero"), both verified verbatim — concludes "there is no convergent pattern to follow," and grounds Spectre's choice in its own shipped rule (`InputBusOccupied`, verified `:171–180`). Convergences are followed and named (`OBS-AB12-ROUTE-003`, `-MIX-002`, `OBS-PP-ARCH-001`). The one benchmark divergence — additive vs Live's exclusive solo — is flagged as a divergence and escalated to §8 Q10 rather than settled. | — |
| 2F. Differentiation | 3 | The iteration-1 gap is closed in §1.2 under "What this makes Spectre do differently," and it is closed with the right claim rather than a slogan: **mixer state travels; structure does not**, tied to §3.6 E5, §6.3's state-word rule, and the alpha bar's "honest telemetry, no fake surfaces" (verified `vision.md:48`). Critically it is paired with Appendix A's matching gap — "no benchmark record in the corpus describes what a DAW does when a structural edit lands while audio is running" — so the differentiation claim is framed as a choice rather than as a superiority claim or a parity claim. No parity claim is made anywhere. | — |
| 2G. Benchmark evidence discipline | 3 | Every `OBS-` ID cited was opened and each record says what the spec claims: `OBS-AB12-ROUTE-003` (17.2.1), `-MIX-001` (18.1), `-MIX-002` (18.1.1), `-MIX-003` (18.1), `-MIX-004` (18.3), `-MIX-005` (18.4), `-MIX-009` (18.9), `-SES-005` (7.5), `-LAUNCH-007` (16.7), `OBS-PP-ARCH-001`, `-ARCH-002`, `-FX-001`, `OBS-VCV-VOLT-005`, `-VOLT-006`, `OBS-BW53-GRID-002`, `-LAUNCH-001`. The `MIX-003` section fix landed correctly — the record reads (18.1) and the spec now says §18.1; §18.3 is `MIX-004`'s, which the spec also has right. Both structural claims reproduce exactly: the Ableton corpus is **85** records spanning **exactly** ARR/AUTO/CLIP/LAUNCH/MIX/REC/ROUTE/SES/WARP, and `grep -in "maximum\|limit"` returns **exactly one** hit, `OBS-AB12-LAUNCH-007`. Counts verified independently: Phase Plant 11, VCV 6, Serum 2 **exactly two** (`OBS-SR2-CPU-001`, `OBS-SR2-KB-001`, neither used), Logic Pro **zero** (`logic-pro.md` `inventory-only`). The quarantined Serum extraction is cited by nothing; the spec's "~250-record" figure is closer to truth than the iteration-1 scorecard's 271 — the file holds **249** unique `OBS-SR2-` IDs. | — |

**Lens average:** 2.857 (20/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s
**Auto-fail triggered:** **No — AF-5 does not trigger.** All nine prohibited conclusions were
checked individually, and the four the brief names by name were checked hardest. **Default
shortcut map:** refused by name at §3.4 with a verified citation; the only `⌘` in the spec is in
the header's Supersedes line describing the discarded spec's defect. **Final native-device list:**
`SumBus` is proposed as a fifth *layout* amendment against `dsp-device-io.md:35–39` (verified to
contain exactly four v1 layouts) and escalated to §8 Q4, not asserted. **Gesture-count or time
budget:** §4.7 states "**No budget threshold is asserted**" with the one-measurement reason;
§3.6 states "**No threshold-derived alarm state is specified**"; no gesture is counted anywhere.
**Workflow archetype:** none promoted — no persona, archetype, or user-type claim appears in the
spec. Also absent: supported-interface model list, monitoring-latency threshold, platform/backend
order (§4.6 makes no Linux claim), command-frequency or feature-priority scores.

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | The slice is exactly R4 slice 4: `docs/status/NEXT.md:26` reads "Introduce the track model with a track-to-master signal path, keeping the graph compilation contract intact" — verified to the line, as are `current-milestone.md:12` ("track to master") and `:83` ("A MIDI clip plays through a track into master"). Every out-of-scope neighbour is deferred against a verified roadmap line: sends, returns, groups, monitoring, compensation, meters, track types → R6 (`rebuild-roadmap.md:31`, verbatim); recording and arming → R7 (`:32`); automation → R9 (`:34`); per-track launcher authority → R10 (`:35`). Mixer depth is refused concretely: §3.3 ships **no meters** because peak/RMS is `OBS-AB12-MIX-001` behavior and R6. Re-parenting the device browser is excluded (§3.1) and raised as §8 Q2. | — |
| 3B. Non-goal respect | 3 | §6.4 3B enumerates the non-goals and none is proposed: no CLAP/LV2/AU hosting, no plugin-format authoring, no cross-DAW preset or project compatibility, no cloud or content store, no video scoring. §4.5 adds no third-party crate and no asset. The `armed` flag is **removed** rather than left inert. | — |
| 3C. Deliberately small first devices | 3 | Exactly one device is added, `SumBus`, with zero parameters, no tone, no character, no UI beyond the master strip; §6.4 3C argues correctly that it is routing infrastructure. `PulseInstrument`, `Gain`, and `Saturator` are reused unchanged — verified against `spectre-dsp/src/lib.rs:11–19`, which exports exactly those four devices. The track fader is the shipped `Gain`, not a new one. Nothing grows toward the R11 flagship (decision 15, verified). | — |
| 3D. Originality | 3 | **AF-4 is fully discharged and iteration 2 introduced no new bound.** Both numeric bounds carry Spectre-derived rationale: `MAX_TRACKS = 16` is argued from the structural necessity of a compile-time bound (`PlanStep`'s `[usize; MAX_FLAT_INPUTS]`, verified `spectre-graph/src/lib.rs:337`, and `process`'s stack array, verified `:497`) plus Spectre's own cost arithmetic. §4.7's figures recompute: `(4n+4) × 512 × 4 B` gives 16,384 B at one track and 139,264 B at sixteen; `[&[f32]; 32]` is 512 B of stack; 34 × 288 B ≈ 9.8 KB; 8,192 f64 adds per block at 16 tracks / 256 frames; 2n+2 = 34 nodes at the ceiling. §7.2 **schedules the ledger rows** `MIX-001` and `MIX-002` into `docs/01-requirements/requirements-ledger.md` and states that §4.2's inline rationale does not discharge PROD-003 — verified at `requirements-ledger.md:64` that rationale must be recorded "in this ledger". Appendix A states explicitly that no benchmark track count exists to copy, which I confirmed. | — |
| 3E. Platform commitment | 3 | §4.6 is platform-neutral by construction and then states that R4-4 makes decision 23's undischarged Linux debt **worse**, because the plan grows from three nodes to up to 34 while the only headroom measurement is macOS on the three-node fixture (verified `current-milestone.md:113`; `:114` reads "not run"). It authorizes no Linux claim and adds a concrete requirement to R4-3's run in §8 Q9. | — |
| 3F. Accessibility trajectory | 3 | Nothing forecloses decision 17. §3.7 removes the only glyph-only state in the sidebar (the `●`/`○` armed indicator, verified `main.rs:131`), carries mute/solo/solo-silencing in words, uses only standard egui widgets, decomposes reorder into two single-activation buttons, and states focus order follows creation order. The real blocker is recorded rather than papered over: `crates/spectre-app/Cargo.toml` builds eframe `0.32.3` with `default-features = false` and only `default_fonts` and `glow` — verified — so no accessibility feature is enabled today and R4-4 claims none. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes
**Auto-fail triggered:** No — 3B is not 0, and **AF-4 does not trigger**: both bounds carry
Spectre-derived rationale *and* scheduled `requirements-ledger.md` rows in §7.2's modified-files
list. Checked specifically per the brief: the remediation introduced **no** new track-count,
channel-count, or send-count bound. Iteration 2's only new quantities are `N` publications per
solo edit (derived from `TrackList::len()`, not a limit) and "one block, on the order of a few
milliseconds" as the partial-apply window — a derived consequence of the block boundary, not an
asserted budget.

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | Both iteration-1 defects are closed, and a third the reviewer missed was found and closed by the author. **(i) The grep transcript now reproduces exactly.** I re-executed `grep -rn track crates --include='*.rs'`: 85 hits across **five** files — `spectre-app/src/lib.rs`, `src/main.rs`, `tests/app_model.rs`, `tests/smoke_cli.rs`, and `spectre-project/src/lib.rs` at **`:162` and `:170`**, both the string `future_track_kind` inside `unknown_fields_survive_rewrite` (the `#[test]` at `:157`, the fn at `:158`). §1.2 and §7.1 both name all five and both explain the two extra hits. The conclusion is still stated and still unhedged — "**No track concept exists outside `crates/spectre-app/`**" — and it is still true. **(ii) No `manifest.md` line citation remains as evidence.** §7.1's sibling-status claims now rest on source I verified: `bridge.rs:181–186` drains parameters into `\|_, _\| {}` and counts them, and `ProjectDoc` at `lib.rs:29–38` carries `id`, `name`, `tempo_map`, `transport`, and a flattened `unknown` map and nothing else. **(iii) The unflagged repair is the serious one and it checks out.** I ran `git show 2e005e5:gauntlet-output/manifest.md`: line 19 read "R4-1 remediation 1 was cut off by a usage cap…", not the sentence iteration 1 quoted, and line 44 recorded R4-1 as `spec-remediation-1` at **2.750 — FAIL (AF-2)**. The retraction at §7.1 states both facts correctly. The replacement claims verify: `crates/spectre-app/src` contains exactly `lib.rs` and `main.rs`, and the crate's `[dependencies]` are `eframe`, `spectre-core`, `spectre-dsp`, `spectre-project`, `serde`, `serde_json` — no `spectre-audio`. Roughly 45 further §7.1 pointers were opened individually and every one lands: `TrackView`'s six public fields (`lib.rs:37–45`), `add_track` with the hard-coded `level: 0.72` (`:405–422`), the default project's `"Pulse"` at `level: 0.78` (`:218–262`), the accessors (`:289–311`), the sidebar (`main.rs:115–160`), the toggles (`:177–179`), the slider (`:181`), the literals at `:184` and `:474`, the smoke line (`:487–496`), `smoke_cli.rs:18`, the two tests (`app_model.rs:33–41`, `:43–49`). Absence claims verify in the other direction too: `grep -rn "struct Track\b\|fn arm\|fn mute" crates` returns nothing; `.muted`/`.solo` are written at `main.rs:177–178` and read by nothing; `grep -rn "MAX_FLAT\|InvalidLayout" crates/*/tests/` returns nothing. | — |
| 4B. Status vocabulary | 3 | Vocabulary used correctly and the `implemented`/`verified` distinction respected explicitly: §7.2 states "Status moves to `implemented`, not `verified`, until §5.4's manual protocol passes," and §7.1 now leans on the definition itself — `docs/README.md:59`, verified verbatim as "code exists but its full evidence gate may remain open" — to explain why a passed sibling spec is not an implementation. The gated block reports decision 22 as design-accepted with implementation at R4-2, GRAPH-002 as `proposed` (verified `requirements-ledger.md:56`), CORE-001 as `implemented` with reorder evidence gated (`:46`), and decision 13 as SD-adopted with one command kind shipped (`command.rs:30–33`). The spec's own status is `proposed`. | — |
| 4C. Traceability | 3 | Every normative claim carries a requirement ID, decision row, `OBS-` ID, or source path, and the high-risk ones verify verbatim: decision 6's f64-accumulator wording, decision 22's "(b) is rejected on its face," decision 21's latest-wins lane, `dsp-device-io.md:27` (every output sample written), `:60` (UI must not redefine DSP ranges), `:35–39` (exactly four v1 layouts), `:81–96` (the runtime-parameter-seam section), `:94` and `:104` (the `Gain`-smoothing contract text), `requirements-ledger.md:30` (RT-003's `OBS-VCV-VOLT-006` provenance), `:62`/`:63`/`:64` (PROD-001/002/003), `vision.md:48`, `docs/README.md:34`. Every iteration-2 line-citation correction is right: `MAX_FLAT_INPUTS` `:15`, `PULSE_PARAMETERS` `source.rs:39–46`, `devices.rs:12–14`, `:186–209`, `:251–271`, `plan_alloc.rs:22–37`, `OBS-AB12-MIX-003` §18.1. | — |
| 4D. Honest gaps | 3 | Ten open questions, three escalated to `decisions-needed.md` because they change accepted material, escalated in the header, §7.2, and §8 — satisfying AF-1 on all three amendments. The strongest instance is still unprompted: §7.1 surfaces a **contradiction between an accepted contract and shipped code** — `dsp-device-io.md:94` says "`Gain` already smooths" and `:104` calls it "click-resistant smoothing"; `effect.rs:29–74` has a single `gain: f32` and a direct multiply with no ramp (verified) — declines to fix it inside the slice, and routes it to §8 Q6 with three named options. Iteration 2 adds a further honest disclosure rather than removing one: the one-block partial-apply window on a solo publication. | — |
| 4E. Evidence commands | 3 | The workspace gate is quoted exactly as `criteria.md` 4E requires. Every feature-scoped command names a real target — the three new test files are declared in §7.2, and `devices.rs`, `containment.rs`, `graph_plan.rs`, `plan_alloc.rs`, `app_model.rs`, `smoke_cli.rs`, `rt_guard.rs` all exist as named. §5.2 makes re-running the graph suite **after** the `MAX_FLAT_INPUTS` change a required gate rather than a formality. | — |
| 4F. No fake surfaces | 3 | Applied against itself. It removes the literal signal-path string at `main.rs:184` and the inert `armed` glyph rather than leaving either; it states in four places that `./spectre` produces no sound today (verified: `toggle_play` at `lib.rs:268–275` mutates an in-memory `Transport`, the transport bar prints the hard-coded `ENGINE OFFLINE` at `main.rs:79`, and the crate has no audio dependency); it states mute/solo/level will **not** be audible in R4-4; §6.3 makes the rebuild notice normative ("an app that hides it is claiming an edit is audible when it is not"); §5.4's Honesty-check row makes "toggle mute — the sound must **not** change" a manual gate. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s
**Auto-fail triggered:** **No.** AF-2 does not trigger: no code is described as existing, partial,
or implemented without a source path containing it, and no absent thing is described that in fact
exists. The iteration-1 finding that came closest — an evidence transcript that did not reproduce
— is now gone, verified by re-execution rather than by reading the amended sentence. AF-6 does not
trigger: no optimistic or promotional phrasing was found, §6.3 audits for it directly, and
iteration 2's additions are all disclosures of limitation rather than of capability.

---

## Auto-fail roll-call

| Rule | Triggered | Basis |
|---|---|---|
| AF-1 — Contradicting accepted authority | No | Three amendments to accepted material are proposed: the `dsp-device-io.md:35–39` layout list (§8 Q4), `MAX_FLAT_INPUTS` (§8 Q5), and `MAX_TRACKS` (§8 Q1). Each is flagged as an amendment, proposes supersession, and is routed to `decisions-needed.md` in the header, §7.2, and §8. Nothing is amended by assertion. |
| AF-2 — Unbacked implementation claims | No | Every existence and absence claim in §7.1 was re-checked individually against `spectre-app/src/{lib,main}.rs`, `spectre-app/tests/`, `spectre-graph/src/lib.rs`, `spectre-dsp/src/`, `spectre-project/src/`, and `spectre-audio/src/`. All existence claims are backed; all absence claims are genuine, including the two absences the paper trail previously got wrong in the *other* direction (`AppModel::add_track` and the sidebar, both correctly stated as existing). No `manifest.md` line citation remains as evidence; the fabricated `manifest.md:19` quotation is gone and its retraction is accurate against `git show 2e005e5`. |
| AF-3 — Realtime discipline violation | No | One new callback-path item, `SumBus::process`; allocation-free accumulate-and-store over borrowed buffers, covered by the existing counting-allocator guard. Latest-wins lane for parameters with the no-overflow property proved from `control.rs:205–213`, off-thread reclamation named, denormal flush and NaN/Inf isolation addressed at two layers. |
| AF-4 — Borrowed numeric limits | No | `MAX_TRACKS`/`MAX_SUM_BUSES`/`MAX_FLAT_INPUTS` all carry Spectre-derived rationale **and** scheduled `requirements-ledger.md` rows `MIX-001`/`MIX-002` in §7.2, verified against PROD-003 at `requirements-ledger.md:64`. Iteration 2 introduced no track-count, channel-count, or send-count bound of any kind. The corpus contains no track-count record to borrow from, which I confirmed independently. |
| AF-5 — Conclusions the evidence does not support | No | All nine checked individually; none asserted. The four the brief names: no default shortcut map (§3.4 refuses one by name, citation verified line-exact); no final native-device list (`SumBus` escalated as an amendment at §8 Q4); no gesture-count or time budget (§4.7 and §3.6 both refuse a threshold in as many words); no workflow archetype promoted (no persona or user-type claim appears anywhere in 1,817 lines). |
| AF-6 — Optimistic language | No | No unevidenced or promotional phrasing found. The spec repeatedly states what does **not** work, and iteration 2 added two more such statements (the partial-apply window, the rebuild's audible gap restated normatively). |

---

## Feasibility Check

Every source path the spec cites was opened and checked at the cited lines, in both directions.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | `Track`, `TrackList`, `TrackInstrument`, `TrackError`, `SumBus`, `TrackPathNodes`, `GainNode`, `InstrumentNode`, `RoutingError` are fully specified in §4.2 with signatures. Every existing type they build on was verified: `ObjectId`, `IdGen`, `DeviceIo`, `AudioProcessor` (`io.rs:163`, `Send` bound present), `NodeId`, `EditableGraph` (`spectre-graph/src/lib.rs:119–125`), `CompiledPlan` (`:416–427`), `Gain`, `PulseInstrument`, `Transport` (`spectre-core/src/transport.rs`). |
| API/interface changes are feasible with current architecture | ✓ | `SumBus` with `audio_inputs: buses * 2` passes `add_node`'s three checks at `:138–143` for `buses ≤ 16` once `MAX_FLAT_INPUTS = 32`; `buses == 0` also passes and compiles, since the `MissingInput` loop at `:227–240` iterates an empty range and `validate_buffers` (`io.rs:181–196`) accepts a zero-input effect. `compile` flattens buses in bus order (`:298–308`) and `process` slices `&inputs[..step.input_channels]` (`:507`), so bus index = track index holds as claimed. Dropping `Eq` from `CommandKind`/`ProjectCommand`/`Transaction` is safe: nothing in `command.rs` compares them — `Transaction::execute`'s rollback (`:92–109`) and `EditHistory` (`:113–182`) compare nothing. |
| Views/screens fit current navigation pattern | ✓ | No new screen. All four modified regions exist as cited: `track_list()` `115–160`, inspector CONTEXT `174–186` (§3.1's `172–186` is a one-line-looser range on the same block), `mix_surface()` `458–478`, `transport()` `50–84`. Sidebar 220 px default / 180 px min (`:118–119`) and the 1060×680 window minimum (`:508`) verified, as are the dark-only palette claims (`main.rs:37`, `:9–15`). |
| Dependencies are available and version-compatible | ✓ | No third-party crate added. Adding `spectre-dsp` and `spectre-graph` to `spectre-project` creates no cycle — verified from the manifests that `spectre-graph` depends only on `spectre-core` + `spectre-dsp`, and `spectre-dsp` on `spectre-core` alone. `spectre-app` and `spectre-offline` both already depend on `spectre-project`, so one builder can serve both. eframe stays pinned at `0.32.3`. |
| Platform/renderer requirements are realistic | ✓ | Portable Rust arithmetic over workspace types; no platform, driver, or OS API named. Only egui widgets already used in `main.rs` are required. The Linux consequence is named as debt, not discharged. |
| Test strategy is executable with current infrastructure | ✓ | All named existing targets exist and the cited fixtures are real: `plan_alloc.rs`'s counting allocator (`:22–37`) and its test (`:45`); `containment.rs`'s `PoisonSource` (`:44`), `Poison::sample` (`:31`), `Poison::Clean` writing `0.5` (`:38`), and its import list (`:9–12`); `devices.rs`'s `output()` helper (`:12–14`), `device_layouts_match_the_v1_contract` (`:186–209`), and `saturator_contains_non_finite_input_and_bounds_output` (`:251–271`); `graph_plan.rs:85–130`. Test 12's clean-`PoisonSource` construction adds exactly one import as claimed. |
| Performance budget is realistic for target hardware | ✓ | §4.7 asserts no threshold, the correct posture on one measurement. Every figure recomputes from verified source. One rounding nit: `PlanStep` at `MAX_FLAT_INPUTS = 4` is 57 bytes of fields (8 + 32 + 8 + 8 + 1) padding to 64, not the "~56 B" stated; the spec hedges with "~" and the delta and the 288 B figure are right. |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares all of them: R4-1 blocks only `build_track_engine_parts` (`engine.rs` confirmed absent — the crate is exactly `lib.rs` and `main.rs`); R4-2 blocks audibility (verified: `bridge.rs:181–186` discards); R4-7 blocks persistence and `EditHistory` routing (verified: `ProjectCommand::apply` takes `&mut ProjectDoc` at `command.rs:52`, and `ProjectDoc` at `lib.rs:29–38` has no track); R4-5 replaces the single-note-node arrangement (verified: one `note_node` at `bridge.rs:122–123`, one `PlanNoteInput` at `:188–191`); R4-9 owns the multi-track headroom measurement. |

**Feasibility verdict:** Feasible
**Caveats:** §8 Q8's ordering question (commands now vs. at R4-7) should still be answered before
implementation begins, since the answer decides whether `command.rs` is touched once or twice.
`TrackList::structure_revision`'s doc comment says the revision advances on "instrument change,"
but §4.2 declares no instrument setter on `Track`, so that clause is currently unreachable
through the declared API.

---

## Protected passage

**Intact — byte-identical.** §7.1's opening refusal (from "Verified by reading the files at commit
`2e005e5`" through "…at the precision the source supports.") was extracted from both the working
tree and `git show HEAD:gauntlet-output/specs/R4-4-track-model.md` and compared with `diff` and
`cmp`: **identical, eleven lines, no byte changed.** The correction that `criteria.md` and
`manifest.md` have since absorbed is recorded in a *separate appended paragraph* immediately after
it, not by editing it, and that paragraph deliberately cites both documents by dated entry rather
than by line number, with the reason stated.

**Still accurate.** All four load-bearing locations were re-verified independently:
`AppModel::add_track` exists at `crates/spectre-app/src/lib.rs:405–422`; the track list sidebar
exists at `crates/spectre-app/src/main.rs:115–160`; the two covering tests are at
`crates/spectre-app/tests/app_model.rs:33–41` and `:43–49`; and `grep -rn "struct Track\b\|fn
arm\|fn mute" crates` returns nothing, so `Track::arm` and `Track::mute` are genuinely absent and
there is no `Track` type. The refusal is right in both directions.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 3.000 | 35% | 1.050 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 3.000 | 20% | 0.600 |
| **Composite** | | | **2.964** |

**Pass conditions (values copied from `criteria.md`, which is binding):**
- [x] Composite ≥ **2.30** — **2.964** (iteration 1: 2.810)
- [x] Every lens average ≥ **2.00** — 3.000 / 2.857 / 3.000 / 3.000
- [x] No criterion scores **0** — none; the lowest score awarded is 2, on 2A alone
- [x] At most **two** criteria score 1 — **zero** criteria scored 1
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear; see roll-call
- [x] Feasibility rule satisfied — every cited source path was opened and checked at the cited
  lines, in both directions. §7.1's description of current state is accurate. No claimed-to-exist
  thing is absent and no claimed-absent thing exists.
- [x] Reviewer personally executed every command claimed as passing when required — the spec
  claims **no** command as currently passing. All five `grep` transcripts it presents as evidence
  were executed and **all five reproduce as written**, including the one that failed at iteration
  1. `git show 2e005e5:gauntlet-output/manifest.md` was run to test the spec's own retraction, and
  the retraction is correct. No `cargo` command is asserted as green, so none was required.

**All conditions met:** Yes → **PASS**

---

## Findings the remediation overturned, and my ruling on each

The brief asked me to say plainly where I believe a prior finding was wrong. Two, and the
remediation is right on both.

1. **`Transaction`'s `Eq` derive is at `command.rs:71`, not `:70`.** Verified: `:70` is the
   comment "// Ordered command group that applies and reverses atomically"; `:71` is
   `#[derive(Debug, Clone, PartialEq, Eq)]`; `:72` is `pub struct Transaction {`. The
   iteration-1 scorecard's Priority 3 item 1 cited `:70`. The spec declined to copy it and is
   correct. `CommandKind` at `:30` and `ProjectCommand` at `:36` were both right and are used
   as given.
2. **PROD-001's non-foreclosure sentence lives in Appendix A, not §4.4.** Verified by reading
   both sections: §4.4 contains no PROD-001 sentence; Appendix A's `OBS-AB12-SES-005` /
   `OBS-BW53-LAUNCH-001` bullet does, citing `requirements-ledger.md:62`. The iteration-1
   Priority 3 item 3 told the author to model the new PROD-002 sentence on "§4.4 as it already
   does for PROD-001." The spec put the PROD-002 material in §3.3 and pointed at Appendix A
   instead. Correct.

Two further places where the spec is more accurate than the iteration-1 scorecard, neither
flagged by the author: `CompiledPlan::step_count` spans `:445–447` (the scorecard said the
accessor is at `:446`, which is the body line), and the quarantined Serum extraction holds
**249** unique `OBS-SR2-` IDs (the scorecard said 271, which is the raw count of ID *mentions*).
The spec's "~250-record" is the better figure.

I found no case in which the remediation overturned a finding and was wrong.

---

## Remediation Brief

The spec passes at 2.964 and every iteration-1 item is closed. Nothing below is a pass blocker,
and there is **no Priority 1 item** — I found no factual error in the spec.

### Priority 1 — Factual corrections required before this spec is used as a source of truth

**None.** Every existence claim, absence claim, line citation, `OBS-` attribution, arithmetic
figure, and grep transcript I checked reproduces. This is the first iteration of this spec for
which that is true.

### Priority 2 — Should fix for quality

1. **§7.1 — the R4-1 score is a live cross-feature status claim with a dated hedge, and it will
   go stale the same way the manifest pins did.** "has passed blind review at 2.950 as of this
   remediation (2026-08-23)" is true right now — I confirmed it against the current
   `manifest.md` — and the dated qualifier is the right instinct. But the spec's own stated
   reason for dropping manifest citations is that sibling status is rewritten as the run
   advances, and a score is exactly that kind of fact. The paragraph's own argument is that the
   *implementation* half is the operative one, and it makes that argument well. Consider
   dropping the numeral and keeping only "has passed its spec gate; no implementation exists,"
   which is durable and is all the spec's reasoning uses.

2. **Header notes, line 26 — "No `manifest.md` line citation remains anywhere in the spec" is
   very slightly overstated.** `manifest.md:19` and `manifest.md:44` do still appear at spec
   line 1437, inside the parenthetical that *retracts* them. That is the right place for them
   and the retraction is accurate, so the substance is fine — but the sentence as written says
   "anywhere," and a later reader running the grep will get two hits and have to reason about
   why. One clause ("…except inside the iteration-2 retraction that names them") closes it.

### Priority 3 — Consider for excellence

1. **§5.1 tests 1, 6, 10, 11 use `?` inside tests that today return `()`.** Test 11 in
   particular extends the existing `device_layouts_match_the_v1_contract`, whose signature is
   `fn(...)` returning unit; `SumBus::new(buses)?` will not compile there without changing the
   signature to `-> Result<(), _>`. The repo's convention in `devices.rs` is `.unwrap()`. This
   is notation rather than a design flaw, but naming it removes an implementer's decision.

2. **§4.2 — `structure_revision`'s doc comment names an edit the API cannot make.** The comment
   says the revision advances on "push, insert, remove, reorder, and instrument change," but
   `Track` declares no `set_instrument`, and `TrackInstrument` has one variant. Either declare
   the setter (and let §5.1 test 5 cover it) or drop the clause until R4-6 adds a second variant.

3. **§4.3 — "on the order of a few milliseconds" is the one quantity in the spec with no
   arithmetic behind it.** It is correct (256 frames at 48 kHz is 5.3 ms) and it is not a
   threshold, so AF-5 and AF-4 are both clear. But every other figure in this spec shows its
   work, and this one would take half a clause: "one block — 5.3 ms at 256 frames and 48 kHz."

4. **§3.1 cites the inspector CONTEXT block as `main.rs:172–186`; §7.1 cites the same block as
   `:174–186`.** `:172` is `.show(ctx, |ui| {`, `:173` is the `CONTEXT` label, `:174` opens the
   `if let Some(track)` block. Both are defensible; they should not differ from each other
   within one document.

5. **`containment.rs:31` is called "its `Poison` helper."** `:31` is `fn sample(self) -> f32`;
   the `Poison` enum itself is at `:19–27`. The pointer is close enough to be useful and the
   behavioral claim it supports (`Clean` writes `0.5`) is cited separately and correctly to
   `:38`, so this is a naming nit only.

---

## Review Scope

Stated plainly so a later reader can judge what this verdict rests on.

**Read in full, start to end:**

- `gauntlet-output/criteria.md`, including the 2026-08-15 correction block
- `gauntlet-output/spec-scorecards/R4-4-track-model-scorecard.md` — the iteration-1 review, read
  as a claim to be tested rather than as authority
- `gauntlet-output/specs/R4-4-track-model.md` — all 1,817 lines, in sequence

**Opened and checked at the exact cited lines (not read end to end):**

- `crates/spectre-graph/src/lib.rs` — `:12`, `:15`, `:16`, `:64–108`, `:119–125`, `:136–145`,
  `:165–185`, `:194–203`, `:225–245`, `:271–282`, `:289–310`, `:320–330`, `:333–340`, `:385–418`,
  `:443–449`, `:495–500`, `:505–525`
- `crates/spectre-app/src/lib.rs` — `:34–48`, `:206`, `:216–232`, `:258–264`, `:266–278`,
  `:287–312`, `:313–315`, `:338–345`, `:405–422`, `:462–474`
- `crates/spectre-app/src/main.rs` — `:7–16`, `:36–38`, `:50`, `:79`, `:84`, `:115–160`,
  `:172–190`, `:369`, `:425–443`, `:458–478`, `:485–498`, `:508`
- `crates/spectre-audio/src/bridge.rs` — `:118–142`, `:158–165`, `:170–195`, `:205–212`
- `crates/spectre-audio/src/control.rs` — `:20–30`, `:122–142`, `:188–195`, `:195–220`, `:310–326`
- `crates/spectre-audio/tests/rt_guard.rs` — `:290–300` (the four-module scan list)
- `crates/spectre-dsp/src/effect.rs` — `:10–18`, `:19–25`, `:27–34`, `:58–72`, `:70–76`, `:115–128`
- `crates/spectre-dsp/src/source.rs` — `:12–26`, `:36–48`
- `crates/spectre-dsp/src/io.rs` — `:160–166`, `:175–198`
- `crates/spectre-dsp/src/parameter.rs` — `:50–60`, `:78–84`, `:124–128`
- `crates/spectre-dsp/src/lib.rs` — `:9–21`
- `crates/spectre-project/src/lib.rs` — `:10–40`, `:150–180`
- `crates/spectre-project/src/command.rs` — `:26–40`, `:50–74`, `:90–115`, `:178–186`, all derives
- `crates/spectre-core/src/transport.rs` — `:80–100`
- `crates/spectre-offline/src/lib.rs` — `:26–36`, `:200–210`, `:265–285`
- Test files: `spectre-graph/tests/{plan_alloc,containment,graph_plan}.rs`,
  `spectre-dsp/tests/devices.rs`, `spectre-app/tests/{app_model,smoke_cli}.rs` — every cited range
- `crates/spectre-app/Cargo.toml` in full; the other crate manifests for dependency direction
- Accepted documents at every cited line: `requirements-ledger.md` (`:28–32`, `:44–48`, `:54–57`,
  `:60–68`), `decision-gates.md` (rows 1, 4, 6, 8, 13, 15, 16, 17, 21, 22, 23),
  `current-milestone.md` (`:10–14`, `:81–85`, `:110–116`), `rebuild-roadmap.md` (`:28–36`),
  `NEXT.md` (`:24–28`), `dsp-device-io.md` (`:25–29`, `:33–41`, `:55–62`, `:79–106`),
  `vision.md` (`:39–48`), `docs/README.md` (`:32–36`, `:57–61`)
- Every `OBS-` record cited — sixteen IDs across `ableton-live-observations.md`,
  `synth-modular-observations.md`, `bitwig-studio-observations.md` — plus the corpus counts,
  category set, and `logic-pro.md`'s research state
- `gauntlet-output/decisions-needed.md` — D-R1 and D-R2 only

**Executed:**

- `grep -rn track crates --include='*.rs'` — reproduces exactly as §1.2 and §7.1 now state it:
  85 hits, five files, `spectre-project/src/lib.rs:162` and `:170`
- `grep -rn "struct Track\b\|fn arm\|fn mute" crates` — nothing
- `grep -rn "MAX_FLAT\|InvalidLayout" crates/*/tests/` — nothing
- `grep -rn "\.muted\|\.solo\b" crates` — exactly the two writes at `main.rs:177–178`
- `grep -in "maximum\|limit"` on `ableton-live-observations.md` — exactly one hit,
  `OBS-AB12-LAUNCH-007`
- `git show 2e005e5:gauntlet-output/manifest.md` — to test the spec's own retraction of the
  fabricated `manifest.md:19` quotation and the `:44` score
- `diff` and `cmp` of the protected §7.1 passage against `git show HEAD:` — byte-identical
- Corpus counts: 85 Ableton, 11 Phase Plant, 6 VCV, 2 Serum 2, 0 Logic Pro; 249 unique
  quarantined `OBS-SR2-` IDs

**Not executed:** no `cargo` command was run. The spec asserts no command as currently passing —
every test it names is one this slice creates or extends — so there was nothing to reproduce. A
reviewer of the *implementation* must run the §5 gate; this scorecard does not stand in for that.

**Not reviewed:** the spec author's reasoning, which I never saw; the R4-1, R4-2, and R4-7 specs,
beyond reading the manifest rows that record their scores; whether `MAX_TRACKS = 16` is the right
number, which §8 Q1 correctly escalates to Jeff; and whether solo should be additive or exclusive,
which §8 Q10 escalates for the same reason.

**Standing note carried forward:** the iteration-1 scorecard's closing note asked that §7.1's
refusal be preserved rather than edited. It was, byte for byte, and the surrounding framing was
updated in a separate paragraph exactly as that note intended. That instruction was followed
precisely.

---

**End of scorecard.**
