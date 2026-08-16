<!--
Author: Jeff
Date: 2026-08-15
Description: Blind verification scorecard for the R4-3 linux-device-qualification spec, iteration 1
Notes: Read in full and checked line by line — crates/spectre-audio/tests/lifecycle_health.rs (all
  295 lines), crates/spectre-audio/src/{cpal_backend.rs, bridge.rs, lib.rs}, spectre-audio/Cargo.toml,
  .github/workflows/ci.yml, docs/README.md, docs/status/{STATUS.md, NEXT.md},
  docs/06-plans/current-milestone.md, docs/01-requirements/{decision-gates.md, requirements-ledger.md},
  gauntlet-output/criteria.md, gauntlet-output/templates/SCORECARD-TEMPLATE.md. Read in part —
  crates/spectre-dsp/src/source.rs:180–215 and effect.rs:105–133 (the silence argument),
  docs/01-requirements/traceability.md (RT rows and known gaps), the three cited OBS records grepped
  to their exact lines, docs/02-reference-research/logic-pro.md header, the AF-5 prohibited-conclusions
  block in workflow-field-study/product-implications.md, gauntlet-output/specs/R4-1-live-audio-wiring.md
  (§4.3 and §4.6 only, to check R4-3's cross-references) and both R4-1 scorecards for calibration.
  QUAL-002's arithmetic was recomputed independently, as was the macOS 173-block discrepancy.
  Not run: no command was executed against hardware; §5.1 is hardware-blocked by the spec's own
  admission and this reviewer has no Linux host either.
-->

# Scorecard: Linux Device Qualification

**Feature ID:** `R4-3` (`linux-device-qualification`)
**Spec file:** `gauntlet-output/specs/R4-3-linux-device-qualification.md`
**Reviewer agent:** blind verification agent, R4-3
**Date:** 2026-08-15
**Spec iteration reviewed:** 1

---

## Verdict: FAIL

**Summary:** The strongest quality is §4.3's asserted-versus-printed table, which is the
correct and load-bearing insight of the whole feature and which I verified line by line:
`xruns` and `worst_headroom` really are printed at `lifecycle_health.rs:284–285` and
asserted nowhere, so a run can emit `xruns=41` and libtest still reports `ok` — the spec
says exactly that and builds its outcome vocabulary on it. The most critical gap is that
the spec, having correctly identified that the granted buffer geometry may differ from the
requested one (§4.6, §8 Q1), never accounts for `RenderBridge::render`'s frame-capacity
refusal at `bridge.rs:167–173`: the compiled plan is built at 256 frames, any callback
larger than that is silently filled with silence and counted into an unprinted counter
without incrementing `blocks_rendered`, so §5.1's T-7 row assigns the wrong meaning
("the driver never called back") to the single most likely ALSA failure the spec itself
predicts. It fails on the feasibility rule for a §7.1 sentence that misdescribes the test
file, and carries a normative bound (`QUAL-002` = 90) that contradicts its own arithmetic
(⌊93.75⌋ − 4 = 89). All four numeric thresholds are cleared; the fix is three bounded edits.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 2 | §4.7 correctly states the render path is unchanged and that `publish_headroom` (`bridge.rs:211–231`) uses `Instant::now`, callback-safe via the vDSO/commpage per `current-milestone.md:67` — both verified exact. §3.6 E-8 correctly describes the cpal error callback as counting into an atomic and nothing else (`cpal_backend.rs:138–141`, verified). But the spec never names the RT-001 guard that covers the drill's callback path (`rt_guard.rs`, recorded at `traceability.md:51`), which criterion 1A asks for by name. | Name the RT-001 guard covering the bridge-render path in §4.7, citing `traceability.md:51`. |
| 1B. Control↔render communication | 2 | §4.4 rule 3 correctly reports that the drill builds its control channel at `lifecycle_health.rs:260` and never sends a note — verified: `_sender` is bound and never used. That is the right observation and it is exactly right. The spec never states the lens-1B disposition explicitly (no new control traffic crosses the boundary, so no lane, overflow policy, or reclamation changes). | Add one sentence to §4.2 or §4.4 stating that no new RT-002 traffic is introduced and the existing lanes are untouched. |
| 1C. Numerical containment | 3 | §5.1 T-9 and §3.6 E-6 tie `contaminated_nodes == 0` (`lifecycle_health.rs:294`, verified) to RT-003 and correctly read a nonzero value as a defect rather than an environment quirk. Appendix A traces RT-003 to `OBS-VCV-VOLT-006`, which I confirmed at `synth-modular-observations.md:50` and at `requirements-ledger.md:30`. No new DSP node exists, so the injection-test clause is structurally inapplicable. | — |
| 1D. Determinism | 2 | The spec claims no live/offline equivalence and needs no hash walk; §5.3's split between "the state machine is sound on this host" and "the driver behaves" is the right analogue and is grounded in the file's own header comment (`lifecycle_health.rs:4–7`, quoted accurately). Weakness: §4.5 treats the callback count as bounded by structure ("exactly four transition points at which a partial block can be lost"), but driver start/stop latency is not bounded by the drill's structure, so a healthy ALSA run could land below the floor for reasons the spec's derivation does not model. | §4.5 should state that block count is timing-dependent and that stream start latency, not just partial-block truncation, contributes to the shortfall. |
| 1E. Graph and plan contract | 2 | §4.1 and §4.2 correctly state that no Rust type and no plan behavior changes, and no per-change recompilation is proposed — GRAPH-001 is respected. But the plan's frame capacity is never mentioned: `fixture(FRAMES)` at `lifecycle_health.rs:259` compiles the plan at 256 frames, and `bridge.rs:167` refuses any block outside `self.plan.max_frames()`. For a spec whose §4.6 and §8 Q1/Q2 turn entirely on granted-versus-requested geometry, that coupling is the missing half of the analysis. | §4.2 must state that the compiled plan is built at `FRAMES = 256` and that a granted buffer above that is refused into silence by `bridge.rs:167–173`. |
| 1F. Failure behavior | 2 | Genuinely strong: branches A–D in §3.2, eight error states in §3.6, and §4.4's binding rules 1–2 (record every outcome; a retry is a new row) are precisely the fail-closed, counted-not-dropped discipline the criterion asks for. §3.6 E-7 correctly names the unasserted-`xruns` case as "the highest-risk error state in the feature, because the tooling reports success." Gap: there is no error state for "callbacks occurred but every block was refused for frame capacity," which is `bridge.rs:167–173`'s own fail-closed path and produces `blocks=0` with a healthy driver. | Add an error state E-9 for frame-capacity refusal, and note that `frame_capacity_rejections` (`bridge.rs:73`) is the disambiguating counter and is not printed. |
| 1G. Test specification | 2 | Every assertion named in §5.1's T-1…T-9 table resolves to the exact line I opened: `:253`, `:254–256`, `:265–271`, `:275`, `:277`, `:279`, `:289–292`, `:293`, `:294`. The commands in §5.1–§5.3 are real and correctly described, including that `--ignored` runs only ignored tests and that `--nocapture` is mandatory because the five telemetry values reach the operator only through `println!` at `:281–288`. The spec is candid that T-10…T-13 are operator-evaluated and cannot be automated without modifying the drill, and says so in criterion 1G's own terms. Two defects: T-11's threshold (90) contradicts its own derivation (89, see Lens 4 / Priority 1), and T-7's stated meaning of an assertion failure is wrong (see 1E/1F). | Fix T-11's number and T-7's failure meaning. |

**Lens average:** 2.143 (15/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | Appendix A's closing paragraph declines this on a structural ground, not a convenient one: the feature "cannot lose selection, zoom, or transport context because it never touches them." §3.1 backs it — no `./spectre` screen, panel, dialog, or menu is added, and `./spectre` does not use `spectre-audio` at all (verified at `STATUS.md:37`). The N/A carries a real reason and a positive obligation rather than an empty section. | — |
| 2B. Linked lenses | 3 | Same paragraph: "it cannot fork state per lens because it owns no state." §4.4 confirms it — the only owned state is a Markdown record in an accepted document, and `BridgeTelemetry` (`bridge.rs:24–38`, eleven counters, verified) already owns the runtime counters. Nothing is forked. | — |
| 2C. Modulation visibility | 3 | "It cannot obscure a modulation contribution because it renders no parameter." Correct: §4.2 adds no type, and the drill never applies a parameter — `drain_parameters` at `bridge.rs:181` is drained into a no-op and counted, which the spec does not need to restate. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 gives the exactly-correct answer to the AF-5 interaction: "**none are defined, and none may be**," citing the prohibition and noting there is no application surface that could carry one. §3.7 additionally requires the outcome be carried by a word rather than colour or an emoji glyph, which is the calm-UI value applied to the only surface that exists. | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A: "none apply. Device qualification has no cross-product convergent pattern in the corpus, because the corpus contains no instance of it. Spectre does not diverge from a pattern here; it operates where the evidence is silent, and says so." That is the disposition criteria.md prescribes for a silent corpus. | — |
| 2F. Differentiation | 3 | Appendix A names a narrow differentiator (publishing which numbers were pass conditions, which were bystanders, and an enumerated list of what a PASS does not authorize) and immediately disclaims the parity trap: "a statement about the corpus's coverage, not a claim that they do not." Ties to the vision release bar rather than to feature parity. | — |
| 2G. Benchmark evidence discipline | 3 | Three `OBS-` IDs cited, all three verified by me at the exact cited line and all three saying what the spec claims: `OBS-VCV-VOLT-006` (`synth-modular-observations.md:50`, 0-on-NaN/infinity), `OBS-AB12-MIX-009` (`ableton-live-observations.md:106`, per-track six-step CPU meter for freeze candidates), `OBS-SR2-CPU-001` (`synth-modular-observations.md:54`, unison CPU guidance). Serum 2 is held to its two records by name; Logic Pro's zero-record state is stated with no Logic Pro claim anywhere (I confirmed `logic-pro.md` is `draft` / `inventory-only`). The absence of any qualification-protocol record across all five benchmarks is named as a gap and routed to §8 Q8 — the behavior criteria.md scores 3. | Minor: Appendix A sources Logic Pro's zero-record status to `criteria.md`'s inventory rather than to `logic-pro.md:10–11`. Cite the primary document. |

**Lens average:** 3.000 (21/7)
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | §6.4 cites `current-milestone.md:88` ("Linux device qualification runs, discharging decision 23's debt") and `NEXT.md:25` (slice 3) — both verified verbatim at those lines. The spec produces exactly that row and defers every enrichment: no drill change (§4.1), no JACK row (§4.6), no CI automation (§4.6), no headroom threshold (§4.5), each with a stated reason and an §8 question. | — |
| 3B. Non-goal respect | 3 | §6.4 walks the list; the feature adds no product surface at all, so no non-goal is reachable. Confirmed against §3.1's surface inventory and §4.1's "adds **no code**." | — |
| 3C. Deliberately small first devices | 3 | §6.4 correctly identifies the fixture as the existing Pulse → Gain → Saturator chain at `lifecycle_health.rs:27–66` (verified: the `fixture` fn spans exactly 27–66) and leaves it unchanged. §7.2's "Explicitly not changed" bars every file under `crates/`. | — |
| 3D. Originality | 3 | Both proposed bounds are derived from this repository's own values, not a vendor's: `QUAL-002` from `lifecycle_health.rs:22–23`, `:264`, `:274–278`, and `QUAL-003` from the xrun definition at `bridge.rs:222` (verified: `if headroom <= 0.0 { xruns.fetch_add(1) }`). The outcome vocabulary and field set are the spec's own. The `QUAL-002` arithmetic slip is a correctness defect, not a borrowing one. | — |
| 3E. Platform commitment | 3 | This is the criterion's live risk addressed head-on. The standing constraint in the header, §1.2, §4.6, §5.6, §6.3, and §7.1 each refuse the Linux claim, and §7.1's "Gated / absent — Linux" bucket reproduces `current-milestone.md:116` and `:127` accurately, including that the 2026-08-09 result is build-and-link only and that a container or VM cannot satisfy the row. Decision 23 is quoted correctly from `decision-gates.md:49`. | — |
| 3F. Accessibility trajectory | 3 | §3.7 forecloses nothing and identifies one real positive obligation: record `hw:CARD=…` device keys verbatim rather than prettified, because a future device picker must display the identity the backend actually uses (`cpal_backend.rs:17–22`, verified — the file's own comment says cpal reports no stable device identity). Outcome is carried by a word, satisfying colour-independent state communication by construction. | — |

**Lens average:** 3.000 (18/6)
**Lens pass:** Yes
**Auto-fail triggered:** No

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 2 | §7.1 is overwhelmingly accurate and I verified it by opening every file. Exact to the line: the drill at `:248` with its three attributes at `:245–247`; the body map (`:252–253`, `:254–256`, `:257`, `:259–262`, `:264`, `:265–271`, `:274–278`, `:279`, `:281–288`, `:289–294`); `CpalBackend` at `cpal_backend.rs:34` implementing `AudioBackend` at `:68`; `open_output` at `:115–151`; `BufferSize::Fixed` at `:126–130`; `OpenFailed` at `:144`, `:181`, `:196`; `error_count` at `:163–168` and its genuine absence from the `AudioStream` trait at `lib.rs:181–196`; all eleven `BridgeTelemetry` fields and all eleven accessor lines `:63`–`:113`; `publish_headroom` at `:211–231` and the xrun test at `:222`; `Cargo.toml:12–15` and `:18` with no JACK feature; `ci.yml:7`, `:23`, `:27–32`, `:47`; the file is 295 lines. The silence argument is correct: `source.rs:202–209`'s `else { 0.0 }` branch and `effect.rs:119–128`'s zero-in-zero-out both check out. **Against that, four claims are wrong.** (i) §7.1 and §5.3 state the six deterministic tests run "against `NullBackend`" — the tests at `:207` and `:226` never construct a backend at all; they call `bridge.render` on a stack `RenderBlock`. (ii) §4.3 says the drill "contains exactly five assertions" — it contains four `assert*!` macros and six `.expect` calls. (iii) §4.7 attributes the drill's footprint to "the bridge's 256-event note scratch (`bridge.rs:19`)" — the constant is at `:20` and the drill passes `64` (`lifecycle_health.rs:261`). (iv) §5.1 T-7 says a `blocks_rendered > 0` failure means "the driver never called back"; it can equally mean every callback was refused at `bridge.rs:167–173`. | See Priority 1 items 1 and 3, Priority 2 item 3. |
| 4B. Status vocabulary | 3 | Spec status is `proposed`; §7.1 partitions into Implemented / Verified / Gated-absent / Absent and respects the distinction — the drill is `implemented` while only macOS is `verified`, and the QUAL rows are `proposed until Jeff accepts`. §7.2 item 3 correctly makes decision 23's disposition conditional on `PASS` only. | — |
| 4C. Traceability | 2 | Citation density is exceptional and nearly everything resolves: decisions 1/16/19/20/23 at `decision-gates.md:25`, `:40`, `:43`, `:44`, `:49` all verified; PROD-003 at `requirements-ledger.md:64`; the five ledger families at `:24`, `:32`, `:42`, `:51`, `:58`; `README.md:50` on creating `05-quality/`, `:39–48`, `:64–66` all exact. It also does the thing R4-1 was marked down for: §7.2 schedules `requirements-ledger.md` in the modified-files list with three QUAL rows carrying their own rationale, so PROD-003 is discharged by record and not by prose. Two defects: `QUAL-002`'s stated value does not follow from its own cited derivation, and §7.2 item 6 describes the traceability edit as adding a platform qualifier where none exists — `traceability.md:19` and `:44` already carry one ("under a live macOS driver", "Linux device qualification has not run"), so the edit is narrower than described. | See Priority 1 item 2; correct §7.2 item 6's premise. |
| 4D. Honest gaps | 3 | §8's ten questions are real, each naming what it blocks. Known gap (c) flags an inconsistency in the repository's own accepted record and I verified both halves independently: `current-milestone.md:113` records 173 blocks; 0.5 s at 256 frames / 48 000 Hz predicts 93.75 callbacks; 173 is 1.85× that and close to the 187.5 a 128-frame buffer would predict. §4.6 states the discrepancy, declines to explain it, and draws two consequences instead. That is the correct handling of an unresolved fact in accepted evidence. | — |
| 4E. Evidence commands | 3 | §5.2's three commands match `criteria.md` §4E and `STATUS.md:46–48` exactly; §5.1's command matches `current-milestone.md:132` and `NEXT.md:25` verbatim, and the three load-bearing properties named for it (`--ignored` runs only ignored tests, `--nocapture` is mandatory, default features must be on so `#[cfg(feature = "cpal-backend")]` compiles the drill in) are all correct. The §5 opening honesty statement declares which commands run today and which cannot — including that a `0 tests` transcript is `INCONCLUSIVE`, never `PASS` (T-13), which is a genuine and easily-missed trap. | — |
| 4F. No fake surfaces | 3 | Nothing implies Linux works or that anything makes sound. §3.1 states `./spectre` does not use `spectre-audio`, matching `STATUS.md:37`. §5.6's nine-item list of what a `PASS` does not authorize is the strongest instance of this discipline in the gauntlet so far, and item 5 is correct on the substance I checked: the drill renders silence, so the headroom figure describes an idle chain. §6.3's self-audit against AF-6 named language is accurate — "should pass", "expected to pass", and "straightforward" appear nowhere about the Linux outcome. | — |

**Lens average:** 2.667 (16/6)
**Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Triggered | Basis |
|---|---|---|
| AF-1 — Contradicting accepted authority | **No** | Decisions 1, 20, and 23 are quoted accurately and followed. Every proposal that would touch accepted authority — a new `QUAL` ledger family, a new `05-quality/` directory, a drill modification, a JACK row — is routed to §8 as a question rather than asserted. |
| AF-2 — Unbacked implementation claims | **No** | No code is described as existing that does not exist. Every path I opened contained what the spec said it did; §7.1's partition is checkable in one `Read` per claim. The four defects at 4A are misdescriptions of real code, not claims about absent code — see the feasibility rule instead. |
| AF-3 — Realtime discipline violation | **No** | No code changes; nothing is added to a callback-reachable path. The spec correctly reads the cpal error callback (`cpal_backend.rs:138–141`) as counting only. |
| AF-4 — Borrowed numeric limits | **No** | `QUAL-002` derives from `lifecycle_health.rs:22–23`, `:264`, `:274–278`; `QUAL-003` from `bridge.rs:222`. Neither comes from a reference product, and §7.2 schedules both as rationale rows in `requirements-ledger.md`, which is what PROD-003 requires beyond prose. The `QUAL-002` arithmetic error is a correctness defect, not a borrowing. |
| AF-5 — Conclusions the evidence does not support | **No** | Checked against all nine prohibited items in `workflow-field-study/product-implications.md`. §3.4 refuses a default shortcut map explicitly. §4.5 refuses a headroom threshold explicitly and names the monitoring-latency adjacency. `QUAL-003` (`xruns == 0`) is a correctness condition defined by `bridge.rs:222`, not a latency target, and the spec argues that distinction correctly. No supported-interface model list — §5.6 item 2 refuses generalizing past the one device opened. No new platform/backend order: §4.6's ALSA-first recommendation restates accepted decision 20 and is routed to §8 Q4. |
| AF-6 — Optimistic language | **No** | The standing constraint, §1.2, §4.6, §5.6, §6.3, and §7.1 each refuse the Linux claim. §1.3 explicitly states "the signal is not 'the row says PASS'." §5.5 gives `FAIL`, `REFUSED`, and `INCONCLUSIVE` equal standing. |
| 3B = 0 | **No** | No non-goal is proposed. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | The record schema (§3.3 table columns + E1–E10) is fully specified and does not exceed what `BridgeTelemetry` and `StreamConfig` can supply. The claim that no granted-config accessor exists in the seam is correct: `AudioStream::config()` (`lib.rs:195`) returns the requested `StreamConfig` that `CpalStream` stored unchanged at `cpal_backend.rs:147`. |
| API/interface changes are feasible with current architecture | ✓ | None are proposed. The reachability claim checks out — `error_count` is an inherent method on `CpalStream` (`cpal_backend.rs:165–167`), the `AudioStream` trait (`lib.rs:181–196`) does not declare it, and `open_output` returns `Box<dyn AudioStream>` (`lib.rs:215`), so it is genuinely unreachable from the drill. |
| Views/screens fit current navigation pattern | ✓ | No application surface. The two real surfaces (terminal transcript, Markdown row) are correctly described; the transcript is two plain `key=value` lines from `:257` and `:281–288`. |
| Dependencies are available and version-compatible | ✓ | `cpal = { version = "0.15.3", optional = true }` at `Cargo.toml:18` behind `default = ["cpal-backend"]` at `:14–15`, exactly as stated. The JACK-unreachability claim is correct: no Spectre feature enables a cpal JACK host. |
| Platform/renderer requirements are realistic | ✓ | `libasound2-dev` is installed at `ci.yml:27–32`; the host commands named (`aplay -l`, `uname`, `/proc/asound/version`, `/etc/os-release`) are present on a stock Linux install. The CI exclusion argument is sound — `ci.yml:47` runs `cargo test --locked --workspace`, which does not include ignored tests. |
| Test strategy is executable with current infrastructure | ✗ | §5.1 is hardware-blocked, which the spec declares openly and is not itself a defect. The defect is interpretive: §5.1's T-7 misreads what a `blocks_rendered > 0` failure proves, and no step covers the frame-capacity refusal path at `bridge.rs:167–173`. §5.2 and §5.3 are executable as written. |
| Performance budget is realistic for target hardware | ✓ | No render-path change. §4.7's wall-clock estimate (~1 s of drill time, gate dominates) is consistent with two 250 ms sleeps. |
| No undeclared dependency on unbuilt features | ✓ | §7.4's dependency table is accurate, including the two cross-spec references I checked: R4-1 §4.6 does flag `BufferSize::Fixed(256)` refusal and a 44 100 Hz default as its two ALSA risks, and `stream_errors` is proposed in R4-1 §4.3. R4-1's recorded review outcome (PASS at composite 2.950) matches its iteration-2 scorecard. |

**Feasibility verdict:** Feasible with caveats
**Caveats:** The protocol is authorable and acceptable now, as claimed. But the binding
feasibility rule is not satisfied: §7.1 states that six deterministic tests "run against
`NullBackend` at `:68`, `:108`, `:133`, `:165`, `:207`, `:226`", and the tests at `:207`
and `:226` construct no backend at all — they build a `RenderBridge` and call
`bridge.render` directly on a stack `RenderBlock`. The enumerated line numbers are all
correct and the behaviors §5.3 attributes to them are all correct; only the harness
attribution is false, for one third of the set.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.143 | 35% | 0.750 |
| 2 — DAW Workflow Depth | 3.000 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.667 | 20% | 0.533 |
| **Composite** | | | **2.633** |

**Pass conditions (from `criteria.md`; `criteria.md` is binding):**
- [x] Composite ≥ 2.30 — **2.633**
- [x] Every lens average ≥ 2.00 — 2.143 / 3.000 / 3.000 / 2.667
- [x] No criterion scores 0
- [x] At most two criteria score 1 — **zero** criteria scored 1
- [x] All auto-fail rules pass
- [ ] **Feasibility rule — FAILS.** §7.1 misdescribes the source file (the `NullBackend`
      attribution above). `criteria.md` binding condition 4 states that such a spec "fails
      regardless of composite," and R4-1 iteration 1 was failed on a single comparable
      sentence at composite 2.750.
- [x] Reviewer personally executed every command claimed as passing — the spec claims no
      command was run on Linux, and correctly declares §5.1 unrunnable. Nothing needed
      execution to verify; every claim was checked by reading source.

**All conditions met:** No → **FAIL**

---

## Remediation Brief

### Priority 1 — Must fix to pass

1. **Correct the `NullBackend` attribution in §7.1 and §5.3.** Both say the six deterministic
   tests run "against `NullBackend`." Verified against `crates/spectre-audio/tests/lifecycle_health.rs`:
   the tests at `:68`, `:108`, `:133`, and `:165` do construct `NullBackend`; the tests at
   `:207` (`an_overrunning_block_is_counted_as_an_xrun`) and `:226`
   (`containment_counters_reach_the_app_thread_through_telemetry`) construct no backend —
   they build a `RenderBridge` and call `bridge.render(&mut RenderBlock::new(&mut interleaved,
   CHANNELS))` directly. Replace with: four run against `NullBackend`; two drive the bridge
   directly with no backend. Neither §5.3's conclusion nor the six behaviors it lists needs
   to change — only the harness claim. This is the item that fails the feasibility rule.

2. **Fix `QUAL-002`'s arithmetic, which contradicts its own derivation.** §4.5 writes
   "⌊93.75⌋ − 4 = **90**"; ⌊93.75⌋ = 93 and 93 − 4 = **89**. The generalized form the same
   row proposes, ⌊0.5 s × granted_rate ÷ granted_frames⌋ − 4, also yields 89 at the drill's
   requested 48 000 Hz / 256 frames, so the fixed constant and the formula disagree at the
   very geometry the drill requests. The number appears four times — §4.5 (twice), §5.1
   T-11, and §7.2's `QUAL-002` row — and all four must move together. Either correct 90 → 89,
   or change the derivation to round up (94 − 4 = 90) and say so; do not leave a proposed
   requirements-ledger row whose value does not follow from the rationale printed beside it.

3. **Account for the compiled plan's frame capacity, and fix §5.1's T-7 row.** The fixture
   compiles the plan at `FRAMES = 256` (`lifecycle_health.rs:23`, `:259`), and
   `RenderBridge::render` refuses any block where `frames == 0 || frames > self.plan.max_frames()`
   — it fills silence, increments `frame_capacity_rejections`, and returns **without**
   incrementing `blocks_rendered` (`crates/spectre-audio/src/bridge.rs:166–173`). Three
   consequences the spec must carry:
   - §5.1 T-7's "what its failure means" column currently reads "the driver never called
     back." A `blocks_rendered == 0` result equally means every callback arrived and every
     one was refused for frame capacity. Rewrite it to name both causes.
   - Add an error state to §3.6 (E-9) for frame-capacity refusal, and note that the
     disambiguating counter `frame_capacity_rejections` (`bridge.rs:73`) is **not** printed
     by the drill, so this failure mode is currently indistinguishable from a dead driver in
     the record. That belongs in §5.6's list of what a run does not establish.
   - §4.6 and §8 Q1/Q2 discuss granted-versus-requested geometry at length without this. A
     granted buffer above 256 frames is precisely the case that produces the confusing
     result, and it is the same ALSA risk §4.6 already flags. Wire the two together.

### Priority 2 — Should fix for quality

1. **§4.3's assertion count.** "Exactly five assertions and two `println!`s" — the two
   `println!`s are right (`:257`, `:281`), but the file has four `assert*!` macros (`:253`,
   `:289`, `:293`, `:294`) and six `.expect` calls (`:252`, `:256`, `:271`, `:275`, `:277`,
   `:279`). The table's substance is right; the count sentence is not. Say "five asserted
   conditions, four `assert!` macros and six `.expect` panics" or drop the count.
   Relatedly, the table has no row for the `open_output` `.expect` at `:271` even though
   §5.1 lists it as T-3.

2. **§4.5's floor should model start latency, not only partial-block truncation.** The
   derivation allows exactly four lost blocks (~21 ms) across two `play()` and two `pause()`
   transitions. Stream start latency on ALSA is not bounded by the drill's structure and can
   exceed that, so a healthy run could be recorded `INCONCLUSIVE`. Either widen the margin
   with a stated rationale or note in `QUAL-002` that a marginal shortfall should be re-run
   before the row is written.

3. **§4.7's note-scratch figure.** "The bridge's 256-event note scratch (`bridge.rs:19`)" —
   `DEFAULT_NOTE_SCRATCH = 256` is at `bridge.rs:20`, and the drill does not use it: it
   passes `64` (`lifecycle_health.rs:261`), so the scratch is 64 events. Correct both.

4. **§7.2 item 6's premise.** It says the traceability edit adds a platform qualifier "rather
   than an unqualified claim," but `traceability.md:19` and `:44` already qualify to macOS
   and already state that Linux qualification has not run. The real edit is narrower: add the
   ALSA result alongside the CoreAudio one. Restate it accurately.

5. **`denormals_flushed` is unmeasured and unnamed.** RT-003 has two halves; the drill asserts
   `contaminated_nodes == 0` but never reads `denormals_flushed` (`bridge.rs:93`), which is
   not printed. §5.6's nine-item list should say the run establishes nothing about denormal
   flushing on the platform. §4.2 covers it generically; §5.6 is where it bites.

### Priority 3 — Consider for excellence

1. Cite Logic Pro's zero-record status to `docs/02-reference-research/logic-pro.md:10–11`
   (`draft` / `inventory-only`) rather than to `gauntlet-output/criteria.md`'s inventory. The
   grading document is not a primary source, and its own correction block is the standing
   argument for not treating it as one.
2. §4.4's "constructed at `:46–54`" for `BridgeTelemetry::default` — the headroom fields
   continue to `:56`. Widen to `:45–57`.
3. §1B is answered implicitly. One sentence stating that no new RT-002 traffic crosses the
   boundary and no lane, overflow policy, or reclamation path changes would close the lens-1
   disposition explicitly rather than by inference.
4. §4.7 could name the RT-001 guard (`rt_guard.rs`, recorded at `traceability.md:51`) that
   covers the bridge-render path the drill exercises, which is what criterion 1A asks for by
   name.
5. The strongest passage in the spec is §4.3's threshold-versus-observation table together
   with §4.4 rule 3's silence argument. Both are correct and both are non-obvious. Once the
   frame-capacity gap in Priority 1 item 3 is closed, that trio becomes the durable content
   of `docs/05-quality/device-qualification-protocol.md` and is worth leading with there.

---

**End of scorecard.**
