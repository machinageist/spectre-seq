<!--
Author: Jeff
Date: 2026-08-21
Description: Blind verification scorecard for the R4-6 first-devices spec, iteration 1
Notes: Opened in full — source.rs, effect.rs, io.rs, parameter.rs (spectre-dsp), param.rs (spectre-core),
  the contain_channel/CompiledPlan regions of spectre-graph/src/lib.rs, spectre-offline/src/lib.rs,
  the cited regions of spectre-app/src/lib.rs, rt_guard.rs, containment.rs, devices.rs, harness.rs,
  criteria.md, decision-gates.md, requirements-ledger.md, dsp-device-io.md, product-implications.md
  §"Prohibited conclusions", vision.md, current-milestone.md, NEXT.md, STATUS.md, README.md, and every
  OBS- record the spec cites. Sampled — spectre-app/src/main.rs (slider-row rendering only),
  the non-cited remainder of spectre-graph and spectre-project, and the R4-1/R4-2 scorecards
  (score values only). Corpus AF-4 cross-check run against fabfilter-observations.md and
  ozone-observations.md as well as the five benchmark dossiers.
-->

# Scorecard: First Devices — Filament and Gloam

**Feature ID:** `R4-6` (`first-devices`)
**Spec file:** gauntlet-output/specs/R4-6-first-devices.md
**Reviewer agent:** blind verification agent, R4-6 iteration 1
**Date:** 2026-08-21
**Spec iteration reviewed:** 1

---

## Verdict: PASS

**Summary:** The strongest quality is source fidelity under adversarial checking: I opened every
path the spec cites and the overwhelming majority of citations are line-exact in both directions —
including the subtle ones (`parameter()` is `pub(crate) const fn` at `parameter.rs:128`;
`minimum()`/`maximum()` are `pub const fn` at `:66–76`; the RT-001 structural lock scan at
`rt_guard.rs:293–298`/`:309` genuinely cannot reach `spectre-dsp`; `device_parameter_snapshot`
genuinely iterates a hardcoded four-entry array rather than `self.devices`). The most critical gap
is in §5.3: the test `opening_a_new_device_focuses_shape` is described as calling
`select_device`, and **no such method exists anywhere in `crates/`** — the drill-in the spec
correctly cites at `crates/spectre-app/src/lib.rs:342–343` is `AppModel::open_device_in_shape`
(`:338`), so that test cannot compile as written. That is a Priority 1 correction, not a
disqualifier: the behavior claim and the cited line range are both true, §7.1 is accurate in both
directions, and no auto-fail trips.

---

## Lens 1 — Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | §4.1 "Why the new work is allocation-free, path by path" walks both `process` bodies and both `set_parameter` bodies: fixed-size scalar state, borrowed buffers (`dsp-device-io.md:26` — verified), `validate_buffers` first (`io.rs:175–198` — verified, allocates nothing), `DspParameter::clamp` quoted character-exact against `parameter.rs:49–51`. The no-panic argument closes all three real holes (bounded indexing after `io.rs:187–193`; unreachable division, §4.3; `f64::clamp` behind a `const ParamSpec` that cannot hold `min > max`, `spectre-core/src/param.rs:77–79` — verified). Critically, §4.1 does **not** claim the structural lock scan covers it: `rt_guard.rs:289–320` reads four `spectre-audio` paths at `:293–298` joined to `env!("CARGO_MANIFEST_DIR")` at `:309`. I opened it — the claim that it cannot reach `crates/spectre-dsp/src/` is **true**, and the spec names the allocation guard at `:259` with its positive control at `:138–155` as its actual RT-001 evidence. | — |
| 1B. Control↔render communication | 3 | §4.4's "RT-002 disposition" states the correct answer for a slice that adds no traffic: no new lane; parameters ride decision 21's latest-wins lane (`decision-gates.md:45` — verified); notes ride the bounded FIFO whose per-quantum capacity is `MAX_NOTE_EVENTS_PER_BLOCK = 1_024` (`io.rs:52` — verified exact). Retired state is answered rather than dodged: neither device allocates, so the plan itself is what travels back on the existing reclaim lane. §4.4's state table names owner, thread, and lifetime for all five state classes. | — |
| 1C. Numerical containment | 3 | §4.3 step 1 places containment **before** the state update and gives the reason a reviewer can check: `contain_channel` (`spectre-graph/src/lib.rs:399–414`) operates on output buffers only and the plan applies it after a node has already updated its own state (`:512–522`) — I verified both, and the argument holds. `Gloam` is the first recursive device, so this is a real new hazard, correctly identified. The denormal-state flush reuses the plan's own predicate `*sample != 0.0 && sample.abs() < f32::MIN_POSITIVE` (`:407–410` — verified) rather than inventing a threshold (DEV-012). Injection tests are named: §5.1 test 7 (NaN/±Inf, no latch), test 8 (state reaches exact zero), and the `containment.rs` sibling to `healthy_native_devices_report_no_containment_activity` (`:327–328` — verified). §5.2 also correctly notes what is *not* testable in-plan and why. | — |
| 1D. Determinism | 2 | §5.2 reuses the **existing** FNV-1a walk via a private helper extracted from `spectre-offline/src/lib.rs:269–284` (seed `:271`, prime `:276` — all verified), satisfying 1D's "not a new one" structurally. §4.6 honestly scopes determinism to within-a-build because `sin`/`exp`/`tanh` are libm calls, and routes cross-platform hashes to Q5 rather than overclaiming. **Gap:** `existing_fixture_hash_is_unchanged` ("assert `render_vertical_slice` still returns the same report as before this slice") has no defined baseline. `crates/spectre-offline/tests/harness.rs` contains no golden hash constant; its regression gate is `plan_render_matches_hand_wired_chain` (`:58`), which recomputes the reference inline. So the proposed test either needs a literal §4.6 argues against, or is already covered. | Priority 2 item 1. |
| 1E. Graph and plan contract | 3 | §4.1's GRAPH-001 paragraph is correct and checkable: both devices are `AudioProcessor` implementations a plan owns, neither touches graph structure, and `CompiledPlan` still exposes no node or edge mutation API — `spectre-graph/src/lib.rs:416–417` says exactly that, verified verbatim. Nothing is recompiled per parameter change; decision 22's rejected option (b) (`decision-gates.md:47` — verified) is not proposed in any form. §4.2 notes the two devices are `Box<dyn AudioProcessor>` in the plan exactly as the four existing devices are. | — |
| 1F. Failure behavior | 3 | §3.6's E1–E8 table is the most complete failure model in the run so far, and every mechanism checks out: E1 const-eval panic (`parameter.rs:140–143` + `param.rs:77–79`); E3 recoverable `UnknownKey` per `dsp-device-io.md:92`; E4 non-finite → descriptor default (`param.rs:119–124`); E6 whole-node silence and `contaminated_nodes` (`graph/lib.rs:512–522`); E7 `validate_buffers` before device arithmetic (`io.rs:175–198`); E8 bounded one-quantum denormal exposure. The three binding rules — fail closed, counted not dropped, no new error vocabulary — are stated and honored: the spec adds no `ProcessError` or `PlanError` variant. | — |
| 1G. Test specification | 2 | Twenty tests across five real files, with real commands, and most would genuinely fail on regression: test 5's exact-`0.0`-plus-`is_sign_positive()` release assertion breaks under an exponential envelope; test 9's peak bound breaks if the coefficient escapes `[0, 1]`; test 12 puts the new descriptors through the existing `native_parameters()` iterator at `devices.rs:37` (verified) and the round-trip policy at `:69–70` (verified); test 8's arithmetic is correct (`damp_a ≈ 0.0794`, decay `≈ 0.9206`, `≈ 1 052` samples ≈ 4.1 quanta, 64 quanta ≈ 15×). **Four defects.** (a) §5.3's `select_device` does not exist — `grep -rn "fn select_device\|select_device("` over `crates/` returns nothing; the method is `open_device_in_shape` (`lib.rs:338`), and the test cannot compile as written. (b) 1D's baseline gap above. (c) Test 6 asserts "within 1 ULP per sample" over a 256-sample recursion without an error-accumulation argument, in a spec that argues that class of point everywhere else. (d) Test 7's setup says a block "containing" NaN/±Inf; the assertion only holds if that block contains *no* finite nonzero samples, otherwise the poisoned and fresh instances hold different legitimate state and the test fails against a correct device. | Priority 1 item 1; Priority 2 items 1–3. |

**Lens average:** 2.714 (19 / 7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, zero 0s

---

## Lens 2 — DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 2 | §3.2's primary flow is an audition loop — hold a note, sweep Lean, hear timbre move without pitch or phase moving — and §3.1 preserves context by adding no view, no modal, and no transport-bar element. The drill-in it rides (`lib.rs:342–343`) is the one `STATUS.md` records as preserving track selection. **Gap:** sketch → branch → grow are not addressed at all, and the spec never states in its own words that selection, zoom, and transport survive the new rows. Correct and functional; thin. | Priority 3 item 1. |
| 2B. Linked lenses | 3 | §3.1 puts both devices in Build and Shape as views over one `DeviceControl` (`lib.rs:57–63` — verified), and §4.4 states "No new state container, no new store, and no second source of truth." §3.3 adds the sharper version: exactly one value per parameter in exactly one place, with the device setter as the only way a value enters DSP state. Nothing forks per lens. | — |
| 2C. Modulation visibility | 3 | §3.3's PROD-002 paragraph is the right answer rather than the flattering one: neither device is a modulation source or destination, so every Shape value is a base value, and inventing a tri-state display now "would be a fake surface." The obligation it accepts is non-foreclosure — one value, one place, one insertion point for a later automation layer. Provenance is real: `OBS-AB12-AUTO-004` (`ableton-live-observations.md:127`) does say a manual change while not recording overrides automation, dims the LED, and offers Re-Enable Automation. PROD-002 sits at `requirements-ledger.md:63` as cited. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 adds no binding and ratifies none, citing the prohibition at `product-implications.md:96` ("a default shortcut map") — verified at that exact line — and pointing at `vision.md:41`'s context-scoped, searchable, remappable commands, also verified. Seven rows across two devices is the opposite of spreadsheet density; no cable surface is introduced. | — |
| 2E. Convergent-pattern grounding | 3 | The one genuine convergence on this surface is that per-voice duplication is the cost center and shared post-FX is the cheaper structure — `OBS-SR2-CPU-001` (`synth-modular-observations.md:54`) plus the file's own cross-cutting pattern 2. §0 follows it (one voice, one shared insert) while explicitly refusing the record's numbers. Where Spectre diverges — VCV's voltage-typed signal contract, `OBS-VCV-VOLT-001`/`-004` — the Appendix states the divergence and grounds it in accepted product direction at `vision.md:39` ("informed by — not copied from"), verified verbatim. | — |
| 2F. Differentiation | 3 | The Appendix's differentiation paragraph claims exactly one thing and disclaims the rest: every numeric bound published with its derivation and a re-open trigger before the device ships. It states outright this is "**not** a claim that these devices sound better than, or are more capable than, anything in the benchmark set. They are smaller than all of it, on purpose." That is 2F satisfied without the parity framing `vision.md:59` prohibits. | — |
| 2G. Benchmark evidence discipline | 3 | I opened every `OBS-` ID cited and each says what the spec claims, at the cited line. `OBS-AB12-MIX-008` (`:105`) does warn against on-stage track-delay changes for clicks/pops. `OBS-AB12-MIX-002` (`:99`) does describe a 32-bit float engine tolerating over-0 dB internally. `OBS-AB12-AUTO-004` (`:127`) is accurate. `OBS-PP-UNI-003` (`synth-modular-observations.md:37`) does record a polyphony limit plus oldest-and-quietest voice recycling. `OBS-VCV-VOLT-006` does say modules should output 0 on NaN/infinity. **Serum 2 is held to exactly the two permitted records** — `OBS-SR2-CPU-001` and `OBS-SR2-KB-001` at `:54–55`, and no claim beyond them appears anywhere. **Logic Pro appears zero times**, and the Appendix names the hole with the dossier's own status line (`logic-pro.md:10–11`, `draft` / `inventory-only` — verified). Named gap 1 (no citable record on any benchmark's internal device implementation) and gap 2 (no anti-aliasing record) are both correct against the corpus. | — |

**Lens average:** 2.857 (20 / 7)
**Lens pass:** Yes

---

## Lens 3 — Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | §6.4 3A maps the feature onto the R4 exit row at `current-milestone.md:84` — "One small original synth and one original effect ship as the alpha's voice" — verified verbatim at that line, with `vision.md:48`'s alpha bar also verbatim. Everything adjacent is deferred **by name and by slice**: track hosting → R4-4, clip playback → R4-5, persistence → R4-7, bounce equivalence → R4-8, QA protocol → R4-9. §7.4 repeats the discipline as a dependency table. | — |
| 3B. Non-goal respect | 3 | §6.4 3B walks `vision.md:53–59` (verified) and `current-milestone.md:77` (verified) item by item. No CLAP/LV2/AU hosting; explicitly no plugin-format authoring — both devices are plain Rust types reachable only through `AudioProcessor`; no preset format at all; no cloud, store, or video; and it names automation and modulation as R5+ per `current-milestone.md:77`. | — |
| 3C. Deliberately small first devices | 3 | This is the spec's spine, not a disclaimer bolted on. §0 is the first section and quotes decision 15 (`decision-gates.md:39`) verbatim — verified — alongside `dsp-device-io.md:107` ("They do not define the eventual flagship instrument or effect catalog"), also verbatim. The delivered scope: seven parameters, one voice, one oscillator, one pole, no unison, no filter bank, no LFO, no modulation matrix, no effect lanes, no wavetable, no sample player, no granular engine, no preset browser, no visualizer. §3.1 even refuses a waveform display on repaint-cost and accessibility-cost grounds and routes it to Q6. There is no drift toward the R11 flagship anywhere in 1,302 lines. | — |
| 3D. Originality | 3 | `grep -rniI "filament\|gloam"` over the whole repository returns nothing outside this spec — verified, so the §6.4 3D claim is true. Neither is a benchmark's device, style, or mode name. The parameter vocabulary (Lean, Rise, Fall, Damp, Depth, Track) is Spectre's own; the one reused word, `level`, is reused from Spectre's own `PULSE_PARAMETERS` (`source.rs:39–46` — verified). The DSP is written out in full — a two-segment piecewise-linear phase reparameterization and a level-tracked one-pole — so the "not transcribed" claim is checkable rather than assertable, which is what §6.2 says it is for. | — |
| 3E. Platform commitment | 3 | §4.6 is the live-risk answer decision 23 demands. No `cfg`, no intrinsic, no CPU control-register manipulation, no `unsafe`; the denormal flush is the same software FTZ-equivalent the project chose *because* it is portable, sourced to `current-milestone.md:59` — verified at that line. Every test in §5.1–§5.3 runs offline or against the null backend, so both devices are provable on either platform with no audio hardware. It then states the boundary: "This spec authorizes no Linux support claim", leaving decision 23's debt (`decision-gates.md:49` — verified) to R4-3. | — |
| 3F. Accessibility trajectory | 3 | §3.7 claims **no** screen-reader support and gives the reason, then argues only non-foreclosure: every one of the seven controls carries a backend name, unit, range, and default, which is what `dsp-device-io.md:117–118` requires (verified). It finds its own weakness — `Lean`, `Damp`, `Track` read poorly aloud — adds it to the M3 manual check, and routes a descriptor description field to Q7 ahead of decision 17's R4 audit (`decision-gates.md:41` — verified). No icon-only control, no color-only state, no drag-only gesture. | — |

**Lens average:** 3.000 (18 / 6)
**Lens pass:** Yes
**Auto-fail triggered:** No

---

## Lens 4 — Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | §7.1 was checked line by line against eleven source files and is accurate **in both directions**. Verified present at the cited lines: `PULSE_PARAMETERS` one parameter, default `0.2` (`source.rs:39–46`); the four-variant `Waveform` at `:103–109` with a getter at `:139–141` and no setter; naive shapes at `:143–157`; no envelope, `:202–209`; note-ID matching at `:188–193`, ignored non-matching off at `:198`, `AllNotesOff` at `:194–197`; `Gain` as one `f32` (`effect.rs:29–31`) with a clamped instantaneous setter (`:45–47`); `Saturator`'s `:119`, `:122–126`, `:127`, `:128`; `AudioProcessor` with exactly `io` and `process` plus `Send` (`io.rs:163–172`); `ProcessContext::new`'s four validations at `:82–84`, `:85–87`, `:88–90`, `:91–101`; `contain_channel` at `:399–414`; `render_plan` at `:204–285`; the four-entry snapshot fixture at `lib.rs:351–356`; `prototype`'s three devices at `:228–250` with subtitles verbatim at `:233`/`:240`/`:247`. Verified **absent** as claimed: `Filament`, `Gloam`, `FILAMENT_PARAMETERS`, `GLOAM_PARAMETERS` (repo-wide grep, zero hits); `AudioProcessor::set_parameter`; any envelope, filter, or smoothing on any shipped device; any in-crate `#[cfg(test)]` module in those four crates. The D-R3 discrepancy is stated exactly right — `dsp-device-io.md:94` and `:104` do claim `Gain` smooths, and the shipped `Gain` does not. Two off-by-one line ranges (`effect.rs:65–69` should start at `:64`; `io.rs:163–172` omits the trait's closing brace at `:173`); neither makes a false claim. | Priority 3 item 2. |
| 4B. Status vocabulary | 3 | §7.1 partitions Implemented / Absent / Gated and adds "Contract-versus-code discrepancy, carried not resolved." The `implemented`/`verified` distinction is respected — R4-2 is described as "spec'd and passed at 3.000; it is **not** implemented" (its scorecard composite is 3.000, confirmed) and R4-1 as "passed its spec gate at 2.950 with zero lines of implementation" (confirmed). The spec's own status is `proposed`; the twelve DEV rows are `proposed` until Jeff accepts. | — |
| 4C. Traceability | 3 | Essentially every normative claim carries a requirement ID, decision row, observation ID, or source path, and the ones I sampled at random all resolved. Decision rows land exactly: 1 at `:25`, 15 at `:39`, 16 at `:40`, 17 at `:41`, 21 at `:45`, 22 at `:47`, 23 at `:49`. Ledger rows land exactly: RT-001/002/003 at `:28–30`, CORE-002 `:47`, GRAPH-001 `:55`, PROD-002 `:63`, PROD-003 `:64`, format `:21–22`, known gaps `:19`. `README.md:23–35` is the conflict precedence and `:68–70` is the working rule, both as cited. | — |
| 4D. Honest gaps | 3 | Three known gaps in the header, nine open questions, and the questions are the real ones rather than decorative: Q1 routes smoothing to D-R3 without resolving it, Q3 routes the ledger family as a proposal to an accepted document, Q5 names cross-platform libm bit-equality as a threat to R4-8's framing, and Q8 discloses that the RT-001 structural lock scan has never covered any device module — a gap the spec found itself and had no obligation to surface. §4.7 states plainly that nothing has been measured because neither device exists. | — |
| 4E. Evidence commands | 3 | The workspace gate is quoted character-exact as `criteria.md` 4E names it. All five targeted commands resolve to files that exist: `spectre-dsp --test devices`, `spectre-graph --test containment`, `spectre-offline --test harness`, `spectre-audio --test rt_guard`, `spectre-app --test app_model`. No command is invented, and §5 does not claim the new test bodies already run. | — |
| 4F. No fake surfaces | 3 | The prohibition is honored in the strongest form available: "**Stated plainly and kept stated until it is false: `./spectre` produces no sound of any kind today**", sourced to `NEXT.md:19` verbatim and corroborated by `STATUS.md` ("Play changes the model's transport state without producing sound"). §5.4 states that none of the manual protocol can be executed today. §7.4 makes R4-2 a hard blocker precisely because devices with dead controls would be a fake surface. §3.7 claims no screen-reader support. §6.3's AF-6 self-audit is real, and I found no promotional phrasing about repository state. | — |

**Lens average:** 3.000 (18 / 6)
**Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Result | Basis |
|---|---|---|
| AF-1 — Contradicting accepted authority | **Pass** | No Accepted row is contradicted. The two changes to accepted documents — a `DEV` ledger family and an addition to `dsp-device-io.md`'s device list — are routed as proposals in §7.2 items 11–12 and §8 Q2/Q3, never asserted. §8 Q1 explicitly declines to resolve D-R3. |
| AF-2 — Unbacked implementation claims | **Pass** | §7.1 distinguishes implemented / absent / gated and every claim resolved in one `Read`, in both directions. The one wrong identifier (`select_device`, §5.3) is a test-plan naming slip whose behavior and cited line range are both correct, not a claim that unwritten code exists. |
| AF-3 — Realtime discipline violation | **Pass** | No allocation, lock, I/O, logging, formatting, or panic on any callback-reachable path; §4.1 argues each by construction and §5.2 names the allocation-guard evidence. Denormals flush (DEV-012, same predicate as `graph/lib.rs:407–410`); NaN/Inf isolate to silence (E5, E6). |
| AF-4 — Borrowed numeric limits | **Pass — checked hard.** | All twelve bounds carry Spectre-internal derivations that I verified at source: `lean` `0…1` is the phase domain (`source.rs:96`, `:205`); `lean` default `0.5` is provably the identity point; `rise/fall/track` min `1.0 ms` = eight sample periods at `MIN_SAMPLE_RATE = 8_000` (`spectre-audio/src/lib.rs:25` — verified), max `1000.0 ms` = two beats at the default project's own `TempoMap::constant(120.0)` (`spectre-offline/src/lib.rs:157` — verified); defaults `31.6` and `632.5` are geometric midpoints (`sqrt(1000) = 31.6228`, `sqrt(400 000) = 632.4555` — both arithmetically correct); `level` default `0.2` is Spectre's own `PULSE_PARAMETERS[0]`; `damp_hz` `20…20 000` is Spectre's own `TONE_PARAMETERS[0]` (`source.rs:28–35` — verified); voice count `1` matches the shipped `Option<(u32, u8)>` at `source.rs:117`; filter order `1` is argued from stability; the flush threshold is the existing `f32::MIN_POSITIVE`. **Corpus cross-check:** the audible band `20 Hz – 20 kHz` does appear in the corpus as documented product behavior (`OBS-OZ-EQ-001`, `ozone-observations.md:126`; `OBS-OZ-DEQ-001`, `:252`), so I investigated rather than assumed coincidence — the spec derives it from Spectre's own shipped descriptor, never cites Ozone, and Ozone is not in the benchmark set. No corpus record matches `1…1000 ms`, `31.6`, `632.5`, `0.2`, one voice, or one pole; grep for those values across `docs/` returns nothing. PROD-003's record obligation is discharged the way `requirements-ledger.md:64` requires: §7.2 item 11 schedules DEV-001…DEV-012 into `requirements-ledger.md` in the modified-files list, in the ledger's own `:21–22` format. |
| AF-5 — Conclusions the evidence does not support | **Pass — checked against the prohibiting text.** | `product-implications.md:90–104` forbids, among others, "a final native-device list", "a synthesis architecture or modulation limit", and "a default shortcut map" (verified verbatim at `:94`, `:95`, `:96`). §0 tabulates four of these and refuses each: it names two devices for R4 while stating no catalog, no naming scheme, no families, no count (Q2); it declares `Filament`'s structure "**this device's implementation**, not Spectre's synthesis architecture", proposes no modulation source, routing, or limit (Q4); §3.4 adds and ratifies no binding; §4.7 gives operation counts and asserts no time, CPU, or headroom target, citing `:97`. Even the device-list addition in §7.2 item 12 preserves `dsp-device-io.md:107`'s disclaiming sentence verbatim for exactly this reason. |
| AF-6 — Optimistic language | **Pass** | §6.3's self-audit holds under inspection. Every audible outcome is marked conditional on R4-1/R4-2; aliasing, libm variance, and the absence of measurement are stated as limitations rather than smoothed. |
| 3B = 0 | **Pass** | No non-goal is proposed. |

---

## Feasibility Check

Read against source, not against §7.1.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | `DeviceIo`, `DeviceClass`, `DspParameter`, `ParamUnit` all exist and are exported (`spectre-dsp/src/lib.rs:11–19`). Every unit the spec uses — `Linear`, `Hertz`, `Milliseconds`, `Percent` — is a real `ParamUnit` variant (`spectre-core/src/param.rs:11–18`). `parameter()` is `pub(crate)`, so the spec's claim that a new module *inside* `spectre-dsp` can build descriptors and one outside cannot is correct. |
| API/interface changes are feasible with current architecture | ✓ | Two new modules plus two `mod`/`pub use` lines beside the four at `lib.rs:6–9`. `AudioProcessor::set_parameter`'s signature matches R4-2's declaration character-for-character (`R4-2-runtime-parameter-seam.md:649`). |
| Views/screens fit current navigation pattern | ✓ | Verified at `crates/spectre-app/src/main.rs:364–376`: each Shape row is `egui::Slider::new(..., descriptor.minimum()..=descriptor.maximum())` with a `Reset` button and hover text naming minimum and maximum — exactly what §3.2 step 3, §3.4, and §5.4 M3 describe. Two appended `DeviceControl::from_descriptors` entries need no new widget. |
| Dependencies are available and version-compatible | ✓ | None added. `spectre-audio`'s test already imports `spectre_dsp` (`rt_guard.rs:19`), so §5.2's guard test needs no manifest change. |
| Platform/renderer requirements are realistic | ✓ | No `cfg`, no intrinsic, no `unsafe`; both devices are provable with no audio hardware on either platform. |
| Test strategy is executable with current infrastructure | ✗ | Four defects, one blocking: §5.3's `select_device` does not exist anywhere in `crates/` (the method is `open_device_in_shape`, `lib.rs:338`), so that test cannot compile as written. Plus the undefined baseline in `existing_fixture_hash_is_unchanged`, test 6's unargued 1-ULP tolerance over a 256-sample recursion, and test 7's ambiguous poison-block composition. All five named test files exist and all five commands resolve. |
| Performance budget is realistic for target hardware | ✓ | Stated as operation counts with no time target, correctly refusing a prohibited conclusion. The claim that one `sin` per sample is the same order as the shipped `Saturator`'s `tanh` per sample per channel is true (`effect.rs:127`). |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares R4-2 as a hard blocker for the setters and R4-1 for audibility, and correctly states that §5.1–§5.3 run offline regardless. Both dependencies' status ("zero lines implemented") is accurate. |

**Feasibility verdict:** Feasible with caveats
**Caveats:** One of roughly twenty tests cannot compile as written because it names a method that does not exist; the fix is a one-word substitution to a method the spec already cites by line. Three further tests are underspecified rather than infeasible. Nothing in the DSP, the descriptors, the app delta, or the offline refactor is blocked.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.714 | 35% | 0.950 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 3.000 | 20% | 0.600 |
| **Composite** | | | **2.864** |

**Pass conditions (from `criteria.md`, binding):**
- [x] Composite ≥ 2.30 — **2.864**
- [x] Every lens average ≥ 2.00 — 2.714 / 2.857 / 3.000 / 3.000
- [x] No criterion scores 0 — lowest score awarded is 2
- [x] At most two criteria score 1 — **zero** criteria score 1
- [x] All auto-fail rules pass — AF-1 through AF-6 checked individually above
- [x] Feasibility rule — §7.1 was read against source and is accurate in both directions; no source path the spec cites misdescribes the file at the cited lines
- [x] Reviewer personally opened every source path the spec cites and every `OBS-` record it names

**All conditions met:** Yes → **PASS**

---

## Remediation Brief

The spec passes; these are corrections to make before implementation, not conditions of the pass.

### Priority 1 — Must fix before implementation

1. **§5.3, `opening_a_new_device_focuses_shape` names a method that does not exist.** The spec
   describes calling `select_device` on `filament`'s instance ID. `grep -rn "fn select_device\|select_device("`
   over `crates/` returns nothing. The drill-in the spec correctly cites at
   `crates/spectre-app/src/lib.rs:342–343` belongs to `AppModel::open_device_in_shape`, declared at
   `:338` with signature `pub fn open_device_in_shape(&mut self, id: ObjectId) -> Result<(), &'static str>`.
   Replace the name and note that the method returns `Result`, so the test asserts `Ok(())` before
   checking `lens == Lens::Shape` and `selected_device_id()`. Nothing else in the test changes.

### Priority 2 — Should fix for quality

1. **§5.2, `existing_fixture_hash_is_unchanged` has no defined baseline.** "The same report as
   before this slice" has no source: `crates/spectre-offline/tests/harness.rs` contains no golden
   hash constant, and its existing regression gate `plan_render_matches_hand_wired_chain` (`:58`)
   recomputes its reference inline via `hand_wired_report` (`:112`). Either (a) state that the test
   pins a literal hash captured at implementation time and reconcile that with §4.6's argument
   against checked-in golden hashes — the argument was about cross-platform equality, and a
   same-machine regression pin is a different claim — or (b) state that the existing hand-wired
   comparison already covers the extraction and drop the new test.
2. **§5.1 test 6's "within 1 ULP per sample" is asserted without an accumulation argument.** The
   device stores `damp_a` as `f32` after computing it in `f64`; if the test's reference runs in
   `f64`, the coefficient difference compounds across 256 recursive samples and can exceed one
   `f32` ULP. Either state that the reference replicates the same `f32` arithmetic (making the
   assertion bit-equality, which is stronger and stabler) or give the tolerance a derivation, as
   §4.3 does for every other numeric claim.
3. **§5.1 test 7's poison block is ambiguous and the assertion depends on the ambiguity.** The
   assertion that the poisoned instance's second block equals a fresh instance's first block holds
   only if the poison block contains no finite nonzero samples — otherwise both instances hold
   different but legitimate state and the test fails against a correct device. State that the
   injected block is composed **entirely** of `NAN`, `INFINITY`, and `NEG_INFINITY`, or replace the
   equality with the property actually wanted: every sample of the following clean block is finite
   and the state is not latched.

### Priority 3 — Consider for excellence

1. **§3.1/§3.2 do not state the loop-first context guarantee in the spec's own words (2A).** The
   drill-in preserves track selection, the transport is untouched, and no view is added — the spec
   relies on the reader inferring it. One sentence naming selection, zoom, and transport survival
   across opening a device would close the only soft criterion in the review.
2. **Two line ranges start one line late.** `effect.rs:65–69` is cited for `Gain`'s non-finite
   mapping; the `if input.is_finite()` guard is at `:64`. `io.rs:163–172` is cited for
   `AudioProcessor`; the trait's closing brace is at `:173`. Neither makes a false claim, and both
   are worth correcting only because the rest of the spec's citations are line-exact.
3. **§4.3's citation of `dsp-device-io.md:112` paraphrases.** The contract line reads "Silence
   remains finite silence through every effect"; the spec renders it as "exact silence" and applies
   it to an instrument. The stronger property the spec actually delivers — bit-exact `0.0`, not
   merely finite — is its own, and is worth claiming as such rather than sourcing to a line that
   says something weaker about a different device class.

---

**End of scorecard.**
