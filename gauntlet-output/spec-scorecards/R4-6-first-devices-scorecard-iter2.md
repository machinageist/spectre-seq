<!--
Author: Jeff
Date: 2026-08-23
Description: Blind re-verification scorecard for the R4-6 first-devices spec, iteration 2 (remediation 1)
Notes: The remediation is new content and was graded as ungraded content: every citation it added
  was opened at source, and iteration 1's findings were re-checked rather than carried forward —
  two of its Priority 3 pointers were wrong and the spec was right to refuse them. Scope statement
  at the end of this file records what was opened in full versus sampled.
-->

# Scorecard: First Devices — Filament and Gloam (iteration 2)

**Feature ID:** `R4-6` (`first-devices`)
**Spec file:** gauntlet-output/specs/R4-6-first-devices.md (1,396 lines)
**Reviewer agent:** blind verification agent, R4-6 iteration 2 re-verification
**Date:** 2026-08-23
**Spec iteration reviewed:** 2 (remediation 1)
**Prior record:** iteration 1 passed at 2.864 (`R4-6-first-devices-scorecard.md`); that scorecard was read but every finding in it was re-checked against source rather than carried forward.

---

## Verdict: PASS

**Summary:** All four remediation items are correct, and I verified each against source rather
than against the spec's own account of them: `open_device_in_shape` is real at
`crates/spectre-app/src/lib.rs:338` with the exact `Result`/`Err("unknown device")`/`:342–343`
shape the spec describes, and the prescribed assertion order is character-for-character the
idiom the existing sibling test already uses at `crates/spectre-app/tests/app_model.rs:217–220`;
the three hand-wired gates offered in place of the withdrawn test exist at the exact cited lines
and do genuinely guard the extraction from outside `spectre-offline`; test 6's deleted tolerance
is replaced by a bit-equality claim that §4.3's own arithmetic actually supports; and test 7's
poison-block proof holds. The remediation introduced **one** new defect: §5.2's forward reference
to `gauntlet-output/specs/R4-8-offline-bounce.md:953–966` for the golden vector is wrong — that
range has never held the vector in any version of that file I could open, and the vector sits
roughly 325 lines further down. The claim's substance is true, so this is a wrong pointer rather
than a fabrication: it costs 4C a point and does not trip the feasibility rule, which is scoped to
source paths and to §7.1, and every path under `crates/` and `docs/` that this spec cites was
opened and is accurate at the cited lines, in both directions.

---

## Lens 1 — Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | §4.1 unchanged and re-verified against source. `validate_buffers` allocates nothing and runs first (`io.rs:175–198` — opened, confirmed; bounded length checks at `:187–193`, note rejection at `:194–196`). `DspParameter::clamp` is quoted character-exact against `parameter.rs:49–51`. The no-panic argument closes all three holes: bounded indexing after `validate_buffers`; unreachable division (§4.3's strict `<`/`else` argument, checked below); `f64::clamp` behind a `const ParamSpec` that cannot hold `min > max` (`spectre-core/src/param.rs:77–79`, reached through `parameter()`'s const panic at `parameter.rs:140–143` — both opened). Critically §4.1 still does **not** claim the structural lock scan covers it; it names the allocation guard at `rt_guard.rs:259` with the positive control at `:138–155` as its actual RT-001 evidence. | — |
| 1B. Control↔render communication | 3 | §4.4 unchanged. No new lane; parameters ride decision 21's latest-wins lane, notes ride the bounded FIFO whose per-quantum capacity is `MAX_NOTE_EVENTS_PER_BLOCK = 1_024` — verified exact at `crates/spectre-dsp/src/io.rs:52`. Retired state is answered rather than dodged: neither device allocates, so the plan itself is what travels back on the existing reclaim lane. The five-row state table names owner, thread, and lifetime for every class. | — |
| 1C. Numerical containment | 3 | §4.3 step 1 still places containment **before** the state update, and the reason is checkable: `contain_channel` (`spectre-graph/src/lib.rs:401–414`) operates on output buffers only, and the plan applies it after a node has already updated its own state (`:510–521` — opened, exact). `Gloam` is the first recursive device, so the hazard is real and correctly identified. The denormal-state flush reuses the plan's own predicate `*sample != 0.0 && sample.abs() < f32::MIN_POSITIVE` (`:407–410` — verified verbatim) rather than inventing a threshold. Injection tests named: §5.1 test 7, test 8, and the `containment.rs` sibling. | — |
| 1D. Determinism | 3 | **Raised from 2.** The iteration-1 gap was `existing_fixture_hash_is_unchanged` having no baseline. The spec now withdraws it and names three external gates instead, and **all three exist at the exact cited lines and do what is claimed**: `plan_render_matches_hand_wired_chain` (`harness.rs:57–58`) folds peak and FNV-1a inline at `:97–105` and asserts against `render_vertical_slice` at `:108–109`; `app_defaults_match_backend_authoritative_default_render` (`:220–221`) and `every_app_parameter_maps_exactly_to_the_compiled_plan` (`:172–173`) both compare a full `RenderReport` (hash and peak included) against `hand_wired_report` (`:112–170`, own inline fold at `:155–163`). Both folds are complete hand-written FNV-1a walks holding their own copies of the seed (`:98`, `:156`) and prime (`:103`, `:161`) — so a one-bit change in §7.2 item 5's extracted helper does fail all three. §4.6 still scopes determinism to within-a-build because `sin`/`exp`/`tanh` are libm calls, and routes cross-platform hashes to Q5. | — |
| 1E. Graph and plan contract | 3 | Unchanged and re-verified. Both devices are `AudioProcessor` implementations a plan owns; `CompiledPlan` still exposes no node or edge mutation API — `spectre-graph/src/lib.rs:416–417` says exactly that, verbatim. Nothing is recompiled per parameter change; decision 22's rejected option (b) is not proposed in any form. | — |
| 1F. Failure behavior | 3 | §3.6's E1–E8 table unchanged; every mechanism re-checked. E1 const-eval panic (`parameter.rs:140–143` + `param.rs:77–79`); E4 non-finite → descriptor default (`param.rs:119–124`); E6 whole-node silence and `contaminated_nodes` (`graph/lib.rs:510–521`); E7 `validate_buffers` before device arithmetic (`io.rs:175–198`); E8 bounded one-quantum denormal exposure. The three binding rules — fail closed, counted not dropped, no new error vocabulary — are stated and honored. | — |
| 1G. Test specification | 3 | **Raised from 2. All four iteration-1 defects are genuinely closed, and I checked each at source rather than accepting the spec's account.** (a) `select_device` is gone; `grep -rn "select_device" crates/` still returns **zero** hits, and the replacement `AppModel::open_device_in_shape` is declared at `crates/spectre-app/src/lib.rs:338` exactly as quoted, with `Err("unknown device")` at `:340`, `self.selected_device = Some(id)` at `:342`, `self.lens = Lens::Shape` at `:343`. Every accessor the test reads exists: `selected_device_id()` at `:324` (the spec's cited line, exact), `lens()` at `:281`, `devices()` at `:313`, and `Lens::Shape` at `:13–16`. The prescribed assertion order is not merely compilable — it is verbatim the shape of the existing sibling `open_device_in_shape_atomically_focuses_existing_device_and_preserves_track` (`app_model.rs:206`), whose body runs `assert_eq!(model.open_device_in_shape(saturator_id), Ok(()))` at `:217` then `model.lens()` at `:218` and `selected_device_id()` at `:220`. (b) The withdrawal is justified — see 1D; the three replacement gates are real and stronger than a self-comparison. (c) Test 6's tolerance is deleted, not derived, and the reasoning holds: at `depth = 0.0`, step 3 gives `0.0 * follower = +0.0` for the non-negative finite `follower` step 2 guarantees, `clamp(+0.0, 0, 1) = +0.0`, and step 4's `damp_a + (1 - damp_a) * (+0.0)` is `damp_a + (+0.0)`, which is bit-exactly `damp_a` for every `damp_a` the spec's own `[0, 1]` argument allows. The no-FMA-contraction claim is correct — Rust emits no fast-math flags — and the unit-step input does keep state out of denormal range. (d) Test 7's proof is sound: with every poison sample non-finite, step 1 maps all to `0.0`, both recursions update by `+= k * (0.0 - 0.0)`, so A's post-poison state is bit-identical to a fresh B's initial state and all three numbered assertions follow. **Three residual soft items, none blocking:** test 7's proof silently requires instance A to be *fresh* before the poison block — the setup says "fresh" for B and not for A, in the one row whose whole remediation was about making a silent dependency explicit; test 4's `1e-6` bound is an unargued tolerance of exactly the species test 6 just removed one of; and the withdrawal paragraph's forward reference is mis-cited (4C). | Priority 2 items 1–2; Priority 3 item 1. |

**Lens average:** 3.000 (21 / 7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, zero 0s

---

## Lens 2 — DAW Workflow Depth (weight: 25%)

§3 and Appendix A are byte-identical to iteration 1. Scores re-derived, not carried.

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 2 | §3.2's primary flow is an audition loop — hold a note, sweep Lean, hear timbre move without pitch or phase moving — and §3.1 adds no view, no modal, and no transport-bar element ("Nothing is added to the transport bar"; Arrange and Mix "unchanged"). **Gap, unchanged from iteration 1:** sketch → branch → grow is not addressed, and the spec never states in its own words that selection, zoom, and transport survive the new rows. It is close enough to reach for it — §5.3 says its new test "is that test's shape", and that sibling asserts `selected_track_id()` preservation at `app_model.rs:219` — but §5.3's own test does not assert it and §3 does not claim it. Correct and functional; thin. | Priority 3 item 2. |
| 2B. Linked lenses | 3 | §3.1 puts both devices in Build and Shape as views over one `DeviceControl` (`crates/spectre-app/src/lib.rs:57–63`), and §4.4 states "No new state container, no new store, and no second source of truth." §3.3 sharpens it: exactly one value per parameter in exactly one place, with the device setter the only way a value enters DSP state. Nothing forks per lens. | — |
| 2C. Modulation visibility | 3 | §3.3's PROD-002 paragraph gives the correct answer rather than the flattering one: neither device is a modulation source or destination, so every Shape value is a base value, and inventing a tri-state display now "would be a fake surface." The obligation accepted is non-foreclosure — one value, one place, one insertion point for a later automation layer. Provenance verified: `OBS-AB12-AUTO-004` does record a manual change overriding automation, the LED dimming, and an explicit re-enable command. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 adds no binding and ratifies none, citing the prohibition at `product-implications.md:96` and pointing at `vision.md:41`'s context-scoped, searchable, remappable commands. Seven rows across two devices is the opposite of spreadsheet density; no cable surface is introduced. | — |
| 2E. Convergent-pattern grounding | 3 | The one genuine convergence on this surface — per-voice duplication is the cost center, shared post-FX is the cheaper structure — is grounded in `OBS-SR2-CPU-001` and followed in §0 (one voice, one shared insert) while explicitly refusing the record's numbers. Where Spectre diverges (VCV's voltage-typed contract, `OBS-VCV-VOLT-001`/`-004`), Appendix A states the divergence and grounds it in `vision.md:39`'s "informed by — not copied from". | — |
| 2F. Differentiation | 3 | Appendix A claims exactly one thing — every numeric bound published with its derivation and a re-open trigger before the device ships — and disclaims the rest in terms: "**not** a claim that these devices sound better than, or are more capable than, anything in the benchmark set. They are smaller than all of it, on purpose." That is 2F without the parity framing `vision.md:59` prohibits. | — |
| 2G. Benchmark evidence discipline | 3 | Every `OBS-` ID cited resolves and says what the spec claims. **Serum 2 is held to exactly the two permitted records** (`OBS-SR2-CPU-001`, `OBS-SR2-KB-001`) and no claim beyond them appears. **Logic Pro appears zero times**, and Appendix A names the hole with the dossier's own status line. Named gaps 1 (no citable record on any benchmark's internal device implementation) and 2 (no anti-aliasing record) are both correct against the corpus. | — |

**Lens average:** 2.857 (20 / 7)
**Lens pass:** Yes

---

## Lens 3 — Product Identity & Scope Discipline (weight: 20%)

§0, §4.2's bound table, §6, and §7.1 are unchanged. Re-verified rather than carried.

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | §6.4 3A maps the feature onto the R4 exit row at `current-milestone.md:84` and `vision.md:48`'s alpha bar, both verbatim. Everything adjacent is deferred by name and by slice: track hosting → R4-4, clip playback → R4-5, persistence → R4-7, bounce equivalence → R4-8, QA protocol → R4-9. §7.4 repeats the discipline as a dependency table. | — |
| 3B. Non-goal respect | 3 | §6.4 3B walks the non-goals item by item. No CLAP/LV2/AU hosting; explicitly no plugin-format authoring — both devices are plain Rust types reachable only through `AudioProcessor`; no preset format at all; no cloud, store, or video; automation and modulation named as R5+. | — |
| 3C. Deliberately small first devices | 3 | Still the spine, and the remediation did not move it: §0 is the first section and quotes decision 15 verbatim alongside `dsp-device-io.md:107`. Delivered scope is unchanged at seven parameters, one voice, one oscillator, one pole, no unison, no filter bank, no LFO, no modulation matrix, no effect lanes, no wavetable, no sample player, no granular engine, no preset browser, no visualizer. §3.1 still refuses a waveform display on repaint and accessibility cost and routes it to Q6. **No drift toward the R11 flagship in 1,396 lines** — and the 94 lines the remediation added are entirely §5 test prose and two §7.2 sentences, not capability. | — |
| 3D. Originality | 3 | `grep -rniI "filament\|gloam"` over the whole repository **excluding `gauntlet-output/`** returns zero hits, so the §6.4 3D claim is true in the strong form. Neither name is a benchmark's device, style, or mode name. The parameter vocabulary is Spectre's own; the one reused word, `level`, is reused from Spectre's own `PULSE_PARAMETERS` (`source.rs:39–46` — verified, default `0.2`). The DSP is written out in full, so "not transcribed" is checkable rather than assertable. | — |
| 3E. Platform commitment | 3 | §4.6 unchanged. No `cfg`, no intrinsic, no CPU control-register manipulation, no `unsafe`; the denormal flush is the portable software FTZ-equivalent sourced to `current-milestone.md:59`. Every test in §5.1–§5.3 runs offline or against the null backend, so both devices are provable on either platform with no audio hardware. It then states the boundary: "This spec authorizes no Linux support claim." | — |
| 3F. Accessibility trajectory | 3 | §3.7 claims **no** screen-reader support and gives the reason, then argues only non-foreclosure: all seven controls carry a backend name, unit, range, and default. It finds its own weakness — `Lean`, `Damp`, `Track` read poorly aloud — adds it to the M3 manual check, and routes a descriptor description field to Q7 ahead of decision 17's R4 audit. No icon-only control, no color-only state, no drag-only gesture. | — |

**Lens average:** 3.000 (18 / 6)
**Lens pass:** Yes
**Auto-fail triggered:** No

---

## Lens 4 — Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | §7.1 is unchanged from iteration 1 and I re-read it line by line against source rather than trusting that. Verified **present** at the cited lines: `PULSE_PARAMETERS` one parameter default `0.2` (`source.rs:39–46`); `TONE_PARAMETERS` `20 … 20 000 / 440` at `:27–37`; the four-variant `Waveform` and the `Option<(u32, u8)>` at `:117`; `:202–209` emitting the oscillator value or exactly `0.0`; `Gain` as one `f32` (`effect.rs:29–31`) with a clamped instantaneous setter (`:45–47`) and the non-finite mapping at `:65–69`; `EFFECT_IO` at `:12–17` and `INSTRUMENT_IO` at `source.rs:20–25`, both byte-for-byte what §4.2 restates; `Saturator`'s `:119`, `:122–126`, `:127`, `:128`; `AudioProcessor` with exactly `io` and `process` plus `Send` (`io.rs:163–172`); `contain_channel` and `CompiledPlan`'s no-mutation comment; `render_plan` `:204–285` with seed `:271` and prime `:276`; `DeviceValues::from_snapshot`'s exactly-four refusal at `:45–47`; the hardcoded four-entry snapshot fixture at `lib.rs:351–356`. Verified **absent** as claimed, in both directions: `Filament`/`Gloam`/`FILAMENT_PARAMETERS`/`GLOAM_PARAMETERS` (repo-wide grep outside `gauntlet-output/`, zero hits); `set_parameter` anywhere in `crates/` (zero hits); any `#[cfg(test)]` module in `spectre-dsp`, `spectre-graph`, `spectre-audio`, `spectre-offline` (zero hits); any smoothing in `spectre-dsp/src` (zero hits for `smooth`, so D-R3 is real). Note on method: `minimum()`/`maximum()` are `pub const fn` at `parameter.rs:66–72`, so I matched on `const fn` rather than concluding absence from a `pub fn` grep. `crates/` is byte-identical between `2e005e5` and `HEAD`, so no source citation has drifted since the spec's stated read commit. The D-R3 discrepancy is stated exactly right and is real: `dsp-device-io.md:94` does read *"`Gain` already smooths"* and `:104` *"stereo linear gain with click-resistant smoothing"*, both verbatim, while `grep -rn "smooth" crates/spectre-dsp/src/` returns nothing. The one blemish here is a stale *pointer*, not a false statement — the D-R3 line number has moved since the spec was written (4C) — and §7.1's description of what D-R3 says is correct against the record as it stands today. | — |
| 4B. Status vocabulary | 3 | §7.1 partitions Implemented / Absent / Gated and adds "Contract-versus-code discrepancy, carried not resolved." The `implemented`/`verified` distinction is respected — R4-2 is "spec'd and passed at 3.000; it is **not** implemented", R4-1 "passed its spec gate at 2.950 with zero lines of implementation". The spec's own status is `proposed`; the twelve DEV rows are `proposed` until Jeff accepts. The iteration-2 header states its own iteration and enumerates its four changes without claiming any of them was verified by anyone. | — |
| 4C. Traceability | 2 | **Lowered from 3, on a defect the remediation introduced.** §5.2's withdrawal argument closes with: *"R4-8 already makes that fold public and declares exactly this vector and constant for it (`gauntlet-output/specs/R4-8-offline-bounce.md:953–966`); the pin belongs there."* **Those lines are not that.** In the committed version of R4-8 they are the §4.4 state-management table (`| State | Owner | Thread | Lifetime |` and the `BouncePanel`/`bounce_into`/`RenderBridge` rows); the golden vector and constant are ~325 lines further down, at `:1288` (`const GOLDEN_INPUT: [f32; 8] = …`) and `:1291` (`const GOLDEN_HASH: u64 = 0xa49a_cc9c_e735_9a37`), inside test `the_shared_fold_matches_its_checked_in_golden_vector` at `:1278` in that version. I checked three copies — `923134b`, the working tree as it stood mid-review, and the version now committed at `988541f` (R4-8's remediation 3, which landed while I was writing) — and **`:953–966` has never held the vector in any of them**, holding three different unrelated passages instead. So this is an authoring error, not concurrent-edit drift, and I checked the ancestors specifically to be able to tell those two apart. The claim's **substance is true** — R4-8 does move the fold to a public `crates/spectre-offline/src/hash.rs` with `pub const FNV_OFFSET_BASIS`/`FNV_PRIME` and does declare that vector — so this is a wrong pointer on a true claim, not a fabrication. **A second stale pointer exists and is expressly *not* charged against the spec:** the header, §4.3, §7.1, and §7.4 all cite `gauntlet-output/decisions-needed.md:102` for D-R3, and `:102` is today `### D-R4 — What does R4's exit mean while the Linux row cannot close?`, with D-R3 at `:146`. I checked the ancestor: at `923134b`, D-R3 **was** at `:102`, and D-R4 — "Raised: 2026-08-21, by the R4-9 spec" — was inserted above it afterward. That is drift in a concurrently-maintained sibling document, not an authoring error, and the substance is exact: D-R3 exists, is open, and its heading reads *"An accepted architecture document asserts `Gain` smooths; the shipped `Gain` does not"*, which is precisely what §7.1 says it says. Distinguishing the two cases is why I opened the committed ancestors rather than only the working tree. Everything else lands, and I mean everything I opened: decision rows 1/15/16/17/21/22/23 at `:25`/`:39`/`:40`/`:41`/`:45`/`:47`/`:49`; ledger rows RT-001/002/003 (`:28–30`), CORE-002 (`:47`), GRAPH-001 (`:55`), PROD-002 (`:63`), PROD-003 (`:64`), format (`:21–22`), known gaps (`:19`); every `dsp-device-io.md` line (`:26`, `:29`, `:36–37`, `:46`, `:58`, `:59`, `:60`, `:67`, `:79`, `:87–94`, `:98–107`, `:112`, `:113`, `:115`, `:117–118`); `vision.md:39`/`:48`/`:59`; `current-milestone.md:84`; `NEXT.md:19`/`:28`; `product-implications.md:94–100`; and every source path in §4 and §7.1. | Priority 1 item 1; Priority 2 item 3. |
| 4D. Honest gaps | 3 | Three known gaps in the header, nine open questions, and the questions are the real ones: Q1 routes smoothing to D-R3 without resolving it, Q3 routes the ledger family as a proposal, Q5 names cross-platform libm bit-equality as a threat to R4-8's framing, Q8 discloses that the RT-001 structural lock scan has never covered any device module — a gap the spec found itself. §4.7 states plainly that nothing has been measured. The remediation adds to this rather than subtracting: §5.2 now states in terms what the withdrawn test could not do ("could only have compared the implementation against itself and **could not fail**") and why a golden vector is declined here, instead of quietly dropping a row. | — |
| 4E. Evidence commands | 3 | The workspace gate is quoted character-exact as `criteria.md` 4E names it. All five targeted commands resolve to files that exist. No command is invented, and §5 does not claim the new test bodies already run. | — |
| 4F. No fake surfaces | 3 | "**Stated plainly and kept stated until it is false: `./spectre` produces no sound of any kind today**", sourced to `NEXT.md:19`. §5.4 states that none of the manual protocol can be executed today. §7.4 makes R4-2 a hard blocker precisely because devices with dead controls would be a fake surface. §3.7 claims no screen-reader support. §6.3's AF-6 self-audit holds under inspection. | — |

**Lens average:** 2.833 (17 / 6)
**Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Result | Basis |
|---|---|---|
| AF-1 — Contradicting accepted authority | **Pass** | No Accepted row is contradicted. The two changes to accepted documents — a `DEV` ledger family and an addition to `dsp-device-io.md`'s device list — are still routed as proposals in §7.2 items 11–12 and §8 Q2/Q3, never asserted. §8 Q1 still explicitly declines to resolve D-R3. |
| AF-2 — Unbacked implementation claims | **Pass** | §7.1 distinguishes implemented / absent / gated and every claim resolved in one `Read`, in both directions. The one non-existent identifier iteration 1 found (`select_device`) is gone and its replacement exists at the exact cited line. No claim that unwritten code exists survives anywhere in the spec. |
| AF-3 — Realtime discipline violation | **Pass** | No allocation, lock, I/O, logging, formatting, or panic on any callback-reachable path; §4.1 argues each by construction and §5.2 names the allocation-guard evidence. Denormals flush with the existing predicate; NaN/Inf isolate to silence (E5, E6). |
| AF-4 — Borrowed numeric limits | **Pass — schedule confirmed intact.** | The remediation **added no bound and removed none**. §4.2's table still carries exactly DEV-001 … DEV-012 and §7.2 item 11 still schedules "rows **DEV-001 … DEV-012** exactly as tabulated in §4.2" into `docs/01-requirements/requirements-ledger.md` in the ledger's own `:21–22` format, each `proposed`. The only number the remediation deleted is test 6's 1-ULP *test tolerance*, which is not a parameter bound and carries no DEV row, so PROD-003's schedule is unaffected. Corpus cross-check re-run: `20 Hz – 20 kHz` does appear in the corpus (`OBS-OZ-EQ-001`, `OBS-OZ-DEQ-001`), so it was investigated rather than assumed coincidental — the spec derives it from Spectre's own `TONE_PARAMETERS[0]` (`source.rs:28–35`, verified `20.0 … 20_000.0`), never cites Ozone, and Ozone is not in the benchmark set. |
| AF-5 — Conclusions the evidence does not support | **Pass — the §0 refusals survived.** | §0 is byte-identical: it still tabulates four prohibited conclusions from `product-implications.md:94–97` and refuses each — two devices but no catalog/naming scheme/families/count (Q2); `Filament`'s structure declared "**this device's implementation**, not Spectre's synthesis architecture" with no modulation source, routing, or limit (Q4); §3.4 adds and ratifies no binding; §4.7 gives operation counts and asserts no time, CPU, or headroom target. §7.2 item 12 still preserves `dsp-device-io.md:107`'s disclaiming sentence verbatim. |
| AF-6 — Optimistic language | **Pass** | §6.3's self-audit holds. Every audible outcome is conditional on R4-1/R4-2; aliasing, libm variance, and the absence of measurement are stated as limitations. The remediation's own prose is if anything more conservative than what it replaced — it says of the withdrawn test that it "could not fail" rather than that it was redundant. |
| 3B = 0 | **Pass** | No non-goal is proposed. |

---

## Feasibility Check

Read against source, not against §7.1 and not against iteration 1's scorecard.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | `DeviceIo`, `DeviceClass`, `DspParameter`, `ParamUnit` all exist and are exported. `parameter()` is `pub(crate) const fn` at `parameter.rs:128`, so the spec's claim that a new module *inside* `spectre-dsp` can build descriptors and one outside cannot is correct. `NORMALIZED_ROUND_TRIP_MAX_ULPS` is `pub const` at `parameter.rs:9` as test 12 needs. |
| API/interface changes are feasible with current architecture | ✓ | Two new modules plus two `mod`/`pub use` lines. `FILAMENT_IO` is byte-identical to `INSTRUMENT_IO` (`source.rs:20–25`) and `GLOAM_IO` to `EFFECT_IO` (`effect.rs:12–17`) — both confirmed field for field. |
| Views/screens fit current navigation pattern | ✓ | Two appended `DeviceControl::from_descriptors` entries need no new widget; Shape rows are descriptor-driven. |
| Dependencies are available and version-compatible | ✓ | None added. `spectre-audio`'s test already imports `spectre_dsp`, so §5.2's guard test needs no manifest change. |
| Platform/renderer requirements are realistic | ✓ | No `cfg`, no intrinsic, no `unsafe`; both devices provable with no audio hardware on either platform. |
| Test strategy is executable with current infrastructure | ✓ | **This is where iteration 1 failed and where the remediation had to land.** Every method and accessor §5.3 names now exists: `open_device_in_shape` (`:338`), `lens()` (`:281`), `devices()` (`:313`), `selected_device_id()` (`:324`), `Lens::Shape` (`:13–16`). `assert_eq!(model.open_device_in_shape(id), Ok(()))` type-checks because `Result<(), &'static str>` is `Debug + PartialEq`, and the identical assertion already compiles at `app_model.rs:217`. Test 6's bit-equality is achievable — the derivation holds. Test 7's three assertions are provable. All five named test files exist and all five commands resolve. |
| Performance budget is realistic for target hardware | ✓ | Operation counts with no time target. One `sin` per sample is the same order as the shipped `Saturator`'s `tanh` per sample per channel (`effect.rs:127`). |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares R4-2 a hard blocker for the setters and R4-1 for audibility, and correctly states §5.1–§5.3 run offline regardless. Both dependencies' status is accurate. |

**Feasibility verdict:** Feasible

**Why the two bad pointers do not trip the feasibility rule, stated explicitly because a sibling
just failed on it.** The rule is scoped to "every source path the spec cites" and to a §7.1 that
misdescribes current state. **Every source path under `crates/` and `docs/` that this spec cites
was opened and is accurate at the cited lines, in both directions** — including all three of §7.1's
absence claims, which I checked by repo-wide grep rather than by inspection: `Filament`/`Gloam`
appear nowhere outside `gauntlet-output/`, `set_parameter` appears nowhere in `crates/`, and
`smooth` appears nowhere in `spectre-dsp/src/`, so the `Gain` claim underpinning D-R3 is true. Both
bad pointers are into `gauntlet-output/` — a sibling spec and the decisions register — and neither
carries a §7.1 statement about code. R4-8's iteration-3 failure was a different animal: a **false
absence census** and an import list that would not compile, both inside its own current-state
section. Nothing of that kind is present here, and the one thing that *could* have been — §5.3
naming a method that does not exist — is exactly what this remediation fixed.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 3.000 | 35% | 1.050 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.833 | 20% | 0.567 |
| **Composite** | | | **2.931** |

**Pass conditions (from `criteria.md`, binding — all four):**
- [x] Composite ≥ 2.30 — **2.931** (iteration 1: 2.864)
- [x] Every lens average ≥ 2.00 — 3.000 / 2.857 / 3.000 / 2.833
- [x] No criterion scores 0, and at most two score 1 — lowest awarded is 2; **zero** criteria score 1
- [x] Feasibility rule — every source path the spec cites was opened and checked at the cited lines, in both directions; §7.1 is accurate; the single mis-citation is to a sibling spec, not a source file
- [x] All auto-fail rules pass — AF-1 through AF-6 checked individually above

**All conditions met:** Yes → **PASS**

---

## Remediation Brief

The spec passes. These are corrections to make before implementation, not conditions of the pass.

### Priority 1 — Must fix before implementation

1. **§5.2's forward reference to R4-8's golden vector cites the wrong lines.** The sentence
   *"R4-8 already makes that fold public and declares exactly this vector and constant for it
   (`gauntlet-output/specs/R4-8-offline-bounce.md:953–966`)"* points at R4-8's §4.4
   state-management table. The vector and constant live in test
   `the_shared_fold_matches_its_checked_in_golden_vector` — at `:1288`/`:1291` in the version
   committed at `923134b`, and at `:1454` as of `988541f` — and the public fold is
   `crates/spectre-offline/src/hash.rs` with `pub const FNV_OFFSET_BASIS`/`FNV_PRIME`. **Do not fix
   this with a new line number.** R4-8 has moved twice while this review was being written (2,200 →
   2,305 → its current length, under `988541f` "R4-8 remediation 3 fixes the feasibility-rule
   failure"), and `:953–966` has held three different unrelated passages across those versions.
   Cite the test by name. Two further notes belong with the fix: R4-8's iteration 3 **failed** its
   gate on the feasibility rule and its remediation is only now landing, so a clause stating that
   the pin's destination is itself unsettled is the honest form of this deferral; and the argument
   for withdrawing `existing_fixture_hash_is_unchanged` does not depend on this sentence at all —
   the three `harness.rs` gates carry it on their own — so the sentence can be trimmed rather than
   repaired if that is simpler.

### Priority 2 — Should fix for quality

1. **§5.1 test 7's proof requires instance A to be fresh, and only B is called fresh.** The
   setup reads "Feed that block to instance A; then feed one clean block to A and the **same**
   clean block to a fresh instance B." The proof below the table then argues "both recursions are
   then driven from zero by zero", which is true only if A's `follower` and `damped` are already
   zero when the poison block arrives. If A had any prior nonzero state, the poison block leaves
   it at `state × (1 - k)` rather than at zero, and assertion (iii) fails against a device behaving
   exactly as §4.3 specifies — the same false-alarm class the remediation just eliminated for the
   block's composition. Write "a fresh instance A" into the setup, with the same "load-bearing"
   note the word "entirely" now carries.
2. **§5.1 test 4's `1e-6` is the last unargued tolerance in the table.** Test 6's remediation
   argues, correctly, that a bound with nothing behind it is what PROD-003 exists to forbid; test 4
   two rows above still asserts `output[0].abs() <= 1e-6` with no derivation. The derivation is
   available and short: the worst case is `lean = 0.0`, where `map(0, 0) = 0.5` and
   `sin(TAU × 0.5)` is `≈1.22e-16` rather than exactly zero, scaled by a first-sample contour of at
   most `1.0` — so the true bound is on the order of `1e-16` and `1e-6` carries ten orders of
   margin. State that, or assert the `1e-16`-scale bound directly. Either way the row stops being
   the one place the spec applies a standard it argues against elsewhere.
3. **D-R3's line number has moved and four citations now point at D-R4.** The header's "Open
   decisions" line, §4.3's "Where smoothing is and is not", §7.1's contract-versus-code paragraph,
   and §7.4's dependency table all cite `gauntlet-output/decisions-needed.md:102`. D-R3 is now at
   `:146`; `:102` is D-R4, inserted on 2026-08-21 by the R4-9 spec. **This is not an authoring
   error** — the citation was correct at `923134b` and a sibling document moved underneath it — so
   it is a refresh, not a correction, and the spec's account of what D-R3 says is still exact.
   Worth doing in the same pass as Priority 1, and worth noting that both stale references in this
   spec point into `gauntlet-output/`, where documents are still moving; citing gauntlet artifacts
   by section or record ID rather than by line would immunize the spec against the next such shift.

### Priority 3 — Consider for excellence

1. **§5.2's "holds no golden constant" is loose about what `harness.rs` does hold.** The file
   holds no golden *report* value — I confirmed that across all 470 lines — but it does hold two
   FNV constants twice over (`:98`/`:103` and `:156`/`:161`), and the sentence's own next paragraph
   depends on those existing. "No baseline report value for the fixture render" would say the true
   thing without appearing to contradict the paragraph beneath it.
2. **§4.3 step 3 still renders `dsp-device-io.md:112` as "exact silence" where the contract says
   "finite silence".** Carried unaddressed from iteration 1's Priority 3, and it is worth doing
   because the spec is *underselling itself*: `:112` reads *"Silence remains finite silence through
   every effect"* — a weaker property, about a different device class — while what §4.3 actually
   delivers is bit-exact `0.0` with `is_sign_positive()`, asserted in §5.1 test 5. Claim the
   stronger property as Spectre's own rather than sourcing it to a line that says something else.
3. **§3.1/§3.2 still do not state the loop-first context guarantee in the spec's own words (2A).**
   This is the one iteration-1 Priority 3 item that was neither adopted nor refused, and it is now
   the only thing holding a criterion below 3. The material is already in hand: the sibling test
   §5.3 explicitly models itself on asserts `selected_track_id()` preservation at
   `app_model.rs:219`. One sentence naming selection, zoom, and transport survival across opening a
   device — and one added assertion in `opening_a_new_device_focuses_shape` — would close it.

**Two iteration-1 Priority 3 pointers were correctly refused, and I confirmed the refusal
independently.** Iteration 1 claimed `effect.rs:65–69` should start at `:64` and that
`io.rs:163–172` omits a closing brace at `:173`. Both claims are wrong: `effect.rs:64` is
`let input = inputs[channel][frame];` and the `if input.is_finite()` conditional begins at `:65`
and ends at `:69`; `io.rs:172` **is** the trait's closing brace and `:173` is blank. The spec's
original ranges were exact and were right not to move.

---

## Scope of this review

**Opened in full and read against the claim at every cited line:** the spec itself (all 1,396
lines); `criteria.md`; the iteration-1 scorecard; `crates/spectre-app/src/lib.rs` `:315–360`
(`selected_device_id`, `selected_device`, `lens`, `open_device_in_shape`,
`device_parameter_snapshot` and its fixture array) plus `Lens`, `lens()`, `devices()` by grep;
`crates/spectre-app/tests/app_model.rs:200–225` (the sibling drill-in test);
`crates/spectre-offline/tests/harness.rs` `:40–235` in full with line numbers, plus a constant
sweep over all 470 lines; `crates/spectre-offline/src/lib.rs` at `:40–50`, `:155–160`, `:200–216`,
`:265–292`, `:300–336`; `crates/spectre-dsp/src/effect.rs:10–75` and `:105–133`;
`crates/spectre-dsp/src/io.rs` `:50–53`, `:135–145`, `:160–200`;
`crates/spectre-dsp/src/parameter.rs` `:5–12`, `:60–80`, `:125–146`;
`crates/spectre-dsp/src/source.rs` `:18–48`, `:90–100`, `:112–120`, `:200–212`;
`crates/spectre-graph/src/lib.rs` `:399–418` and `:505–525`; and both the working-tree and
committed (`923134b`) copies of `gauntlet-output/specs/R4-8-offline-bounce.md` around `:945–970`
and `:1278–1302`.

**Verified by exhaustive repo-wide grep** (absence claims, both directions): `select_device`,
`set_parameter`, `filament`/`gloam` outside `gauntlet-output/`, `#[cfg(test)]` in the four
crates, `smooth` in `spectre-dsp/src`, and `git diff 2e005e5..HEAD -- crates/` (empty, so no
citation has drifted since the spec's stated read commit). `minimum()`/`maximum()` were matched
as `pub const fn`, not `pub fn`.

Also opened at the cited lines rather than assumed from iteration 1: `crates/spectre-audio/tests/rt_guard.rs`
at `:6–10`, `:17–20`, `:65–68`, `:136–156`, `:257–261`, `:289–311` — which is how I confirmed
independently that `RT_MODULES` at `:293–298` joined to `env!("CARGO_MANIFEST_DIR")` at `:309`
genuinely cannot reach `crates/spectre-dsp/src/`, the concession §4.1 and §8 Q8 are built on;
`crates/spectre-graph/tests/containment.rs:1–8` and `:325–330`;
`crates/spectre-dsp/tests/devices.rs:35–40`, `:67–72`, `:184–189` (`native_parameters()` at `:37`,
the round-trip test at `:69–70`, `device_layouts_match_the_v1_contract` at `:186–187` — all exact);
`crates/spectre-audio/src/lib.rs:25` (`MIN_SAMPLE_RATE = 8_000`, DEV-003's derivation base);
`crates/spectre-app/src/lib.rs:55–65`, `:90–94`, `:226–252`, `:455–459` (the three class subtitles
at `:233`/`:240`/`:247` are verbatim what §3.3 reuses). **Every `OBS-` record the spec cites was
opened** — `OBS-AB12-MIX-002` (`:99`), `-MIX-008` (`:105`), `-AUTO-004` (`:127`), `OBS-PP-UNI-003`
(`:37`), `OBS-SR2-CPU-001`/`-KB-001` (`:54–55`), `OBS-VCV-VOLT-006` (`:50`) — plus
`logic-pro.md:10–11` and `product-implications.md:90–100`. On the Serum 2 quarantine: a much larger
`serum-2-observations.md` now exists in the tree carrying `OBS-SR2-VER-*`, `-VOICE-*` and more, and
**the spec cites none of it** — it holds to the two permitted records, which is the correct behavior
under D-R2. Corpus AF-4 cross-check re-run against `ozone-observations.md` and
`fabfilter-observations.md` as well as the benchmark dossiers.

**Sampled rather than read in full:** the non-cited remainder of `spectre-graph`,
`spectre-project`, `spectre-core`, and `spectre-app/src/main.rs`; the bodies of `rt_guard.rs`,
`containment.rs`, and `devices.rs` away from the cited regions; the R4-2 spec (its `set_parameter`
signature only) and R4-8 (its golden-vector and public-fold regions only, in two versions); the
R4-1/R4-2 scorecards (score values only, to check §7.1's "2.950" and "3.000"). The four remediation
items, §7.1's full census, and every citation the remediation newly introduced were opened in full
and are the basis of this verdict.

---

**End of scorecard.**
