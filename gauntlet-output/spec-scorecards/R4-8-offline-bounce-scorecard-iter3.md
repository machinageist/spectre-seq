<!--
Author: Jeff
Date: 2026-08-23
Description: Blind verification scorecard for the R4-8 offline-bounce spec, iteration 3 (remediation 2)
Notes: Delta review with a regression sample, scope stated honestly below rather than claimed exhaustive.
  Read in full: the spec (all 2,200 lines), criteria.md, the iteration-1 scorecard,
  crates/spectre-offline/src/lib.rs (all 334 lines), crates/spectre-offline/src/main.rs (all 32),
  crates/spectre-audio/tests/bridge_plan.rs (all 245 lines), all three cited Cargo.toml manifests.
  Read at the cited ranges plus surrounding context: crates/spectre-graph/src/lib.rs
  (:200-226, :318-330, :398-416, :448-470, :508-545), crates/spectre-audio/src/bridge.rs
  (:1-25, :117-150, :245-292), crates/spectre-offline/tests/harness.rs (:1-20, :35-80, :95-172,
  :205-230), crates/spectre-dsp/src/effect.rs (:45-75, :105-134), source.rs (:110-122, :155-165,
  :190-212), io.rs (:48-56, :88-102, :163-172), crates/spectre-graph/tests/containment.rs
  (:17-46, :73-102), crates/spectre-audio/tests/rt_guard.rs (:290-315),
  crates/spectre-audio/src/control.rs (:195-203), crates/spectre-project/src/lib.rs (:10-40),
  crates/spectre-app/src/main.rs (:45-55, :74-90, :113-120, :160-168, :430-450, :505-512).
  Docs read at cited lines: requirements-ledger.md (:26-32, :36-40, :53-56, :62-66),
  decision-gates.md (:38-48), current-milestone.md (:12, :61, :63, :77, :86, :109-115),
  NEXT.md (:28-32), STATUS.md (:17-21, :35-39), dsp-device-io.md (:23, :81, :85, :94, :104, :113).
  Every OBS- ID the spec cites was opened (13 of them), plus ableton-live.md :64/:113/:139,
  logic-pro.md :64, serum-2-observations.md :8-40, decisions-needed.md D-R2,
  loudness-standards-observations.md :10-12. R4-1's spec was grepped, not read, for
  ENGINE_BUFFER_FRAMES (:393), LiveEngine::config (:564), start_audition (:573).
  Workspace-wide greps run: the two FNV constants, 48_000/48000 under crates/*/src/,
  filesystem writes under crates/, Transport in graph/dsp src, Transport::advance callers,
  and every symbol in bridge_plan.rs's stay-used list.
  All arithmetic recomputed independently, including the golden vector, which was recomputed
  from the literal array in Python rather than trusted. Did NOT execute cargo — the crate does
  not exist yet; the -D warnings finding below is derived from reading the trait definition and
  every use site, not from a build. Did NOT read the manifest's run-history entries for
  iteration 3, to preserve blindness.
-->

# Scorecard: Offline Bounce — iteration 3

**Feature ID:** `R4-8` (`offline-bounce`)
**Spec file:** `gauntlet-output/specs/R4-8-offline-bounce.md`
**Reviewer agent:** blind verification agent, R4-8 iteration 3 (fresh context; did not author or observe either remediation)
**Date:** 2026-08-23
**Spec iteration reviewed:** 3 (remediation 2)
**Prior scorecard:** `R4-8-offline-bounce-scorecard.md` (iteration 1: composite 2.848, PASS)

---

## Verdict: FAIL — feasibility rule (pass condition 4)

**Summary:** Both remediation rounds did what they claimed, and I verified the hard parts rather
than the easy ones: the 48× arithmetic error is gone and every replacement figure recomputes
exactly, the golden vector `0xa49acc9ce7359a37` reproduces from its own literal array, the new
`BOUNCE_FALLBACK_SAMPLE_RATE` is the best-evidenced of the three constants and §7.2 really does
schedule three ledger rows, and §4.4(1) and §7.2 now say the same thing because a shared builder
is actually scheduled. Two defects survive, both found by opening files the spec cites. The
blocking one is that §7.1's absence census is false: it states the FNV constants exist in exactly
two files and that `bridge_plan.rs:80-92` is the workspace's only independently written fold,
when `crates/spectre-offline/tests/harness.rs` holds **two more** hand-written FNV folds with the
same two constants (`:98`/`:103` and `:156`/`:161`) — a §7.1 misdescription of current state,
which pass condition 4 says fails regardless of composite. The second is that the `-D warnings`
knock-on list is wrong by one entry in the direction that matters: `AudioProcessor` is used
**only** inside the `fixture_plan` body this slice deletes, so the scheduled `bridge_plan.rs`
edit would not compile under the gate the spec itself names.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A Callback-path discipline | 3 | Re-verified rather than carried over. `RenderBridge`'s plan is private (`bridge.rs:120`) and its public surface is exactly `new`/`telemetry`/`transport`/`render` (`:133`/`:152`/`:157`/`:162`), so §4.1's "no API through which a bounce could reach it" holds structurally. `RT_MODULES` is at `rt_guard.rs:293-298` with exactly the four entries at `:294-297`, and `FORBIDDEN` at `:299-307` with exactly the seven needles at `:300-306` — both verbatim as quoted. Remediation 2 changed nothing under `crates/spectre-audio/src/`; §4.1's table still touches `spectre-audio` only in `tests/`. | — |
| 1B Control↔render communication | 3 | Iteration 1's only 1B gap is closed, and closed cheaply. §4.4's handoff is `Option<JoinHandle<Result<BounceReport, BounceError>>>`, `is_finished()` polled once per frame and joined only after it returns `true` — no channel, no `Mutex`, no `Arc` around the result, no new dependency. The claim that the poll actually happens is verified: `crates/spectre-app/src/main.rs:443` is `ctx.request_repaint_after(std::time::Duration::from_millis(250))`, unconditional, at the end of `update`, immediately after the five region calls at `:438-442`. So no timing constant is introduced, exactly as claimed. Still correctly states there is **no** RT-002 traffic: the bounce sends nothing on the lanes and reads nothing from them, and its shared state is app↔worker only. | — |
| 1C Numerical containment | 3 | Unchanged and re-verified against source: per-sample denormal flush at `crates/spectre-graph/src/lib.rs:407-411` with the sign-preserving branch at `:409`, whole-quantum `left[..frames].fill(0.0)` / `right[..frames].fill(0.0)` at `:517-518`, `contaminated_nodes += 1` at `:519`, `containment()` at `:451-453`. §3.6 E9 still surfaces a nonzero count rather than shipping a clean-looking report. | — |
| 1D Determinism | 3 | Still the spec's best section, and the fixture extraction does not weaken it — it strengthens it. One FNV-1a fold, same offset basis and prime, plus a *traversal*; no second hash function, no tolerance, no correlation. §4.3 keeps stating the consequence out loud (`BounceReport::hash != RenderReport::hash`) and test 4 pins the inequality. §4.4(4)'s per-device block-invariance argument re-verified: `PulseInstrument`'s per-frame body at `source.rs:202-209` reads only carried state and `context.sample_rate()`, `Gain` (`effect.rs:62-71`) and `Saturator` (`effect.rs:121-129`) are memoryless. §4.4(6)'s three-way transport check re-verified in all three parts: `process` takes only `sample_rate`/`frames`/`note_inputs` (`graph/lib.rs:456-461`), `Transport` appears nowhere under `spectre-graph/src/` or `spectre-dsp/src/`, and `Transport::advance` (`core/transport.rs:96`) is called only from `spectre-core`'s own tests. | — |
| 1E Graph and plan contract | 3 | `compile` allocates the channel pool at `graph/lib.rs:324` (`vec![vec![0.0; max_frames]; channel_count]`) and runs once before the block loop. The new `fixture.rs` does not disturb this: it holds the `EditableGraph` → `compile` half and returns an immutable plan, which is exactly GRAPH-001's split (`requirements-ledger.md:55`, verified). `render_plan`'s existing `process`/hash tail (`lib.rs:259-285`) stays put, and `compile_fixture_plan_with` returning `(CompiledPlan, NodeId)` supplies the `pulse` node `PlanNoteInput` needs at `lib.rs:262-265` — the extraction is signature-coherent. No per-parameter recompilation anywhere. | — |
| 1F Failure behavior | 3 | Unchanged, and one dangling symbol fixed: §3.6 E3 now cites `bounce::max_frames(sample_rate)`, which §4.2 actually declares, in place of the `BOUNCE_MAX_FRAMES` that §4.2 never declared. E1–E9 remain explicit and fail-closed; E7 keeps the file because it is evidence and says so. §5.2 test 15's `frame_capacity_rejections() == 0` / `blocks_rendered() == 16` pair is still the correct guard against `bridge.rs:167-173`'s silent-refusal path. | — |
| 1G Test specification | 2 | **Two things moved in opposite directions.** Iteration 1's unfailable test 1 is genuinely fixed: I recomputed the fold over `GOLDEN_INPUT` from its literal array and got `0xa49acc9ce7359a37` (decimal 11,861,017,542,899,636,791), matching the spec exactly, and all eight bit patterns it lists (`0x00000000, 0x80000000, 0x3f800000, 0xbf800000, 0x3f000000, 0xbe800000, 0x40700000, 0xbb000000`) are correct. The value is fixed outside the code under test, so the test can fail. New test 21 can also genuinely fail, and its reasoning about why nothing else pins node-ID order checks out: `plan_render_matches_hand_wired_chain` (`harness.rs:58`) builds processors directly with no `NodeId` anywhere (`:65-67`), and test 15 takes its note node from the builder's own return value. Its `IdGen`/`NodeId` reachability claim is also right — both crates are `[dependencies]` of `spectre-offline`, and `harness.rs` already imports across that boundary. **But test 17 is designated a "required gate" and cannot run as specified.** The `bridge_plan.rs` edit it gates leaves `AudioProcessor` imported and unused (see the feasibility check), and `cargo clippy --locked --workspace --all-targets -- -D warnings` fails before any assertion in the file executes. Scored 2 rather than 1 because every assertion in all 21 tests is sound and the defect is one wrong entry in an import census. | Add `AudioProcessor` to the drop list in §4.3 and §7.2; the count is **seven** now-unused imports, not six — P1 |

**Lens average:** 2.857
**Lens pass:** Yes — avg ≥ 2.0, no criterion scores 1, no 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A Loop-first core loop | 3 | Unchanged. §3.1 still argues the non-modal panel from the workflow rather than from taste, §3.2 step 4 states selection/zoom/lens/transport are untouched, and §5.4's long-render row makes it a manual check. | — |
| 2B Linked lenses | 3 | Iteration 1's P3-1 is closed with more care than it asked for. §3.1 no longer claims the right edge: it places the bounce panel as a **second right-hand `SidePanel`** created between `inspector(ctx)` and `workspace(ctx)`. Verified exactly — `main.rs:438-442` is `transport / lenses / track_list / inspector / workspace` in that fixed order, `SidePanel::right("inspector")` is at `:163`, `SidePanel::left("tracks")` at `:116`, and `with_min_inner_size([1060.0, 680.0])` at `:508`. §5.4's screen-size row now checks all four panels at that minimum. The panel still stays off `AppModel`, so no lens forks state. | — |
| 2C Modulation visibility | 3 | Up from 2. §6.3 now answers PROD-002 in its own terms rather than leaving it implied, and it answers it against the real control list: destination path, length, two read-only derived fields, one checkbox, one button, a progress line, a report block — nothing automatable. `requirements-ledger.md:63` verified verbatim ("distinct visible states for automated / manually-overridden, with an explicit restore action"). The live version of the question stays routed to §8 Q2 rather than answered by the checkbox. | — |
| 2D Keyboard-first, calm UI | 3 | Unchanged and still the AF-5-correct handling: §3.4 refuses to assign any shortcut, cites the prohibited-conclusions list by name, states the remappable/context-scoped requirement the accepted direction does support, and gives the structural reason nothing forecloses it. No color-only state (§3.7). | — |
| 2E Convergent-pattern grounding | 3 | Every convergence and divergence re-opened. `OBS-AB12-MIX-002` (`ableton-live-observations.md:99`) says what the spec says, including "clipping matters only at physical outputs, the main output, or file export." `OBS-AB12-ARR-008` (`:32`) does say Consolidate incorporates clip-level gain/warp/pitch/envelopes **but not track effects** — so the deliberate divergence is grounded. `OBS-PP-ARCH-001` (`synth-modular-observations.md:25`, "At least one output module is required to produce sound") and `OBS-VCV-VOLT-006` (`:50`) both check out, as does `OBS-PP-ARCH-002`'s one-sample aux latency (`:26`). | — |
| 2F Differentiation | 3 | Unchanged and still correctly bounded: Spectre makes equivalence a CI gate with a localizable failure, and the spec explicitly does **not** claim no other DAW does this, "only that this repository holds no record of one." | — |
| 2G Benchmark evidence discipline | 3 | All thirteen cited `OBS-` IDs opened; every one says what the spec claims. Gap-naming is exact: `ableton-live.md:64` is `inventory-only` for chapter 20 "Bounce to Audio"; `:113` and `:139` are `section-inventoried`, and `:139`'s unresolved list is quoted **verbatim** ("signal-path equivalence, interruption, plugin realtime requirements, metadata, dither, SRC, partial-output cleanup, and failure reporting"); `logic-pro.md:64` reads `unreviewed`. Serum 2 discipline is exact: the spec cites only `OBS-SR2-CPU-001`, names `OBS-SR2-KB-001` as the other citable record, and **refuses** `OBS-SR2-GLOB-008` (which does exist, at `serum-2-observations.md:298`) against the quarantine banner at `:10-38` — verified verbatim, including "Do not cite this file, promote any record from it, or treat it as closing any gap," the missing `source-ledger.json` record, and the D-R2 tracking at `:38` / `decisions-needed.md:60`. Loudness is named as a gap with no conformance claim: `OBS-R128-005`, `OBS-R128-007` (−1 dBTP, ±0.3 dB), and `OBS-BS1770-006` (48 kHz coefficients only) all verified, and the file's status is `draft` with research state `in-review` (`:10-11`), exactly as described. | — |

**Lens average:** 3.000
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A Milestone fit | 3 | Anchors re-verified at their exact lines: `current-milestone.md:12` scope ends "save/reload, bounce"; `:86` is verbatim "Offline bounce renders the same project deterministically and matches the live path's computation"; `NEXT.md:30` is slice 8 with "extending the hash-equivalence approach the callback bridge already uses." §7.4 still defers stem export, codecs, dither, 16/24-bit, SRC, region export, realtime bounce, metadata, and batch renders by name. | — |
| 3B Non-goal respect | 3 | Unchanged. §6.4 enumerates each non-goal; the output is a plain WAV of Spectre's own render. `current-milestone.md:77` verified as covering recording and latency compensation as R5+. | — |
| 3C Deliberately small first devices | 3 | "**No device is added or modified**" still holds after remediation 2 — `fixture.rs` moves definitions and adds no DSP. §6.4 still declines a limiter, a fade-out, and a normalizer for the stated reason. | — |
| 3D Originality | 3 | Up from 2, and this is where remediation 2's new content lands cleanly. The 48× error is gone; I recomputed every replacement figure independently and all are exact: 86,400 × 48,000 = 4,147,200,000 frames, ÷ 256 = 16,200,000 blocks, × 8 B = 129.6 MB; 180 × 48,000 ÷ 256 = 33,750 blocks with 4,185 of them **exactly** 12.4%; the file it sits beside is 33.2 GB and the log is exactly 1/256 of it. **Three constants, three ledger rows, and I counted the rows myself** — §7.2 has one `requirements-ledger.md` entry scheduling three, §4.2 declares exactly three, §6.4 says three; they agree. Each rationale is Spectre-derived and each was checked: `BOUNCE_MAX_SECONDS` from TIME-003's own 24-hour horizon (`requirements-ledger.md:38`, verified); `BOUNCE_FALLBACK_BLOCK_FRAMES` from R4-1 §4.2's `ENGINE_BUFFER_FRAMES = 256` (verified at that spec's line 393); and the **new** `BOUNCE_FALLBACK_SAMPLE_RATE` from Spectre's own render corpus — which is the best-evidenced of the three, and every particular of it is true: `48_000` occurs exactly 24 times in `harness.rs`, `bridge_plan.rs:19` is `SAMPLE_RATE: f64 = 48_000.0`, and under `crates/*/src/` the only occurrences at all are `spectre-core/src/tempo.rs:124` and `time.rs:174`, both inside `#[cfg(test)]` modules (`:118`, `:158`). The iteration-3 retraction is itself correct rather than cosmetic: `current-milestone.md:111-113`'s table has columns for platform, date, backend/device, blocks, xruns, worst headroom, plan errors, and contaminated — **no sample-rate column and no block-size column** — so the earlier attribution really was an overclaim. `FIXTURE_SEED` and the four `FIXTURE_*` values are moved identities and fixture data, correctly excluded from PROD-003's scope. | Minor: §7.2's "the list is exhaustive" sentence omits `FNV_OFFSET_BASIS`/`FNV_PRIME` and the test file's `GOLDEN_HASH`. None needs a row; the exhaustiveness claim just isn't — P3 |
| 3E Platform commitment | 3 | Unchanged. One code path, `std::fs` and `std::thread` only, no audio device, headless on both platforms. Verified that nothing in the bounce path is platform-conditional and that `spectre-offline` has no platform-conditional dependency. Discharges none of decision 23's debt and forbids recording a Linux result. | — |
| 3F Accessibility trajectory | 3 | Re-verified: `crates/spectre-app/Cargo.toml:13` builds eframe with `default-features = false` and only `default_fonts` and `glow`, so no accessibility feature is enabled and the spec claims none. Standard widgets, text-carried progress and results, no icon-only control, `MISMATCH` must not be color-only. The `right_to_left` cluster it reasons against exists at `main.rs:78`. | — |

**Lens average:** 3.000
**Lens pass:** Yes
**Auto-fail triggered:** No — see roll-call below

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A Current-state accuracy | **1** | I opened every source path §7.1 cites and checked roughly seventy line references. Almost all are exact, including everything the remediations touched: the six-row fixture census (`lib.rs:210-258` topology; `lib.rs:297-300` and `:328-331` values; `bridge_plan.rs:24-27` + seed `:30`; `bridge_plan.rs:33-77` second topology; `harness.rs:65-67` hand-wired) is correct in every row; `render_plan` is private at `:204-285`, compiles at `:243`, calls `process` once at `:259-267`; `DeviceValues::from_snapshot` really does span `:45-146`; `fixture_events` is `:178-201`; `ProjectDoc`'s five fields are at `spectre-project/src/lib.rs:29-38`; `spectre-app/src/` really contains only `lib.rs` and `main.rs`; the "only filesystem access in `crates/` is two reads" claim (`spectre-offline/src/main.rs:16`, `rt_guard.rs:311`) is **exactly true** on a workspace-wide grep. **The defect is a false exhaustiveness claim, stated twice in §7.1 and once more in §4.3.** §7.1's Absent block says "The only hash is the inline loop at `lib.rs:271-278` and its verbatim duplicate at `bridge_plan.rs:80-92`" and "the FNV constants are duplicated verbatim across those two files." A grep for the two constants over `crates/` returns eight lines at **four** sites in **three** files: `lib.rs:271`/`:276`, `bridge_plan.rs:81`/`:87`, **`harness.rs:98`/`:103`** (inside `plan_render_matches_hand_wired_chain`) and **`harness.rs:156`/`:161`** (inside `hand_wired_report`). Both harness folds are complete, hand-written, channel-major-planar FNV-1a walks, so §4.3's "it is the workspace's only independently written implementation of the FNV walk" is false as well. Also: `lib.rs` is **334** lines, not the 335 §7.1 states (`wc -l` = 334; `cat -n`'s last line is 334, file ends with a newline). Scored 1 rather than 0 because the overwhelming majority of §7.1 is exact and the error's direction is conservative — it undercounts existing independent verification rather than inventing code that is not there. | Correct the census in §7.1 and §4.3; state why `harness.rs`'s two folds are or are not in scope for the de-duplication — P1 |
| 4B Status vocabulary | 3 | §7.1's Implemented / Planned / Gated / Absent partition is accurate and its cross-spec statuses check out: R4-1 "spec'd and passed at 2.950" matches `R4-1-live-audio-wiring-scorecard-iter2.md` (composite 2.950, PASS) and `manifest.md:44`, and `crates/spectre-app/src/engine.rs` genuinely does not exist; R4-7 at 3.000 confirmed. §7.2 still moves status to `implemented`, **not** `verified`, until §5.4's manual checks pass. | — |
| 4C Traceability | 3 | Up from 2 — iteration 1's only 4C defect was the 48× arithmetic, and it is gone with every replacement figure recomputing exactly. All six ledger IDs re-verified at their exact lines (RT-001 `:28`, RT-002 `:29`, RT-003 `:30`, TIME-003 `:38`, GRAPH-001 `:55`, PROD-002 `:63`, PROD-003 `:64`) and both decision rows (15 at `decision-gates.md:39`, 16 at `:40`, 17 at `:41`, 22 at `:47` including "(b) is rejected on its face"). The one citation that does not support its claim — `AudioProcessor` at `bridge_plan.rs:223`, which is a comment — is a codebase-accuracy failure and is charged to 4A and 1G rather than double-counted here. | — |
| 4D Honest gaps | 3 | Ten open questions, none decorative, and Q9 is now a stronger argument against the spec's own bundling than before, because the two extractable modules are the whole of the diff into `bridge_plan.rs`. Q4's libm reasoning re-verified (`f32::tanh` at `effect.rs:119` and `:127`, `f64::powf` at `source.rs:204`). Q5 still refuses to edit `dsp-device-io.md:113` by assertion — that line reads exactly "Identical initial state and input produce bit-identical offline output." The `Gain`-smoothing documentation divergence is re-verified true in both directions: `dsp-device-io.md:94` says "`Gain` already smooths" and `:104` says "click-resistant smoothing", while `effect.rs:62-71` multiplies by `self.gain` with no smoothing state at all. | — |
| 4E Evidence commands | 3 | The workspace gate is quoted exactly as criteria.md requires. Every `-p X --test Y` target either exists today (`spectre-offline --test harness`, `spectre-audio --test bridge_plan`, `--test rt_guard`, `spectre-graph --test containment` — all four present) or is created by this spec and named as such. Signatures the tests call were checked against source and match: `RenderBridge::new(plan, control, note_node, sample_rate, note_scratch)` at `bridge.rs:133-139`, `control_channel(targets, note_capacity, transport_capacity)` at `control.rs:195-199`, `NodeId::new` at `graph/lib.rs:24`. The gate *would fail* on the scheduled `bridge_plan.rs` edit, but that is the plan being wrong, not the command — charged to 1G. | Minor: test 15 widens the note lane to `control_channel(&[], 1024, 8)` where every existing test uses `64`, with no word about why, in a spec otherwise scrupulous about every number — P3 |
| 4F No fake surfaces | 3 | Verified against `STATUS.md:19` ("`./spectre` does not use the audio crate, so nothing launchable makes sound") and `:37` ("This is not a live engine … only from a test, never from `./spectre`"). §3.3's empty-state mandate survives remediation 2 intact, and it is still correct that `ProjectDoc` has no tracks, clips, or devices. §4.7 still reports a measured realtime factor and asserts no target. | — |

**Lens average:** 2.667
**Lens pass:** Yes — avg ≥ 2.0

---

## Auto-fail roll-call

| Rule | Result | Basis |
|---|---|---|
| AF-1 Contradicting accepted authority | **Pass** | The one narrowing of an accepted statement (`dsp-device-io.md:113`) is flagged in §7.2 and routed to §8 Q5 rather than edited. No accepted ledger requirement or decision row is contradicted; decision 22's rejection of option (b) is respected. |
| AF-2 Unbacked implementation claims | **Pass, narrowly** | AF-2 as written covers code "described as existing, partial, or implemented without a source path that actually contains it." Every positive claim I checked resolved in one `Read`, including all the new ones from remediation 2. The census error at 4A runs the *other* direction — claimed-absent, actually present — which AF-2's text does not reach but the feasibility rule does. Recorded here so the distinction is on the record rather than silently absorbed. |
| AF-3 Realtime discipline violation | **Pass** | The new completion handoff adds nothing on a callback-reachable path. `JoinHandle::is_finished()` does not block; the `join()` happens only after it returns `true`, so it does not wait on the render; both live on the app thread, which is not the callback thread. All file I/O stays on the bounce worker inside `wav.rs`. The polling cadence is an existing unconditional `request_repaint_after` (`main.rs:443`), so no new timer, no spin, and no wait was introduced. `RenderBridge::render` is untouched. |
| AF-4 Borrowed numeric limits | **Pass** | This was the live risk and it is clean. Three constants; I enumerated every `const` in the spec and found no fourth bound (`FNV_*` are moved algorithm constants, `FIXTURE_*` and `FIXTURE_SEED` are moved identities and fixture data, `GOLDEN_HASH` is a test vector, `max_frames` is derived). All three rationales are Spectre-derived and all three are scheduled into `requirements-ledger.md` in §7.2; I counted the scheduled rows and the spec's "three, not two" claim is correct. The new constant's rationale survives every check I could put to it, and the spec's own retraction of the qualification-table attribution is correct — that table has neither column. |
| AF-5 Conclusions the evidence does not support | **Pass** | §3.4 refuses a default shortcut map by name and cites the prohibited-conclusions list. No native-device list, synthesis architecture, gesture budget, interface model list, monitoring-latency threshold, platform order, or archetype promotion. Loudness stays out of scope with **no** conformance claim; `peak` is explicitly the plain sample-peak maximum already computed at `lib.rs:273` and "must never be labelled" true-peak. |
| AF-6 Optimistic language | **Pass** | Iteration 1 flagged "bounded and cheap" as a close call against wrong arithmetic; the arithmetic is now right and the claim is correspondingly narrowed — §4.4(8) retracts "cheap enough to always be on" by name and ties the flag to its only consumer. The spec's posture on state remains the opposite of optimistic. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every new type is fully specified with fields and signatures. `fixture.rs` sits as a child module of the crate root, so `pub(crate) compile_fixture_plan_with(DeviceValues, …)` can name the root-private `DeviceValues` — the same mechanism `bounce.rs` relies on. `BounceConfig` correctly has no `Default`; `fallback_config(frames)` replaces the two `BounceConfig::default()` call sites that iterations 1 and 2 wrote against a derive list that never contained it. |
| API/interface changes are feasible with current architecture | ✓ | `compile_fixture_plan_with` returning `(CompiledPlan, NodeId)` supplies exactly what `render_plan` needs for its `PlanNoteInput { node: pulse, … }` at `lib.rs:262-265`, so the extraction is signature-coherent. |
| **Crate boundary argument** | ✓ **all four claims verified** | `crates/spectre-audio/Cargo.toml:24` is `spectre-offline = { path = "../spectre-offline" }` under `[dev-dependencies]` — so no new edge for either `hash` or `fixture`. Both crates take `spectre-graph` by the **same workspace path** (`spectre-audio/Cargo.toml:21` and `spectre-offline/Cargo.toml:15`, both `path = "../spectre-graph"`), so `CompiledPlan` and `NodeId` are the same types on both sides. `spectre-offline`'s `[dependencies]` already hold everything the builder needs, so **no manifest change**. And the recorded consequence is true: no `pub` item in `spectre-offline` today names a `spectre-graph` type (the public surface is `OfflineReport`, `RenderReport`, `default_project`, `inspect_project`, `fixture_events`, `render_vertical_slice`, `render_app_snapshot`, `render_silence`), while `spectre-dsp`'s already appear via `render_app_snapshot`'s `&[DeviceParameterSnapshot]` — so `spectre-graph` types do enter the public API for the first time. The `spectre-app` cycle hazard is also real: `spectre-offline/Cargo.toml:21` dev-depends on `spectre-app`. |
| **Duplication census** | ✓ **all six rows exact** | `lib.rs:210-258` holds the seed (`:210`), three `NodeId`s (`:211-213`), three `add_node` calls, the two `connect` calls (`:231-240`), and `compile(saturator, …)` with its factory closure (`:242-258`). `lib.rs:297-300` and `:328-331` are each `0.3 / 0.7 / 2.5 / 0.35`. `bridge_plan.rs:24-27` are the four consts with `FIXTURE_SEED` at `:30`; `:33-77` is the second full topology. `harness.rs:65-67` is the hand-wired third copy of the values. |
| **`harness.rs:65-67` kept deliberately — does the reasoning hold?** | ✓ **yes** | Verified rather than accepted. `plan_render_matches_hand_wired_chain` (`harness.rs:58`) builds `ProcessContext` directly (`:62-63`), constructs the three processors from its own literals (`:65-67`), calls `AudioProcessor::process` on each **outside the graph** (`:73-95`), folds its own hash (`:97-105`), and asserts `plan_report.hash == hash` and `plan_report.peak == peak` against `render_vertical_slice` (`:107-109`). It shares no code with the builder and has no `NodeId` anywhere. So it genuinely is the independent check on what `fixture.rs` would build, and the "de-duplicate the specimen, never the last independent instrument" rule is applied correctly to it. |
| **`-D warnings` knock-on** | ✗ **one wrong entry** | The drop list is right — `Gain`, `PulseInstrument`, `Saturator`, `Waveform` (`:14`) and `EditableGraph`, `Connection` (`:16`) are used only inside `fixture_plan`. The stay-used list is right for four of five: `IdGen` at `:207` is a real use (`IdGen::new(0x0050_4152_414d)`), `NoteEvent` at `:178` and `NoteEventKind` at `:181` are real uses, and `CompiledPlan`/`NodeId` at `:33` survive because the signature is kept. **`AudioProcessor` is wrong.** Its only occurrences in the file are the import at `:14` and a *comment* at `:223` ("// processor needs an AudioProcessor parameter seam…"). Its only real uses are the three `.io()` calls at `:45`, `:48`, `:52` — all inside `fixture_plan`'s body — and `io()` is a method of `pub trait AudioProcessor` (`crates/spectre-dsp/src/io.rs:163-164`), so the trait must be in scope for them. Once `fixture_plan` delegates, `AudioProcessor` is an unused import and `-D warnings` fails. The correct count is seven imports, not six. |
| **Arithmetic** | ✓ **every figure recomputed** | 129.6 MB chain: 86,400 × 48,000 = 4,147,200,000; ÷ 256 = 16,200,000; × 8 B = 129,600,000 B. Three-minute example: 180 × 48,000 ÷ 256 = 33,750, and 4,185/33,750 = 0.124 exactly. Pool/scratch: 6 × 256 × 4 = 6,144 B and 256 × 2 × 4 = 2,048 B, summing to 32 B × `block_frames` → 8,192 / 16,384 / 32,768 / 65,536 at 256 / 512 / 1,024 / 2,048, all correct, and 65,536 B is 64 KiB. Single-pass rejection: 6 × 8,640,000 × 4 = 207.36 MB for a 69.12 MB result. 24-hour file 33.18 GB, and the log is exactly 1/256 of it. Test 13's `128 × 2 × 4 = 1024`. Test 16's `2,113 ÷ 256 = 8` with `frame_in_block = 65`. Test 6's geometry: a NaN at absolute frame 300 silences 0–511 in one 512-frame quantum and only 256–511 in two 256-frame quanta. **Golden vector recomputed from the literal array, not trusted:** `0xa49acc9ce7359a37`, decimal 11,861,017,542,899,636,791, and all eight bit patterns as listed. |
| **§4.4(1) / §7.2 agreement** | ✓ **fixed, not asserted** | The iteration-2 defect is genuinely resolved. §4.4(1) says the bounce calls `fixture::compile_fixture_plan_with`; §4.1's table lists `fixture.rs` as a new file; §7.2's New-files block schedules it with its full contents; §7.2's `lib.rs` entry schedules edits (i), (ii), and (iii) that match §4.3's call-site table row for row; and §7.2's `bridge_plan.rs` entry schedules the `fixture_plan` replacement on `:33-77` while explicitly keeping `hash_interleaved` on `:80-92`. The two disjoint ranges are genuinely disjoint. All four sections now say the same thing. |
| Views/screens fit current navigation pattern | ✓ | Improved over iteration 1. The second-right-hand-`SidePanel` placement is coherent with `main.rs:438-442`'s fixed region order, and the inspector at `:163` keeps its width and position. |
| Dependencies available and version-compatible | ✓ | No new crate dependency and no new dependency edge, as verified above. |
| Platform/renderer requirements realistic | ✓ | `std::fs` and `std::thread` only; no device; headless on both platforms. |
| Test strategy executable with current infrastructure | ✗ | Test 21's imports are reachable and its assertion can fail; test 1's golden vector is correct and independent. But test 17 — a declared required gate — is blocked by the `-D warnings` defect above. |
| Performance budget realistic | ✓ | Now correct at every figure, and the ceiling case for `block_frames` that iteration 1 asked for is stated (32 B × `block_frames`, 64 KiB at 2,048). |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares each. The core proof composes only implemented components. |

**Feasibility verdict:** Not feasible as written — two mechanical corrections away
**Caveats:** the scheduled `bridge_plan.rs` edit does not compile under the spec's own gate; §7.1's FNV census is false in the claimed-absent direction.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.857 | 35% | 1.000 |
| 2 — DAW Workflow Depth | 3.000 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.667 | 20% | 0.533 |
| **Composite** | | | **2.883** |

**Pass conditions (values copied from criteria.md, which is binding):**
- [x] Composite ≥ **2.30** — 2.883 (iteration 1: 2.848)
- [x] Every lens average ≥ **2.00** — 2.857 / 3.000 / 3.000 / 2.667
- [x] No criterion scores **0** — lowest is 1
- [x] At most **two** criteria score 1 — exactly one (4A)
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear; see roll-call
- [ ] **Feasibility rule — NOT satisfied.** §7.1 misdescribes current state: "The only hash is the inline loop at `lib.rs:271-278` and its verbatim duplicate at `bridge_plan.rs:80-92`" and "the FNV constants are duplicated verbatim across those two files" are both false. `crates/spectre-offline/tests/harness.rs` holds two more complete hand-written FNV-1a folds with the same two constants, at `:98`/`:103` and `:156`/`:161`. criteria.md: "A spec whose §7.1 misdescribes current state fails regardless of composite."
- [x] Reviewer personally executed every command claimed as passing where required — no command in this spec is claimed as currently passing except the four existing regression targets, which are asserted only as "must continue to pass"; I verified those targets exist as files rather than running them, and recomputed every arithmetic claim including the golden vector by hand.

**All conditions met:** No → **FAIL (feasibility rule)**

---

## Where I disagree with the iteration-1 scorecard

Stated because this run has documented four consecutive cases of a review being overturned by
what came after it, and because the same discipline should apply to me.

1. **Iteration 1 marked the feasibility rule satisfied and wrote "§7.1 does not misdescribe
   current state anywhere I could find."** It does, in the passage above. Iteration 1's own 1D
   entry also says R4-8 "adds **no third copy** of the constants" and its 4A entry treats the
   two-file duplication as established. A `grep` for `cbf2_9ce4_8422_2325` over `crates/`
   returns four sites in three files. Iteration 1 read `bridge_plan.rs` in full and verified the
   `harness.rs` regression target only as a file that exists — which is exactly where the miss
   comes from, and it is a reasonable miss, not a careless one.
2. **Iteration 1's Priority 1 item 5 ("state what is lost by collapsing `bridge_plan.rs:80-92`")
   rests on the same false premise.** Its reasoning — that collapsing `hash_interleaved` would
   delete "the workspace's only *independent* implementation of the FNV walk" — is wrong on the
   facts; `harness.rs` holds two more. The *decision* remediation 1 made in response (keep the
   independent de-interleaving walk, swap only the two literals) is still the right one, because
   `hash_interleaved` is the only independent **interleaved** walk and it is the one that sits on
   the live/offline seam. But the spec should say that, not the superlative it currently says.
3. **Two claims I checked specifically because they looked like the kind of thing a prior review
   would have waved through, and which turned out to be right:** the spec's iteration-3
   retraction of the qualification-table provenance (the table genuinely has neither a
   sample-rate nor a block-size column, so the earlier attribution was an overclaim), and the
   §7.2 row count changing from two to three (correct; I counted, and the new constant genuinely
   was being asserted unnamed by §3.2 step 3's UI string in the earlier iterations).

---

## Remediation Brief

Both blocking items are corrections to sentences, not to design. No section needs rewriting, no
decision needs revisiting, and nothing in §4's architecture is affected.

### Priority 1 — Must fix before this spec can pass

1. **Add `AudioProcessor` to the `bridge_plan.rs` drop list, in both §4.3 and §7.2.** The
   "verified to stay used" line currently reads `AudioProcessor` (`:223`); `:223` is a comment.
   Its only real uses are the three `.io()` calls at `:45`, `:48`, `:52`, all inside the
   `fixture_plan` body that §7.2 deletes, and `io()` is a trait method of
   `spectre_dsp::AudioProcessor` (`crates/spectre-dsp/src/io.rs:163-164`), so the import cannot
   survive the delegation. The corrected sentence is **seven** now-unused imports —
   `AudioProcessor`, `Gain`, `PulseInstrument`, `Saturator`, `Waveform` from `:14`, and
   `EditableGraph`, `Connection` from `:16`. The rest of the stay-used list is correct and should
   stay: `IdGen` (`:207`), `NoteEvent` (`:178`), `NoteEventKind` (`:181`), `CompiledPlan` and
   `NodeId` (`:33`, kept by keeping the signature).
2. **Correct the FNV census in §7.1 and §4.3.** The constants exist at four sites in three files:
   `crates/spectre-offline/src/lib.rs:271`/`:276`, `crates/spectre-audio/tests/bridge_plan.rs:81`/`:87`,
   `crates/spectre-offline/tests/harness.rs:98`/`:103` (in `plan_render_matches_hand_wired_chain`),
   and `crates/spectre-offline/tests/harness.rs:156`/`:161` (in `hand_wired_report`). Both harness
   folds are complete hand-written channel-major-planar FNV-1a walks. Three sentences change:
   §7.1's "The only hash is…", §7.1's "duplicated verbatim across those two files", and §4.3's
   "the workspace's only independently written implementation of the FNV walk." The accurate
   superlative for `hash_interleaved` is that it is the only independently written **interleaved**
   walk, and the only one sitting on the live/offline seam — which is a better argument for
   keeping it than the one currently written, since it is the walk whose independence the
   equivalence test actually depends on.
3. **Say whether `harness.rs`'s two folds are in scope for the de-duplication, and why not if
   not.** §7.2's `harness.rs` entry ("not modified, and the absence is the claim") justifies only
   the *device values* at `:65-67`. It should also say that the file's two FNV folds stay
   hand-written for the same reason — they are the independent instrument pinning the shared
   specimen — or schedule them. Silence is the one option that leaves §7.2's rule stated more
   broadly than it is applied. Note the favourable consequence worth recording: because those
   folds exist, §5.2 test 19 remains a genuine two-implementation cross-check after `hash.rs`
   lands, which is a stronger position than the spec currently claims for itself.

### Priority 2 — Should fix for quality

1. **§7.1 — `crates/spectre-offline/src/lib.rs` is 334 lines, not 335.** Trivial in isolation, but
   §7.1 is the section graded on exactness and the number is in its heading. (Iteration 1's
   scorecard header carried the same figure, so this is inherited rather than introduced.)
2. **§5.2 test 15 — justify or drop the widened note lane.** It calls
   `control_channel(&[], 1024, 8)` where every existing test in `bridge_plan.rs` uses `64`, and at
   most a handful of events cross that lane during the test. In a spec this careful about every
   number it writes, an unexplained 16× widening reads as an accident. Either state the reason
   (per-block re-based events for sixteen blocks) or use `64`.

### Priority 3 — Consider for excellence

1. **§7.2 — the "exhaustive" ledger-row paragraph is not quite exhaustive.** It accounts for the
   tail, timeout, log-cap, and `block_frames` refusals, for `max_frames`, for `FIXTURE_SEED`, and
   for the four `FIXTURE_*` values, but not for `FNV_OFFSET_BASIS`/`FNV_PRIME` (moved algorithm
   constants) or the test file's `GOLDEN_HASH` (a checked-in vector). None owes a row under
   PROD-003, which governs limits; one clause naming them would make the claim true as stated.
2. **§4.3 — `NodeId`'s survival is thinner than the list implies.** After the refactor `NodeId` is
   used at exactly one place in `bridge_plan.rs`: `fixture_plan`'s return type at `:33`. That is
   sufficient, and §7.2 correctly commits to keeping the signature — but the dependency is worth
   naming, because a future simplification of that signature silently takes the import with it.
3. **§7.1 — the bridge test's block is driven at `:106-108`, not `:106-109`.** `:109` is blank.

---

**End of scorecard.**
