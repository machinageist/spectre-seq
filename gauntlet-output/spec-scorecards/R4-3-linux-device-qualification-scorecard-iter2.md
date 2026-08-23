<!--
Author: Jeff
Date: 2026-08-16
Description: Blind re-verification scorecard for the R4-3 linux-device-qualification spec, iteration 2
Notes: Read in full — the spec (all 1,331 lines), gauntlet-output/criteria.md, the iteration-1
  scorecard, templates/SCORECARD-TEMPLATE.md, crates/spectre-audio/tests/lifecycle_health.rs (all
  295 lines), crates/spectre-audio/src/bridge.rs (all 291 lines). Read in part but at every cited
  line — crates/spectre-audio/src/lib.rs (:20–30, :54–110, :178–220), cpal_backend.rs (:14–24,
  :30–36, :64–72, :113–200), crates/spectre-graph/src/lib.rs (:430–450 plus a max_frames grep),
  crates/spectre-dsp/src/{source.rs:196–212, effect.rs:115–130}, spectre-audio/Cargo.toml (whole),
  .github/workflows/ci.yml (:7, :23, :27–32, :41–47), docs/06-plans/current-milestone.md (:14–18,
  :22–26, :65–69, :86–90, :108–135), docs/01-requirements/{decision-gates.md rows 1/16/19/20/23,
  requirements-ledger.md :24/:30/:32/:42/:51/:58/:64, traceability.md :15–25 and :42–55},
  docs/README.md (:39–50, :64–66), docs/status/{STATUS.md :19/:34/:37/:46–48, NEXT.md :18/:25},
  docs/02-reference-research/{logic-pro.md:8–12, synth-modular-observations.md:50/:54,
  ableton-live-observations.md:106}, gauntlet-output/manifest.md (feature table) and
  specs/R4-1-live-audio-wiring.md (grepped for the two cross-references only). All arithmetic
  recomputed independently; every occurrence of 89/90/93/94/188 grepped across the spec.
  Not run: no command was executed against hardware. §5.1 is hardware-blocked by the spec's own
  admission and this reviewer has no Linux host. §5.2/§5.3 were not executed either — nothing in
  this review depended on their execution, and no claim of passing execution is made below.
-->

# Scorecard: Linux Device Qualification

**Feature ID:** `R4-3` (`linux-device-qualification`)
**Spec file:** `gauntlet-output/specs/R4-3-linux-device-qualification.md`
**Reviewer agent:** blind verification agent, R4-3 iteration 2
**Date:** 2026-08-16
**Spec iteration reviewed:** 2 (remediation 1)

---

## Verdict: PASS

**Summary:** All three iteration-1 Priority 1 items are genuinely fixed, and I confirmed each
against source rather than against the spec's own account of itself: the `NullBackend`
attribution is now split four-and-two at the exact constructor lines, `QUAL-002` reads 89 at
every one of its seven occurrences with the constant and the generalized formula now agreeing
at the drill's geometry, and the compiled plan's 256-frame refusal path is carried through
§4.2, §3.6 E-9, §5.1 T-7, §5.5, §5.6 item 10, and §8 Q1/Q2 with `bridge.rs:166–173` cited
correctly every time. Iteration 1's load-bearing insight survived intact and was strengthened,
not over-edited: `xruns` (`:284`) and `worst_headroom` (`:285`) are still printed and asserted
nowhere, and §4.3's table now also names `frame_capacity_rejections` as measured-and-discarded.
The remaining findings are small: three uncited or imprecise claims — a comparative-likelihood
clause about ALSA in T-7, an uncited assumption about cpal's `BufferSize::Fixed` semantics, and
a `traceability.md:44` quotation that says "ends" where the sentence is mid-row — none of which
is blocking.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | Iteration 1's only gap here is closed. §4.7 now has an "RT-001 coverage" bullet naming `rt_guard.rs` and quoting `traceability.md:51` verbatim — I compared it character by character against the file: "zero violations across bridge render, frame-capacity and empty-block refusals, control drain, retire, and the plan driven through a real backend callback" is exact, as is the state string "implemented for the offline-driven callback path; a live driver is still unqualified". The spec then refuses the tempting inference, stating that `rt_guard.rs` is a separate test the hardware drill does not run, so no drill result on any platform is evidence about the guard under a live driver. `publish_headroom` (`bridge.rs:211–231`) and its `Instant::now` justification (`current-milestone.md:67`) both verified exact. | — |
| 1B. Control↔render communication | 3 | §4.2 now carries an explicit **RT-002 disposition** paragraph: no new control↔render traffic, no lane, overflow policy, or reclamation change, grounded on the drill building its channel at `lifecycle_health.rs:260` and never sending. Verified: line 260 is `let (_sender, receiver) = control_channel(&[], 64, 16).unwrap();` and `_sender` is never used anywhere in the drill body (`:248–295`). | — |
| 1C. Numerical containment | 3 | §5.1 T-9 and §3.6 E-6 still tie `contaminated_nodes == 0` (`lifecycle_health.rs:294`, verified) to RT-003, provenance `OBS-VCV-VOLT-006` confirmed at `synth-modular-observations.md:50` and `requirements-ledger.md:30`. Strengthened in iteration 2: §5.6 item 11 now states the run establishes **nothing** about denormal flushing, because the drill never reads `denormals_flushed` (`bridge.rs:93`, verified — printed nowhere in `:281–288`) and the fixture renders silence so no denormal is generated. The deterministic-half coverage it points at (`lifecycle_health.rs:238`) is real: `assert_eq!(telemetry.denormals_flushed(), 0)`. | — |
| 1D. Determinism | 3 | No live/offline equivalence is claimed, so no hash walk is owed; §5.3's "state machine is sound on this host" versus "the driver behaves" split is grounded in the file's own header (`:1–7`, quoted exactly). Iteration 1's weakness is answered head-on by §4.5's new paragraph "What the four-block allowance does and does not model," which states that stream start latency is **not** bounded by the drill's structure — verified: the drill sleeps 250 ms from `start()`'s return at `:275–276`, not from the first callback — and that a healthy run can therefore land under the floor. It refuses to widen the margin, citing PROD-003, and substitutes a procedural single unmodified re-run. | — |
| 1E. Graph and plan contract | 3 | The missing half of iteration 1's analysis is now the strongest new passage in the spec. §4.2's "The compiled plan's frame capacity, and why the record has to care" traces `FRAMES = 256` (`lifecycle_health.rs:23`) through `fixture(FRAMES)` at `:259` and `graph.compile(saturator, frames, …)` at `:54–55` to `CompiledPlan::max_frames` — which I confirmed is at `crates/spectre-graph/src/lib.rs:441`, exactly as cited — and then quotes `RenderBridge::render`'s guard in full. Verified against `bridge.rs:167–172`: `if frames == 0 || frames > self.plan.max_frames()` fills silence, increments `frame_capacity_rejections`, and returns before `apply_transport`, before `plan.process`, and before `publish_headroom` at `:207`. GRAPH-001 is explicitly unaffected and no per-block recompilation is proposed. | — |
| 1F. Failure behavior | 3 | §3.6 gains **E-9** for frame-capacity refusal, naming `frame_capacity_rejections` (`bridge.rs:73`, verified as the accessor) as the disambiguating counter and stating that it is not printed. §3.2 gains branch E; §5.5 moves T-7's outcome word from `FAIL` to `INCONCLUSIVE` and states why in the table's own footnote — "`FAIL` would assert the first cause on evidence that supports either." §5.5's four outcome definitions now cover all thirteen T-conditions with no condition unassigned (T-3 → `REFUSED`; T-4/5/6/8/9/10 → `FAIL`; T-1/2/7/11/12/13 → `INCONCLUSIVE`). | Minor (Priority 2): §5.5 states no precedence when two conditions fail at once — e.g. every block hitting the plan-error path (`bridge.rs:197–199`) yields `blocks_rendered == 0` **and** `plan_errors > 0`, which maps to `INCONCLUSIVE` (T-7) and `FAIL` (T-8) simultaneously. |
| 1G. Test specification | 3 | Every line in §5.1's T-1…T-9 table resolves to what the spec says it does; I opened all of them: `:253` `assert!(!devices.is_empty(), …)`, `:254–256`, `:265–271`, `:275`, `:277`, `:279`, `:289–292`, `:293`, `:294`. Iteration 1's two defects are fixed: T-11 now reads `blocks ≥ 89` and T-7 now names **both** causes of `blocks_rendered == 0` with the `bridge.rs:166–173` citation. §4.3's assertion count is corrected and now correct: four `assert*!` macros at `:253`, `:289`, `:293`, `:294` and six `.expect` panics at `:252`, `:256`, `:271`, `:275`, `:277`, `:279` — I counted them in the file — plus two `println!`s at `:257`, `:281`, ten conditions in total. T-13's premise checks out: the file contains exactly one `#[ignore]`, so `--ignored` yields `1 passed`. | — |

**Lens average:** 3.000 (21/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | Appendix A's structural N/A survives remediation unchanged: the feature "cannot lose selection, zoom, or transport context because it never touches them." §3.1's surface inventory still adds no `./spectre` screen, and the underlying fact is still true — `STATUS.md:37` states "nothing in `./spectre` uses them," verified this iteration. | — |
| 2B. Linked lenses | 3 | "It cannot fork state per lens because it owns no state." §4.4 confirms it: the only owned state is a Markdown record, and `BridgeTelemetry` (`bridge.rs:24–38`) already owns the runtime counters. Iteration 2 widened the construction citation to `:45–57`, which is exactly right — the `Default` body runs 45–57 and its eleven fields are 46–56. | — |
| 2C. Modulation visibility | 3 | "It cannot obscure a modulation contribution because it renders no parameter." Still true; `drain_parameters` (`bridge.rs:181`) is drained into a no-op and counted, and the drill sends nothing. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 still gives the correct AF-5 answer — "**none are defined, and none may be**" — and §3.7 still requires the outcome be carried by a word rather than colour or an emoji glyph. Untouched by remediation. | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A unchanged: no cross-product convergent pattern exists in the corpus for device qualification, so Spectre "operates where the evidence is silent, and says so." | — |
| 2F. Differentiation | 3 | Appendix A's narrow differentiator is intact and still disclaims the parity trap ("a statement about the corpus's coverage, not a claim that they do not"). | — |
| 2G. Benchmark evidence discipline | 3 | Three `OBS-` IDs, all three re-verified at the exact cited line this iteration: `OBS-VCV-VOLT-006` (`synth-modular-observations.md:50`), `OBS-SR2-CPU-001` (`:54`), `OBS-AB12-MIX-009` (`ableton-live-observations.md:106`). Iteration 1's Priority 3 item is fixed: Logic Pro's zero-record status is now sourced to the primary document, `logic-pro.md:10–11`, which I confirmed reads `Status: draft` / `Research state: inventory-only`, with the spec noting explicitly that it cites the primary rather than the grading document's inventory. | — |

**Lens average:** 3.000 (21/7)
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | §6.4 cites `current-milestone.md:88` ("Linux device qualification runs, discharging decision 23's debt") and `NEXT.md:25` (slice 3) — both verified verbatim at those lines this iteration. Every enrichment stays deferred: no drill change (§4.1), no JACK row (§4.6), no CI automation (§4.6), no headroom threshold (§4.5). Iteration 2 added no scope. | — |
| 3B. Non-goal respect | 3 | §6.4 walks the list; the feature adds no product surface, so no non-goal is reachable. Confirmed against §3.1 and §4.1's "adds **no code**", which the delta in §7.2 honours — every changed path is under `docs/`. | — |
| 3C. Deliberately small first devices | 3 | §6.4 identifies the fixture as the existing Pulse → Gain → Saturator chain at `lifecycle_health.rs:27–66` (verified: `fn fixture` spans exactly 27–66) and leaves it unchanged. §7.2's "Explicitly not changed" still bars every file under `crates/`. | — |
| 3D. Originality | 3 | Both bounds remain derived from this repository: `QUAL-002` from `lifecycle_health.rs:22–23`, `:264`, `:274–278` (all verified — `SAMPLE_RATE`/`FRAMES` at 22–23, `StreamConfig::stereo(48_000, FRAMES)` at 264, the two 250 ms cycles at 274–278) and `QUAL-003` from `bridge.rs:222` (`if headroom <= 0.0 { xruns.fetch_add(1) }`, verified). The iteration-1 arithmetic slip is corrected in the direction the derivation dictates, not toward a rounder number. | — |
| 3E. Platform commitment | 3 | Still the criterion's live risk answered head-on. Decision 23 quoted correctly from `decision-gates.md:49`; decision 1 from `:25`; §7.1's Gated/absent bucket reproduces `current-milestone.md:116` and `:127` accurately, including that a container or VM cannot satisfy the row. Iteration 2 strengthened this indirectly: §4.6's granted-geometry asymmetry means a Linux `blocks=0` is now recorded as `INCONCLUSIVE` rather than as an adverse Linux finding, which is the more honest treatment of the co-first-class commitment. | — |
| 3F. Accessibility trajectory | 3 | §3.7 unchanged and still correct: verbatim `hw:CARD=…` device keys because a future picker must display the identity the backend uses (`cpal_backend.rs:17–22`, verified — the function's own comment says cpal reports no stable device identity), and outcome carried by a word. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes
**Auto-fail triggered:** No

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | **The feasibility-rule defect is fixed and I verified the fix at source, not at the spec's word.** §7.1 now says the six deterministic tests come in **two** harness shapes: the first four construct a `NullBackend` at `:70`, `:110`, `:135`, `:167`, and the last two construct none and call `RenderBridge::render` directly on a stack `RenderBlock` at `:216–217` and `:233–234`. Every one of those eight line numbers is exact — `let backend = NullBackend::new();` is the first body line of the tests at `:68`, `:108`, `:133`, `:165`, and the tests at `:207`/`:226` go `fixture(FRAMES)` → `control_channel` → `RenderBridge::new` → `bridge.render(&mut RenderBlock::new(&mut interleaved, CHANNELS))` with no backend in the path. §5.3 carries the identical corrected description. The rest of §7.1 I re-verified line by line: drill at `:248` with attributes `:245–247`; body map `:252–253`, `:254–256`, `:257`, `:259–262`, `:264`, `:265–271`, `:274–278`, `:279`, `:281–288`, `:289–294`; file length 295 lines; `CpalBackend` at `cpal_backend.rs:34` implementing `AudioBackend` at `:68`; `open_output` `:115–151`; `BufferSize::Fixed` `:126–130`; `OpenFailed` `:144`; error callback `:138–141`; `error_count` `:163–168` absent from the `AudioStream` trait at `lib.rs:181–196`; eleven `BridgeTelemetry` counters with accessors at `:63`–`:115`; `publish_headroom` `:211–231` and the xrun at `:222`; `ci.yml:7`, `:23`, `:27–32`; and the "Absent" claim — `ls docs/` returns exactly `00-product`, `01-requirements`, `02-reference-research`, `03-architecture`, `06-plans`, `README.md`, `status`, with no `05-quality/`. The note-scratch error is fixed: §4.7 now says 64 events, not 256, citing the drill's argument at `:261` against the signature at `bridge.rs:133–139`, with `DEFAULT_NOTE_SCRATCH = 256` correctly relocated to `bridge.rs:20`. I found no false claim in §7.1. | — |
| 4B. Status vocabulary | 3 | Status `proposed`; §7.1 still partitions Implemented / Verified (macOS only) / Gated-absent / Absent, and respects the `implemented` vs. `verified` distinction — the drill is implemented, only macOS is verified. §7.2 item 3 still makes decision 23's disposition conditional on `PASS` alone. | — |
| 4C. Traceability | 2 | Citation density remains exceptional and almost everything resolves — I spot-checked roughly forty distinct citations and every one landed, including `decision-gates.md:25/:40/:43/:44/:49`, `requirements-ledger.md:24/:30/:32/:42/:51/:58/:64`, `README.md:39–48/:50/:64–66`, `STATUS.md:19/:37/:46–48`, `NEXT.md:18/:25`, `traceability.md:19/:44/:50/:51`, `Cargo.toml:12–18`, and `ci.yml:41–47`. Iteration 1's two defects are fixed: `QUAL-002` is arithmetically consistent everywhere, and §7.2 item 6's premise is restated correctly — the traceability document does already carry the platform qualifier in both places, and the edit is now described as additive. **Three residual blemishes hold this at 2.** (i) §5.1's T-7 cell asserts "§4.6 gives the geometry reason the second is the more likely of the two on ALSA" — a comparative-likelihood claim about ALSA behavior with no `OBS-`, decision, or source citation anywhere behind it. (ii) §4.6 states "cpal's `BufferSize::Fixed` is a request to the host API" with no cpal source path or version-pinned documentation reference; it is the assumption the entire E-9 "quieter half" scenario rests on, and it is the one load-bearing claim in the spec that cannot be checked in one `Read` of this repository. (iii) §7.2 item 6 says the `traceability.md:44` row "already **ends** 'Linux device qualification has not run.'" — the row does not end there; two further sentences about the aarch64 container result follow. The substance (a qualifier already exists) is true; the description of the file is not. | See Priority 2 items 1–3. |
| 4D. Honest gaps | 3 | §8's ten questions are real and each names what it blocks, and iteration 2 added a fourth known gap (d) declaring the `blocks=0` ambiguity in the header block rather than burying it. Q1 is handled exactly right: I independently recomputed the macOS discrepancy — `current-milestone.md:113` records 173 blocks; 0.5 s at 256 frames / 48 000 Hz predicts 93.75 callbacks; 173 is 1.85× that — and iteration 2 adds the structural asymmetry that a narrower grant merely raises the count while a wider one drives it to zero. That is derived, verifiable (`bridge.rs:167` refuses only `frames > max_frames`; `spectre-graph/src/lib.rs:462` applies the same bound inside `process`), and still declines to assert an explanation. | — |
| 4E. Evidence commands | 3 | §5.2's three commands match `criteria.md` §4E, `STATUS.md:46–48`, and `ci.yml:41–47` exactly; §5.1's command matches `current-milestone.md:132` and `NEXT.md:25` verbatim. The three load-bearing properties are all correct, including that a `0 tests` transcript is `INCONCLUSIVE` and never `PASS`. The §5 opening honesty statement still declares which commands run today and which cannot. | — |
| 4F. No fake surfaces | 3 | Nothing implies Linux works or that anything makes sound; §3.1's claim that `./spectre` does not use `spectre-audio` matches `STATUS.md:37`. §5.6's list grew from nine items to eleven and both additions are correct at source: item 10 (frame-capacity blind spot) and item 11 (denormal flushing unmeasured). Item 10 is carefully bounded — it concedes that a `PASS` with `blocks ≥ 89` **does** establish that many in-capacity blocks, so it does not manufacture doubt about a passing run, only about a failing one. §6.3's AF-6 self-audit is still accurate: "should pass", "expected to pass", and "straightforward" appear nowhere about the Linux outcome. | — |

**Lens average:** 2.833 (17/6)
**Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Triggered | Basis |
|---|---|---|
| AF-1 — Contradicting accepted authority | **No** | Decisions 1, 16, 19, 20, and 23 are quoted accurately against `decision-gates.md:25/:40/:43/:44/:49` and followed. Every proposal touching accepted authority — the `QUAL` family, `docs/05-quality/`, a drill modification, a JACK row — is routed to §8 as a question. Iteration 2 added no new normative assertion; the one outcome-word change (T-7 `FAIL` → `INCONCLUSIVE`) makes the spec's own proposed rule *weaker*, not an amendment to anything accepted. |
| AF-2 — Unbacked implementation claims | **No** | No code is described as existing that does not exist. The iteration-1 misdescriptions are all corrected, and every path I opened contained what the spec said. The new `frame_capacity_rejections` claims are the most checkable in the document: field at `bridge.rs:27`, default at `:48`, accessor at `:73`, increment at `:169–171`, absent from the drill's `println!` at `:281–288`. |
| AF-3 — Realtime discipline violation | **No** | No code changes; nothing is added to a callback-reachable path. Iteration 2's new analysis of the refusal branch is a reading of existing code, and it correctly notes (§4.7) that `traceability.md:51` already records RT-001 coverage of the frame-capacity refusal, so the branch is realtime-safe as well as fail-closed. |
| AF-4 — Borrowed numeric limits | **No** | `QUAL-002` = 89 derives from `lifecycle_health.rs:22–23`, `:264`, `:274–278`; `QUAL-003` from `bridge.rs:222`. **No bound was widened or invented in remediation.** §4.5 explicitly declines to widen the callback floor to absorb ALSA start latency on the ground that "any wider number would be a guess rather than arithmetic over Spectre's own constants and PROD-003 (`requirements-ledger.md:64`) forbids an unrationalized bound," and substitutes a procedural re-run instead. The correction moved 90 → 89 *toward* the derivation, and §7.2 still schedules all three rows in the ledger with their own rationale. |
| AF-5 — Conclusions the evidence does not support | **No** | Re-checked against all nine prohibited items in `workflow-field-study/product-implications.md`. No monitoring-latency threshold — §4.5 refuses a headroom floor outright and names the AF-5 adjacency itself. No supported-interface model list — §5.6 item 2 refuses generalizing past the one device opened, and iteration 2 did not add a device list anywhere. No platform/backend order — §4.6's raw-ALSA-first line restates accepted decision 20 and is routed to §8 Q4, unchanged from iteration 1. §3.4 still refuses a default shortcut map. `QUAL-003` remains a correctness condition defined by `bridge.rs:222`, not a latency target. |
| AF-6 — Optimistic language | **No** | The standing constraint, §1.2, §4.6, §5.6, §6.3, and §7.1 each refuse the Linux claim; §1.3 still states "the signal is not 'the row says PASS'"; §5.5 gives `FAIL`, `REFUSED`, and `INCONCLUSIVE` equal standing. Decision 23's bar — no Linux support claim until the run happens — is honoured, and the drill has still never been run on Linux hardware, which the spec states in its header, its standing constraint, and its §5 command-honesty statement. |
| 3B = 0 | **No** | No non-goal is proposed. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | The record schema (§3.3 columns + E1–E10) does not exceed what `BridgeTelemetry` and `StreamConfig` can supply. The no-granted-config claim is correct: `AudioStream::config()` (`lib.rs:194–195`) returns the requested `StreamConfig` that `CpalStream` stores unchanged at `cpal_backend.rs:147`. |
| API/interface changes are feasible with current architecture | ✓ | None are proposed. `error_count` is inherent to `CpalStream` (`cpal_backend.rs:163–168`), absent from the `AudioStream` trait (`lib.rs:181–196`), and `open_output` returns `Box<dyn AudioStream>` (`lib.rs:210–215`), so it is genuinely unreachable from the drill. |
| Views/screens fit current navigation pattern | ✓ | No application surface. The transcript is two `key=value` lines from `:257` and `:281–288`, exactly as described. |
| Dependencies are available and version-compatible | ✓ | `cpal = { version = "0.15.3", optional = true }` at `Cargo.toml:18` behind `default = ["cpal-backend"]` at `:14–15`. The JACK-unreachability claim holds: the manifest declares exactly two features and cpal carries no feature list. |
| Platform/renderer requirements are realistic | ✓ | `libasound2-dev` at `ci.yml:27–32`; `aplay -l`, `uname`, `/proc/asound/version`, `/etc/os-release` are stock. The CI-exclusion argument is sound — `ci.yml:47` runs `cargo test --locked --workspace`, which excludes ignored tests. |
| Test strategy is executable with current infrastructure | ✓ | §5.1 remains hardware-blocked, which the spec declares openly and is not a defect. Iteration 1's interpretive defect is gone: T-7 now names both causes of `blocks_rendered == 0` and §3.6 E-9 covers the frame-capacity refusal path. §5.2 and §5.3 are executable as written; §5.3's harness description is now accurate. |
| Performance budget is realistic for target hardware | ✓ | No render-path change. §4.7's ~1 s estimate is consistent with two 250 ms sleeps plus enumeration and open, and the note-scratch figure is now correct at 64 events. |
| No undeclared dependency on unbuilt features | ✓ | §7.4's table is accurate. Both cross-spec references check out: R4-1 §4.6 flags `BufferSize::Fixed(256)` refusal and a 44 100 Hz default as its two ALSA risks (spec lines 715–716), `stream_errors` is proposed in R4-1 (`fn stream_errors(&self) -> u64;`), and `manifest.md:44` still records R4-1 as **spec-pass at 2.950**, matching §7.4's claim. |

**Feasibility verdict:** Feasible
**Caveats:** Execution of §5.1 remains blocked on Linux hardware, which the spec declares in its
header, known gaps (a), §5's honesty statement, and §7.4 — a declared blocker, not a
misdescription. **The binding feasibility rule is now satisfied:** I opened every source path
§7.1 cites and found no statement that misdescribes the file. The one file-description
imprecision I did find is in §7.2, not §7.1, and concerns a documentation row rather than
current code state (see Priority 2 item 3).

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 3.000 | 35% | 1.050 |
| 2 — DAW Workflow Depth | 3.000 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.833 | 20% | 0.567 |
| **Composite** | | | **2.967** |

**Pass conditions (from `criteria.md`; `criteria.md` is binding):**
- [x] Composite ≥ 2.30 — **2.967**
- [x] Every lens average ≥ 2.00 — 3.000 / 3.000 / 3.000 / 2.833
- [x] No criterion scores 0
- [x] At most two criteria score 1 — **zero** criteria scored 1
- [x] All auto-fail rules pass — AF-1 through AF-6 and 3B all clear
- [x] **Feasibility rule — PASSES.** I read every source path the spec cites and confirmed the
      claim against the file. §7.1 describes current state accurately, including the two-harness
      split that failed iteration 1.
- [x] Reviewer personally executed every command claimed as passing — the spec claims no command
      was run, declares §5.1 unrunnable, and makes no execution claim of its own. Nothing required
      execution to verify; every claim was checked by reading source.

**All conditions met:** Yes → **PASS**

---

## Verification of iteration 1's three Priority 1 items

Recorded separately because a remediation review's central duty is to say whether the blocking
findings were actually discharged, at source.

1. **`NullBackend` harness claim — FIXED.** §7.1 and §5.3 now both state four-and-two. Verified
   independently: `lifecycle_health.rs:70`, `:110`, `:135`, `:167` are each `let backend =
   NullBackend::new();` as the first body line of the tests at `:68`, `:108`, `:133`, `:165`; the
   tests at `:207` and `:226` contain no `NullBackend`, no `open_null_output`, and no stream —
   they build `fixture(FRAMES)`, a `control_channel`, and a `RenderBridge`, then call
   `bridge.render(&mut RenderBlock::new(&mut interleaved, CHANNELS))` at `:217` and `:234`. The
   six behaviors §5.3 attributes to the set are unchanged and remain correct.
2. **`QUAL-002`'s value versus its derivation — FIXED, and fixed everywhere.** I grepped the
   whole spec for `89`, `90`, `93`, `94`, `188`, and `187`. The old value **90 appears exactly
   once**, in the iteration-2 header note describing what was corrected. Every operative site now
   reads 89: §4.5's headline (line 514), §4.5's derivation (line 524, `⌊93.75⌋ − 4 = 93 − 4 = 89`),
   §4.5's generalized form (line 534), §4.5's start-latency paragraph (line 541), §4.6 consequence
   (b) (line 663), §5.1 T-11 (line 783), §5.6 item 10 (line 914), and §7.2's `QUAL-002` row (lines
   1091, 1094). No site was missed. Arithmetic recomputed independently: 256 / 48 000 = 5.333 ms;
   500 / 5.333 = 93.75; ⌊93.75⌋ = 93; 93 − 4 = 89. The generalized formula
   ⌊0.5 s × granted_rate ÷ granted_frames⌋ − 4 gives ⌊0.5 × 48 000 ÷ 256⌋ − 4 = 89 at the drill's
   requested geometry, so constant and formula now agree at exactly the point where they
   previously disagreed. The added justification for flooring ("the 94th block does not fit inside
   the window") is correct: 93 whole blocks occupy 496 ms of a 500 ms window.
3. **Frame capacity — FIXED, all three sub-items.** (a) §5.1 T-7 now reads "**Two causes, and the
   transcript cannot separate them**" and names both, with the outcome word moved to
   `INCONCLUSIVE` and the change explained in §5.5's footnote. (b) §3.6 gains E-9, which names
   `frame_capacity_rejections` (`bridge.rs:73`) as the disambiguating counter and states that the
   drill does not print it — verified: `:281–288` prints `blocks_rendered`, `xruns`,
   `worst_headroom`, `plan_errors`, `contaminated_nodes` and nothing else. §5.6 item 10 carries
   the same limitation into the authorization list. (c) §4.6 wires it to the granted-versus-
   requested ALSA risk in the "quieter half" passage, and adds a genuinely new and correct
   observation: `worst_headroom` printing `inf` marks the surrounding zeroes as untouched
   defaults, because `BridgeTelemetry::default` seeds it at `f32::INFINITY` (`bridge.rs:55–56`)
   and `publish_headroom` (`:207`) is unreachable when every block is refused. §5.4 manual check 5
   operationalizes that. §8 Q1 and Q2 both carry the coupling, and Q2 proposes the cheap partial
   fix (print the counter; no seam change needed) with §4.1's macOS-re-run constraint still
   applied to it.

**Did iteration 1's strengths survive?** Yes. §4.3's asserted-versus-printed table is intact and
larger: `xruns` is still "**Threshold in the protocol, unasserted in the drill**" and
`worst_headroom` still "**Observation only. No threshold.**" — verified at source, the drill's
only assertions are `blocks_rendered > 0` (`:289–292`), `plan_errors == 0` (`:293`), and
`contaminated_nodes == 0` (`:294`), so a run can print `xruns=41` and libtest reports `ok`.
Remediation added two rows (the `open_output` `.expect`, which iteration 1 asked for, and
`frame_capacity_rejections` as "measured and discarded") and corrected the count sentence. The
insight was strengthened, not diluted. §4.4 rule 3's silence argument is likewise intact and
still correct at `source.rs:202–209` (`else { 0.0 }`) and `effect.rs:119–128` (dry 0 → wet 0 →
output 0).

---

## Remediation Brief

The spec passes; nothing below blocks it. These are recorded so the durable protocol document
(`docs/05-quality/device-qualification-protocol.md`) inherits the corrections rather than the
defects.

### Priority 1 — Must fix to pass

None. All three iteration-1 Priority 1 items are discharged and re-verified at source.

### Priority 2 — Should fix for quality

1. **§5.1 T-7's likelihood clause is uncited.** The cell ends "§4.6 gives the geometry reason
   the second is the more likely of the two on ALSA." Nothing in the repository, the decision
   record, or the `OBS-` corpus supports a claim about which of the two causes is more likely on
   ALSA, and the spec is otherwise scrupulous about exactly this. The outcome word is
   `INCONCLUSIVE` either way, so the clause carries no weight — delete it, or restate it as what
   §4.6 actually establishes: that the granted geometry is unrecorded and that a wider grant is a
   named, structurally-possible cause.
2. **§4.6's cpal assumption should be labelled as one.** "cpal's `BufferSize::Fixed` is a request
   to the host API, and nothing in `cpal_backend.rs` re-reads what was granted" — the second half
   is verified true (`cpal_backend.rs:147`; `AudioStream::config()` at `lib.rs:194–195`), but the
   first half is an uncited claim about a third-party crate at a pinned version, and it is the
   assumption the entire "quieter half" scenario rests on. Either cite cpal 0.15.3's own
   documentation or source for `BufferSize::Fixed` semantics on the ALSA host, or mark it
   explicitly as an unverified assumption about the dependency and route it to §8. It would be
   the only in-spec claim a reader cannot check from this repository, which is worth flagging in
   a document whose subject is checkability.
3. **§7.2 item 6 misquotes the shape of `traceability.md:44`.** It says the row "already **ends**
   'Linux device qualification has not run.'" The row does not end there — that sentence sits
   mid-cell and is followed by the aarch64 container build result and the fail-closed
   confirmation. The substantive claim (the platform qualifier already exists, so the edit is
   additive) is correct and I verified both `:19` and `:44` carry it. Change "ends" to "states",
   or quote the cell's actual tail.
4. **§5.5 states no precedence for compound failures.** If every block took the plan-error path
   (`bridge.rs:197–199`), the run yields `blocks_rendered == 0` **and** `plan_errors > 0` — T-7
   maps to `INCONCLUSIVE` and T-8 to `FAIL`, and the table says nothing about which word wins.
   One sentence fixes it: `FAIL` outranks `REFUSED` outranks `INCONCLUSIVE`, and any run meeting a
   `FAIL` condition is recorded `FAIL` regardless of what else is unmet. This matters because the
   outcome vocabulary *is* the deliverable.
5. **`QUAL-002` has no escape hatch from a permanent `INCONCLUSIVE`.** §4.5 correctly refuses to
   widen the floor for ALSA start latency and prescribes one unmodified re-run. But if a healthy
   host systematically lands two blocks short, the protocol yields `INCONCLUSIVE` forever, and
   decision 23's debt cannot be discharged on that machine. That consequence is not routed to §8
   as a question for Jeff, though every other unresolved judgement in the spec is. Add it to Q5
   or open a Q11.

### Priority 3 — Consider for excellence

1. **`≈ 188` contradicts the spec's own flooring convention.** §4.6 and §8 Q1 say a 128-frame
   grant "would predict (≈ 188)"; under the rule §4.5 just established — only whole callbacks
   count — 0.5 s at 128 frames / 48 000 Hz predicts ⌊187.5⌋ = 187. The `≈` makes it defensible,
   but the document is now precise about flooring everywhere else.
2. **The iteration-2 header note says the constant was "corrected to 89 in all four places."** It
   now appears at eight operative sites, because remediation added references in §4.6, §5.6, and
   §4.5's new paragraph. The note undercounts its own thoroughness; say "at every occurrence."
3. **§5.6 item 11's parenthetical could be tightened.** It says the fixture "renders silence in
   any case (§4.4 rule 3), so no denormal is generated to flush" — true for this fixture, and the
   deterministic-half assertion at `lifecycle_health.rs:238` is `denormals_flushed() == 0`, which
   is consistent. Worth one clause noting that the counter therefore proves nothing in either
   direction on any platform, rather than only on Linux.
4. **§4.3's "This is the most important table in the spec"** is an editorial judgement in a
   document that otherwise states facts and cites them. It is accurate, and iteration 1 agreed —
   but the durable protocol document will be read without this framing, so lead with the table
   itself rather than with the claim about it.

---

**End of scorecard.**
