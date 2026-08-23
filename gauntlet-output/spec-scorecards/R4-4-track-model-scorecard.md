<!--
Author: Jeff
Date: 2026-08-15
Description: Blind verification scorecard for the R4-4 track-model spec, iteration 1
Notes: Read in full — criteria.md, the 1,610-line spec, crates/spectre-app/src/{lib,main}.rs,
  crates/spectre-graph/src/lib.rs (all 545 lines), crates/spectre-dsp/src/{effect,lib}.rs, all six
  crate manifests, every OBS- record cited, and every accepted-doc line cited. Sampled by targeted
  range and grep — spectre-offline/src/lib.rs, spectre-audio/src/{bridge,control}.rs,
  spectre-dsp/src/{source,io,parameter}.rs, spectre-project/src/{lib,command}.rs, and the six test
  files the spec modifies. Every existence and absence claim in §7.1 was checked individually.
  Two §7.1 inaccuracies were found and are recorded; neither is a false claim about code.
-->

# Scorecard: Track Model

**Feature ID:** `R4-4` (`track-model`)
**Spec file:** `gauntlet-output/specs/R4-4-track-model.md`
**Reviewer agent:** blind verification agent, R4-4
**Date:** 2026-08-15
**Spec iteration reviewed:** 1

---

## Verdict: PASS

**Summary:** The strongest quality is evidence integrity under the exact conditions that
broke the two predecessor artifacts on this subject: the spec independently refused the
false `AppModel::add_track` claim it was handed, and — at `§4.1` — it states the
`rt_guard.rs` structural scan list correctly (`src/bridge.rs`, `src/control.rs`,
`src/spsc.rs`, `src/null.rs`, verified at `crates/spectre-audio/tests/rt_guard.rs:294–297`)
**and then correctly refuses to draw a `SumBus` safety conclusion from it** — the precise
sentence R4-1 got wrong. Roughly 150 citations were opened; the load-bearing ones verify
to the exact line, including all fourteen `OBS-` records, all six accepted-doc line
citations, and every figure in `§4.7`'s cost arithmetic. The most critical gap is in
`§1.2` and `§7.1`, where a stated verification transcript — `grep -rn track crates
--include='*.rs'` "hits only" four `spectre-app` files — does not reproduce: it also
returns `crates/spectre-project/src/lib.rs:162,170`. The conclusion that grep supports
("no track concept outside `crates/spectre-app/`") is nonetheless **true** — both hits are
the fictitious JSON key `future_track_kind` inside a serde forward-compatibility test — so
this is an overstated evidence artifact, not a false claim about code, and it does not
trigger AF-2 under the standard applied to R4-1.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | §4.1 names exactly one new callback-reachable item, `SumBus::process`, and places everything else (model, commands, builder, compilation, parameter publication) on the app thread, correctly noting `RenderBridge::render` calls only `CompiledPlan::process` and private helpers (verified `bridge.rs:162–208`). It names the RT-001 guard that covers it — the counting global allocator at `plan_alloc.rs:22–39` asserting on `plan_process_is_allocation_free` (`:45`) — and extends that fixture rather than inventing a new guard (§5.1 test 14). Decisively, it then **limits** the claim: the `rt_guard.rs` lock scan "reads `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, and `src/null.rs` … it does not scan `spectre-dsp` or `spectre-graph` at all, so it says nothing about `SumBus` and this spec does not claim otherwise." Verified verbatim at `crates/spectre-audio/tests/rt_guard.rs:294–297`. | — |
| 1B. Control↔render communication | 2 | §4.4's state table names owner, thread, and lifetime per state, puts mixer edits on the RT-002 latest-wins parameter lane (decision 21, verified), and names off-thread reclamation for the retired plan (`RetiredState = Box<dyn Send>`, verified `control.rs:192`, `:314–324`). §3.2 is honest that the value is only counted as `parameters_pending` until R4-2 (verified `bridge.rs:181–186`). **One real gap:** solo is a per-track parameter whose toggle changes the *effective gain of every other track* (§4.3's `effective_gain` formula), but neither §3.2, §4.3, nor §4.4 states that one solo toggle must republish all N targets on the lane. An implementer following §3.2 literally ("publishes the track's effective linear gain") would publish one value and produce a solo that silences nothing. | See Priority 2 item 1. |
| 1C. Numerical containment | 3 | §4.3 gives `SumBus` boundary containment matching the shipped precedent — non-finite input treated as `0.0`, the same posture verified in `Gain` (`effect.rs:65–69`) and `Saturator` (`effect.rs:122–126`) — and then names the second guard accurately: `CompiledPlan::process` runs `contain_channel` on every node's output before it can reach a downstream node (verified `spectre-graph/src/lib.rs:512–522`), counting into `ContainmentStats` (`:389–397`), with denormals flushing to signed zero (`:401–414`). Injection tests are named specifically: test 10 (NaN and infinity buses on the device) and test 12 (a poisoned track silenced *before* the sum, reusing the existing `PoisonSource` at `containment.rs:44`). | — |
| 1D. Determinism | 3 | §1.3's success signal is a hash comparison using the **existing** FNV-1a walk, quoted with the verbatim offset basis `0xcbf2_9ce4_8422_2325` and prime `0x0000_0100_0000_01b3` (both verified at `spectre-offline/src/lib.rs:271,276`), with an explicit refusal to add a second comparison method, restated in §4.3 and §5.2. §5.2 assertion 1 carries the anti-vacuity guard (`peak > 0.0` on both renders) so two silent buffers cannot agree. Test 7 pins node-ID determinism against a fixed `IdGen` seed. Test 8 is unusually honest: it asserts bit equality only for the case where the arithmetic guarantees it, and for the genuinely reordered sum it permits one f32 ULP, "documenting the non-associativity §4.3 admits rather than asserting an equality the arithmetic does not guarantee." | — |
| 1E. Graph and plan contract | 3 | The criterion's two failure modes are both named and refused explicitly. §4.4 consequence 1: "The compiled plan is never mutated," citing `CompiledPlan` "exposes no node or edge mutation API by design" (verified `spectre-graph/src/lib.rs:416–417`); structural change produces a *new* plan on the app thread. §4.4 consequence 2: "Recompilation is never proposed per parameter change," citing decision 22's rejection of option (b) — verified verbatim at `decision-gates.md` row 22: "(b) is rejected on its face: plan compilation allocates and cannot run per knob turn." The `structure_revision` split is the mechanism, and §5.1 test 5 asserts it in both directions (mixer edits must not advance it; structural edits must). The `MAX_FLAT_INPUTS` widening is a pure widening — verified that `add_node`'s other two checks and every rule at `:171–180`, `:227–240`, `:242–273`, `:275–281` are untouched. | — |
| 1F. Failure behavior | 3 | §3.6's eight rows are fail-closed throughout, every one "Data loss: no", with atomic non-mutation on E3/E4 following the `open_device_in_shape` precedent (verified `lib.rs:338–345`). Two binding rules are stated normatively and both are correct: no error string is formatted on the audio thread, and **no threshold-derived alarm is specified** because Spectre owns exactly one headroom measurement (verified `current-milestone.md:113`, macOS, 0.990, three-node chain) and one data point cannot justify a threshold. E8 correctly classifies all-muted as exact silence rather than an error. E6 states plainly that a `TooManyTracks` build failure "is a defect, and the message says so rather than implying user error." | — |
| 1G. Test specification | 3 | Sixteen unit tests plus three integration assertions, each with setup, assertion, and a named edge case, and each able to fail on a stated regression. Test 6 catches a builder that silently drops a track via `step_count` (verified accessor at `spectre-graph/src/lib.rs:446`). Test 9 asserts bit equality against `0.0_f32.to_bits()` so a `-0.0` is caught. Test 13 covers the `InvalidLayout` input-count branch that is genuinely untested today (`grep -rn "MAX_FLAT\|InvalidLayout" crates/*/tests/` verified to return nothing). Test 14 puts the new device inside the existing allocation guard rather than beside it. §5.3 is honest that the GUI surfaces are unreachable from `crates/spectre-app/tests/` (verified: that directory contains exactly `app_model.rs` and `smoke_cli.rs`) and routes them to §5.4's manual protocol. Two trivial unstated consequences: a `CommandKind` variant carrying a `Track` (f32 fields) breaks the existing `Eq` derive at `command.rs:30`, and test 12 needs a `ToneSource` import `containment.rs` does not have. | See Priority 3 items 1 and 2. |

**Lens average:** 2.857 (20/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s
**Auto-fail triggered:** No — AF-3 does not trigger. The only new callback-path code is
`SumBus::process`, an accumulate-and-store loop over borrowed planar buffers with no
allocation, lock, I/O, logging, or panic; overflow policy and off-thread reclamation are
both named.

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 2 | §2's stories are the right ones (sketch a second track, audition by mute, compare without rebuilding), and §3.2's mute flow explicitly preserves "selection, lens, and transport position." But the loop this feature is named for does not actually close in R4-4, and the spec knows it: mute/solo/level are **not audible** until R4-2 (§3.2), and adding a track requires an explicit rebuild that stops and restarts the stream with an audible gap (§3.2 step 4, §4.4 consequence 3). The honesty is exemplary and correctly routed to §8 Q3 — but the criterion grades whether the feature supports audition, and here it substantially defers it. §3.2 step 4 also never states what happens to **transport position** across the stream restart, which is the one context the criterion names. | See Priority 2 item 2. |
| 2B. Linked lenses | 3 | Sidebar, inspector CONTEXT block, and Mix lens all read one `TrackList` on `AppModel`; §4.6 deletes `TrackView` outright rather than shipping beside it, with the reason stated against this criterion: "two track types in one binary is exactly the forked state criterion 2B fails." Stable identity is carried by `ObjectId` from the project `IdGen` and preserved across reorder (test 1) and across undo via the whole-`Track` inverse (§4.3). Selection survives every flow in §3.2. | — |
| 2C. Modulation visibility | 2 | The parameter-honesty half is handled well and prominently: the inspector's hard-coded `0.0..=1.0` slider (verified `main.rs:181`) is replaced by the accepted descriptor range `GAIN_PARAMETERS[0]` (verified `effect.rs:19–20`, `0.0..=2.0`, default `1.0`) so UI stops contradicting DSP per `dsp-device-io.md:60` (verified verbatim), and the Mix bar is normalized through `to_normalized` (verified `parameter.rs:53–56`) so bar and slider agree. But PROD-002 itself is never named. R4-4 introduces a fader — a parameter surface that will later carry automation and modulation contribution — and the spec says nothing about keeping base / automation / modulation distinct on it, nor about a restore action, beyond deferring automation to R9 in §6.4. Non-foreclosing, but unaddressed. | See Priority 3 item 3. |
| 2D. Keyboard-first, calm UI | 3 | §3.4 is the correct answer to the AF-5 hazard, not an evasion: "**No keyboard shortcut is assigned to any of these commands, and none is proposed**," citing `product-implications.md` §"Prohibited conclusions at current evidence level" lines 90–102 — **verified exactly**: the section header is at line 90 and "promotion of any workflow archetype" at 102, with "a default shortcut map" at 96. It commits only to the allowed shape (named, context-scoped, remappable commands with stable identifiers) and explicitly disowns the prototype's pre-existing `Space`/`1`–`4` handlers (verified `main.rs:425–437`) rather than ratifying them. Reorder is two ordinary buttons, not drag-only, "which keeps reorder reachable by keyboard from day one." Calm density: the action cluster appears on the selected row only, citing `vision.md`'s "no spreadsheet density" (verified `vision.md:41`). | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A does the harder thing correctly. On summing it identifies a genuine **divergence** between two researched systems — `OBS-PP-ARCH-002` (Phase Plant generators "each mixing onto the signal from above") versus `OBS-BW53-GRID-002` (Bitwig: "an in port accepts exactly one cable; unconnected in ports read zero") — both verified verbatim, and concludes "there is no convergent pattern to follow," then states why Spectre keeps the explicit model and grounds it in its own shipped rule (`InputBusOccupied`, verified `spectre-graph/src/lib.rs:171–180`). Convergences are followed and named (`OBS-AB12-ROUTE-003` stereo-throughout, `OBS-AB12-MIX-002` no internal limiter, `OBS-PP-ARCH-001` explicit output). The one divergence from a benchmark — additive vs Live's exclusive solo — is flagged as a divergence and escalated to §8 Q10 rather than settled. | — |
| 2F. Differentiation | 2 | The spec is thorough about what it defers and thin about what makes Spectre different. §4.4's "mixer state travels; structure does not" rule, the mandatory rebuild notice, and §6.3's normative copy rules (a state word must name the *actual* cause; the notice must appear whenever revisions differ) are genuinely differentiating — a DAW that refuses to imply an edit is audible when it is not — but the spec never frames any of it as differentiation, and §6.4 3D's originality argument is about names and numbers rather than behavior. No parity claim is made, so the non-goal half of the criterion is clean. | See Priority 3 item 4. |
| 2G. Benchmark evidence discipline | 3 | Every `OBS-` ID cited was opened and each record says what the spec claims: `OBS-AB12-ROUTE-003`, `-MIX-001`, `-MIX-002`, `-MIX-003`, `-MIX-004`, `-MIX-005`, `-MIX-009`, `-SES-005`, `-LAUNCH-007`, `OBS-PP-ARCH-001`, `-ARCH-002`, `-FX-001`, `OBS-VCV-VOLT-005`, `-VOLT-006`, `OBS-BW53-GRID-002`, `-LAUNCH-001`. Two structural claims verified exactly: the Ableton corpus spans **exactly** the nine categories ARR/AUTO/CLIP/LAUNCH/MIX/REC/ROUTE/SES/WARP, and `grep -in "maximum\|limit"` on it returns **exactly one** hit, `OBS-AB12-LAUNCH-007`, as stated. The four gaps are named rather than filled: no track-count record, no summing-bus or fader-law record, Logic Pro zero (verified `logic-pro.md:11` `inventory-only`), Serum 2 two records **correctly not used**, with the 271-record quarantined extraction cited by nothing. One citation nit: `OBS-AB12-MIX-003` is attributed to Live §18.3; the record reads (18.1). | See Priority 3 item 5. |

**Lens average:** 2.571 (18/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s
**Auto-fail triggered:** **No — AF-5 does not trigger**, and each prohibited conclusion was
checked individually. No default shortcut map (§3.4 refuses one by name and citation). No
final native-device list — `SumBus` is proposed as a fifth *layout* amendment and escalated
to `decisions-needed.md` as §8 Q4, not asserted. No gesture-count or time budget — §4.7
states "**No budget threshold is asserted**" with the one-measurement reason. No
monitoring-latency threshold, no supported-interface list, no command-frequency or
feature-priority scores. No platform/backend order — §4.6 makes no Linux claim. No workflow
archetype is promoted.

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | The slice is exactly R4 slice 4: `docs/status/NEXT.md:26` reads "Introduce the track model with a track-to-master signal path, keeping the graph compilation contract intact" — **verified to the line**, as are `current-milestone.md:12` ("track to master") and `:83` ("A MIDI clip plays through a track into master"). Every out-of-scope neighbour the brief flags is named and deferred with a verified roadmap line: sends, returns, groups, monitoring, compensation, meters and track types → R6 (`rebuild-roadmap.md:31`, verified verbatim); recording and arming → R7 (`:32`); automation → R9 (`:34`); per-track launcher authority → R10 (`:35`). Mixer depth is refused concretely rather than rhetorically: §3.3 ships **no meters** because peak/RMS is `OBS-AB12-MIX-001` behavior and R6, and refuses to dress a fader-position bar as one. Re-parenting the device browser onto tracks is explicitly excluded (§3.1) and raised as §8 Q2. | — |
| 3B. Non-goal respect | 3 | §6.4 3B enumerates the non-goals and none is proposed: no CLAP/LV2/AU hosting, no plugin-format authoring, no cross-DAW preset or project compatibility, no cloud or content store, no video scoring. §4.5 adds no third-party crate and no asset. The `armed` flag is **removed** rather than left inert — the opposite of implying recording exists. | — |
| 3C. Deliberately small first devices | 3 | Exactly one device is added, `SumBus`, with zero parameters, no tone, no character, and no UI beyond the master strip; §6.4 3C argues correctly that it is routing infrastructure rather than an instrument or effect. `PulseInstrument`, `Gain`, and `Saturator` are reused unchanged — verified against `crates/spectre-dsp/src/lib.rs:11–19`, which exports exactly those four devices. The track fader is the shipped `Gain`, not a new one. Nothing grows toward the R11 flagship (decision 15, verified). | — |
| 3D. Originality | 3 | **AF-4 is fully discharged.** Both numeric bounds carry Spectre-derived rationale: `MAX_TRACKS = 16` is argued from the structural necessity of a compile-time bound (`PlanStep`'s `[usize; MAX_FLAT_INPUTS]`, verified `spectre-graph/src/lib.rs:337`, and `process`'s stack array, verified `:497`) plus Spectre's own cost arithmetic — **every figure in §4.7 recomputes correctly**: 16,384 B at one track, 139,264 B at sixteen, 288 B per `PlanStep`, 512 B of stack per step, 9.8 KB of steps at 34, 8,192 adds per block. Critically, §7.2's modified-files list **schedules the ledger rows** — `MIX-001` and `MIX-002` into `docs/01-requirements/requirements-ledger.md` — and states outright that §4.2's inline rationale does not discharge PROD-003 (verified at `requirements-ledger.md:64`: rationale must be recorded "in this ledger"). Names are original; Appendix A states explicitly that no benchmark track count exists to copy. | — |
| 3E. Platform commitment | 3 | §4.6 is platform-neutral by construction (no platform, driver, OS API, or backend is named) and then does the harder thing: it states that R4-4 makes decision 23's undischarged Linux debt **worse**, because the alpha's plan grows from three nodes to up to 34 while the only headroom measurement Spectre owns is macOS on the three-node fixture (verified `current-milestone.md:113`, and `:114` reads "not run" for Linux). It authorizes no Linux claim and adds a concrete requirement to R4-3's run in §8 Q9. | — |
| 3F. Accessibility trajectory | 3 | Nothing forecloses decision 17. §3.7 removes the only glyph-only state in the sidebar (the `●`/`○` armed indicator, verified `main.rs:131`), carries mute/solo/solo-silencing in words so a monochrome reading loses nothing, uses only standard egui widgets, decomposes reorder into two single-activation buttons so no custom action is needed, and states focus order follows creation order. It also records the real blocker honestly rather than claiming support: `crates/spectre-app/Cargo.toml` builds eframe `0.32.3` with `default-features = false` and only `default_fonts` and `glow` — **verified** — so no accessibility feature is enabled today and R4-4 claims none. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes
**Auto-fail triggered:** No — 3B is not 0, and **AF-4 does not trigger**: both bounds carry
Spectre-derived rationale *and* scheduled `requirements-ledger.md` rows in §7.2.

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 2 | §7.1 uses the required implemented / prototyped / planned / gated / absent partition and nearly all of it verifies to the exact line. Confirmed individually: `TrackView`'s six public fields (`lib.rs:37–45`); `add_track` **exists** and does exactly what is described, including the hard-coded `level: 0.72` (`lib.rs:405–422`); the sidebar exists (`main.rs:115–160`) with the armed glyph at `:131`; the toggles at `:177–179`; the `0.0..=1.0` slider at `:181`; the literal `"MIDI clip  →  Native synth  →  Master"` at `:184`; `"Master route"` at `:474`; the default project's single `"Pulse"` track at `level: 0.78` (`lib.rs:218–262`); `smoke_cli.rs:18`'s `tracks=1`; the two tests at `app_model.rs:33–49`. The absence claims verify: `grep -rn "struct Track\b\|fn arm\|fn mute" crates` returns **nothing**; `grep -rn "MAX_FLAT\|InvalidLayout" crates/*/tests/` returns **nothing**; `.muted`/`.solo` are written at `main.rs:177–178` and **read by nothing**; `crates/spectre-app` has **no** `spectre-graph` or `spectre-audio` dependency; `crates/spectre-app/src/engine.rs` does not exist. The spec also **refused the false claim it was handed** about `add_track`, and was right to. **Against that, two §7.1 statements do not hold.** (i) `grep -rn track crates --include='*.rs'` is stated in §1.2 and §7.1 to hit only four `spectre-app` files; it also hits `crates/spectre-project/src/lib.rs:162,170`. The conclusion survives — both are `future_track_kind`, a fictitious JSON key in a serde forward-compatibility test, not a track concept — but the transcript as written does not reproduce. (ii) §7.1 states R4-2 and R4-7 are `spec-in-progress` and that "neither has a spec file"; both spec files exist in this repository now, and at the pinned commit `2e005e5` the manifest recorded both as `pending`, not `spec-in-progress`. The claim was accurate against the dirty working tree when written and has gone stale through concurrent batch progress; the operative half ("neither has an implementation") remains true. Roughly a dozen line ranges are off by one (`MAX_FLAT_INPUTS` cited at `:16`, actually `:15`; `PULSE_PARAMETERS` at `source.rs:38-45`, actually `39–46`; several test-range ends). | See Priority 1 items 1 and 2, and Priority 3 item 6. |
| 4B. Status vocabulary | 3 | The vocabulary is used correctly throughout and the `implemented`/`verified` distinction is respected explicitly rather than incidentally: §7.2 states "Status moves to `implemented`, not `verified`, until §5.4's manual protocol passes." The spec's own status is `proposed`. §7.1's gated block correctly reports decision 22 as design-accepted with implementation at R4-2, GRAPH-002 as `proposed` with only implicit-cycle rejection built, CORE-001 as `implemented` with reorder evidence gated, and decision 13 as SD-adopted with one command kind shipped — all four verified against `requirements-ledger.md:46,55,56` and `decision-gates.md` rows 13 and 22. | — |
| 4C. Traceability | 3 | Every normative claim carries a requirement ID, decision row, `OBS-` ID, or source path. Sampled heavily and found no uncited normative claim. The high-risk ones verify verbatim: decision 6's f64-accumulator wording, decision 22's "(b) is rejected on its face," `dsp-device-io.md:27` (every output sample written), `:60` (UI must not redefine DSP ranges), `:35–39` (exactly four v1 layouts), `:94` and `:104` (the `Gain`-smoothing contract text), `vision.md:48` ("honest telemetry, no fake surfaces"), `docs/README.md:34` (the governing rule). | — |
| 4D. Honest gaps | 3 | Ten open questions, three of them escalated to `decisions-needed.md` because they change accepted material, and the escalation is done in the header, §7.2, and §8 rather than by implication — satisfying AF-1 on all three amendments. The strongest instance is unprompted: §7.1's absent block surfaces a **contradiction between an accepted contract and shipped code** — `dsp-device-io.md:94`/`:104` describe `Gain` as smoothing; `crates/spectre-dsp/src/effect.rs:29–74` has a single `gain: f32` and a direct multiply with no ramp (verified) — declines to fix it inside this slice, and routes it to §8 Q6 with three named options. §4.2 also states CORE-001's reorder evidence is only **half** discharged here. | — |
| 4E. Evidence commands | 3 | The workspace gate is quoted exactly as `criteria.md` 4E requires: `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`. Every feature-scoped command names a real target — the three new test files are declared in §7.2, and `devices.rs`, `containment.rs`, `graph_plan.rs`, `plan_alloc.rs`, `app_model.rs`, `smoke_cli.rs`, `rt_guard.rs` all exist as named. §5.2 makes re-running the graph suite **after** the `MAX_FLAT_INPUTS` change a required gate rather than a formality. | — |
| 4F. No fake surfaces | 3 | This is the spec's spine and it is applied against itself. It removes the literal signal-path string at `main.rs:184` and the inert `armed` glyph rather than leaving either; it states in four places that `./spectre` produces no sound today (verified: `toggle_play` at `lib.rs:268–275` mutates an in-memory `Transport`, the transport bar prints the hard-coded `ENGINE OFFLINE` at `main.rs:79`, and the crate has no audio dependency); it states that mute/solo/level will **not** be audible in R4-4; it makes the rebuild notice normative in §6.3 ("an app that hides it is claiming an edit is audible when it is not"); and §5.4's Honesty-check row makes "toggle mute — the sound must **not** change" a manual gate. §5.4 also warns to run with system volume low because there is no limiter. | — |

**Lens average:** 2.833 (17/6)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s
**Auto-fail triggered:** **No.** AF-2 does not trigger: no code is described as existing,
partial, or implemented without a source path containing it, and no absent thing is
described that in fact exists. Applying the standard used against R4-1 — *"the path does
not contain the claim"* — the claim at issue here ("no track concept outside
`crates/spectre-app/`") **is** contained by the source; only its supporting grep transcript
is incomplete, which the R4-1 review classed as Priority 3 imprecision rather than
auto-fail. AF-6 does not trigger: no optimistic or promotional phrasing was found, and §6.3
audits for it directly.

---

## Auto-fail roll-call

| Rule | Triggered | Basis |
|---|---|---|
| AF-1 — Contradicting accepted authority | No | Three amendments to accepted material are proposed: the `dsp-device-io.md:35–39` layout list (§8 Q4), `MAX_FLAT_INPUTS` (§8 Q5), and `MAX_TRACKS` (§8 Q1). Each is flagged as an amendment, proposes supersession, and is routed to `decisions-needed.md` in the header, §7.2, and §8. Nothing is amended by assertion. |
| AF-2 — Unbacked implementation claims | No | Every existence and absence claim in §7.1 was checked individually against `spectre-app/src/{lib,main}.rs`, `spectre-app/tests/`, `spectre-graph/src/lib.rs`, `spectre-core/`, and `spectre-dsp/src/`. All existence claims are backed; all absence claims are genuine. The two §7.1 inaccuracies found (Priority 1) are an incomplete grep transcript whose conclusion holds, and a stale sibling-spec status outside the codebase. |
| AF-3 — Realtime discipline violation | No | One new callback-path item, `SumBus::process`; allocation-free accumulate-and-store over borrowed buffers, covered by the existing counting-allocator guard. Latest-wins lane for parameters, off-thread reclamation named, denormal flush and NaN/Inf isolation addressed at two layers. |
| AF-4 — Borrowed numeric limits | No | `MAX_TRACKS`/`MAX_SUM_BUSES`/`MAX_FLAT_INPUTS` all carry Spectre-derived rationale **and** scheduled `requirements-ledger.md` rows `MIX-001`/`MIX-002` in §7.2. The spec states explicitly that the corpus contains no track-count record to borrow from. |
| AF-5 — Conclusions the evidence does not support | No | All nine prohibited conclusions checked individually; none asserted. §3.4 refuses a default shortcut map by name and cites `product-implications.md:90–102`, verified exactly. |
| AF-6 — Optimistic language | No | No unevidenced or promotional phrasing found. The spec repeatedly states what does **not** work. |

---

## Feasibility Check

Every source path the spec cites was opened and checked at the cited lines.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | `Track`, `TrackList`, `TrackInstrument`, `TrackError`, `SumBus`, `TrackPathNodes`, `RoutingError` are fully specified in §4.2 with signatures. Every existing type they build on was verified: `ObjectId`, `IdGen`, `DeviceIo`, `AudioProcessor` (`io.rs:163`, `Send` bound present), `NodeId`, `EditableGraph`, `CompiledPlan`, `Gain`, `PulseInstrument`. |
| API/interface changes are feasible with current architecture | ✓ | `SumBus` with `audio_inputs: buses * 2` passes `add_node`'s three checks at `spectre-graph/src/lib.rs:138–143` for `buses ≤ 16` once `MAX_FLAT_INPUTS = 32`; `buses == 0` also passes (0 is a multiple of 2 and under bound) and compiles, since the `MissingInput` loop at `:227–240` iterates an empty range. `validate_buffers` (`io.rs:181–196`) accepts a zero-input effect. `compile` flattens buses in bus order (`:298–308`) and `process` slices `&inputs[..input_channels]` (`:507`), so bus index = track index holds as claimed. Test 13's boundary (32 accepted, 34 refused) is arithmetically correct. |
| Views/screens fit current navigation pattern | ✓ | No new screen. All four modified regions exist as cited: `track_list()` `115–160`, `inspector()` CONTEXT block `172–186`, `mix_surface()` `458–478`, `transport()` `50–84`. Sidebar 220 px default / 180 px min (`:118–119`) and the 1060×680 window minimum (`:508`) verified. |
| Dependencies are available and version-compatible | ✓ | No third-party crate added. Adding `spectre-dsp` and `spectre-graph` to `spectre-project` creates no cycle — verified from the manifests that `spectre-graph` depends only on `spectre-core` + `spectre-dsp`, and `spectre-dsp` only on `spectre-core`. `spectre-app` and `spectre-offline` both already depend on `spectre-project`, so one builder can serve both as claimed. eframe stays pinned at `0.32.3`. |
| Platform/renderer requirements are realistic | ✓ | Portable Rust arithmetic over workspace types; no platform, driver, or OS API named. Only egui widgets already used in `main.rs` are required. The Linux consequence is named as debt, not discharged. |
| Test strategy is executable with current infrastructure | ✓ | All named existing targets exist and the cited fixtures are real (`plan_alloc.rs`'s counting allocator `22–37`, `containment.rs`'s `PoisonSource` `:44`, `devices.rs`'s `output()` helper and `device_layouts_match_the_v1_contract` `186–209`). Two trivial unstated consequences: a `CommandKind` variant carrying `Track` (f32 fields) breaks the `Eq` derive at `command.rs:30`, and test 12 needs a `ToneSource` import. |
| Performance budget is realistic for target hardware | ✓ | §4.7 asserts no threshold, which is the correct posture on one measurement. Every figure recomputes exactly from the verified source: channel pool `(4n+4) × max_frames × 4 B`, `[&[f32]; 32]` = 512 B of stack, 34 × 288 B ≈ 9.8 KB of steps, 8,192 f64 adds per block at 16 tracks / 256 frames. |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares all of them: R4-1 blocks only `build_track_engine_parts` (`engine.rs` confirmed absent — the crate is exactly `lib.rs` and `main.rs`); R4-2 blocks audibility; R4-7 blocks persistence and `EditHistory` routing; R4-5 replaces the single-note-node arrangement; R4-9 owns the multi-track headroom measurement. The spec states which parts are implementable today without any of them. |

**Feasibility verdict:** Feasible
**Caveats:** The five new `CommandKind` variants cannot carry a `Track` while `CommandKind`,
`ProjectCommand`, and `Transaction` keep their `Eq` derives; dropping `Eq` is the obvious
fix but the spec does not mention it. §8 Q8's ordering question (commands now vs. at R4-7)
should be answered before implementation begins, since the answer decides whether
`command.rs` is touched once or twice.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.857 | 35% | 1.000 |
| 2 — DAW Workflow Depth | 2.571 | 25% | 0.643 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.833 | 20% | 0.567 |
| **Composite** | | | **2.810** |

**Pass conditions (values copied from `criteria.md`, which is binding):**
- [x] Composite ≥ **2.30** — **2.810**
- [x] Every lens average ≥ **2.00** — 2.857 / 2.571 / 3.000 / 2.833
- [x] No criterion scores **0** — none; the lowest score awarded is 2
- [x] At most **two** criteria score 1 — **zero** criteria scored 1
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear; see roll-call
- [x] Feasibility rule satisfied — every cited source path was opened and checked at the cited lines; §7.1's substantive description of current state is accurate in both directions
- [x] Reviewer personally executed every command claimed as passing when required — the spec claims **no** command as currently passing. The three `grep` transcripts it presents as evidence were each executed: two reproduce exactly, one does not (Priority 1 item 1). No `cargo` command is asserted as green, so none was required to be run.

**All conditions met:** Yes → **PASS**

---

## Remediation Brief

The spec passes. These are corrections to make before implementation, not pass blockers.
Priority 1 items are factual repairs to §7.1 and must land regardless.

### Priority 1 — Factual corrections required before this spec is used as a source of truth

1. **§1.2 and §7.1 — the `track` grep transcript does not reproduce.** Both sections state
   that `grep -rn track crates --include='*.rs'` hits only `crates/spectre-app/`'s
   `src/lib.rs`, `src/main.rs`, `tests/app_model.rs`, and `tests/smoke_cli.rs`. It also
   returns `crates/spectre-project/src/lib.rs:162` and `:170`. Both hits are the string
   `future_track_kind`, a fictitious JSON key inside an in-file test asserting unknown-field
   preservation — so the **conclusion** ("no track concept outside `crates/spectre-app/`")
   is correct and should be kept. Amend both sentences to list the fifth file and state in
   one clause why it is not a counterexample. Do not weaken the conclusion.

2. **§7.1 "Planned" — the R4-2 / R4-7 status is stale and its citation does not match the
   pinned commit.** The spec states both are `spec-in-progress` per `manifest.md:45, 50` and
   that "neither has a spec file." Both spec files now exist
   (`gauntlet-output/specs/R4-2-runtime-parameter-seam.md`,
   `…/R4-7-project-persistence.md`) and both features are `spec-pass` at 3.000; at the
   spec's own pinned commit `2e005e5` the manifest recorded them as `pending`. The
   engineering half of the claim — neither has an **implementation** — is still true and is
   the only half the spec's reasoning uses. Rewrite the sentence to assert only that, and
   drop the manifest line-number citation, which cannot be pinned to a commit while
   `manifest.md` is being written concurrently. Separately, §7.1's opening paragraph
   describes the manifest's 2026-08-12 run-history entry in the present tense; that entry
   has since been corrected (`manifest.md:89–93`), largely because of this spec — note the
   correction rather than describing the superseded text as current.

### Priority 2 — Should fix for quality

1. **§3.2, §4.3, §4.4 — state that a solo toggle republishes every track's gain.** Solo is
   defined so that toggling it on one track changes `effective_gain` for *all* tracks, but
   every statement about the parameter lane is singular ("publishes the track's effective
   linear gain"). An implementer following that literally ships a solo that silences
   nothing. Add one binding sentence: any edit to solo, or to the solo set, republishes
   `effective_gain` for every track in `TrackList`, not only the edited one. Add an
   assertion to §5.1 test 16 that a single `set_track_soloed` produces N publications.

2. **§3.2 step 4 — say what happens to transport position across the engine rebuild.** The
   rebuild stops the stream, installs a new bridge, and restarts. §3.2's mute flow promises
   the user keeps "selection, lens, and transport position," but the structural flow never
   says whether position survives the restart. Criterion 2A grades exactly this. State it
   either way — preserved, or reset with the copy saying so.

### Priority 3 — Consider for excellence

1. **§4.3 / §7.2 — note that the five new `CommandKind` variants force dropping `Eq`.**
   `CommandKind`, `ProjectCommand`, and `Transaction` derive `Eq` at `command.rs:30`, `:36`,
   `:70`; a variant carrying a `Track` (or an f32 mix value) cannot. One line in §7.2.

2. **§5.1 test 12 — `containment.rs` does not import `ToneSource`.** Its imports are
   `AudioProcessor, DeviceClass, DeviceIo, Gain, NoteEvent, ProcessContext, ProcessError,
   PulseInstrument, Waveform`. Name the import addition, or use the file's existing clean
   `PoisonSource` variant instead.

3. **§3.3 / §6.4 — name PROD-002 on the track fader.** R4-4 introduces the first parameter
   surface a track owns. The spec never states that the fader must later keep base value,
   automation, and modulation contribution distinct with a restore action. One
   non-foreclosure sentence, as §4.4 already does for PROD-001, would close criterion 2C.

4. **§6.4 or §1.2 — state the differentiation explicitly.** "Mixer state travels; structure
   does not," the mandatory rebuild notice, and §6.3's rule that a state word must name the
   actual cause are genuinely unlike the benchmark set's behavior, and the spec never says
   so. Criterion 2F asks for exactly this claim.

5. **Appendix A — `OBS-AB12-MIX-003` is attributed to Live §18.3; the record reads (18.1).**
   §18.3 is `OBS-AB12-MIX-004`'s section. Content is accurate; only the manual section is
   wrong.

6. **Off-by-one line citations.** `MAX_FLAT_INPUTS` is at `spectre-graph/src/lib.rs:15`, not
   `:16` (`FLAT_OUTPUTS` is at `:16`, and the spec cites *that* correctly in Appendix A);
   `PULSE_PARAMETERS` is at `source.rs:39–46`, not `:38-45`. Several test-file range ends
   overshoot by one blank line (`devices.rs:12–15`, `:186–210`, `:251–272`;
   `plan_alloc.rs:22–39`). Also fix the typo `cratests/spectre-graph/tests/containment.rs`
   in §7.2's modified-files list.

### Standing note — not a remediation item

The one thing in this spec that should be preserved verbatim rather than edited is §7.1's
opening refusal. The spec author was briefed with an accepted document's claim that
`AppModel::add_track()` and the track list sidebar were inventions of the discarded
2026-08-12 `track-management.md`. That claim was false — both have existed since commit
`6c397d9`, `add_track` at `crates/spectre-app/src/lib.rs:405–422` and the sidebar at
`crates/spectre-app/src/main.rs:115–160`, with two covering tests at
`crates/spectre-app/tests/app_model.rs:33–49` — and the spec checked the source, refused
the claim in writing, and stated the corrected partition instead. I verified all four of
those source locations independently and the spec's version is right.

That refusal is the behavior the feasibility rule exists to produce, and it is worth
recording that it worked against an *accepted* document rather than only against a spec:
`criteria.md` was reproducing the same error inside its own AF-2 illustration, which is to
say inside the rule that forbids exactly that. Priority 1 item 2 asks the author to update
the surrounding present-tense framing now that `manifest.md` carries the correction — it
does **not** ask for any change to the refusal itself or to the corrected partition.

---

## Review Scope

Stated plainly so a later reader can judge what this verdict rests on.

**Read in full, start to end:**

- `gauntlet-output/criteria.md`, including the 2026-08-15 correction block
- `gauntlet-output/templates/SCORECARD-TEMPLATE.md`
- `gauntlet-output/specs/R4-4-track-model.md` — all 1,610 lines, in three passes
- `crates/spectre-app/src/lib.rs` (474 lines) and `crates/spectre-app/src/main.rs` (519)
- `crates/spectre-graph/src/lib.rs` — all 545 lines, the file the spec changes
- `crates/spectre-dsp/src/effect.rs` and `crates/spectre-dsp/src/lib.rs`
- All six crate manifests: `spectre-app`, `spectre-project`, `spectre-graph`,
  `spectre-dsp`, `spectre-offline`, and the dependency direction they establish
- `crates/spectre-app/tests/smoke_cli.rs`
- Every `OBS-` record the spec cites — sixteen IDs across
  `ableton-live-observations.md`, `synth-modular-observations.md`, and
  `bitwig-studio-observations.md`
- Every accepted-document line the spec cites in `requirements-ledger.md`,
  `decision-gates.md`, `current-milestone.md`, `rebuild-roadmap.md`, `NEXT.md`,
  `dsp-device-io.md`, `vision.md`, `docs/README.md`, and `STATUS.md`

**Sampled by targeted range or grep, not read end to end:**

- `crates/spectre-offline/src/lib.rs` — `render_plan` and the FNV-1a walk (195–290),
  `RenderReport` (20–40), plus greps for both hash constants
- `crates/spectre-audio/src/bridge.rs` — the `RenderBridge` struct, `new`, and `render`
  (110–215), plus greps for the note-node and parameter-drain lines
- `crates/spectre-audio/src/control.rs` — `ParameterTarget`, `RetiredState`,
  `control_channel`, `retire` (18–30, 188–215, 310–326)
- `crates/spectre-audio/tests/rt_guard.rs` — **grep only**, for the structural scan list at
  `294–297`. This is the single claim R4-1 failed on, so I checked it directly rather than
  by inference, but I did not read the rest of the file.
- `crates/spectre-dsp/src/source.rs` (1–60), `io.rs` (1–60, 150–198),
  `parameter.rs` (45–60, 76–130)
- `crates/spectre-project/src/lib.rs` (1–45 plus greps), `command.rs` (25–182)
- `crates/spectre-app/tests/app_model.rs` (25–55),
  `crates/spectre-graph/tests/{plan_alloc,containment,graph_plan}.rs` and
  `crates/spectre-dsp/tests/devices.rs` — the cited fixtures and ranges only
- `gauntlet-output/manifest.md` — the header, feature table, and run-history block, at both
  the working tree and commit `2e005e5`

**Executed:** the three `grep` transcripts the spec presents as its own evidence, plus
`git show 2e005e5:gauntlet-output/manifest.md` to test the spec's commit pin. Two of the
three greps reproduce exactly; the third is Priority 1 item 1.

**Not executed:** no `cargo` command was run. The spec asserts no command as currently
passing — every test it names is one this slice creates or extends — so there was nothing
to reproduce. A reviewer of the *implementation* must run the §5 gate; this scorecard does
not stand in for that.

**Not reviewed:** the spec author's reasoning, which I never saw; the R4-2, R4-3, and R4-7
specs, beyond confirming that the two files exist for Priority 1 item 2; and whether
`MAX_TRACKS = 16` is the right number, which §8 Q1 correctly escalates to Jeff and which no
reviewer should settle.

---

**End of scorecard.**
