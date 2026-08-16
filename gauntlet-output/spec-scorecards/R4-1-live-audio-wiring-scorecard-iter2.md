<!--
Author: Jeff
Date: 2026-08-15
Description: Blind re-verification scorecard for the R4-1 live-audio-wiring spec, iteration 2
Notes: Reviewer had no sight of the remediation work; the spec was read cold at iteration 2.
  Every citation the remediation introduced or changed was opened against source, plus a
  regression sample of iteration 1's already-verified set. Scope of re-verification is
  stated explicitly below rather than implied.
-->

# Scorecard: Live Audio Wiring

**Feature ID:** `R4-1` (`live-audio-wiring`)
**Spec file:** `gauntlet-output/specs/R4-1-live-audio-wiring.md`
**Reviewer agent:** blind re-verification, R4-1 iteration 2 (fresh context; did not author or observe remediation 1)
**Date:** 2026-08-15
**Spec iteration reviewed:** 2 (remediation 1)
**Prior scorecard:** `R4-1-live-audio-wiring-scorecard.md` (iteration 1: composite 2.750, FAIL on AF-2)

---

## Verdict: PASS

**Summary:** All four Priority 1 defects are fixed, and each fix was checked against
source rather than accepted on the spec's word. The false RT-scan claim is gone and the
RT-001 argument has been rebuilt on the *content* of the `null.rs` edit with a post-edit
`rt_guard` run made a required gate — a stronger argument than the untouched-set one it
replaces. The invalid note-ordering guarantee is replaced by the counted fail-closed
behavior actually present in the shipped code, pinned by a new test 14 and routed to §8
Q9. Tests 3 and 9 now compile against the declared API. The one remaining gap is that
§4.4's binding rule — the spec's own "one genuinely new correctness hazard" — has no
automated coverage, because it lives in `main.rs`, which the spec correctly identifies as
unreachable from tests. That is recorded as Priority 2, not a blocker.

**Scope of this re-verification, stated honestly.** Every citation the remediation
introduced or altered was opened individually: §4.1's scan and forbidden lists, §4.2's new
`EngineParts` field and generic `LiveEngine`, §4.3's two trait methods and the entire
note-ordering paragraph, §4.4's binding rule and state table, §4.6's `Send` resolution,
§5.1 tests 3/5/7/9/14, §5.2's gate reasoning, and §7.2's modified-file list — 31 source
locations across nine files, plus the external `eframe 0.32.3` claim. Iteration 1's ~70
already-verified citations were sampled rather than re-read in full: seven `OBS-` IDs,
the `main.rs` UI literals, the FNV constants, and the offline/bridge fixture path were
re-checked for regression and all hold. This is a delta review with a regression sample,
not a second full audit, and the pass rests on that.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | Fixed and improved. §4.1 now reads the scan verbatim: `RT_MODULES` at `rt_guard.rs:293–298`, four entries at `:294–297` — `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs` — confirmed character-exact against the file, with `FORBIDDEN` at `:299–307` (seven needles at `:300–306`) and the substring assert at `:313–318`, all confirmed. Critically, the spec no longer argues from an untouched set: it states outright that its own table modifies `null.rs`, then argues from what that edit contains (two bodies returning `48_000` and `0`), and §5.2 makes the post-edit `rt_guard` run the evidence rather than the paragraph. The closure claim is now correctly hedged — `lifecycle_health.rs:269` reads `Box::new(move \|mut block: RenderBlock\| bridge.render(&mut block)),` and the spec says so and calls the identity behavioral, not textual. | — |
| 1B. Control↔render communication | 3 | Both iteration-1 gaps closed. §3.2 step 6 now gates on "the transport send returned `Ok`" and defers to §4.4; §4.4's binding rule enumerates all three outcomes with a reason for each, including the non-obvious correct one — `Err(Note(..))` flips the UI anyway, because a queued transport command cannot be recalled (`spsc.rs` has no un-push) and the honest UI is the one matching what the render thread will see. `AuditionError` carries the `Transport`/`Note` split that makes the rule expressible. Lane depths verify at `control.rs:17–19` (1,024 / 64 / 32); `reclaim` is drained per frame. | — |
| 1C. Numerical containment | 3 | Unchanged and still correct: R4-1 adds no DSP node, so RT-003 applies through the existing plan. `contaminated_nodes` is surfaced in `EngineHealth` and asserted zero in test 10. | — |
| 1D. Determinism | 3 | Strengthened. Test 5 reuses the existing FNV-1a walk verbatim (offset basis `0xcbf2_9ce4_8422_2325`, prime `0x0000_0100_0000_01b3` — both confirmed at `bridge_plan.rs:81/87`) and drives events through `ControlSender::send_note(NoteEvent)` (`control.rs`), which takes a whole event, so `fixture_events(256)`'s real frame offsets (`On` at 0, `Off` at 255 — confirmed at `offline/src/lib.rs:178–200`) survive the trip. This is exactly the path `bridge_plan.rs:102–103` already uses. No second comparison method is introduced. Test 6 pins repeatability. | — |
| 1E. Graph and plan contract | 3 | `plan_max_frames` is carried on `EngineParts` from the `max_frames` argument passed to `EditableGraph::compile` (`graph/lib.rs:196–201`, confirmed), rather than by adding an accessor to `RenderBridge`. The split stays intact and no per-edit recompilation is proposed. | — |
| 1F. Failure behavior | 3 | The ordering hazard is now handled the way the criteria ask: explicit, fail-closed, counted, surfaced. §4.3 traces the refusal through real code — `PlanError::Process` at `graph/lib.rs:487–492`, exact silence and `plan_errors` increment at `bridge.rs:192–200`, scratch cleared at `:258`, transport applied before notes at `:175–176` — all confirmed line-for-line, and correctly concludes no note is left stuck. §3.6 E10 and the §3.3 counter cluster surface it. | See Priority 3 item 2 for one over-broad "harmless". |
| 1G. Test specification | **2** | Fourteen tests; tests 3 and 9 are now executable and test 14 pins the ordering behavior with three assertions that each fail on a distinct regression. **But §4.4's binding rule has no automated coverage.** The rule lives in `main.rs`'s `toggle_play`, which §4.1 itself establishes is unreachable from `crates/spectre-app/tests/`. Test 7 gestures at it — "assert `AppModel::is_playing()` is unchanged" — but a `live_engine.rs` test never wires the engine result into a model, so that half of the assertion cannot fail. §5.3 is honest that no GUI harness exists and defers to R4-9, and §5.4's Honesty-check row covers it manually, so this is a stated limit rather than a hidden one. | See Priority 2 item 1. |

**Lens average:** 2.857
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, zero 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | Unchanged from iteration 1. Engine events never disturb selection, zoom, or lens; test 8 asserts it. | — |
| 2B. Linked lenses | 3 | `AppModel` stays the single model; `LiveEngine` is owned by the `eframe::App` implementor as an `Option`, not forked per lens (§4.4, confirmed `AppModel` at `app/lib.rs:205`). | — |
| 2C. Modulation visibility | 3 | §5.4's Honesty check requires a Shape slider to *not* change sound and the copy to say why, rather than implying a live parameter path R4-2 has not built. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 still refuses a default shortcut map and cites the prohibiting section. | — |
| 2E. Convergent-pattern grounding | 3 | Unchanged. | — |
| 2F. Differentiation | 3 | Unchanged; no parity-as-completeness claim. | — |
| 2G. Benchmark evidence discipline | 3 | Re-sampled and holds. All seven cited IDs resolve in the corpus: `OBS-AB12-MIX-002`, `-MIX-009`, `-ROUTE-001`, `OBS-PP-ARCH-001`, `OBS-VCV-VOLT-006`, `OBS-SR2-CPU-001`, `OBS-SR2-KB-001`. The Serum 2 pair is used only within its two-record limit. The dangling `§2G` pointer is fixed — the header now says "Appendix A ('Named gaps') and §8", and Appendix A exists with that heading. | — |

**Lens average:** 3.00
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | Still exactly `NEXT.md` slice 1; no new render path, no DSP. | — |
| 3B. Non-goal respect | 3 | No hosting, no cross-DAW formats, no cloud. §4.4 explicitly refuses to persist engine state and routes device memory to R4-7. | — |
| 3C. Deliberately small first devices | 3 | The audition voice is the existing fixture Pulse; §7.4 states R4-5 replaces it rather than accumulating alongside it. | — |
| 3D. Originality | 3 | Both constants keep Spectre-derived rationale, and the ledger rows PROD-003 requires are now scheduled — `docs/01-requirements/requirements-ledger.md` appears in §7.2's modified files with a row each for `ENGINE_BUFFER_FRAMES` and `ENGINE_PLAN_FRAME_MARGIN`, and the entry names the PROD-003 line (`requirements-ledger.md:64`) that compels it. | — |
| 3E. Platform commitment | 3 | §4.6 is stronger than iteration 1's: it names the two concrete ALSA risks (`BufferSize::Fixed(256)` refusal; a 44 100 default) and ties the second to why `default_sample_rate` exists at all, rather than treating Linux as a footnote. No Linux claim is made. | — |
| 3F. Accessibility trajectory | 3 | No surface forecloses keyboard completeness; §5.4 checks text-size and screen-size extremes against the real 62 px / 1060 px constraints. | — |

**Lens average:** 3.00
**Lens pass:** Yes

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | The false claim is gone and the four imprecise ranges are fixed. Re-verified: `null.rs` row now `:18–176` with per-item anchors, every one exact (`NullBackend` `:18–19`, `open_null_output` `:38`, `NullStream` `:80`, `pump` `:104`, `last_block` `:118`; the file is 176 lines). `lifecycle_health.rs` is now split correctly into the deterministic half `:68–240` and the `#[ignore]`d hardware drill `:245–295`, with `#[ignore]`/`#[cfg]` at `:246–247` — exact. `RenderBridge.transport` is now cited at `bridge.rs:127`, which is the field. Regression sample on `main.rs` holds: `ENGINE OFFLINE` `:79`, `CPU —` `:80`, `001 · 01 · 000` `:77`, Record button `:72–73`, Build footer string `:330`. | See Priority 3 item 1. |
| 4B. Status vocabulary | 3 | §7.2 states the slice moves status to `implemented`, **not** `verified`, until the manual protocol and hardware re-run pass. That is the distinction `docs/README.md` draws, applied correctly. | — |
| 4C. Traceability | 3 | All three iteration-1 defects fixed: the `§2G` pointer resolves to Appendix A, the requirements-ledger rows are scheduled, and the ordering-key tuple is now cited to `io.rs:96` — confirmed as `let order = (event.frame_offset, event.kind.rank(), event.sequence);`, with `UnsortedEvents` at `:97–99` and `rank` at `:152–157`, both exact. | — |
| 4D. Honest gaps | 3 | Improved. The spec now says outright "This spec makes no claim that the app cannot construct an unsorted batch, because it can", names the 5.33 ms exposure window, and routes the fix to §8 Q9 instead of asserting it. Retracting its own prior guarantee in the text is the behavior this criterion is meant to reward. | — |
| 4E. Evidence commands | 3 | §5's lead is corrected: "All of them run today except `cargo test -p spectre-app --test live_engine`, whose test file this slice creates (§7.2)". The workspace gate remains character-exact. | — |
| 4F. No fake surfaces | 3 | Test 9 exists specifically so `Opened` cannot be reported as `Running` before a callback lands, and §5.4 requires all four resting states be produced by hand. | — |

**Lens average:** 3.00
**Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Triggered | One-line finding |
|---|---|---|
| AF-1 — Contradicting accepted authority | No | No accepted row is amended. The one behavior change the ordering analysis suggests (stamping a nonzero `frame_offset` on the second event of a same-block pair) is routed to §8 Q9 rather than adopted. |
| AF-2 — Unbacked implementation claims | **No — cleared** | The iteration-1 trigger is fixed at both sites. §4.1 and §7.2 now state the scanned set as `bridge.rs`, `control.rs`, `spsc.rs`, `null.rs`, matching `rt_guard.rs:294–297` exactly, and both say plainly that `null.rs` **is** modified by this slice. Every newly-introduced API claim was checked: `ControlSender::send_note(NoteEvent)`, `RenderBridge`'s four public methods at `bridge.rs:133/152/157/162`, `AudioStream` object-safety at `audio/lib.rs:181–196`, `DeviceParameterSnapshotError` at `app/lib.rs:175`, `AppModel::prototype`/`device_parameter_snapshot` at `:218`/`:348`. All present as described. |
| AF-3 — Realtime discipline violation | No | Nothing new on a callback-reachable path. The two added trait methods return a constant and a relaxed atomic load on the app thread; the generic parameter on `LiveEngine` is a type-level change with no runtime effect. |
| AF-4 — Borrowed numeric limits | No | Unchanged, and the ledger rows are now scheduled, closing the loop iteration 1 flagged under 4C. |
| AF-5 — Conclusions the evidence does not support | No | Unchanged; no prohibited conclusion appears. |
| AF-6 — Optimistic language | No | The remediation *added* pessimism where it was owed: the ordering guarantee is retracted in the spec's own text, and the header Notes name the retraction. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Re-confirmed for everything the remediation touches. `EngineParts.plan_max_frames` is a new field on a new type, so it needs no existing API. `LiveEngine<S: ?Sized = dyn AudioStream>` is valid Rust: the default type parameter satisfies the struct's own `?Sized` bound, `Box<S>` is `Sized` for unsized `S`, and the `AudioStream` bound sits on the `impl` rather than the struct. |
| API/interface changes are feasible | ✓ | `AudioStream` is object-safe as the spec claims — every method takes `&self`/`&mut self`, none is generic, none returns `Self` (`audio/lib.rs:181–196`) — so `Box<NullStream> → Box<dyn AudioStream>` coerces and `open_default` can return the erased `LiveEngine`. Both seam additions remain genuinely required: `DeviceInfo` (`:47–53`) still carries no format, and `error_count` is still inherent to `CpalStream` (`cpal_backend.rs:163–168`). |
| Views/screens fit current navigation | ✓ | No change from iteration 1. |
| Dependencies available and compatible | ✓ | The `Send` question is now **resolved rather than deferred**, and resolved correctly: `eframe 0.32.3`'s `pub trait App` carries no `Send` supertrait (`epi.rs:137` — exact) and `AppCreator` boxes `Box<dyn 'app + App>` with no `Send` bound, so a non-`Send` `LiveEngine` can be an ordinary field. Verified in the vendored crate source, not assumed. |
| Platform requirements realistic | ✓ | `OUTPUT_CHANNELS: u16 = 2` at `audio/lib.rs:22` and the stereo bus at `graph/lib.rs:12` both confirmed. |
| **Test strategy executable** | ✓ | **Both iteration-1 blockers are cleared.** Test 3 now asserts against `parts.plan_max_frames`, adding no accessor to `bridge.rs`. Test 9 holds `LiveEngine<NullStream>` built by `from_open_stream` and pumps through `stream_mut()`, which is precisely why the generic exists — and §5.1 opens with an explicit table of which tests hold a `LiveEngine` and which hold bare parts, answering the question iteration 1 raised. `rt_guard.rs:258–287` and `:289–320` are both cited correctly in §5.2. |
| Performance budget realistic | ✓ | Unchanged from iteration 1's recomputation; the margin's 6,144 B is consistent with the new `plan_max_frames` framing. |
| No undeclared dependency on unbuilt features | ✓ | Unchanged. |

**Feasibility verdict:** Feasible. Every source path checked in this pass contains what
the spec says it contains, and all fourteen tests compile against the API the spec
declares.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.857 | 35% | 1.000 |
| 2 — DAW Workflow Depth | 3.000 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 3.000 | 20% | 0.600 |
| **Composite** | | | **2.950** |

**Pass conditions (from `criteria.md`, which is binding):**
- [x] Composite ≥ 2.30 — **2.950** (iteration 1: 2.750)
- [x] Every lens average ≥ 2.00 — 2.857 / 3.000 / 3.000 / 3.000
- [x] No criterion scores 0 — none
- [x] At most two criteria score 1 — zero
- [x] All auto-fail rules pass — **AF-2 cleared**
- [x] Feasibility satisfies `criteria.md` — every checked path contains its claim; all fourteen tests compile against the declared API
- [x] Reviewer personally opened the source paths graded above

**All conditions met:** Yes → **PASS**

---

## Remediation Brief

### Priority 1 — Must fix to pass

None. All four iteration-1 Priority 1 items are fixed and verified against source.

### Priority 2 — Should fix for quality

1. **Give §4.4's binding rule a test that can fail, or say plainly that it cannot have one until R4-9.** The rule is the spec's own "one genuinely new correctness hazard", and no automated test exercises it: it lives in `main.rs`'s `toggle_play`, which §4.1 correctly establishes is unreachable from `crates/spectre-app/tests/`. Test 7's "assert `AppModel::is_playing()` is unchanged" cannot fail in a `live_engine.rs` test, because nothing there wires the engine result into a model. Two acceptable fixes: (a) extract the rule as a pure function in `engine.rs` — for example `fn apply_transport_result(model: &mut AppModel, result: Result<(), AuditionError>)` — and test all three outcomes against a real `AppModel`, which also removes the duplicated decision from `main.rs`; or (b) drop the model half of test 7's assertion, keep the `Err` variant assertion, and add a row to §5.4 stating that the binding rule is manual-only until R4-9 builds a UI harness. (a) is preferable: it converts a manual check into a regression gate at the cost of one small function.

### Priority 3 — Consider for excellence

1. **Two remaining one-line citation offsets, both outside §7.1.** §4.6 cites `AppCreator` boxing at `epi.rs:48–49`; `pub type AppCreator<'app> =` is at line 49 and the `Box<…>` line at 50. (`pub trait App` at `:137` is exact.) §4.6 also says `CpalStream` "is documented thread-affine at `cpal_backend.rs:154`"; line 154 is the struct declaration and line 153 is the documenting comment. Neither changes a claim.
2. **Narrow §4.4's "harmless" to the case it actually covers.** For `stop_audition`, `Err(Transport(..))` means the release may already be queued while Stop was refused. The spec justifies this as "a release with no matching attack is a no-op at the instrument (`source.rs:194`)" — the citation is right and the arm does clear `active_note`/`velocity`, so it is a no-op *when nothing is sounding*. When a note **is** sounding, the release is not a no-op: the voice goes silent while the UI correctly continues to read "playing". The better justification is reachability — the transport lane holds 64 (`control.rs:18`), so refusing a Stop implies the render thread has stopped draining, which implies no audio is being produced anyway. Swapping the reason costs one sentence and makes the claim true in both sub-cases.
3. **§8 Q9 is now the right home for the ordering question, and it should carry the cost.** The spec routes the `frame_offset` stamping change to Q9 without pricing it. One line — that the fix touches only `LiveEngine::send_note`'s stamping and no realtime code — would let Jeff decide Q9 without reopening §4.3.

---

**End of scorecard.**
