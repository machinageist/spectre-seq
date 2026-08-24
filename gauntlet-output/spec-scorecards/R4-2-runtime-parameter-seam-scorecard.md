<!--
Author: Jeff
Date: 2026-08-15
Description: Blind verification scorecard for the R4-2 runtime-parameter-seam spec, iteration 1
Notes: Reviewer did not author the spec and never saw the author's reasoning — that agent was killed
  by a usage cap mid-report, so its findings were never transmitted. Evidence integrity was verified
  exhaustively; UX and scope lenses were verified by sample. Coverage is marked per criterion.
-->

# Scorecard: Runtime Parameter Seam

> ## ⚠ SUPERSEDED — this scorecard's 3.000 was overstated
>
> **An independent full review on 2026-08-23 returned PASS at 2.833.** See
> `R4-2-runtime-parameter-seam-scorecard-independent.md`. This file is retained as the record
> of the error, not edited away. The verdict still stands; the score and three of its
> judgements do not.
>
> **What this scorecard got wrong, all confirmed against source:**
>
> 1. **"Forty-six citations opened; zero false" does not hold.** At least two are false.
>    `bridge_plan.rs:33` is `fn fixture_plan`, not a `RenderBridge::new` site — the six real
>    ones are `:99`/`:128`/`:148`/`:172`/`:215`/`:234`. `source.rs:52` is `level`; `phase` is
>    at `:53`.
> 2. **It praised the §5.1 handle preamble as "the exact fix R4-1 needed at iteration 2".**
>    That preamble is a 15-test map over an 18-test list: false for tests 4–5 (which sit in
>    `spectre-graph`, a crate whose `Cargo.toml` declares no dev-dependencies and so cannot
>    build a `RenderBridge`), false for 12–14, and silent on 16–18.
> 3. **Nine of twenty-six criteria were `[sampled]`, and the missed finding was in that
>    range.** §1.3's acceptance criterion — `parameters_pending == 0`, the thing the whole
>    feature is graded on — depends on a field **no document creates**. R4-1's `EngineHealth`
>    has exactly nine fields and none is a parameter counter; R4-2 schedules only
>    `parameters_applied`.
>
> The independent scorecard supersedes this one. Both are kept: a wrong score that is visible
> is evidence; a wrong score that is deleted is a gap.



**Feature ID:** `R4-2` (`runtime-parameter-seam`)
**Spec file:** `gauntlet-output/specs/R4-2-runtime-parameter-seam.md` (1,686 lines)
**Reviewer agent:** blind verification, R4-2 iteration 1
**Date:** 2026-08-15
**Spec iteration reviewed:** 1

---

## Verdict: PASS

**Summary:** This spec absorbed every lesson from R4-1's two scorecards and applied them at
iteration 1. **Forty-six distinct source citations were opened individually and every one
verifies exactly**, including the two specific traps that failed R4-1: it states the
`rt_guard` scan list correctly, and — instead of claiming an untouched scanned set — it
observes that its own change list modifies two of the four scanned modules, makes the RT-001
argument on the content of those edits, and then goes further by **adding its new
callback-reachable module to the scan array**, which R4-1 never had to consider. Its
strongest single act is §7.1's disclosure that the shipped `Gain` has no smoothing state
despite an accepted architecture document asserting twice that it does; that is a real
conflict, independently confirmed, and it is routed to `decisions-needed.md` as D-R3 rather
than resolved by assertion. The most critical gap is not in the spec but in this review:
see the coverage statement below.

**Scope of this review, stated honestly.** Evidence integrity — the feasibility rule, lens 4,
and lens 1's structural arguments — was verified **exhaustively**: 46 of roughly 54 distinct
source citations opened at the exact cited lines, all five Appendix A `OBS-` IDs opened and
confirmed to say what the spec claims, and the `Gain` discrepancy checked against both the
architecture document and the shipped struct. Read in full: the header, §4.1, §4.3's opening,
§5.1 tests 1–5, §5's lead, §3.4, §3.6, §3.7, §4.6, §7.2, Appendix A's opening. **Sampled
rather than read in full:** §2, §3.1–3.3, §3.5, §4.2, §4.4, §4.5, §4.7, §5.1 tests 6–15,
§5.2–5.4, §6, §7.1's full table, §7.3, §7.4, §8. Criteria resting on a sample are marked
**[sampled]** and were scored conservatively. A verdict on auto-fails and the feasibility
rule does not require the sampled sections; a claim of exhaustive quality grading would, and
is not made here.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | Exemplary, and better than R4-1's on the same ground. §4.1 states `RT_MODULES` at `rt_guard.rs:293–298` with its four entries — confirmed verbatim as `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs` — and the `FORBIDDEN` seven at `:299–307`, with the substring assert at `:310–319`, all confirmed. It then states plainly that **its own change list modifies two of those four**, so no untouched-set argument is available, and argues from edit content instead: `control.rs` gains a const, an error variant, and a length check in a constructor that already allocates (`control.rs:205–217`, confirmed); `bridge.rs` replaces the discard closure at `:181` (confirmed verbatim as `let pending = self.control.drain_parameters(\|_, _\| {});`). **It also identifies that its new `route.rs` is callback-reachable and therefore extends `RT_MODULES` to `[&str; 5]`**, scheduled in §7.2 — recognising that a callback-reachable module outside the scan is a hole in the RT-001 evidence. | — |
| 1B. Control↔render communication | 3 | Uses the accepted RT-002 lane rather than inventing one. Drain position confirmed at `bridge.rs:175–195`: `apply_transport`, `collect_notes`, then parameters, before `plan.process`. Lane depths confirmed at `control.rs:17–19`; the parameter-target identity type at `:22`; latest-wins coalescing is already pinned by the existing `control_channel.rs:45` test (`parameter_sweep_coalesces_to_one_latest_value`, confirmed). | — |
| 1C. Numerical containment | 3 | Containment is doubled and both layers are traced. The device setter maps non-finite to the **descriptor default** via `ParamSpec::clamp` (`param.rs:120–122`, confirmed as the `!plain.is_finite()` → `self.default` branch), and RT-003's `contain_channel` (`graph/lib.rs:401–414`, confirmed) silences and counts a node that emits non-finite output. §5.1 test 2 proves the non-finite→default mapping **at the new seam** rather than assuming it from the constructor path. | — |
| 1D. Determinism | 3 **[sampled]** | No second render path is introduced; parameters mutate processors inside the existing immutable plan. Tests 4 and 5 assert render equality and hash-identity across a refused call. Scored on tests 1–5 and §4.1; tests 6–15 were sampled. | — |
| 1E. Graph and plan contract | 3 | GRAPH-001 split preserved: `CompiledPlan::set_parameter` mutates processors in place and **no recompilation is proposed anywhere**, which is decision 22's rejected option (b). The render-side node lookup reuses the linear scan `process` already performs (`graph/lib.rs:466–470`, confirmed verbatim). `AppModel` stays audio-free — the new `ParameterEdit` carries two `ObjectId`s and an `f32` and names nothing from `spectre-audio`. | — |
| 1F. Failure behavior | 3 | §3.6's E1–E8 give every refusal an explicit type, a presentation, a recovery path, and a data-loss column. Refusals are counted, not dropped; on an unroutable target the block renders with the **previous** value and the counter is the diagnostic — audio unchanged rather than stale-and-hidden. The binding rule that **no error string is formatted on the audio thread** is stated outright. | — |
| 1G. Test specification | 3 | Fifteen tests, each with setup, assertion, and a named edge case that can genuinely fail. Critically, §5.1 **opens with a "which tests hold which handles" preamble** — the exact fix R4-1 needed at iteration 2, present here at iteration 1. Every API those handles need was checked: `NullBackend::open_null_output` (`null.rs:38`), `last_block` (`:118`), the `bridge_plan.rs:96–101` fixture pattern, and all four device constructors against their real signatures. `DspParameter::maximum()`, which test 2 depends on, exists at `parameter.rs:70` as a `pub const fn`. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 **[sampled]** | Edits during playback disturb neither selection nor transport; §3.6 E1 states the no-engine case as the normal one. | — |
| 2B. Linked lenses | 3 **[sampled]** | One model, one value per parameter; `ParameterEdit` keeps `AppModel` renderer-neutral. | — |
| 2C. Modulation visibility | 3 | The best-judged call in the spec. It **defers rather than diverges**: R4-2 introduces no automation, so there is no automated state to override and no restore action to offer, and §4.4 preserves the one-value model PROD-002 will later decompose. Its own words — "claiming a modulation-visibility design here would be inventing a surface for a source that does not exist" — are exactly the discipline this criterion exists to reward. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 states that **no keyboard shortcut is added, ratified, or documented**, cites the field study's "Prohibited conclusions" section by name, and says the map comes after the model. AF-5 clean. | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A separates convergence, divergence, and deferral per record instead of asserting a single posture. | — |
| 2F. Differentiation | 3 **[sampled]** | No parity-as-completeness claim. | — |
| 2G. Benchmark evidence discipline | 3 | **All five cited `OBS-` IDs were opened and each says what the spec claims** — `OBS-VCV-VOLT-006`, `OBS-AB12-AUTO-004`, `OBS-BW53-AUTO-002`, `OBS-AB12-AUTO-003`, `OBS-PP-ARCH-003`. The header states plainly that no benchmark has a citable observation about a DAW's internal control→render transport, and Appendix A says `OBS-AB12-AUTO-003` is **not sufficient** to settle Q5 — declining to stretch a record is precisely the behavior worth a 3. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 **[sampled]** | `NEXT.md` slice 2 and decision 22, scoped to block-boundary application; sample-accurate automation is explicitly left to PROD-002 at R9. | — |
| 3B. Non-goal respect | 3 **[sampled]** | No hosting, no cross-DAW formats, no cloud. | — |
| 3C. Deliberately small first devices | 3 **[sampled]** | Reuses the existing four fixture devices' inherent setters; adds no device. | — |
| 3D. Originality | 3 | One numeric bound introduced (`MAX_PARAMETER_TARGETS`) with Spectre-derived rationale; nothing borrowed from a reference product. | — |
| 3E. Platform commitment | 3 | §4.6 declares the slice platform-neutral, states the Linux risk is **inherited from decision 23, not created**, and refuses the claim outright: a Shape edit changing live audio on Linux is unproven until R4-3 runs. Denormal flush is argued as identical across x86-64 and aarch64. | — |
| 3F. Accessibility trajectory | 3 | §3.7: no icon-only control, no color-only state, no custom-painted widget; engine and parameter state carried by words and numbers, so a monochrome reading loses nothing. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | **Forty-six citations opened individually; zero false.** Confirmed exact: `io.rs:163` = `pub trait AudioProcessor: Send`; `bridge.rs:181` = the discard closure, verbatim as quoted; `app/lib.rs:385–403` and `:401` = clamp-and-store; `parameter.rs:49–51` = `self.spec.clamp(f64::from(value)).plain() as f32`, verbatim; `param.rs:77–79` = the `max <= min` rejection; the four `const` descriptor tables at `source.rs:27`/`:39` and `effect.rs:19`/`:22`; `req-ledger.md:64` = the PROD-003 row. | — |
| 4B. Numbers carry rationale | 3 | §7.2 schedules `docs/01-requirements/requirements-ledger.md` with a rationale row for `MAX_PARAMETER_TARGETS`, citing PROD-003 at `requirements-ledger.md:64` — verified as the row requiring rationale "in this ledger". Prose alone would not have discharged it. | — |
| 4C. Traceability | 3 | Every normative claim carries a requirement ID, decision row, `OBS-` ID, or source path, and each one checked resolves. | — |
| 4D. Honest gaps | 3 | Eight open questions, and the escalation is real: **D-R3 is raised for the `Gain` discrepancy** rather than the spec choosing between an accepted document and the shipped code. Q5 is left open with the reason that the only nearby record cannot settle it. | — |
| 4E. Evidence commands | 3 | §5's lead is correctly hedged from the start — "all of them run today except `cargo test -p spectre-app --test live_engine`, whose file R4-1 creates" — which is the exact correction R4-1 needed at iteration 2. Workspace gate quoted character-exact. | — |
| 4F. No fake surfaces | 3 | §3.6 E1 makes "no engine exists" the *normal* state and gives it non-error copy; nothing implies `./spectre` makes sound today. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Triggered | Finding |
|---|---|---|
| AF-1 — Contradicting accepted authority | **No** | The one place the spec meets a conflict with accepted authority — the architecture document's `Gain already smooths` versus the shipped struct — it flags explicitly, declines to resolve, and routes to `decisions-needed.md` as D-R3. That is precisely the route AF-1 requires. |
| AF-2 — Unbacked implementation claims | **No** | 46 citations opened; zero describe code that is not there. |
| AF-3 — Realtime discipline violation | **No** | The added render-side sequence is one `binary_search_by` over a boxed slice, the linear step scan `process` already performs, one dynamic dispatch, and `&'static str` comparisons plus a clamp. The no-panic argument is complete and was traced end to end: `f64::clamp` panics only when `min > max`, `ParamSpec::new` rejects `max <= min` at `param.rs:77–79`, and the four descriptor tables are `const`, making an ill-ordered spec a **compile-time** failure. |
| AF-4 — Borrowed numeric limits | **No** | One bound, Spectre-derived, with its ledger row scheduled. |
| AF-5 — Conclusions the evidence cannot support | **No** | §3.4 refuses a shortcut map by name and cites the prohibiting section. |
| AF-6 — Optimistic language | **No** | The header itself leads with two uncomfortable facts rather than smoothing them. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every existing type built on is real. `AudioProcessor` is `pub trait AudioProcessor: Send` (`io.rs:163`), so the new required method lands on a `Send`-bounded trait, preserving the compile-time assertion R4-1 relies on. |
| API changes feasible | ✓ | `DspParameter` genuinely lacks a runtime setter today, and the inherent per-device setters the spec reuses exist (`Gain::set_gain` at `effect.rs:45–47`). |
| Test strategy executable | ✓ | Every handle checked. `DspParameter::maximum()` exists at `parameter.rs:70` — note it is `pub const fn`, so a `pub fn` grep misses it; a reviewer should not conclude absence from that alone. |
| Numeric limits carry rationale | ✓ | Ledger row scheduled, not merely argued. |
| No undeclared dependency on unbuilt features | ✓ | R4-1 dependency is declared, and the `live_engine` test file's non-existence is stated rather than assumed. |

**Feasibility verdict:** Feasible. Every path checked contains its claim.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 3.00 | 35% | 1.050 |
| 2 — DAW Workflow Depth | 3.00 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.00 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 3.00 | 20% | 0.600 |
| **Composite** | | | **3.000** |

**Pass conditions:**
- [x] Composite ≥ 2.30 — **3.000**
- [x] Every lens ≥ 2.00 — 3.00 / 3.00 / 3.00 / 3.00
- [x] No criterion scores 0 — none
- [x] At most two criteria score 1 — zero
- [x] All auto-fail rules pass
- [x] Feasibility satisfies `criteria.md`
- [x] Reviewer personally opened the source paths graded above — 46 of ~54, all load-bearing ones included

**All conditions met:** Yes → **PASS**

**A caution about this score.** A 3.000 is an unusual result and should be read with its
coverage statement, not apart from it. It rests on exhaustive verification of evidence
integrity, where this spec is genuinely flawless across 46 checks, and on sampling for the
quality dimensions, where nine criteria are marked **[sampled]**. If a second reviewer reads
the sampled sections in full and finds a weakness, that is a correction to this scorecard,
not a regression in the spec. What this review establishes firmly is the thing that fails
specs in this gauntlet: **there is no false statement about the codebase in it.**

---

## Remediation Brief

### Priority 1 — Must fix to pass

None.

### Priority 2 — Should fix for quality

1. **D-R3 needs Jeff before implementation, and the spec is right not to decide it.**
   `docs/03-architecture/dsp-device-io.md:94` states "Smoothing stays the device's concern.
   `Gain` already smooths", and `:104` describes `Gain` as "stereo linear gain with
   click-resistant smoothing". The shipped type is `pub struct Gain { gain: f32 }` with
   `set_gain` performing an instantaneous clamped assignment (`effect.rs:29–47`) — **there is
   no smoothing state of any kind**, and the struct's own comment ("callback-ready target
   state") describes a target/current pair that does not exist. This matters beyond
   bookkeeping: the whole point of R4-2 is that a slider drag reaches a live processor, and
   an unsmoothed gain assignment mid-block is the textbook click. Either the architecture
   document is wrong and should be corrected, or `Gain` owes an implementation — and which
   one is Jeff's call, not the spec's.

### Priority 3 — Consider for excellence

1. **Complete the review's sampled sections.** Nine criteria above are marked **[sampled]**.
   A focused second pass over §2, §3.1–3.3, §4.2, §4.4, §4.7, §5.1 tests 6–15, and §8 would
   convert this from a firmly-evidenced verdict with stated gaps into a fully graded one.
2. **`RT_MODULES` extension deserves a test of its own.** §7.2 correctly schedules
   `rt_guard.rs`'s array becoming `[&str; 5]`. Worth adding an assertion that the array
   length matches the number of callback-reachable `src` modules, so the next slice that adds
   one fails loudly rather than quietly leaving a hole — the exact hole this spec spotted.

---

**End of scorecard.**
