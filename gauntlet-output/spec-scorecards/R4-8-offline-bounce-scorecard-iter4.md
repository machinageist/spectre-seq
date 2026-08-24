<!--
Author: Jeff
Date: 2026-08-23
Description: Blind verification scorecard for the R4-8 offline-bounce spec, iteration 4 (remediation 3)
Notes: Re-verification of the iteration-3 feasibility failure plus a full grade of the new content.
  Scope stated honestly below rather than claimed exhaustive.
  Read in full: the spec (all 2,436 lines), criteria.md (all 294), the iteration-3 scorecard
  (all 283), crates/spectre-offline/src/main.rs (all 32).
  Read at every cited range plus surrounding context: crates/spectre-offline/src/lib.rs
  (:19-34, :45-46, :144-150, :163-178, :204-215, :240-245, :255-300, :319-334),
  crates/spectre-offline/tests/harness.rs (:55-175 in full, plus a symbol map of all 470 lines),
  crates/spectre-audio/tests/bridge_plan.rs (:1-121 in full, :166-176, :227-245, plus
  a full symbol map), crates/spectre-graph/src/lib.rs (:203, :209-224, :324, :401-414, :451-464,
  :512-521, :533-544), crates/spectre-dsp/src/source.rs (:113-119, :194-197, :202-209),
  effect.rs (:50-52, :62-71, :107-109, :119, :121-129), io.rs (:52, :93, :96-99, :163-164),
  crates/spectre-audio/src/bridge.rs (:20, :119-120, :133, :152, :157, :162, :167-175, :250-254,
  :274-289), tests/rt_guard.rs (:290-312), crates/spectre-graph/tests/containment.rs (:19-22,
  :44, :75, :100), crates/spectre-project/src/lib.rs (:13, :29-38), crates/spectre-app/src/main.rs
  (:51, :78, :87, :116, :163, :438-443, :508), and all four cited Cargo.toml manifests in full.
  Workspace-wide greps run and re-derived rather than carried forward: both FNV constants,
  every `wrapping_mul` in crates/, `48_000` in harness.rs, the four device values in harness.rs,
  every import and every `fixture_plan` call site in bridge_plan.rs.
  Docs read at cited lines: requirements-ledger.md (:26-32, :36-40, :53-56, :62-66),
  decision-gates.md (:38-52), current-milestone.md (:12, :61, :63, :77, :86, :109-115),
  NEXT.md (:28-32), STATUS.md (:17-21, :35-39), dsp-device-io.md (:23, :81, :85, :94, :104, :113).
  Every OBS- ID the spec cites was opened: ableton-live-observations.md :32/:92/:99/:106,
  synth-modular-observations.md :25/:26/:50/:54/:55, loudness-standards-observations.md
  :10-11/:67/:114/:116; plus ableton-live.md :64/:113/:139, logic-pro.md :64,
  serum-2-observations.md :8-40 and :298, decisions-needed.md D-R2 (:60).
  R4-1's spec was grepped, not read, for ENGINE_BUFFER_FRAMES (:393), LiveEngine::config (:564),
  start_audition (:573); R4-1's iteration-2 scorecard was opened only for its composite;
  manifest.md was grepped for the R4-1/R4-7/R4-8 feature rows.
  All arithmetic recomputed independently in Python, including the golden vector, which was
  folded from its own literal array rather than trusted.
  To isolate what remediation 3 actually changed I diffed the spec file against its
  iteration-3 state (`git diff 767b88a HEAD -- <spec>`). That is the artifact at two
  revisions, not the author's reasoning; I did not read any commit message, plan, or
  remediation note. The manifest's run-history narrative is stale at iteration 3 and was not
  read for iteration 4.
  Did NOT execute cargo — spectre-offline's hash.rs, fixture.rs, bounce.rs and wav.rs do not
  exist yet, so the -D warnings finding below is re-derived from the trait definition and
  every use site, not from a build.
  SECOND PASS, after the first draft: I closed every range I had sampled rather than opened,
  so no claim in this scorecard rests on a partial read. Newly opened in full and confirmed:
  crates/spectre-graph/tests/containment.rs :19-45 (the Poison enum is :19-27 and its sample()
  impl :29-41, so the spec's ":19-41" covers the type and its only method — the two things
  test 6's PoisonAtFrame uses), crates/spectre-audio/src/bridge.rs :1-10 (the :3-8 header Sec 1.2
  cites says exactly what Sec 1.2 quotes), crates/spectre-audio/tests/bridge_plan.rs :94-105,
  crates/spectre-dsp/src/effect.rs :40-74, docs/03-architecture/dsp-device-io.md :81-96,
  docs/02-reference-research/serum-2-observations.md :10-38 in full, synth-modular-observations.md
  :25 in full, crates/spectre-app/src/lib.rs (symbol map), and R4-1's spec :421-432 and :560-578.
  Three Sec 7.1 claims I had not personally confirmed in the first pass, now confirmed:
  harness.rs holds exactly 21 `#[test]` functions (counted); spectre-offline's public surface is
  exactly OfflineReport, RenderReport, default_project, inspect_project, fixture_events,
  render_vertical_slice, render_app_snapshot, render_silence; and NO pub signature in that crate
  names a spectre-graph type today, so Sec 4.5's "spectre-graph types enter spectre-offline's
  public API for the first time" is true rather than assumed. Every second-pass check confirmed
  the spec; none changed a score.
-->

# Scorecard: Offline Bounce — iteration 4

**Feature ID:** `R4-8` (`offline-bounce`)
**Spec file:** `gauntlet-output/specs/R4-8-offline-bounce.md` (2,436 lines)
**Reviewer agent:** blind verification agent, R4-8 iteration 4 (fresh context; did not author or observe any of the three remediations)
**Date:** 2026-08-23
**Spec iteration reviewed:** 4 (remediation 3)
**Prior scorecards:** `R4-8-offline-bounce-scorecard.md` (iteration 1: 2.848, PASS) · `R4-8-offline-bounce-scorecard-iter3.md` (iteration 3: 2.883, **FAIL**, feasibility rule)

---

## Verdict: PASS — composite 2.967

**Summary:** All three blocking items are genuinely fixed, and I confirmed each against source
rather than against the iteration-3 scorecard: the FNV census is now four sites in three files
and it is corrected consistently at every one of the eight places the spec states it; the
`bridge_plan.rs` drop list is seven imports with `AudioProcessor` moved to the correct side and
every stay-used entry independently re-located; and §7.2 now says explicitly, with a coherent
reason, that `harness.rs`'s two folds are out of scope. The sharpened rule and the new
`hash_interleaved` superlative both survive testing — I traced all four folds' traversals and
the interleaved/planar split is real. One new defect entered with this round: §5.2 test 15 now
justifies its `control_channel(&[], 64, 8)` as "the same lane widths every existing test in
`bridge_plan.rs` uses," and `notes_beyond_the_scratch_stay_queued_rather_than_dropped`
(`bridge_plan.rs:169`) uses `256`. Minor, non-blocking, and charged to 4E — but it is the same
defect class this spec has now acquired during remediation three rounds running, and it was
inherited verbatim from my predecessor's own prose without being checked.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A Callback-path discipline | 3 | Re-verified from source, not carried over. `RenderBridge`'s plan is private (`bridge.rs:120`) and its public surface is exactly `new`/`telemetry`/`transport`/`render` (`:133`/`:152`/`:157`/`:162`), so §4.1's "no API through which a bounce could reach it" holds structurally. `RT_MODULES` is at `rt_guard.rs:293-298` with exactly the four entries at `:294-297`; `FORBIDDEN` at `:299-307` with the seven needles at `:300-306` — both verbatim as §4.1 quotes them. Remediation 3 touched nothing under `crates/spectre-audio/src/`; the §4.1 table still reaches `spectre-audio` only in `tests/`. | — |
| 1B Control↔render communication | 3 | Unchanged by this round and re-verified. `Option<JoinHandle<Result<BounceReport, BounceError>>>`, `is_finished()` polled once per frame, joined only after it returns `true` — no channel, no `Mutex`, no `Arc` around the result, no new dependency. The poll's existence is real: `spectre-app/src/main.rs:443` is an unconditional `ctx.request_repaint_after(Duration::from_millis(250))` at the end of `update`, immediately after the five region calls at `:438-442`. No RT-002 traffic; the bounce's shared state is app↔worker only. | — |
| 1C Numerical containment | 3 | Re-verified at source. Per-sample denormal flush at `graph/lib.rs:407-411` with the sign-preserving branch at `:409`; whole-quantum `left[..frames].fill(0.0)` / `right[..frames].fill(0.0)` at `:517-518`; `contaminated_nodes += 1` at `:519`; `containment()` at `:451-453`. §3.6 E9 still surfaces a nonzero count instead of shipping a clean-looking report. | — |
| 1D Determinism | 3 | Still the spec's strongest section, and remediation 3 improved the *argument* without changing the design. One FNV-1a fold, same basis and prime, plus a second traversal; no second hash function, no tolerance, no correlation. §4.3's consequence is still stated out loud (`BounceReport::hash != RenderReport::hash`) and test 4 pins the inequality. §4.4(4)'s per-device block-invariance argument re-verified in all three parts: `PulseInstrument`'s per-frame body at `source.rs:202-209` reads only carried state and `context.sample_rate()`; `Gain` (`effect.rs:62-71`) and `Saturator` (`effect.rs:121-129`) are memoryless. §4.4(6)'s three-way transport check re-verified: `process` takes only `sample_rate`/`frames`/`note_inputs` (`graph/lib.rs:456-461`), `Transport` appears nowhere under `spectre-graph/src/` or `spectre-dsp/src/`, and `RenderBridge::render` applies commands (`bridge.rs:250-254`) without advancing position. | — |
| 1E Graph and plan contract | 3 | `compile` allocates the channel pool at `graph/lib.rs:324` and runs once before the block loop. `fixture.rs` holds the `EditableGraph` → `compile` half and returns an immutable plan, which is GRAPH-001's split verbatim (`requirements-ledger.md:55`). `compile_fixture_plan_with` returning `(CompiledPlan, NodeId)` supplies exactly the `pulse` node `render_plan`'s `PlanNoteInput` needs at `lib.rs:262-265`. No per-parameter recompilation; decision 22's option (b) respected (`decision-gates.md:47`, "(b) is rejected on its face" — verified verbatim). | — |
| 1F Failure behavior | 3 | E1–E9 explicit and fail-closed; E3 cites `bounce::max_frames(sample_rate)`, which §4.2 does declare. E7 keeps the file because it is evidence and says so. §5.2 test 15's `frame_capacity_rejections() == 0` / `blocks_rendered() == 16` pair is the correct guard against `bridge.rs:167-173`'s silent-refusal path — both accessors verified to exist (`bridge.rs:63`, `:68`, `:73`). | — |
| 1G Test specification | 3 | **Up from 2 — iteration 3's blocking test defect is gone and I re-derived both halves.** Test 17 can now run: the drop list is seven imports and `AudioProcessor` is on the correct side, so `cargo clippy --locked --workspace --all-targets -- -D warnings` no longer fails before the file's assertions execute. Test 15's widened lane is fixed to `control_channel(&[], 64, 8)`; the width is adequate on the substance — `fixture_events` returns `[NoteEvent; 2]` (`lib.rs:178`, verified), on at frame 0 and off at `frames - 1`, so with per-block re-basing exactly one event crosses the lane before block 0 and one before block 15 — and every assertion in the test (hash equality, `peak > 0.0`, `blocks_rendered() == 16`, `plan_errors() == 0`, `frame_capacity_rejections() == 0`, `block_hashes.len() == 16`) can fire. Test 1's golden vector recomputed from its own literal array: `0xa49acc9ce7359a37`, decimal 11,861,017,542,899,636,791, with all eight bit patterns exactly as listed — fixed outside the code under test, so it can fail. Test 21's reachability and its "nothing else pins node order" argument both check out (`harness.rs:65-67` has no `NodeId` anywhere; test 15 takes its note node from the builder's return value). No test in §5 is unfailable. | — |

**Lens average:** 3.000
**Lens pass:** Yes

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A Loop-first core loop | 3 | Unchanged. §3.1 argues the non-modal panel from the workflow rather than from taste; §3.2 step 4 states selection/zoom/lens/transport are untouched; §5.4's long-render row makes it a manual check. | — |
| 2B Linked lenses | 3 | Re-verified against source rather than carried. `main.rs:438-442` is `transport / lenses / track_list / inspector / workspace` in that fixed order; `SidePanel::right("inspector")` at `:163`; `SidePanel::left("tracks")` at `:116`; `with_min_inner_size([1060.0, 680.0])` at `:508`. The bounce panel is a second right-hand `SidePanel` between `inspector(ctx)` and `workspace(ctx)`, and §5.4 checks all four panels at the 1060 px minimum. The panel stays off `AppModel`, so no lens forks state. | — |
| 2C Modulation visibility | 3 | §6.3 answers PROD-002 in its own terms against the real control list, and `requirements-ledger.md:63` is verbatim as quoted ("distinct visible states for automated / manually-overridden, with an explicit restore action"). The live version of the question stays routed to §8 Q2. | — |
| 2D Keyboard-first, calm UI | 3 | §3.4 refuses to assign any shortcut, cites the prohibited-conclusions list by name, states the remappable/context-scoped requirement the accepted direction does support, and gives the structural reason nothing forecloses it. No color-only state (§3.7). | — |
| 2E Convergent-pattern grounding | 3 | Every convergence and divergence re-opened at its line. `OBS-AB12-MIX-002` (`ableton-live-observations.md:99`) says what the spec says, including "clipping matters only at physical outputs, the main output, or file export." `OBS-AB12-ARR-008` (`:32`) does say Consolidate incorporates clip-level gain/warp/pitch/envelopes **but not track effects**, so the deliberate divergence is grounded. `OBS-AB12-ROUTE-006` (`:92`) is the realtime resampling path, correctly distinguished. `OBS-PP-ARCH-001` (`synth-modular-observations.md:25`) and `OBS-PP-ARCH-002`'s one-sample aux latency (`:26`) both check out, as does `OBS-VCV-VOLT-006` (`:50`). | — |
| 2F Differentiation | 3 | Unchanged and still correctly bounded: Spectre makes equivalence a CI gate with a localizable failure, and the spec explicitly does **not** claim no other DAW does this, "only that this repository holds no record of one." | — |
| 2G Benchmark evidence discipline | 3 | Every cited `OBS-` ID opened; each says what the spec claims. Gap-naming exact: `ableton-live.md:64` is `inventory-only` for chapter 20 "Bounce to Audio"; `:113` and `:139` are `section-inventoried`; `:139`'s unresolved list is quoted **verbatim** ("signal-path equivalence, interruption, plugin realtime requirements, metadata, dither, SRC, partial-output cleanup, and failure reporting"); `logic-pro.md:64` reads `unreviewed`. Serum 2 discipline is exact and is the sharpest test here: the spec cites only `OBS-SR2-CPU-001`, names `OBS-SR2-KB-001` as the other citable record (both at `synth-modular-observations.md:54-55`), and **refuses** `OBS-SR2-GLOB-008`, which does exist at `serum-2-observations.md:298` and is precisely an offline-render quality preference — refused against the quarantine banner at `:10-38`, verified verbatim including "Do not cite this file, promote any record from it, or treat it as closing any gap," with D-R2 tracked at `decisions-needed.md:60`. Loudness is named as a gap with **no** conformance claim: `OBS-R128-005` (`:114`, both BS.1770 equation-(7) gating **and** EBU Tech 3341), `OBS-R128-007` (`:116`, −1 dBTP with ±0.3 dB), and `OBS-BS1770-006` (`:67`, 48 kHz coefficients only) all verified, and the file's status is `draft` / `in-review` (`:10-11`) exactly as described. | — |

**Lens average:** 3.000
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A Milestone fit | 3 | Anchors re-verified at their exact lines: `current-milestone.md:12` scope ends "save/reload, bounce"; `:86` is verbatim "Offline bounce renders the same project deterministically and matches the live path's computation"; `NEXT.md:30` is slice 8 with "extending the hash-equivalence approach the callback bridge already uses." §7.4 defers stem export, codecs, dither, 16/24-bit, SRC, region export, realtime bounce, metadata, and batch renders by name. | — |
| 3B Non-goal respect | 3 | §6.4 enumerates each non-goal; the output is a plain WAV of Spectre's own render. `current-milestone.md:77` verified as covering recording and latency compensation as R5+. | — |
| 3C Deliberately small first devices | 3 | "**No device is added or modified**" still holds. `fixture.rs` moves definitions and adds no DSP; `hash.rs` adds a traversal, not a device. §6.4 still declines a limiter, a fade-out, and a normalizer for the stated reason. | — |
| 3D Originality | 3 | I recomputed every figure independently and all are exact: 86,400 × 48,000 = 4,147,200,000 frames, ÷ 256 = 16,200,000 blocks, × 8 B = 129,600,000 B; 180 × 48,000 ÷ 256 = 33,750 blocks with 4,185 **exactly** 12.4%; the 24-hour file is 33,177,600,000 B and the log is exactly 1/256 of it; the single-pass rejection is 6 × 8,640,000 × 4 = 207,360,000 B for a 69,120,000 B result; the pool/scratch pair is `32 B × block_frames` giving 8,192 / 16,384 / 32,768 / 65,536 at 256 / 512 / 1,024 / 2,048, with 65,536 B correctly called 64 KiB. **Three constants, three ledger rows, counted myself** — §4.2 declares exactly three, §7.2's ledger entry schedules exactly three, §6.4's 3D bullet says three. Each rationale is Spectre-derived and each was checked: `BOUNCE_MAX_SECONDS` from TIME-003's own 24-hour horizon (`requirements-ledger.md:38`, verified verbatim); `BOUNCE_FALLBACK_BLOCK_FRAMES` from R4-1 §4.2's `ENGINE_BUFFER_FRAMES = 256` (verified at that spec's line 393); `BOUNCE_FALLBACK_SAMPLE_RATE` from Spectre's own render corpus — `48_000` occurs exactly 24 times in `harness.rs` (counted), `bridge_plan.rs:19` is `SAMPLE_RATE: f64 = 48_000.0`, and no shipping source defines a rate. The qualification-table retraction is itself correct: `current-milestone.md:111-113` has columns for platform, date, backend/device, blocks, xruns, worst headroom, plan errors, and contaminated — **no sample-rate column and no block-size column**. Remediation 3 introduced no fourth number; I enumerated every `const` in the spec again after the diff. | — |
| 3E Platform commitment | 3 | Unchanged. One code path, `std::fs` and `std::thread` only, no audio device, headless on both platforms. `spectre-offline`'s manifest has no platform-conditional dependency. Discharges none of decision 23's debt (`decision-gates.md:49`, verified) and forbids recording a Linux result. | — |
| 3F Accessibility trajectory | 3 | Re-verified: `crates/spectre-app/Cargo.toml:13` builds eframe with `default-features = false` and only `default_fonts` and `glow`, so no accessibility feature is enabled and the spec claims none. Standard widgets, text-carried progress and results, no icon-only control, `MISMATCH` must not be color-only. The `right_to_left` cluster it reasons against exists at `main.rs:78`. | — |

**Lens average:** 3.000
**Lens pass:** Yes
**Auto-fail triggered:** No — see roll-call below

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A Current-state accuracy | **3** | **Up from 1. The iteration-3 blocking defect is genuinely fixed, and I re-derived the census by `grep` before reading a word of the spec's version of it.** `grep -rn` over `crates/` for `cbf2_9ce4_8422_2325` and `0x0000_0100_0000_01b3` returns eight lines at **four sites in three files**: `spectre-offline/src/lib.rs:271`/`:276`, `spectre-audio/tests/bridge_plan.rs:81`/`:87`, `spectre-offline/tests/harness.rs:98`/`:103`, `harness.rs:156`/`:161`. The spec now states that census at **eight** places and they agree at every one: §7.1's Absent block (full four-row census plus the per-constant line list), §7.1's `harness.rs` table row, §4.3's four-row table, §4.1's `hash.rs` row, §4.2's `FNV_OFFSET_BASIS` comment, §4.2's `FNV_PRIME` comment, §4.2's `SampleHasher` comment, and §7.2's `bridge_plan.rs` (ii) rationale. I grepped the whole spec for surviving "two files" / "only independent" / "third copy" / "last copy" wording; every hit is inside the header's dated change record, none in a normative section. The rest of §7.1 was re-checked line by line and is exact: `lib.rs` is **334** lines (`wc -l` = 334, corrected from 335); the bridge test's block is driven at `:106-108` with `:109` blank (verified); `render_plan` is private at `:204-285`, compiles at `:243`, calls `process` once at `:259-267`, hashes at `:269-278`; `DeviceValues::from_snapshot` spans `:45-146`; `fixture_events` is `:178-201` and returns `[NoteEvent; 2]`; `ProjectDoc`'s five fields are at `spectre-project/src/lib.rs:29-38`; `spectre-app/src/` holds only `lib.rs` and `main.rs`; `spectre-app/tests/` holds only `app_model.rs` and `smoke_cli.rs`; the "only filesystem access in `crates/` is two reads" claim (`spectre-offline/src/main.rs:16`, `rt_guard.rs:311`) is exactly true on a workspace grep. The two new narrowed claims also survive: `hand_wired_report` really is at `harness.rs:112-125` hand-wiring the topology at caller-supplied values (`:123-125`) and is called from `:198`, `:225`, `:461`; and grepping `harness.rs` for `0.3` / `0.7` / `2.5` / `0.35` returns `:65-67` and nothing else, so "the one hand-written copy of these four **values**" is true as narrowed. | — |
| 4B Status vocabulary | 3 | §7.1's Implemented / Planned / Gated / Absent partition is accurate and its cross-spec statuses check out against the manifest rather than against memory: `manifest.md:44` records R4-1 as **spec-pass** at **2.950** (matching `R4-1-live-audio-wiring-scorecard-iter2.md:154`), `manifest.md:50` records R4-7 at **3.000**, and `crates/spectre-app/src/engine.rs` genuinely does not exist. §7.2 still moves status to `implemented`, **not** `verified`, until §5.4's manual checks pass. | — |
| 4C Traceability | 3 | All six ledger IDs re-verified at their exact lines (RT-001 `:28`, RT-002 `:29`, RT-003 `:30`, TIME-003 `:38`, GRAPH-001 `:55`, PROD-002 `:63`, PROD-003 `:64`) and every decision row (15 `:39`, 16 `:40`, 17 `:41`, 22 `:47`, 23 `:49`). Every arithmetic claim recomputed. The one citation iteration 3 caught as unsupported — `AudioProcessor` at `bridge_plan.rs:223`, a comment — is now correctly described *as* a comment, in §4.3, §7.2, and §5.1 test 17. | — |
| 4D Honest gaps | 3 | Ten open questions, none decorative. Q9 is now the sharpest it has been: with `hash.rs` and `fixture.rs` being the whole of the diff into `bridge_plan.rs`, the split argument is stated with its counter-argument intact and left to Jeff. Q4's libm reasoning re-verified (`f32::tanh` at `effect.rs:119` and `:127`, `f64::powf` at `source.rs:204`). Q5 still refuses to edit `dsp-device-io.md:113` by assertion — that line reads exactly "Identical initial state and input produce bit-identical offline output." The `Gain`-smoothing divergence is re-verified true in both directions: `dsp-device-io.md:94` and `:104` claim smoothing; `effect.rs:62-71` multiplies by `self.gain` with no smoothing state. | — |
| 4E Evidence commands | **2** | The workspace gate is quoted exactly as criteria.md requires, and every `-p X --test Y` target either exists today (`spectre-offline --test harness`, `spectre-audio --test bridge_plan`, `--test rt_guard`, `spectre-graph --test containment` — all four present) or is created by this spec and named as such. Every signature the tests call was checked against source: `RenderBridge::new(plan, control, note_node, sample_rate, note_scratch)` at `bridge.rs:133`, `control_channel(targets, note_capacity, transport_capacity)` at `control.rs:195`, the three telemetry accessors at `bridge.rs:63`/`:68`/`:73`. **The one defect is new in this round.** §5.2 test 15 now justifies `control_channel(&[], 64, 8)` as "the same lane widths **every existing test** in `bridge_plan.rs` uses." That is false: of the six `control_channel` call sites in the file (`:97`, `:126`, `:146`, `:169`, `:213`, `:232`), `notes_beyond_the_scratch_stay_queued_rather_than_dropped` uses `control_channel(&[], 256, 8)` at `:169`, deliberately, so that a scratch of `4` (`:171`) forces the deferral path. `:213` also passes a non-empty target list. The chosen number is still right and the substantive half of the reason is right — `fixture_events` returns `[NoteEvent; 2]` (`lib.rs:178`, verified), so at most one event crosses the lane per block — but the universal quantifier is an uncited codebase claim that is wrong, and it was carried over verbatim from the iteration-3 scorecard's own prose rather than checked. Scored 2, not 1: it changes no command, no parameter, and no outcome, and every command in §5 remains real and runnable. | Narrow the sentence to "the width five of the six existing `control_channel` call sites use; `:169` uses `256` deliberately, to overrun a scratch of 4" — or drop the comparison and keep only the `[NoteEvent; 2]` reason, which is sufficient on its own — P2 |
| 4F No fake surfaces | 3 | Verified against `STATUS.md:19` ("`./spectre` does not use the audio crate, so nothing launchable makes sound") and `:37` ("This is not a live engine … only from a test, never from `./spectre`"). §3.3's empty-state mandate is intact and still correct that `ProjectDoc` has no tracks, clips, or devices. §4.7 reports a measured realtime factor and asserts no target. `spectre-app`'s `[dependencies]` are exactly `eframe`, `spectre-core`, `spectre-dsp`, `spectre-project`, `serde`, `serde_json` — no `spectre-audio`, as §1.2 states. | — |

**Lens average:** 2.833
**Lens pass:** Yes — avg ≥ 2.00

---

## The three blocking items, tested individually

**(1) The FNV census — FIXED, and corrected consistently.**
Re-derived by `grep` before reading the spec's version. Four sites, three files, eight lines:
`spectre-offline/src/lib.rs:271`/`:276`, `spectre-audio/tests/bridge_plan.rs:81`/`:87`,
`spectre-offline/tests/harness.rs:98`/`:103`, `harness.rs:156`/`:161`. The brief asked me to
check five sites in the spec; there are eight, and all eight agree (listed under 4A). A grep of
the spec body for the superseded wording returns only header-record hits. **The traversal claims
attached to the census are also true**, and they matter because the argument for keeping
`hash_interleaved` now rests on them. I read all four folds:

| Fold | Buffer | Traversal | Verified |
|---|---|---|---|
| `lib.rs:271-278` | planar | `output[0].iter().chain(output[1].iter())` (`:272`) | yes |
| `bridge_plan.rs:80-92` | **interleaved** | `for channel { for frame { samples[frame*channels+channel] } }` (`:82-84`) | yes |
| `harness.rs:98-105` | planar | `wired[0].iter().chain(wired[1].iter())` (`:99`) | yes |
| `harness.rs:156-163` | planar | same shape (`:157`) | yes |

So the new superlative holds: `hash_interleaved` is the only hand-written fold that walks an
**interleaved** buffer, the other three all walk planar channel-major. And the seam claim holds
too — `bridge_plan.rs:111` folds the live interleaved buffer and `:113-116` asserts it against
`render_vertical_slice`'s `RenderReport::hash`, while both `harness.rs` folds compare offline
against offline. That is a materially better argument than the count ever was, and the spec
makes it rather than hedging.

**(2) The import list — FIXED, seven, and every entry independently re-located.**
I mapped every symbol in `bridge_plan.rs` rather than trusting either document. `AudioProcessor`
(`:14`) occurs at `:45`, `:48`, `:52` as `.io()` calls — all inside `fixture_plan`'s body — and
at `:223` inside a comment; `io()` is a trait method (`spectre-dsp/src/io.rs:163-164`, quoted
verbatim by the spec and correct), so the import cannot survive the delegation. `Gain`,
`PulseInstrument`, `Saturator`, `Waveform` (`:14`) and `EditableGraph`, `Connection` (`:16`) all
occur **only** within `:33-77`. Seven is right. The stay-used list is right in every entry:
`IdGen` really is used at `:207` (`IdGen::new(0x0050_4152_414d)`), `NoteEvent` at `:178`,
`NoteEventKind` at `:181`, `CompiledPlan` at `:33` only. **The `NodeId` claim specifically:**
its uses are `:33` (return type), `:35`, `:36`, `:37` — the last three inside the deleted body —
so it survives on the signature alone, exactly as §4.3 says, and §7.2 commits to keeping that
signature. The spec names the fragility itself. All six `fixture_plan` call sites (`:96`, `:125`,
`:145`, `:168`, `:212`, `:231`) are exactly as §7.2 lists them.

**(3) `harness.rs` scope — FIXED, and the reason is coherent.**
§7.2's `harness.rs` entry now has two explicit sub-items: (i) the device values at `:65-67`, and
(ii) **both** FNV folds, named at `:98-105` and `:156-163` with their constants at `:98`/`:103`
and `:156`/`:161`. It states plainly that "the FNV constants remain duplicated at two sites after
this slice, by decision rather than by oversight." The reason given — they are the independent
instrument the shared specimen is measured against, and delegating them would make four named
tests compare `hash.rs` with itself — is coherent and checkable: all four named tests exist at
the cited lines (`:58`, `:173`, `:221`, `:451`) and all four reach a hand-wired hash, three of
them through `hand_wired_report` (`:198`, `:225`, `:461`). §5.2 test 19 records the favourable
consequence. **Silence is gone; the rule is now applied as broadly as it is stated.**

**The sharpened rule, tested against the spec's own scheduling.** "De-duplicate the specimen,
never an independent instrument that stands between two paths under comparison," explicitly not
"never the last copy." I checked whether the scheduling matches. It does, and `harness.rs` is the
case that proves it rather than the case that strains it: those two folds are *not* the last
copy of anything — after the refactor `hash.rs` holds the shared one and `hash_interleaved`
holds a third — so a "last copy" rule would not protect them, and a pure de-duplication rule
would absorb them. Only the positional rule reaches them, and the spec applies it there. The
device values at `:65-67` are handled by the same rule from the specimen side. One asymmetry
worth naming for honesty, and the spec does name it: `harness.rs`'s folds sit between two
*offline* paths, not between live and offline, so they are a weaker instance of "between two
paths under comparison" than `hash_interleaved` is. §7.2 says exactly that — they pin
`hash.rs`'s output against walks `hash.rs` never touches — which is the right framing.

**The two self-reported defects — both narrowings survive.**
§4.3's `fixture.rs` header now reads "The one hand-written copy of these four **values**," with
`hand_wired_report` (`harness.rs:112-125`) named as hand-wiring the same topology at
caller-supplied values. Verified: the constructors at `:123-125` take `level`, `gain_value`,
`drive`, `mix` from the signature at `:112-119`, and a grep of `harness.rs` for `0.3` / `0.7` /
`2.5` / `0.35` returns `:65-67` and nothing else. §7.1's matching sentence is narrowed the same
way ("the workspace's only independent statement of what the fixture's *device values* are").
Both narrowed claims are true as written.

**The deliberate non-edit — sound record-keeping, imperfectly executed; not blocking.**
The header preserves round 1's item (d) with its false "the workspace's only independent
implementation of the FNV walk," and round 2's item (e) with its "six now-unused imports,"
correcting both in round 3's items (j) and (k). **My verdict: this is the right call.** The
header is a dated change record, not a normative section; the pattern is the one `criteria.md`
uses on its own AF-6 passage, for the reason that passage gives — a document that silently
rewrites its own history teaches nothing, and both decisions being recorded were correct even
where their stated reasons were not. Nothing in §3–§8 carries the false claim, and every
normative statement of the census is right.

Where the execution falls short of `criteria.md`'s pattern: `criteria.md` marks its wrong text
**at the wrong text**, with "Corrected 2026-08-15 — this passage was itself wrong" immediately
above the quoted original. Here the false sentences sit at header lines 45 and 80 and their
corrections at lines 158–172 and 181–186 — 110 and 100 lines later. A reader who stops at item
(d) is left with a false claim and no signal that one exists. One inline clause each ("corrected
in (j)" / "corrected in (k)") would close it entirely. **P3, not blocking:** this is a
presentation gap in a change log, not a surviving claim in the artifact's normative body.

---

## Auto-fail roll-call

| Rule | Result | Basis |
|---|---|---|
| AF-1 Contradicting accepted authority | **Pass** | The one narrowing of an accepted statement (`dsp-device-io.md:113`) is flagged in §7.2 and routed to §8 Q5 rather than edited. No accepted ledger requirement or decision row is contradicted; decision 22's rejection of option (b) is respected (`decision-gates.md:47`). |
| AF-2 Unbacked implementation claims | **Pass** | Every positive claim I checked resolved in one `Read` at the cited line — roughly ninety line references across fourteen source files, four manifests, seven docs, and twelve `OBS-` records. The claimed-absent direction, which is what failed iteration 3 and which AF-2's text does not reach, is now also correct: the FNV census, the fixture-values census, the "no filesystem write in `crates/`" claim, the `spectre-app` dependency claim, and the `engine.rs` absence all verify. |
| AF-3 Realtime discipline violation | **Pass** | Nothing R4-8 adds sits on a callback-reachable path. `JoinHandle::is_finished()` does not block; the `join()` happens only after it returns `true`; both live on the app thread. All file I/O stays on the bounce worker inside `wav.rs`. The polling cadence is an existing unconditional `request_repaint_after` (`main.rs:443`), so no new timer, no spin, no wait. `RenderBridge::render` is untouched, and remediation 3 changed nothing under `crates/spectre-audio/src/`. |
| AF-4 Borrowed numeric limits | **Pass** | I counted the ledger rows myself after the diff: §7.2's `requirements-ledger.md` entry schedules exactly **three**, one each for `BOUNCE_FALLBACK_SAMPLE_RATE`, `BOUNCE_FALLBACK_BLOCK_FRAMES`, and `BOUNCE_MAX_SECONDS`; §4.2 declares exactly those three; §6.4's 3D bullet says three. **Remediation 3 introduced no numeric bound** — I re-enumerated every `const` and every number in the changed ranges: the only figure that moved is test 15's lane width, which drops from `1024` to `64` and is a test parameter matching existing call sites, not a bound. `FNV_*` are moved algorithm constants, `FIXTURE_*` and `FIXTURE_SEED` moved identities and fixture data, `GOLDEN_HASH` a checked-in vector, `max_frames` derived — and §7.2 now names all of them in its exhaustiveness paragraph, which closes iteration 3's P3-1 and makes the claim true as written. All three rationales are Spectre-derived, and the refusal of the qualification table as provenance for either fallback is correct: that table has neither column. |
| AF-5 Conclusions the evidence does not support | **Pass** | §3.4 refuses a default shortcut map by name and cites the prohibited-conclusions list. No native-device list, synthesis architecture, gesture budget, interface model list, monitoring-latency threshold, platform order, or archetype promotion. **Loudness stays out of scope with no conformance claim** — §6.3 and Appendix A name `OBS-R128-005`, `OBS-R128-007`, and `OBS-BS1770-006` as the reason a casual claim would be fabricated, and `peak` is explicitly the plain sample-peak maximum already computed at `lib.rs:273` and "must never be labelled" true-peak. |
| AF-6 Optimistic language | **Pass** | §4.4(8) still retracts "cheap enough to always be on" by name and ties the flag to its only consumer against the corrected 129.6 MB figure. Remediation 3's new prose is if anything more conservative than what it replaced: it converts a superlative that was false into one that is narrower, checkable, and true, and it records "the FNV constants remain duplicated at two sites after this slice" as an outcome rather than hiding it. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every new type is fully specified with fields and signatures. `fixture.rs` is a child module of the crate root, so `pub(crate) compile_fixture_plan_with(DeviceValues, …)` can name the root-private `DeviceValues` — the same mechanism `bounce.rs` uses. `BounceConfig` correctly has no `Default`; `fallback_config(frames)` states all four field values. |
| API/interface changes feasible with current architecture | ✓ | `compile_fixture_plan_with` returning `(CompiledPlan, NodeId)` supplies exactly what `render_plan` needs for `PlanNoteInput { node: pulse, … }` at `lib.rs:262-265`. Signature-coherent. |
| **FNV census (the iteration-3 failure)** | ✓ **re-derived by grep, correct at eight sites** | Four sites, three files, eight lines. See the blocking-item section above. All four traversals read and classified; the interleaved/planar split the argument rests on is real. |
| **`-D warnings` knock-on (the second iteration-3 failure)** | ✓ **seven, and every entry located** | Drop list: `AudioProcessor`, `Gain`, `PulseInstrument`, `Saturator`, `Waveform` (`:14`), `EditableGraph`, `Connection` (`:16`) — all used only inside `:33-77`. Stay list: `IdGen` (`:207`), `NoteEvent` (`:178`), `NoteEventKind` (`:181`), `CompiledPlan`/`NodeId` (`:33`). `NodeId`'s single-use survival verified and named by the spec itself. The scheduled edit now compiles under the gate the spec names. |
| **`harness.rs` scope** | ✓ **stated for both the values and both folds** | §7.2's entry is explicit in two sub-items with a coherent, checkable reason. The four tests it names all exist at the cited lines and all reach a hand-wired hash. |
| **Duplication census (fixture)** | ✓ **all rows exact** | `lib.rs:210-258` topology (seed `:210`, three `NodeId`s `:211-213`, three `add_node`, two `connect`, `compile(saturator, …)` at `:243` with its factory closure); `lib.rs:297-300` and `:328-331` each `0.3 / 0.7 / 2.5 / 0.35`; `bridge_plan.rs:24-27` plus seed `:30`; `:33-77` the second topology; `harness.rs:65-67` the hand-wired third copy of the values — and `harness.rs:112-125` the second hand-wired topology, at caller-supplied values, correctly distinguished. |
| **Crate boundary argument** | ✓ **all four claims verified** | `spectre-audio/Cargo.toml:24` is `spectre-offline = { path = "../spectre-offline" }` under `[dev-dependencies]` — no new edge for `hash` or `fixture`. Both crates take `spectre-graph` by the same workspace path (`spectre-audio/Cargo.toml:21`, `spectre-offline/Cargo.toml:15`), so `CompiledPlan` and `NodeId` are the same types on both sides. `spectre-offline`'s `[dependencies]` already hold everything the builder needs — no manifest change. The recorded consequence is true: no `pub` item in `spectre-offline` today names a `spectre-graph` type, while `spectre-dsp`'s already appear via `render_app_snapshot`. The `spectre-app` cycle hazard is real: `spectre-offline/Cargo.toml:21` dev-depends on `spectre-app`, and §4.5 says verify with `cargo metadata` rather than assume. |
| **Arithmetic** | ✓ **every figure recomputed independently** | 4,147,200,000 frames → 16,200,000 blocks → 129,600,000 B. 33,750 blocks with 4,185 = 12.4% exactly. Pool/scratch 6,144 + 2,048 = 8,192 B = 32 × 256, scaling to 65,536 B = 64 KiB at 2,048. Single-pass 207,360,000 B for 69,120,000 B. 24-hour file 33,177,600,000 B, log exactly 1/256 of it. Test 13's 128 × 2 × 4 = 1024. Test 16's 2,113 ÷ 256 = block 8, `frame_in_block` 65. Test 6's geometry correct. **Golden vector folded from the literal array in Python:** `0xa49acc9ce7359a37` = 11,861,017,542,899,636,791, with all eight bit patterns as listed. No figure is off. |
| **§4.4(1) / §4.1 / §4.3 / §7.2 agreement** | ✓ | §4.4(1) says the bounce calls `fixture::compile_fixture_plan_with`; §4.1's table lists `fixture.rs`; §4.3's call-site table is exhaustive; §7.2's New-files block schedules it and the `lib.rs` entry schedules edits (i)–(iii) matching §4.3 row for row. The two `bridge_plan.rs` edits sit on genuinely disjoint ranges (`:33-77` replaced, `:80-92` kept). |
| Views/screens fit current navigation pattern | ✓ | Second-right-hand-`SidePanel` placement coherent with `main.rs:438-442`'s fixed region order; the inspector at `:163` keeps width and position. |
| Dependencies available and version-compatible | ✓ | No new crate dependency and no new dependency edge. |
| Platform/renderer requirements realistic | ✓ | `std::fs` and `std::thread` only; no device; headless on both platforms. |
| Test strategy executable with current infrastructure | ✓ | Test 17's blocker is removed; test 15's lane is adequate for `[NoteEvent; 2]` with per-block re-basing; test 1's vector is correct and independent; test 21's imports are reachable and its assertion can fail. Every named `-p X --test Y` target either exists or is created by this spec and named as such. |
| Performance budget realistic | ✓ | Correct at every figure, with the `block_frames` ceiling case stated and no cap introduced. |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares each; the core proof composes only implemented components. |
| **Second-pass §7.1 spot checks** | ✓ **three more claims confirmed** | `harness.rs` holds exactly **21** `#[test]` functions, matching §7.1's "21 harness tests" and §5.2 test 19's "its 21 existing tests." `spectre-offline`'s public surface is exactly `OfflineReport`, `RenderReport`, `default_project`, `inspect_project`, `fixture_events`, `render_vertical_slice`, `render_app_snapshot`, `render_silence` — and **no** `pub` signature in that crate names a `spectre-graph` type today, so §4.5's recorded consequence ("`spectre-graph` types enter `spectre-offline`'s **public** API for the first time") is true rather than assumed, and `spectre-dsp`'s really do already appear via `render_app_snapshot`. `crates/spectre-app/src/lib.rs` declares no `pub mod` at all today, so §7.2's scheduled `pub mod bounce_panel;` is a clean addition to a single-file lib root. |
| **Residual** | ⚠ **one false uncited claim, non-blocking** | §5.2 test 15: "the same lane widths every existing test in `bridge_plan.rs` uses." `:169` uses `control_channel(&[], 256, 8)`. Not in §7.1, does not affect the chosen value, does not affect compilability or any assertion. Charged to 4E; P2. |

**Feasibility verdict:** Feasible as written. Every source path the spec cites was opened and
checked at its exact lines, in both directions; §7.1 does not misdescribe current state.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 3.000 | 35% | 1.050 |
| 2 — DAW Workflow Depth | 3.000 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.833 | 20% | 0.567 |
| **Composite** | | | **2.967** |

**Pass conditions (values copied from criteria.md, which is binding):**
- [x] Composite ≥ **2.30** — 2.967 (iteration 1: 2.848; iteration 3: 2.883)
- [x] Every lens average ≥ **2.00** — 3.000 / 3.000 / 3.000 / 2.833
- [x] No criterion scores **0** — lowest is 2
- [x] At most **two** criteria score 1 — none do
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear; see roll-call
- [x] **Feasibility rule — satisfied.** I opened every source path the spec cites and checked the claim at the cited lines, in both directions, with the absence claims checked hardest because that is what failed iteration 3. The FNV census, the fixture-values census, the "no filesystem write in `crates/`" claim, the `spectre-app` dependency list, and the `engine.rs` absence all verify. §7.1 does not misdescribe current state anywhere I could find, and I re-derived its two load-bearing censuses by `grep` rather than reading them first.
- [x] Reviewer personally executed every command claimed as passing where required — no command in this spec is claimed as currently passing except the four existing regression targets, asserted only as "must continue to pass"; I verified those targets exist as files rather than running them, and recomputed every arithmetic claim including the golden vector by hand.

**All conditions met:** Yes → **PASS**

---

## Where I disagree with the prior scorecards

Stated because this run has documented repeated cases of a review being overturned by what came
after it, and because the same discipline must apply to me.

1. **Iteration 3 was right on both blocking items, and I confirmed both from source before
   reading its scorecard's version of them.** The census really was four sites in three files;
   `AudioProcessor` really was on the wrong list. Neither finding was overstated.
2. **Iteration 3's P2-2 was right about the number and wrong about the reason, and the spec
   inherited the wrong reason.** Its remediation brief says test 15 "calls
   `control_channel(&[], 1024, 8)` where **every existing test** in `bridge_plan.rs` uses `64`."
   Five of the six call sites use `64`; `:169` uses `256`, deliberately, to overrun a scratch of
   `4`. Dropping to `64` was still correct — `fixture_events` is `[NoteEvent; 2]` — but the spec
   copied the false universal quantifier into §5.2 rather than checking it, and that is now the
   only false codebase claim in the artifact. **This is the run's documented failure mode
   appearing one more time: a remediation verified against a document instead of against source.**
   The remediation caught two of my predecessor's blind spots on its own initiative (§4.2's
   `SampleHasher` comment and §4.3's `fixture.rs` header) and then swallowed this one whole.
3. **Iteration 3's P3-3 was right** — the bridge test's block is driven at `:106-108` and `:109`
   is blank; the spec now says so. **Its P2-1 was right** — `lib.rs` is 334 lines, and the
   heading now says 334. Both checked, not assumed.
4. **On one point I looked hardest at, iteration 3 and the spec are both correct and I could not
   break it:** the claim that `harness.rs`'s two folds should stay hand-written. I tested the
   opposite case — that the positional rule is a rationalization for leaving work undone — and
   it does not hold: delegating them would make `plan_render_matches_hand_wired_chain`,
   `every_app_parameter_maps_exactly_to_the_compiled_plan`,
   `app_defaults_match_backend_authoritative_default_render`, and
   `model_snapshot_contains_nonfinite_edit_before_render` compare `hash.rs` with itself, and all
   four exist at the cited lines and all four reach a hand-wired hash.

---

## Remediation Brief

**Nothing here blocks this spec.** It passes all four binding conditions. The items below are
quality work for whoever implements it or writes the next iteration.

### Priority 1 — Must fix before this spec can pass

**None.** All three of iteration 3's blocking items are genuinely fixed and verified against
source.

### Priority 2 — Should fix for quality

1. **§5.2 test 15 — correct the lane-width justification.** The sentence reads "the same lane
   widths every existing test in `bridge_plan.rs` uses." Of the six `control_channel` call sites
   (`:97`, `:126`, `:146`, `:169`, `:213`, `:232`), `:169` is
   `control_channel(&[], 256, 8)` inside `notes_beyond_the_scratch_stay_queued_rather_than_dropped`,
   which pairs it with a scratch of `4` (`:171`) to force the deferral path; `:213` passes a
   non-empty target list. Either narrow to "the width five of the six existing call sites use,"
   or drop the comparison entirely and keep only the reason that carries the argument on its own:
   `fixture_events` returns `[NoteEvent; 2]` (`lib.rs:178`), note-on at frame 0 and note-off at
   `frames - 1`, so with per-block re-basing exactly one event crosses the lane before block 0
   and one before block 15. The chosen value `64` is correct either way.

### Priority 3 — Consider for excellence

1. **Mark the header's superseded claims where they sit.** Round 1's item (d) still carries "the
   workspace's only independent implementation of the FNV walk" and round 2's item (e) still
   carries "six now-unused imports," corrected 110 and 100 lines later in items (j) and (k).
   Preserving them is the right call and matches `criteria.md`'s own AF-6 pattern — but that
   passage marks its wrong text *at* the wrong text. One inline clause each ("corrected in (j)",
   "corrected in (k)") would make the record self-consistent for a reader who stops at item (d).
2. **§4.3's opening paragraph cites `bridge_plan.rs:83-84` for the de-interleaving traversal
   where §4.3's own table and §7.2 cite `:82-84`.** Both are defensible — `:82` opens the channel
   loop, `:83-84` are the frame loop and the index expression — but two ranges for one construct
   in one section invites a reader to think one is wrong. Pick `:82-84`.
3. **§7.2's `harness.rs` entry names three callers of `hand_wired_report` and four tests that
   reach a hand-wired hash; the header's item (l) names three tests.** §4.3 and §7.2 name four,
   including `model_snapshot_contains_nonfinite_edit_before_render` (`:451`), which is the
   complete list and the correct one. The header is the outlier; align it or drop the enumeration
   from the header.
4. **§7.1's heading still says "Verified by reading the files at commit `2e005e5`" while the
   metadata block says the sources were re-read at `767b88a`.** Both are true and the spec proves
   they are equivalent (`git diff b5af060..767b88a -- crates/ docs/01-requirements/
   docs/06-plans/` is empty — I confirmed this, and confirmed the same for `2e005e5..HEAD`), but
   the two commit hashes in two places make a reader do that work. One sentence in §7.1 naming
   the equivalence would settle it.

---

**End of scorecard.**
