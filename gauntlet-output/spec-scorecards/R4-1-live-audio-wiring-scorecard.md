<!--
Author: Jeff
Date: 2026-08-15
Description: Blind verification scorecard for the R4-1 live-audio-wiring spec, iteration 1
Notes: Every source path the spec cites was opened and checked against the file at branch
  rename/geist-to-spectre. One assertion about the codebase is false and one stated
  ordering guarantee does not follow from its own premises; both are recorded below.
-->

# Scorecard: Live Audio Wiring

**Feature ID:** `R4-1` (`live-audio-wiring`)
**Spec file:** `gauntlet-output/specs/R4-1-live-audio-wiring.md`
**Reviewer agent:** blind verification agent, R4-1
**Date:** 2026-08-15
**Spec iteration reviewed:** 1

---

## Verdict: FAIL

**Summary:** This is the strongest spec I have graded in this repository. Roughly seventy
source citations were opened individually and verify to the exact line — including
verbatim UI string literals, the FNV-1a offset basis and prime, the `173 / 0 / 0.990`
macOS qualification row, all six `OBS-` records, and the exact set of nine Ableton
observation categories. It fails on two things, both narrow and both fatal by design of
`criteria.md`: **§4.1 and §7.2 assert that `rt_guard.rs:293–298` scans `bridge.rs`,
`control.rs`, `spsc.rs`, and `midi.rs` — the file actually scans `null.rs`, not
`midi.rs`, and the spec's own §4.1 table modifies `null.rs` while the same paragraph
claims the scanned four are untouched.** That is a false statement about the codebase
carrying the spec's central RT-001 safety argument, and it triggers AF-2 and the
feasibility rule. The single most important fix is one sentence in §4.1 and one bullet
in §7.2.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 2 | §4.1 correctly isolates the only callback-reachable path — the closure `move \|mut block\| bridge.render(&mut block)` — and §5.1 test 11 names the RT-001 guard covering it, correctly noting the allocator must be re-declared because `#[global_allocator]` is per-test-binary (pattern verified at `crates/spectre-audio/tests/rt_guard.rs:38–64`). **But the structural argument is factually wrong:** §4.1 names `midi.rs` as one of the four RT-scanned modules; `crates/spectre-audio/tests/rt_guard.rs:293–298` lists `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs`. The §4.1 table three rows above modifies `null.rs`. | See Priority 1 item 1. |
| 1B. Control↔render communication | 2 | §4.4's state table names owner, thread, and lifetime per state; §3.6 E7 gives the strict-FIFO overflow behavior (`TransportLaneFull`/`NoteLaneFull`, counted, UI unchanged); §4.3's `reclaim()` names off-thread reclamation once per UI frame. All verified against `control.rs:17–19`, `:248–281`, `:314–324`. Two gaps: (a) §4.3's ordering guarantee is invalid — see Priority 1 item 2; (b) §3.2 step 6 flips UI state "only if **both** sends returned `Ok`" while §4.4's binding rule conditions only on the transport send, and `start_audition` (§4.3) returns one `Result` covering both, leaving the transport-Ok/note-Err case unspecified. | See Priority 1 item 2 and Priority 2 item 1. |
| 1C. Numerical containment | 3 | R4-1 adds no DSP node, and the spec handles that correctly rather than skipping it: §7.2 "no DSP is written"; §3.3 item 8 surfaces `contaminated_nodes` and `frame_capacity_rejections`; §5.1 test 10 asserts `contaminated_nodes == 0`; Appendix A cites `OBS-VCV-VOLT-006` as RT-003's recorded provenance and states R4-1 inherits `CompiledPlan`'s containment unchanged. Verified at `spectre-graph/src/lib.rs:401` (`contain_channel`) and `bridge.rs:234–247`. | — |
| 1D. Determinism | 3 | §5.1 test 5 names the existing comparison method and forbids a new one, quoting offset basis `0xcbf2_9ce4_8422_2325` and prime `0x0000_0100_0000_01b3` — both verified verbatim at `spectre-offline/src/lib.rs:271,276` and `spectre-audio/tests/bridge_plan.rs:81,87`. It adds the anti-vacuity guard (`peak > 0.0`) that the existing test uses at `bridge_plan.rs:120`. Test 6 covers repeat determinism. I independently confirmed the hash equality is achievable despite the live plan's `max_frames` of 512 vs the offline 256, because `CompiledPlan::last_output` slices to `rendered_frames` (`spectre-graph/src/lib.rs:536–543`). | — |
| 1E. Graph and plan contract | 3 | §4.1 keeps `AppModel` audio-free and renderer-neutral with no new field; §4.3 splits `build_engine_parts` from `open_default` so the plan is compiled on the app thread before any device is touched, and §5.1 test 1 asserts that construction order. No per-parameter recompilation is proposed anywhere; §7.1's Gated block correctly records decision 22 as design-accepted with implementation at R4-2 and the bridge counting `parameters_pending` (verified `bridge.rs:181–186`). | — |
| 1F. Failure behavior | 3 | §3.6's nine-row table is fail-closed throughout, with two binding rules stated normatively: no error is formatted on the audio thread, and no threshold-derived alarm is specified because Spectre owns exactly one hardware measurement (decision 16 / PROD-003, both verified). E1's refusal to fall back to `NullBackend` — "a null stream that renders into a discarded buffer would be a fake 'running' surface" — is the correct call. E9 correctly identifies the oversized-block refusal as already shipped (`bridge.rs:167–173`). | — |
| 1G. Test specification | 2 | Thirteen tests, each with setup, assertion, and a named edge case; test 11 is a real RT-001 guard with a positive control; test 5 cannot pass vacuously. **Two are not executable as written.** Test 3 reads "`parts.bridge`'s plan capacity … through `CompiledPlan::max_frames`", but `RenderBridge` (`bridge.rs:119–290`) exposes only `new`, `telemetry()`, `transport()`, `render()` — no plan or `max_frames` accessor — and §4.1/§7.2 forbid modifying `bridge.rs`, while `EngineParts` (§4.2) carries no plan handle. Test 9 asserts `LiveEngine::state()` transitions across a pump, but `LiveEngine` holds `Box<dyn AudioStream>`, `pump()` is inherent to `NullStream` and absent from the trait (`lib.rs:181–196`), and §4.3 declares no constructor that gives a test both a `LiveEngine` and a pumpable stream. Also, no test exercises the same-block `On` → `AllNotesOff` ordering hazard. | See Priority 1 items 3 and 4. |

**Lens average:** 2.57 (18/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | §5.1 test 8 asserts an engine open failure leaves lens, selected track, and selected device unchanged, and names the rule: "an engine event never disturbs selection context." All nine §3.6 rows carry "Data loss: no", none uses a modal, and §3.2's failure branch keeps the rest of the app fully usable. The audition voice adds the audition leg; branch and grow are correctly left to R4-4/R4-5 and named as such in §7.4. | — |
| 2B. Linked lenses | 3 | §3.1 places engine state in the transport bar with a stated reason — "the only region visible from all four lenses, which keeps engine state one global fact rather than a per-lens fork" — citing `vision.md`'s "One project, linked lenses". §4.4 keeps one `Option<LiveEngine>` on the app struct rather than per-lens state. Nothing forks. | — |
| 2C. Modulation visibility | 3 | R4-1 adds no automation or modulation, and the spec handles the parameter-honesty hazard directly: §3.3 rewrites the Shape header to say edits do not reach live audio until R4-2, calling this "required, not cosmetic: without it, moving a slider while hearing sound implies a connection that does not exist," and §5.4's Honesty-check row makes it a manual gate. That is the right answer for this slice. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 explicitly refuses to "extend, ratify, or document a default shortcut map", citing `workflow-field-study/product-implications.md` §"Prohibited conclusions at current evidence level" — verified: line 96 of that file lists "a default shortcut map". It assigns no binding to `Retry engine`, relying on egui's standard focus handling, and correctly labels the existing `Space`/`1`–`4` bindings (verified `main.rs:425–437`) as prototype scaffolding it neither ratifies nor removes. Calm UI: §3.5 adds no animation and no new timing constant; §3.3 item 8 hides permanently-zero counters. | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A both follows and diverges with reasons, and every record verifies exactly. `OBS-PP-ARCH-001` ("At least one output module is required to produce sound") → Spectre converges by construction. `OBS-AB12-ROUTE-001` (§17.1, "Auto … monitor only while armed and not playing clips") → Spectre keeps the stream open across transport changes rather than opening on Play, a load-bearing design choice actually derived from the record. `OBS-AB12-MIX-009` (§18.9, six-step per-track CPU meter) + `OBS-SR2-CPU-001` → Spectre **diverges** on presentation, with the stated reason that it has no track-level DSP attribution and inventing one would be a fake surface. | — |
| 2F. Differentiation | 3 | Appendix A's final gap states what Spectre does that no benchmark record covers — exposing per-block callback headroom and containment counts — and immediately declines to claim superiority on evidence, grounding it in the vision's published realtime contract instead. No parity-as-completeness claim appears anywhere. | — |
| 2G. Benchmark evidence discipline | 3 | Exemplary, and the whole gap block was independently verified. The Ableton corpus is 85 unique `OBS-AB12-` records (exact count confirmed) spanning exactly the nine prefixes the spec lists — ARR, SES, CLIP, WARP, LAUNCH, ROUTE, MIX, REC, AUTO — with no preferences or audio-device category (confirmed). Logic Pro has zero behavioral records (confirmed). Serum 2 has exactly `OBS-SR2-CPU-001` and `OBS-SR2-KB-001` with a `blocked-source-gap` dossier (confirmed), and only the CPU record is used, only for a narrow claim. The spec states that every device-open, format-negotiation, and failure-surface decision is Spectre's own and records it as a research need in the header's Known gaps. | Cross-reference fix only — see Priority 2 item 2. |

**Lens average:** 3.00 (21/7)
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | R4-1 is verbatim slice 1 of `docs/status/NEXT.md` ("Wire `./spectre` to the qualified backend so Play produces sound through the existing compiled plan… No new render path, no new DSP") and the first exit-evidence row of `docs/06-plans/current-milestone.md`. §6.4 3A names track model, clip, device, and persistence and leaves each to its own leaf; §7.4 states what blocks what. Q6 surfaces the seam-additions bundling as a scoping question for Jeff rather than burying it. | — |
| 3B. Non-goal respect | 3 | No hosting, plugin authoring, cross-DAW compatibility, cloud service, or video scoring appears. Recording is *removed* from the surface rather than merely omitted — §3.3 item 3 disables the Record button with a reason, which is stronger than silence, and matches `current-milestone.md` §Non-goals placing recording at R5. | — |
| 3C. Deliberately small first devices | 3 | No device is added or modified. §3.2 explicitly declines to give `PulseInstrument` an amplitude envelope even though it would make the audition prettier, assigning it to R4-6 under decision 15, and instead warns in copy and in §5.4 that the audition will click. Verified: `spectre-dsp/src/source.rs:202–209` is a hard note gate with no envelope. | — |
| 3D. Originality | 3 | No reference product's numbers, layout, or naming is transcribed. Both new constants carry Spectre-derived rationale rows in §4.2, and the one adopted number — 256 frames — is Spectre's own prior measurement, confirmed at `lifecycle_health.rs:23` (`FRAMES = 256`) and `:264` (`StreamConfig::stereo(48_000, FRAMES)`). AF-4 does not trigger. | Ledger row scheduling — see Priority 2 item 3. |
| 3E. Platform commitment | 3 | One code path through the `AudioBackend` trait; nothing in `engine.rs` names cpal, CoreAudio, or ALSA. The Linux gap is stated three times and no Linux claim is made, matching decision 23 exactly. The strongest evidence that Linux was designed *for* rather than mentioned: `default_sample_rate` exists specifically so a 48 kHz assumption is not baked in for a device locked to 44 100, and Q4 names `BufferSize::Fixed` (verified `cpal_backend.rs:129`) as the concrete ALSA risk. | — |
| 3F. Accessibility trajectory | 3 | §3.7 correctly declines to claim screen-reader support and states the verified reason: `crates/spectre-app/Cargo.toml` builds eframe with `default-features = false` and only `default_fonts` and `glow` (confirmed exactly), so no accessibility feature is enabled. It names the right-to-left focus-order hazard (`main.rs:78`, confirmed) and commits to recording any mismatch for decision 17's R4 audit rather than papering over it. No icon-only control, no color-only state — state is carried by words. | — |

**Lens average:** 3.00 (18/6)
**Lens pass:** Yes
**Auto-fail triggered:** No for this lens (3B is not 0; AF-4 does not trigger). AF-2 triggers under Lens 4 — see the auto-fail roll-call below.

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | 2 | §7.1 itself is accurate and every row is checkable in one `Read`, with a correct implemented / prototyped / absent / gated partition. Verified exactly: `main.rs:79` `ENGINE OFFLINE`, `:80` `CPU —`, `:75–77` the three literals, `:330` the Build footer string verbatim, `:443` the 250 ms cadence, `:508` the 1060 px minimum, `:37` `dark_mode = true`, `:9–15` the color constants, `:78` the `right_to_left` layout, `:72–73` the enabled Record button; `lib.rs:268–275` `toggle_play`, `:348–383` the four-entry snapshot; the exact `spectre-app` dependency list; `spectre` line 9; `cpal_backend.rs:129/139–141/154/163–168`; `bridge.rs:20/157/167–173/175–176/181–186`; `control.rs:17–18/314–324`; `audio/lib.rs:22/25–28/47–53/181/198/199–216`; `null.rs:38`; `io.rs:152/154/163`; `source.rs:194/202–209`; `graph/lib.rs:12/324/401/456/536`; `offline/lib.rs:178–334` and both FNV constants; `lifecycle_health.rs:22` and `:269` (which is character-exact but for a `: RenderBlock` annotation); `rt_guard.rs:24/30–36/38–64`; `bridge_plan.rs:19`. **Against that, one claim is false** (the RT-scan module list, Priority 1 item 1) and four line ranges are imprecise (Priority 3). | See Priority 1 item 1 and Priority 3. |
| 4B. Status vocabulary | 3 | Header `Status: proposed` is correct for an unaccepted spec. §7.1 partitions cleanly into implemented-and-verified / implemented / prototyped / absent / gated. §7.2's final bullet applies `docs/README.md`'s distinction correctly and unprompted: "Status must move to `implemented`, not `verified`, until the manual protocol and a hardware re-run pass." | — |
| 4C. Traceability | 2 | Nearly every normative claim carries a requirement ID, decision row, `OBS-` ID, or source path, and all of them resolve. Three defects: (a) the header's Known gaps points the reader to "§2G-adjacent notes below" — there is no §2G in this document; the content is in Appendix A, so the pointer resolves to nothing; (b) decision 16 and PROD-003 require every numeric bound's rationale to be recorded *in `requirements-ledger.md`*, and §7.2's modified-files list omits that file, so the two constants' mandated ledger rows are unscheduled; (c) §4.3 cites the ordering-key tuple to `io.rs:152`, which is `pub fn rank`; the tuple is at `io.rs:96`. | See Priority 2 items 2 and 3, Priority 3. |
| 4D. Honest gaps | 3 | Eight open questions, each with an explicit "blocks" pointer, several genuinely hard: Q1 identifies that a transport gate would cut reverb tails, conflict with `OBS-AB12-ROUTE-001`, and require editing eight existing R3 tests; Q2 correctly says moving the open off-thread needs a decision row and not a spec assertion; Q3 admits the fixture chain is assembled in four places; Q8 states there is no output limiter anywhere in the chain and that changing the audition level would break test 5's equality. §1.2 states the no-sound reality in bold, and §7.1's Gated block states the consequence that a Shape slider still will not change live audio after this slice ships. | — |
| 4E. Evidence commands | 2 | The workspace gate is quoted exactly as `criteria.md` and `STATUS.md` name it, character for character. Every feature-scoped command exists and runs, including the hardware drill's exact invocation as `current-milestone.md` records it. One defect: §5's lead sentence, "Every command below is real and runs today," is false for `cargo test -p spectre-app --test live_engine`, whose test file §5.1 immediately identifies as new. | See Priority 2 item 4. |
| 4F. No fake surfaces | 3 | The best-answered criterion in the spec. The `Opened` vs `Running` split — pinned by §5.1 test 9 and made normative in §6.3 ("the word 'running' may only appear when `blocks_rendered > 0`") — is a genuine invention. E1 refuses a `NullBackend` fallback as a fake running surface; the Record button is disabled; the frozen `001 · 01 · 000` becomes `—`; the Shape copy states the parameter gap; §3.6 refuses to derive a health alarm from n=1; §5.4 makes the honesty check a manual gate. | — |

**Lens average:** 2.50 (15/6)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, no 0s

---

## Auto-fail roll-call

| Rule | Triggered | One-line finding |
|---|---|---|
| AF-1 — Contradicting accepted authority | **No** | The spec amends no accepted row. Decisions 1, 8, 15, 16, 17, 19, 20, 21, 22, 23 and RT-001/002/003, GRAPH-001/002, PROD-003 were each read and the spec's characterizations match; §8 routes every would-be change (Q2's `Send` bound, Q5's playhead) to Jeff rather than asserting it. |
| AF-2 — Unbacked implementation claims | **YES** | §4.1 and §7.2 describe `midi.rs` as one of the four modules scanned by `rt_guard.rs:293–298`; that cited path contains `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs`. The path does not contain the claim. |
| AF-3 — Realtime discipline violation | **No** | Nothing new is placed on a callback-reachable path. The render closure is unchanged, both new trait methods are app-thread-only (`audio/lib.rs:198` documents the trait that way) and return constants or a relaxed atomic load, and control traffic stays on the accepted bounded wait-free lanes with counted overflow and off-thread reclaim. |
| AF-4 — Borrowed numeric limits | **No** | `ENGINE_BUFFER_FRAMES = 256` derives from Spectre's own macOS drill; `ENGINE_PLAN_FRAME_MARGIN = 2` is argued from unproven-absent oversized blocks and priced at 6,144 B. Neither comes from a reference product; both carry rationale in §4.2. (The ledger row PROD-003 also requires is unscheduled — that is a 4C traceability defect, not AF-4.) |
| AF-5 — Conclusions the evidence does not support | **No** | §3.4 explicitly refuses a default shortcut map and cites the prohibiting section, which I verified lists exactly that. No final device list, synthesis architecture, gesture/time budget, interface model list, monitoring-latency threshold, platform/backend order, frequency score, or archetype promotion appears. |
| AF-6 — Optimistic language | **No** | The register is conspicuously non-promotional: §1.2 bolds "`./spectre` currently produces no sound of any kind", §3.2 warns the audition will click, §4.6 refuses any Linux claim, and §6.3's self-audit is accurate on every line I checked. |

---

## Feasibility Check

Every source file cited by the spec was opened and checked against the claim.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every existing type the spec builds on is real: `AudioBackend`/`AudioStream`/`StreamConfig`/`BackendError`/`DeviceInfo` (`audio/lib.rs:31–216`), `NullBackend::open_null_output` (`null.rs:38`), `RenderBridge`/`BridgeTelemetry` (`bridge.rs`), `ControlSender`/`ControlError` (`control.rs`), `DeviceParameterSnapshotError` (`app/lib.rs:175`), `CompiledPlan::max_frames` (`graph/lib.rs:441`), `GraphError` (`graph/lib.rs:45` — §3.6 E6 names it correctly), `NoteEventKind::AllNotesOff { channel: Option<u8> }` (`io.rs:46`). The nine new `engine.rs` types are fully specified with field lists. |
| API/interface changes are feasible with current architecture | ✓ | **Both claimed absences are genuine, contrary to the possibility that an existing API already covers them.** `AudioBackend` (`audio/lib.rs:199–216`) has exactly `name`, `output_devices`, `default_output_device`, `open_output` — no format query; `DeviceInfo` (`:47–53`) carries only `id`, `name`, `max_output_channels`, `is_default`, so a device's sample rate is genuinely unreachable. `error_count` is inherent to `CpalStream` (`cpal_backend.rs:163–168`) and absent from the `AudioStream` trait (`:181–196`), so it is genuinely unreachable through `Box<dyn AudioStream>`. Both seam additions are justified. |
| Views/screens fit current navigation pattern | ✓ | No new screen. The three modified regions exist as described: `transport()` at `main.rs:50–84`, the Build footer label at `:329–333`, the Shape header label at `:337–342`. The 62 px fixed height (`:52`) and 1060 px minimum width (`:508`) are correct, so the truncation constraint in §3.4 is a real constraint. |
| Dependencies are available and version-compatible | ✓ | `spectre-audio` and `spectre-graph` are workspace members; `cpal 0.15.3` is already a `spectre-audio` dependency under a default-on `cpal-backend` feature, exactly as §4.5 describes. The dev-dependency cycle is real — `spectre-offline`'s `[dev-dependencies]` does contain `spectre-app` — and it resolves cleanly, because `spectre-offline`'s *lib* dependencies are only core/dsp/graph/project, so no lib-level cycle forms. The spec's hedge plus named fallback is correct. |
| Platform/renderer requirements are realistic | ✓ | With one correction in the spec's favor: §4.6 flags a possible `Send` bound on `eframe::App` as a build risk. `eframe 0.32.3` declares `pub trait App` with no `Send` bound (`epi.rs:137`) and `AppCreator` boxes `Box<dyn 'app + App>` (`:49–50`), so a non-`Send` `LiveEngine` can be a field of the app struct. The risk resolves in the spec's favor; the hedge is over-cautious, not wrong. |
| Test strategy is executable with current infrastructure | ✗ | Eleven of thirteen tests are executable. **Test 3 is not:** `RenderBridge` exposes no plan or `max_frames` accessor (`bridge.rs:119–290` — only `new`, `telemetry`, `transport`, `render`), `EngineParts` carries no plan handle, and §4.1/§7.2 forbid modifying `bridge.rs`. **Test 9 is not:** `pump()` is inherent to `NullStream` and absent from the `AudioStream` trait, `LiveEngine` holds `Box<dyn AudioStream>`, and §4.3 declares only `open_default` as a constructor, so no test can hold both a `LiveEngine` and a pumpable stream. The homes for tests 12–13 (`backend_seam.rs`) and the smoke assertions (`smoke_cli.rs:17–19`, exactly `lens=Arrange`, `tracks=1`, `selected_device=Pulse(pulse)`) are correct. |
| Performance budget is realistic for target hardware | ✓ | Recomputed from source and correct. `compile` allocates `channel_count × max_frames` f32 (`graph/lib.rs:324`); the fixture's three nodes at two channels each give 6, so 6 × 512 × 4 B = 12,288 B and the margin's share is 6,144 B — both figures match §4.2 and §4.7 exactly. Lane depths verify (`control.rs:17–19`: 1,024 / 64 / 32), and the total lands just under the stated 64 KB. `EngineHealth` has exactly nine fields, matching the "nine relaxed atomic loads" claim. |
| No undeclared dependency on unbuilt features | ✓ | Every composed component is implemented and tested today. §7.4 states plainly that nothing blocks authorship or macOS implementation, that R4-3 blocks only the Linux *claim*, and that R4-5 will replace the audition voice rather than accumulate alongside it. |

**Feasibility verdict:** Infeasible as written — two of thirteen tests cannot compile against the API the spec declares, and one asserted fact about the codebase is false.

**Caveats:** Both defects are narrow and cheaply fixed — one sentence, one bullet, one added `EngineParts` field or accessor, and one test-visible `LiveEngine` constructor. Nothing in the spec's architecture is unsound, and the two seam additions it proposes are genuinely required rather than duplicative.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.57 | 35% | 0.900 |
| 2 — DAW Workflow Depth | 3.00 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.00 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.50 | 20% | 0.500 |
| **Composite** | | | **2.750** |

**Pass conditions (from `criteria.md`, which is binding):**
- [x] Composite ≥ 2.30 — **2.750**
- [x] Every lens average ≥ 2.00 — 2.57 / 3.00 / 3.00 / 2.50
- [x] No criterion scores 0 — none
- [x] At most two criteria score 1 — **zero** criteria scored 1
- [ ] All auto-fail rules pass — **AF-2 triggered**
- [ ] Feasibility satisfies `criteria.md` — **the feasibility rule fails**: a cited source path does not contain the claim made about it, and two specified tests cannot compile
- [x] Reviewer personally opened every source path the spec cites and checked the claim against the file

**All conditions met:** No → **FAIL**

Note for the orchestrator: this spec fails on evidence integrity, not on quality. It clears
every numeric threshold with room to spare and has no criterion below 2. The four
remediation items in Priority 1 are all small and local.

---

## Remediation Brief

### Priority 1 — Must fix to pass

1. **§4.1 and §7.2 state a false fact about the RT module scan, and contradict the spec's own change list.**
   §4.1 says: "**`bridge.rs`, `control.rs`, `spsc.rs`, and `midi.rs` are not modified.** Those are exactly the four modules `rt_guard.rs:293–298` scans for blocking primitives." §7.2 repeats it under "Deliberately not modified".
   `crates/spectre-audio/tests/rt_guard.rs:293–298` actually reads:
   ```rust
   const RT_MODULES: [&str; 4] = [
       "src/bridge.rs",
       "src/control.rs",
       "src/spsc.rs",
       "src/null.rs",
   ];
   ```
   The fourth entry is `null.rs`, not `midi.rs` — and §4.1's own table two rows earlier assigns `crates/spectre-audio/src/null.rs` the change "implement both new methods". Fix by (a) correcting the module list in both §4.1 and §7.2 to `bridge.rs`, `control.rs`, `spsc.rs`, `null.rs`; (b) restating the structural argument accurately — three of the four scanned modules are untouched, and the fourth, `null.rs`, gains only two app-thread methods that return a constant and a relaxed atomic load and therefore name none of the seven primitives the scan forbids (`Mutex`, `RwLock`, `Condvar`, `thread::sleep`, `println!`, `eprintln!`, `dbg!`, at `rt_guard.rs:299–307`); and (c) adding to §5.2 the requirement that `cargo test -p spectre-audio --test rt_guard` must still pass *after* the `null.rs` edit, since it is no longer an untouched-module argument. Do not move the new methods off `null.rs` — the change is safe, only the description is wrong.

2. **§4.3's note-ordering guarantee does not follow from its own premises.**
   §4.3 states: "`send_note` … stamps `frame_offset: 0` and a monotonically increasing `sequence` internally. That guarantees the plan's ordering contract … at a single frame offset, rank puts `Off`/`AllNotesOff` before `On` … and `sequence` is unique, so the key is a total order. The app never constructs an unsorted batch."
   A total order on the key does not make the app's *emission* order ascending in that key. `ProcessContext::new` (`crates/spectre-dsp/src/io.rs:91–101`) rejects any batch whose keys are not strictly increasing, returning `ProcessError::UnsortedEvents`. With every event stamped at `frame_offset: 0`, a note-on (`rank` 1, `sequence` N) followed by an all-notes-off (`rank` 0, `sequence` N+1) in the *same* block yields `(0,1,N)` then `(0,0,N+1)` — strictly decreasing. The block is refused into silence and `plan_errors` increments (`bridge.rs:192–200`). Exposure is small (Play and Stop must land inside one ~5.33 ms block at 256/48 kHz) and the failure is fail-closed, but the stated guarantee is wrong. Fix by either (a) having `LiveEngine` track the last emitted rank and stamp a `frame_offset` of 1 for any event whose rank would not increase within the current block, or (b) deleting the "never constructs an unsorted batch" claim, documenting the same-block Play→Stop collapse as a counted `plan_errors` outcome, and adding a row to §3.6. Whichever is chosen, add a §5.1 test that sends `On` then `AllNotesOff` through `LiveEngine` with no pump between them and asserts the chosen behavior.

3. **§5.1 test 3 cannot compile — `RenderBridge` exposes no plan capacity.**
   Test 3 asserts "`parts.bridge`'s plan capacity equals `ENGINE_BUFFER_FRAMES * ENGINE_PLAN_FRAME_MARGIN`, read through `CompiledPlan::max_frames`". `RenderBridge` (`crates/spectre-audio/src/bridge.rs:119–290`) has exactly four public methods — `new`, `telemetry`, `transport`, `render` — and its `plan` field is private; §4.1 and §7.2 both forbid modifying `bridge.rs`. Fix by adding a `pub plan_max_frames: usize` field to `EngineParts` in §4.2 (set by `build_engine_parts` from the `max_frames` it passed to `EditableGraph::compile`) and rewriting test 3 to assert against that field. Do not add an accessor to `RenderBridge`.

4. **§5.1 test 9 cannot compile — no `LiveEngine` can be built over a pumpable stream.**
   Test 9 requires calling `LiveEngine::state()` before and after a pump. `LiveEngine` holds `stream: Box<dyn AudioStream>` (§4.2); `pump()` is inherent to `NullStream` (`crates/spectre-audio/src/null.rs:104`) and is not on the `AudioStream` trait (`crates/spectre-audio/src/lib.rs:181–196`), and the trait offers no downcast; §4.3 declares `open_default(&dyn AudioBackend, snapshot)` as the only constructor, which erases the concrete type. Fix by declaring in §4.3 a second constructor — for example `pub fn from_parts(parts: EngineParts, stream: Box<dyn AudioStream>, backend_name: &'static str, device_name: String) -> LiveEngine` — and specifying that tests build `NullStream` via `NullBackend::open_null_output` (`null.rs:38`), pump it *before* boxing, or hold the pump handle alongside. State explicitly which of tests 4, 9, 10, and 11 use `LiveEngine` and which use bare `EngineParts` plus a concrete `NullStream`; as written, tests 4 and 11 work on bare parts, test 10 could read `parts.telemetry`, and only test 9 strictly needs the new constructor.

### Priority 2 — Should fix for quality

1. **Resolve the §3.2/§4.4 disagreement on which sends gate the UI mutation.** §3.2 step 6 says the app flips UI transport state "only if **both** sends returned `Ok`"; §4.4's binding rule conditions only on `LiveEngine::send_transport`; §4.3's `start_audition` returns a single `Result` for a two-send sequence. Specify the transport-`Ok`/note-`Err` case explicitly: whether the queued `TransportCommand::Play` is left in the lane, whether the UI flips, and which of E7's wordings is shown. Pick one rule and make §3.2, §4.3, and §4.4 agree.

2. **Fix the header's dangling cross-reference.** The Known-gaps field says the research need is "recorded as a research need in §2G-adjacent notes below and in §8". There is no §2G in this spec — `2G` is a `criteria.md` criterion ID. The content is in Appendix A under "Named gaps". Change the pointer to "Appendix A and §8".

3. **Schedule the requirements-ledger rows that PROD-003 requires.** PROD-003 reads "Every numeric limit in Spectre MUST have its own rationale recorded in **this ledger**" (`docs/01-requirements/requirements-ledger.md:64`), and decision 16 makes it a standing rule. §4.2 gives both constants excellent rationale, but §7.2's modified-files list omits `requirements-ledger.md`, so the mandated rows would never be written. Add `docs/01-requirements/requirements-ledger.md` to §7.2's modified files with rows for `ENGINE_BUFFER_FRAMES` and `ENGINE_PLAN_FRAME_MARGIN`.

4. **Correct §5's opening sentence.** "Every command below is real and runs today" is false for `cargo test -p spectre-app --test live_engine`, whose file §5.1 introduces as new. Reword to something like "Every command below is real; all but `--test live_engine` run today, and that file is created by this slice."

### Priority 3 — Consider for excellence

1. **Tighten four line citations.** §4.4 cites `RenderBridge.transport` at `bridge.rs:128`; it is at line 127 (128 is `telemetry`). §4.3 cites the ordering-key tuple to `io.rs:152`, which is `pub fn rank`; the tuple `(event.frame_offset, event.kind.rank(), event.sequence)` is at `io.rs:96`. §7.1's `null.rs:52–176` row names `NullBackend`, whose declaration is at `null.rs:19`, outside the range. §7.1's `lifecycle_health.rs:245–295` row names both the deterministic drill and the ignored hardware drill, but that range covers only the hardware drill; the deterministic tests are at `:68–243`.
2. **Soften §4.1's "character-for-character".** `lifecycle_health.rs:269` reads `Box::new(move |mut block: RenderBlock| bridge.render(&mut block)),` — the spec's version drops the `: RenderBlock` annotation. Substantively identical; the phrase overstates it.
3. **Resolve §4.6's `Send` question rather than deferring it.** `eframe 0.32.3` declares `pub trait App` with no `Send` bound (`epi.rs:137`) and `AppCreator` boxes `Box<dyn 'app + App>` (`:49–50`), so `LiveEngine` can be an ordinary field. Replacing the conditional with the confirmed answer removes an open build risk at no cost.
4. **Consider adopting Q3's shared fixture builder within this slice.** The spec correctly observes the fixture chain will be assembled in a fourth place (`engine.rs`) and that test 5 pins them together. Since test 5's hash equality is the slice's central proof, a single builder would make that proof structural rather than maintained — worth pricing against the placement awkwardness §8 Q3 already names.

---

**End of scorecard.**
